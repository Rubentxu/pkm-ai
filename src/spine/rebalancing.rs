//! Spine rebalancing implementation
//!
//! Auto-organizes sections by density and rebalances sequence weights.

#![allow(dead_code)]

use crate::db::Database;
use crate::models::{Block, BlockType, FractionalIndex, LinkType};
use crate::NexusResult;
use ulid::Ulid;
use std::collections::HashMap;

/// Rebalancing action
#[derive(Debug, Clone)]
pub enum RebalanceAction {
    /// Split a section into subsections
    Split {
        section_id: Ulid,
        clusters: Vec<SectionCluster>,
    },
    /// Merge multiple sections into one
    Merge {
        source_ids: Vec<Ulid>,
        target_id: Ulid,
    },
    /// Reassign sequence weights
    Reweight {
        section_id: Ulid,
        changes: Vec<WeightChange>,
    },
    /// Move a block to a different section
    Move {
        block_id: Ulid,
        from_section: Ulid,
        to_section: Ulid,
        new_weight: FractionalIndex,
    },
}

/// A cluster of blocks for splitting
#[derive(Debug, Clone)]
pub struct SectionCluster {
    /// Cluster ID
    pub id: Ulid,
    /// Suggested title
    pub title: String,
    /// Block IDs in this cluster
    pub block_ids: Vec<Ulid>,
    /// Average density
    pub density: usize,
}

/// A weight change to apply
#[derive(Debug, Clone)]
pub struct WeightChange {
    /// Block ID
    pub block_id: Ulid,
    /// Old weight
    pub old_weight: FractionalIndex,
    /// New weight
    pub new_weight: FractionalIndex,
}

/// Rebalancing report
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RebalanceReport {
    /// Actions taken
    pub actions: Vec<RebalanceAction>,
    /// Sections affected
    pub sections_affected: usize,
    /// Blocks moved
    pub blocks_moved: usize,
    /// Density improvement (ratio)
    pub density_improvement: f32,
}

/// Spine rebalancer
#[allow(dead_code)]
pub struct SpineRebalancer<'a> {
    db: &'a Database,
}

#[allow(dead_code)]
impl<'a> SpineRebalancer<'a> {
    /// Create a new SpineRebalancer
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// Analyze density and detect problems
    pub async fn analyze(&self) -> NexusResult<DensityAnalysis> {
        // Get all structure blocks
        let structures = self.db.blocks().list_by_type(BlockType::Structure).await?;

        let mut section_densities: Vec<SectionDensity> = Vec::new();

        for structure in &structures {
            let edges = self.db.edges().outgoing_from(&structure.id).await?;
            let next_count = edges.iter().filter(|e| e.link_type == LinkType::Next).count();

            // Calculate vacancy
            let vacancy = match next_count {
                0..=2 => Vacancy::High,
                3..=10 => Vacancy::Medium,
                _ => Vacancy::Low,
            };

            section_densities.push(SectionDensity {
                section_id: structure.id,
                title: structure.title.clone(),
                density: next_count as u32,
                vacancy,
                status: if next_count > 20 {
                    SectionStatus::Overfull
                } else if next_count <= 1 {
                    SectionStatus::Underfull
                } else {
                    SectionStatus::Normal
                },
            });
        }

        // Calculate overall statistics
        let total_blocks: u32 = section_densities.iter().map(|s| s.density).sum();
        let avg_density = if section_densities.is_empty() {
            0.0
        } else {
            total_blocks as f32 / section_densities.len() as f32
        };

        let max_density = section_densities
            .iter()
            .map(|s| s.density)
            .max()
            .unwrap_or(0);

        let min_density = section_densities
            .iter()
            .map(|s| s.density)
            .min()
            .unwrap_or(0);

        let imbalance_ratio = if min_density > 0 {
            max_density as f32 / min_density as f32
        } else {
            0.0
        };

        Ok(DensityAnalysis {
            sections: section_densities,
            total_sections: structures.len(),
            total_blocks,
            average_density: avg_density,
            max_density,
            min_density,
            imbalance_ratio,
        })
    }

    /// Rebalance sequence weights within a section
    ///
    /// With FractionalIndex, we rebalance by reassigning positions using after_last
    pub async fn rebalance_weights(&self, section_id: Ulid) -> NexusResult<Vec<WeightChange>> {
        // Get NEXT edges from this section
        let edges = self.db.edges().outgoing_from(&section_id).await?;
        let mut next_edges: Vec<_> = edges
            .into_iter()
            .filter(|e| e.link_type == LinkType::Next)
            .collect();

        if next_edges.is_empty() {
            return Ok(Vec::new());
        }

        // Sort by current weight (FractionalIndex already implements Ord)
        next_edges.sort_by(|a, b| a.sequence_weight.cmp(&b.sequence_weight));

        let mut changes = Vec::new();

        // Reassign weights using after_last to ensure proper ordering
        let mut last_position = FractionalIndex::first();
        for edge in next_edges.iter_mut() {
            let new_weight = FractionalIndex::after_last(&last_position);
            if new_weight != edge.sequence_weight {
                changes.push(WeightChange {
                    block_id: edge.to,
                    old_weight: edge.sequence_weight.clone(),
                    new_weight: new_weight.clone(),
                });
                edge.sequence_weight = new_weight;
            }
            last_position = edge.sequence_weight.clone();
        }

        // Update edges in database
        for edge in &next_edges {
            self.db.edges().update(edge.clone()).await?;
        }

        Ok(changes)
    }

    /// Rebalance all sections
    pub async fn rebalance_all(&self) -> NexusResult<RebalanceReport> {
        let mut actions = Vec::new();
        let blocks_moved = 0;

        // Analyze current state
        let analysis = self.analyze().await?;

        // Get all structure blocks
        let structures = self.db.blocks().list_by_type(BlockType::Structure).await?;

        for structure in &structures {
            // Rebalance weights for each section
            let changes = self.rebalance_weights(structure.id).await?;
            if !changes.is_empty() {
                actions.push(RebalanceAction::Reweight {
                    section_id: structure.id,
                    changes,
                });
            }
        }

        // Calculate improvement (simplified: based on reduction in max/min ratio)
        let new_analysis = self.analyze().await?;
        let improvement = if new_analysis.imbalance_ratio > 0.0 {
            analysis.imbalance_ratio / new_analysis.imbalance_ratio
        } else {
            1.0
        };

        Ok(RebalanceReport {
            actions,
            sections_affected: structures.len(),
            blocks_moved,
            density_improvement: improvement,
        })
    }

    /// Suggest optimal section structure based on clusters
    ///
    /// Uses semantic clustering to group related blocks
    pub async fn suggest_split(&self, section_id: Ulid) -> NexusResult<Vec<SectionCluster>> {
        // Get all blocks in this section
        let edges = self.db.edges().outgoing_from(&section_id).await?;
        let next_edges: Vec<_> = edges
            .into_iter()
            .filter(|e| e.link_type == LinkType::Next)
            .collect();

        if next_edges.len() < 10 {
            // Not enough blocks to split
            return Ok(Vec::new());
        }

        // Simple clustering based on keywords
        // A full implementation would use embeddings and k-means or DBSCAN
        let mut clusters: HashMap<String, Vec<Ulid>> = HashMap::new();

        for edge in &next_edges {
            if let Some(block) = self.db.blocks().get(&edge.to).await? {
                let keywords = self.extract_keywords(&block);
                if let Some(first_keyword) = keywords.first() {
                    clusters
                        .entry(first_keyword.clone())
                        .or_default()
                        .push(block.id);
                }
            }
        }

        // Convert to SectionCluster list
        let mut result: Vec<SectionCluster> = clusters
            .into_iter()
            .map(|(keyword, block_ids)| SectionCluster {
                id: Ulid::new(),
                title: format!("Subsection: {}", keyword),
                block_ids: block_ids.clone(),
                density: block_ids.len(),
            })
            .collect();

        // Sort by density descending
        result.sort_by(|a, b| b.density.cmp(&a.density));

        Ok(result)
    }

    /// Extract keywords from a block for clustering
    fn extract_keywords(&self, block: &Block) -> Vec<String> {
        let mut keywords = Vec::new();

        // Add title words
        for word in block.title.split_whitespace() {
            let cleaned = word.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase();
            if cleaned.len() > 2 {
                keywords.push(cleaned);
            }
        }

        // Add tags
        keywords.extend(block.tags.iter().map(|t| t.to_lowercase()));

        keywords
    }
}

/// Density analysis result
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DensityAnalysis {
    /// Per-section densities
    pub sections: Vec<SectionDensity>,
    /// Total sections
    pub total_sections: usize,
    /// Total blocks
    pub total_blocks: u32,
    /// Average density
    pub average_density: f32,
    /// Maximum density
    pub max_density: u32,
    /// Minimum density
    pub min_density: u32,
    /// Imbalance ratio (max/min)
    pub imbalance_ratio: f32,
}

/// Section density info
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SectionDensity {
    pub section_id: Ulid,
    pub title: String,
    pub density: u32,
    pub vacancy: Vacancy,
    pub status: SectionStatus,
}

/// Vacancy level
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vacancy {
    High,
    Medium,
    Low,
}

/// Section status
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionStatus {
    Overfull,
    Underfull,
    Normal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rebalance_action_display() {
        let action = RebalanceAction::Move {
            block_id: Ulid::new(),
            from_section: Ulid::new(),
            to_section: Ulid::new(),
            new_weight: FractionalIndex::first(),
        };

        let debug = format!("{:?}", action);
        assert!(debug.contains("Move"));
    }

    #[test]
    fn test_density_analysis_stats() {
        let analysis = DensityAnalysis {
            sections: vec![
                SectionDensity {
                    section_id: Ulid::new(),
                    title: "Section 1".to_string(),
                    density: 10,
                    vacancy: Vacancy::Medium,
                    status: SectionStatus::Normal,
                },
                SectionDensity {
                    section_id: Ulid::new(),
                    title: "Section 2".to_string(),
                    density: 2,
                    vacancy: Vacancy::High,
                    status: SectionStatus::Underfull,
                },
            ],
            total_sections: 2,
            total_blocks: 12,
            average_density: 6.0,
            max_density: 10,
            min_density: 2,
            imbalance_ratio: 5.0,
        };

        assert_eq!(analysis.total_sections, 2);
        assert!((analysis.average_density - 6.0).abs() < 0.001);
        assert!((analysis.imbalance_ratio - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_section_status() {
        assert_eq!(SectionStatus::from_density(25), SectionStatus::Overfull);
        assert_eq!(SectionStatus::from_density(5), SectionStatus::Normal);
        assert_eq!(SectionStatus::from_density(1), SectionStatus::Underfull);
    }

    impl SectionStatus {
        fn from_density(density: u32) -> Self {
            if density > 20 {
                SectionStatus::Overfull
            } else if density <= 1 {
                SectionStatus::Underfull
            } else {
                SectionStatus::Normal
            }
        }
    }
}
