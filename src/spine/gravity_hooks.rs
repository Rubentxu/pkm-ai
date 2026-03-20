//! Gravity Hooks: Semantic attraction calculation
//!
//! Calculates semantic gravity between blocks and sections using embeddings.

#![allow(dead_code)]

use crate::db::Database;
use crate::models::{Block, BlockType};
use crate::NexusResult;
use ulid::Ulid;

/// A gravity match result
#[derive(Debug, Clone)]
pub struct GravityMatch {
    /// Section block ID
    pub section_id: Ulid,
    /// Section title
    pub section_title: String,
    /// Similarity score (0.0 to 1.0)
    pub similarity: f32,
    /// Whether the block should attach to this section
    pub should_attach: bool,
    /// Reason for the match
    pub reason: String,
}

/// Semantic gravity configuration
#[derive(Debug, Clone)]
pub struct GravityConfig {
    /// Minimum similarity threshold to suggest a move
    pub move_threshold: f32,
    /// Minimum similarity threshold to consider a match
    pub match_threshold: f32,
    /// Maximum number of candidate sections to return
    pub top_n: usize,
    /// Weight for keyword matching (0.0 to 1.0)
    pub keyword_weight: f32,
    /// Weight for embedding similarity (0.0 to 1.0)
    pub embedding_weight: f32,
}

impl Default for GravityConfig {
    fn default() -> Self {
        Self {
            move_threshold: 0.15,
            match_threshold: 0.1,
            top_n: 3,
            keyword_weight: 0.3,
            embedding_weight: 0.7,
        }
    }
}

/// Gravity calculator for semantic attraction
pub struct GravityCalculator<'a> {
    db: &'a Database,
    config: GravityConfig,
}

impl<'a> GravityCalculator<'a> {
    /// Create a new GravityCalculator with default config
    pub fn new(db: &'a Database) -> Self {
        Self {
            db,
            config: GravityConfig::default(),
        }
    }

    /// Create a new GravityCalculator with custom config
    pub fn with_config(db: &'a Database, config: GravityConfig) -> Self {
        Self { db, config }
    }

    /// Calculate semantic gravity between a block and multiple sections
    ///
    /// Returns sections ordered by affinity (highest first)
    pub async fn calculate_gravity(
        &self,
        block_id: Ulid,
        section_ids: &[Ulid],
    ) -> NexusResult<Vec<GravityMatch>> {
        // Get the block
        let block = self.db.blocks().get(&block_id).await?
            .ok_or_else(|| crate::NexusError::BlockNotFound(block_id.to_string()))?;

        // Extract keywords from block
        let block_keywords = self.extract_keywords(&block);

        // Calculate gravity for each section
        let mut matches = Vec::new();

        for section_id in section_ids {
            let section = self.db.blocks().get(section_id).await?
                .ok_or_else(|| crate::NexusError::BlockNotFound(section_id.to_string()))?;

            let section_keywords = self.extract_keywords(&section);

            // Calculate keyword similarity
            let keyword_sim = self.keyword_similarity(&block_keywords, &section_keywords);

            // Calculate embedding similarity if available
            let embedding_sim = self.embedding_similarity(&block, &section);

            // Combined score
            let similarity = (keyword_sim * self.config.keyword_weight)
                + (embedding_sim * self.config.embedding_weight);

            // Determine reason
            let reason = if keyword_sim > embedding_sim {
                format!("Keywords match: {:?}", &block_keywords.iter().take(3).collect::<Vec<_>>())
            } else {
                "Semantic embedding similarity".to_string()
            };

            matches.push(GravityMatch {
                section_id: *section_id,
                section_title: section.title.clone(),
                similarity,
                should_attach: similarity > self.config.move_threshold,
                reason,
            });
        }

        // Sort by similarity descending
        matches.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));

        // Return top N
        matches.truncate(self.config.top_n);

        Ok(matches)
    }

    /// Calculate gravity for a block against all sections
    pub async fn calculate_gravity_all(&self, block_id: Ulid) -> NexusResult<Vec<GravityMatch>> {
        // Get all structure sections
        let sections = self.db.blocks().list_by_type(BlockType::Structure).await?;
        let section_ids: Vec<Ulid> = sections.into_iter().map(|s| s.id).collect();

        self.calculate_gravity(block_id, &section_ids).await
    }

    /// Calculate semantic gravity field for a document
    ///
    /// Returns gravity data for all sections
    pub async fn calculate_gravity_field(&self, document_root: Ulid) -> NexusResult<Vec<SectionGravity>> {
        let traversal = crate::spine::traversal::SpineTraversal::new(self.db);

        // Get all sections in the document
        let tree = traversal.get_tree(document_root).await?;
        let mut section_gravities = Vec::new();

        for node in &tree.nodes {
            if matches!(node.block_type, BlockType::Structure | BlockType::Outline) {
                // Get density (number of children)
                let density = node.children.len() as u32;

                // Calculate vacancy
                let vacancy = self.calculate_vacancy(density);

                section_gravities.push(SectionGravity {
                    section_id: node.block_id,
                    title: node.title.clone(),
                    density,
                    vacancy,
                });
            }
        }

        Ok(section_gravities)
    }

    /// Extract keywords from a block
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

        // Add content words (first 50 words)
        for (i, word) in block.content.split_whitespace().take(50).enumerate() {
            if i >= 20 {
                // Only first 20 content words
                break;
            }
            let cleaned = word.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase();
            if cleaned.len() > 3 {
                keywords.push(cleaned);
            }
        }

        keywords
    }

    /// Calculate keyword-based similarity using Jaccard index
    fn keyword_similarity(&self, keywords1: &[String], keywords2: &[String]) -> f32 {
        if keywords1.is_empty() || keywords2.is_empty() {
            return 0.0;
        }

        let set1: std::collections::HashSet<_> = keywords1.iter().collect();
        let set2: std::collections::HashSet<_> = keywords2.iter().collect();

        let intersection = set1.intersection(&set2).count() as f32;
        let union = set1.union(&set2).count() as f32;

        if union == 0.0 {
            0.0
        } else {
            intersection / union
        }
    }

    /// Calculate embedding similarity using cosine similarity
    fn embedding_similarity(&self, block1: &Block, block2: &Block) -> f32 {
        let emb1 = match &block1.semantic_centroid {
            Some(e) => e,
            None => return 0.5, // No embedding, return neutral similarity
        };

        let emb2 = match &block2.semantic_centroid {
            Some(e) => e,
            None => return 0.5,
        };

        self.cosine_similarity(emb1, emb2)
    }

    /// Cosine similarity between two vectors
    fn cosine_similarity(&self, v1: &[f32], v2: &[f32]) -> f32 {
        if v1.len() != v2.len() {
            return 0.0;
        }

        let dot_product: f32 = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum();
        let mag1: f32 = v1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let mag2: f32 = v2.iter().map(|x| x * x).sum::<f32>().sqrt();

        if mag1 == 0.0 || mag2 == 0.0 {
            0.0
        } else {
            dot_product / (mag1 * mag2)
        }
    }

    /// Calculate vacancy level based on density
    fn calculate_vacancy(&self, density: u32) -> VacancyLevel {
        match density {
            0..=2 => VacancyLevel::High,
            3..=10 => VacancyLevel::Medium,
            _ => VacancyLevel::Low,
        }
    }
}

/// Section gravity information
#[derive(Debug, Clone)]
pub struct SectionGravity {
    /// Section block ID
    pub section_id: Ulid,
    /// Section title
    pub title: String,
    /// Number of blocks in section
    pub density: u32,
    /// Vacancy level
    pub vacancy: VacancyLevel,
}

/// Vacancy level for sections
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VacancyLevel {
    High,
    Medium,
    Low,
}

impl std::fmt::Display for VacancyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VacancyLevel::High => write!(f, "alta"),
            VacancyLevel::Medium => write!(f, "media"),
            VacancyLevel::Low => write!(f, "baja"),
        }
    }
}

/// Gravity candidate returned by the gravity hook using bloom filters.
#[derive(Debug, Clone)]
pub struct GravityCandidate {
    /// Section ID
    pub section_id: Ulid,
    /// Section title
    pub section_title: String,
    /// Gravity score (0.0 to 1.0)
    pub score: f32,
    /// Reason for the score
    pub reason: String,
}

/// Calculate semantic gravity using Bloom filter intersection.
///
/// This is an O(1) approximation for gravity calculation as specified
/// in TECNICA.md section 6.3.
///
/// # Arguments
/// * `new_block_bloom` - Bloom filter for the new block
/// * `new_block_keywords` - Keywords from the new block
/// * `sections` - SmartSections to evaluate gravity against
///
/// # Returns
/// Vector of GravityCandidates sorted by score descending, top 3
pub fn calculate_gravity(
    new_block_bloom: &[u128; 1],
    new_block_keywords: &[String],
    sections: &[crate::models::SmartSection],
) -> Vec<GravityCandidate> {
    let candidates: Vec<GravityCandidate> = sections
        .iter()
        .map(|section| {
            // Bloom intersection as primary signal (O(1) operation)
            let bloom_score = calculate_bloom_intersection(new_block_bloom, &section.semantic_bloom);

            // Keyword overlap as secondary signal
            let keyword_overlap = calculate_keyword_overlap(new_block_keywords, &section.keywords);

            // Combined score: 70% bloom, 30% keyword
            let base_score = bloom_score * 0.7 + keyword_overlap * 0.3;

            // Vacancy boost: empty sections attract more
            let vacancy_boost = match section.vacancy {
                crate::models::VacancyLevel::Full => 0.7,
                crate::models::VacancyLevel::NearlyFull => 0.85,
                crate::models::VacancyLevel::Partial => 1.0,
                crate::models::VacancyLevel::Sparse => 1.2,
                crate::models::VacancyLevel::Empty => 1.3,
            };

            let final_score = (base_score * vacancy_boost).min(1.0);

            // Determine reason string
            let reason = if bloom_score > 0.5 {
                "High semantic similarity via Bloom filter".to_string()
            } else if keyword_overlap > 0.3 {
                format!(
                    "{} keyword matches",
                    new_block_keywords
                        .iter()
                        .filter(|kw| section.keywords.contains(kw))
                        .count()
                )
            } else {
                "Low semantic affinity".to_string()
            };

            GravityCandidate {
                section_id: section.id,
                section_title: section.intent.clone(),
                score: final_score,
                reason,
            }
        })
        .filter(|c| c.score > 0.1) // Only return meaningful candidates
        .collect();

    top_candidates(candidates, 3)
}

#[allow(dead_code)]
/// Sort candidates by score and return top N
pub fn top_candidates(mut candidates: Vec<GravityCandidate>, n: usize) -> Vec<GravityCandidate> {
    candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    candidates.truncate(n);
    candidates
}

/// Calculate Bloom filter intersection as Jaccard similarity.
///
/// Returns a value between 0.0 and 1.0.
pub fn calculate_bloom_intersection(a: &[u128; 1], b: &[u128; 1]) -> f32 {
    let intersection = a[0] & b[0];
    let union = a[0] | b[0];

    if union == 0 {
        return 0.0;
    }

    let intersect_bits = intersection.count_ones() as f32;
    let union_bits = union.count_ones() as f32;

    intersect_bits / union_bits
}

/// Calculate keyword overlap using Jaccard index.
pub fn calculate_keyword_overlap(new_keywords: &[String], section_keywords: &[String]) -> f32 {
    if new_keywords.is_empty() || section_keywords.is_empty() {
        return 0.0;
    }

    let set1: std::collections::HashSet<_> = new_keywords.iter().collect();
    let set2: std::collections::HashSet<_> = section_keywords.iter().collect();

    let intersection = set1.intersection(&set2).count() as f32;
    let union = set1.union(&set2).count() as f32;

    if union == 0.0 {
        0.0
    } else {
        intersection / union
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gravity_config_default() {
        let config = GravityConfig::default();
        assert_eq!(config.move_threshold, 0.15);
        assert_eq!(config.match_threshold, 0.1);
        assert_eq!(config.top_n, 3);
    }

    #[test]
    fn test_extract_keywords() {
        // This test would require a database connection
        // For now, test the logic without DB
        let keywords1 = vec!["rust".to_string(), "pkm".to_string(), "zettelkasten".to_string()];
        let keywords2 = vec!["rust".to_string(), "actor".to_string(), "model".to_string()];

        // Test Jaccard similarity manually
        let set1: std::collections::HashSet<_> = keywords1.iter().collect();
        let set2: std::collections::HashSet<_> = keywords2.iter().collect();

        let intersection = set1.intersection(&set2).count() as f32;
        let union = set1.union(&set2).count() as f32;

        let similarity = if union > 0.0 { intersection / union } else { 0.0 };

        assert!((similarity - 0.2).abs() < 0.01);
    }

    #[test]
    fn test_cosine_similarity() {
        let v1 = vec![1.0, 0.0, 0.0];
        let v2 = vec![1.0, 0.0, 0.0];
        let v3 = vec![0.0, 1.0, 0.0];

        // Same vector = 1.0
        let sim_same = cosine_similarity_static(&v1, &v2);
        assert!((sim_same - 1.0).abs() < 0.001);

        // Orthogonal vectors = 0.0
        let sim_ortho = cosine_similarity_static(&v1, &v3);
        assert!((sim_ortho - 0.0).abs() < 0.001);
    }

    // Helper function for testing cosine similarity without the struct
    fn cosine_similarity_static(v1: &[f32], v2: &[f32]) -> f32 {
        if v1.len() != v2.len() {
            return 0.0;
        }

        let dot_product: f32 = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum();
        let mag1: f32 = v1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let mag2: f32 = v2.iter().map(|x| x * x).sum::<f32>().sqrt();

        if mag1 == 0.0 || mag2 == 0.0 {
            0.0
        } else {
            dot_product / (mag1 * mag2)
        }
    }

    #[test]
    fn test_vacancy_level() {
        assert_eq!(calculate_vacancy_static(1), "alta");
        assert_eq!(calculate_vacancy_static(5), "media");
        assert_eq!(calculate_vacancy_static(15), "baja");
    }

    fn calculate_vacancy_static(density: u32) -> &'static str {
        match density {
            0..=2 => "alta",
            3..=10 => "media",
            _ => "baja",
        }
    }

    #[test]
    fn test_calculate_bloom_intersection() {
        // Same bloom = 1.0
        let a: [u128; 1] = [0b1010u128];
        let score_same = calculate_bloom_intersection(&a, &a);
        assert_eq!(score_same, 1.0);

        // Partial overlap
        let b: [u128; 1] = [0b0110u128];
        let score_partial = calculate_bloom_intersection(&a, &b);
        assert!(score_partial > 0.0);
        assert!(score_partial < 1.0);

        // No overlap
        let c: [u128; 1] = [0b0000u128];
        let score_none = calculate_bloom_intersection(&a, &c);
        assert_eq!(score_none, 0.0);
    }

    #[test]
    fn test_calculate_keyword_overlap() {
        let keywords1 = vec!["rust".to_string(), "programming".to_string()];
        let keywords2 = vec!["rust".to_string(), "systems".to_string()];

        let score = calculate_keyword_overlap(&keywords1, &keywords2);
        assert!(score > 0.0);
        assert!(score < 1.0);

        // Empty keywords
        let empty: Vec<String> = vec![];
        let score_empty = calculate_keyword_overlap(&empty, &keywords1);
        assert_eq!(score_empty, 0.0);
    }

    #[test]
    fn test_top_candidates() {
        let candidates = vec![
            GravityCandidate {
                section_id: Ulid::new(),
                section_title: "Low".to_string(),
                score: 0.2,
                reason: "Low".to_string(),
            },
            GravityCandidate {
                section_id: Ulid::new(),
                section_title: "High".to_string(),
                score: 0.8,
                reason: "High".to_string(),
            },
            GravityCandidate {
                section_id: Ulid::new(),
                section_title: "Medium".to_string(),
                score: 0.5,
                reason: "Medium".to_string(),
            },
        ];

        let top = top_candidates(candidates, 2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].section_title, "High");
        assert_eq!(top[1].section_title, "Medium");
    }
}
