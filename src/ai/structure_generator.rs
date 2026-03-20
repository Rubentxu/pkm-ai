//! Structure Generator: AI-powered structure suggestions for organizing blocks
//!
//! This module suggests hierarchical structures (Structure Notes, Hub Notes)
//! based on semantic clustering of blocks.

#![allow(dead_code)]

use crate::ai::embeddings::EmbeddingGenerator;
use crate::ai::semantic_clustering::SemanticClusterer;
use crate::models::{Block, BlockType, VacancyLevel};
use anyhow::Result;
use std::collections::HashMap;
use ulid::Ulid;

/// A suggested structure for organizing blocks
#[derive(Debug, Clone)]
pub struct StructureSuggestion {
    /// Suggested structure block
    pub structure: Block,
    /// Blocks that should belong to this structure
    pub member_ids: Vec<Ulid>,
    /// Confidence score for the suggestion
    pub confidence: f32,
    /// Reasoning for the suggestion
    pub reasoning: String,
}

#[allow(dead_code)]
impl StructureSuggestion {
    /// Create a new structure suggestion
    pub fn new(
        structure: Block,
        member_ids: Vec<Ulid>,
        confidence: f32,
        reasoning: impl Into<String>,
    ) -> Self {
        Self {
            structure,
            member_ids,
            confidence: confidence.clamp(0.0, 1.0),
            reasoning: reasoning.into(),
        }
    }
}

/// Configuration for structure generation
#[derive(Debug, Clone)]
pub struct StructureConfig {
    /// Minimum number of blocks to form a structure
    pub min_cluster_size: usize,
    /// Maximum number of structures to suggest
    pub max_structures: usize,
    /// Similarity threshold for clustering
    pub similarity_threshold: f32,
    /// Include hub notes (broader categorization)
    pub include_hubs: bool,
    /// Include outline notes (document skeletons)
    pub include_outlines: bool,
}

impl Default for StructureConfig {
    fn default() -> Self {
        Self {
            min_cluster_size: 3,
            max_structures: 10,
            similarity_threshold: 0.5,
            include_hubs: true,
            include_outlines: true,
        }
    }
}

/// Structure generator for organizing blocks into hierarchies
///
/// This struct provides structure suggestion functionality working with in-memory block data.
#[derive(Debug, Clone)]
pub struct StructureGenerator {
    embeddings: EmbeddingGenerator,
    clusterer: SemanticClusterer,
    config: StructureConfig,
}

impl StructureGenerator {
    /// Create a new structure generator with default configuration
    pub fn new() -> Self {
        Self {
            embeddings: EmbeddingGenerator::new(),
            clusterer: SemanticClusterer::new(),
            config: StructureConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: StructureConfig) -> Self {
        Self {
            embeddings: EmbeddingGenerator::new(),
            clusterer: SemanticClusterer::new(),
            config,
        }
    }

    /// Suggest structures for a set of blocks
    ///
    /// Analyzes the semantic content and clustering of blocks
    /// and suggests Structure Notes that could organize them.
    pub async fn suggest_structure(
        &self,
        blocks: &[Block],
        domain: Option<&str>,
    ) -> Result<Vec<StructureSuggestion>> {
        if blocks.len() < self.config.min_cluster_size {
            return Ok(Vec::new());
        }

        let meaningful_blocks: Vec<_> = blocks.iter()
            .filter(|b| !b.content.is_empty() || !b.title.is_empty())
            .cloned()
            .collect();

        if meaningful_blocks.len() < self.config.min_cluster_size {
            return Ok(Vec::new());
        }

        // Generate embeddings sequentially to avoid lifetime issues with JoinSet
        let mut embeddings: Vec<Vec<f32>> = Vec::new();
        for block in &meaningful_blocks {
            if let Ok(emb) = self.embeddings.embed(block).await { embeddings.push(emb) }
        }

        let k = self.estimate_optimal_k(meaningful_blocks.len());
        let cluster_indices = self.clusterer.cluster(&embeddings, k).await?;

        let mut suggestions = Vec::new();

        for (cluster_idx, indices) in cluster_indices.iter().enumerate() {
            if indices.is_empty() || indices.len() < self.config.min_cluster_size {
                continue;
            }

            let cluster_blocks: Vec<_> = indices.iter()
                .filter_map(|&i| meaningful_blocks.get(i))
                .collect();

            let centroid = self.calculate_centroid(&embeddings, indices);
            let (title, description) = self.generate_structure_content(&cluster_blocks, domain)?;
            let coherence = self.calculate_coherence(&embeddings, indices);
            let vacancy = self.calculate_vacancy(indices.len());

            let mut structure = Block::structure(&title);
            structure.content = description;
            structure.metadata.insert("semantic_centroid".to_string(), serde_json::json!(centroid));
            structure.metadata.insert("coherence_score".to_string(), serde_json::json!(coherence));
            structure.metadata.insert("vacancy".to_string(), serde_json::json!(format!("{:?}", vacancy)));
            structure.metadata.insert("cluster_id".to_string(), serde_json::json!(cluster_idx));

            if let Some(d) = domain {
                structure.tags.push(d.to_string());
            }

            let member_ids: Vec<_> = indices.iter()
                .filter_map(|&i| meaningful_blocks.get(i))
                .map(|b| b.id)
                .collect();

            let reasoning = format!(
                "Cluster of {} blocks with coherence {:.0}%, vacancy: {:?}. Topics: {}",
                indices.len(),
                coherence * 100.0,
                vacancy,
                self.extract_cluster_keywords(&cluster_blocks)
            );

            suggestions.push(StructureSuggestion {
                structure,
                member_ids,
                confidence: coherence,
                reasoning,
            });
        }

        suggestions.sort_by(|a, b| b.member_ids.len().cmp(&a.member_ids.len()));
        suggestions.truncate(self.config.max_structures);

        Ok(suggestions)
    }

    pub async fn generate_from_tags(
        &self,
        blocks: &[Block],
        tags: &[String],
    ) -> Result<Vec<StructureSuggestion>> {
        if tags.is_empty() {
            return Ok(Vec::new());
        }

        let blocks_with_tags: Vec<_> = blocks.iter()
            .filter(|b| b.tags.iter().any(|t| tags.contains(t)))
            .cloned()
            .collect();

        if blocks_with_tags.len() < self.config.min_cluster_size {
            return Ok(Vec::new());
        }

        let tags_joined = tags.to_vec().join(", ");
        let title = format!("Hub: {}", tags_joined);
        let description = format!(
            "This hub note aggregates content related to: {}.\n\nIt contains {} linked blocks.",
            tags_joined,
            blocks_with_tags.len()
        );

        let mut structure = Block::new(BlockType::Hub, &title);
        structure.content = description;
        structure.tags.extend(tags.iter().cloned());

        let member_ids: Vec<_> = blocks_with_tags.iter().map(|b| b.id).collect();

        // Generate embeddings sequentially to avoid lifetime issues with JoinSet
        let mut embeddings: Vec<Vec<f32>> = Vec::new();
        for b in &blocks_with_tags {
            if let Ok(emb) = self.embeddings.embed(b).await { embeddings.push(emb) }
        }

        let coherence = if !embeddings.is_empty() {
            let indices: Vec<usize> = (0..embeddings.len()).collect();
            self.calculate_coherence(&embeddings, &indices)
        } else {
            0.5
        };

        Ok(vec![StructureSuggestion {
            structure,
            member_ids,
            confidence: coherence,
            reasoning: format!("Hub note for tags: {}. Contains {} blocks.", tags_joined, blocks_with_tags.len()),
        }])
    }

    pub async fn generate_outline(
        &self,
        blocks: &[Block],
        topic: &str,
        expected_sections: usize,
    ) -> Result<Option<StructureSuggestion>> {
        let topic_lower = topic.to_lowercase();
        let related_blocks: Vec<_> = blocks.iter()
            .filter(|b| {
                b.title.to_lowercase().contains(&topic_lower) ||
                b.content.to_lowercase().contains(&topic_lower) ||
                b.tags.iter().any(|t| t.to_lowercase().contains(&topic_lower))
            })
            .cloned()
            .collect();

        if related_blocks.is_empty() {
            return Ok(None);
        }

        // Generate embeddings sequentially to avoid lifetime issues with JoinSet
        let mut embeddings: Vec<Vec<f32>> = Vec::new();
        for b in &related_blocks {
            if let Ok(emb) = self.embeddings.embed(b).await { embeddings.push(emb) }
        }

        let k = expected_sections.min(related_blocks.len());
        let cluster_indices = self.clusterer.cluster(&embeddings, k).await?;

        let mut outline_content = format!("# Outline: {}\n\n", topic);
        let mut member_ids = Vec::new();

        for (i, indices) in cluster_indices.iter().enumerate() {
            if indices.is_empty() {
                continue;
            }

            let section_blocks: Vec<_> = indices.iter()
                .filter_map(|&idx| related_blocks.get(idx))
                .collect();

            if section_blocks.is_empty() {
                continue;
            }

            let section_title = self.generate_section_title(&section_blocks, i + 1);
            outline_content.push_str(&format!("## Section {}: {}\n\n", i + 1, section_title));

            for block in section_blocks.iter().take(3) {
                if !block.title.is_empty() && block.title != "Fleeting Note" {
                    outline_content.push_str(&format!("- {}\n", block.title));
                    member_ids.push(block.id);
                }
            }
            outline_content.push('\n');
        }

        outline_content.push_str("---\n*Generated by AI structure generator*\n");

        let mut structure = Block::outline(topic);
        structure.content = outline_content;

        Ok(Some(StructureSuggestion {
            structure,
            member_ids,
            confidence: 0.7,
            reasoning: format!("Outline with {} sections on topic '{}'", k, topic),
        }))
    }

    pub async fn suggest_parent(
        &self,
        block: &Block,
        structures: &[Block],
    ) -> Result<Option<StructureSuggestion>> {
        if structures.is_empty() {
            return self.suggest_new_parent(block, &[]).await;
        }

        let structure_blocks: Vec<_> = structures.iter()
            .filter(|b| matches!(b.block_type, BlockType::Structure | BlockType::Hub))
            .cloned()
            .collect();

        if structure_blocks.is_empty() {
            return self.suggest_new_parent(block, &[]).await;
        }

        let block_emb = self.embeddings.embed(block).await?;

        let mut best_match: Option<(&Block, f32)> = None;

        for structure in &structure_blocks {
            let struct_emb = self.embeddings.embed(structure).await?;
            let similarity = EmbeddingGenerator::cosine_similarity(&block_emb, &struct_emb);

            match &best_match {
                None => best_match = Some((structure, similarity)),
                Some((_, best_sim)) if similarity > *best_sim => {
                    best_match = Some((structure, similarity));
                }
                _ => {}
            }
        }

        if let Some((structure, similarity)) = best_match
            && similarity >= self.config.similarity_threshold {
                return Ok(Some(StructureSuggestion {
                    structure: structure.clone(),
                    member_ids: vec![block.id],
                    confidence: similarity,
                    reasoning: format!(
                        "Block '{}' is semantically similar ({:.0}%) to structure '{}'",
                        block.title,
                        similarity * 100.0,
                        structure.title
                    ),
                }));
            }

        self.suggest_new_parent(block, &[]).await
    }

    fn estimate_optimal_k(&self, n_blocks: usize) -> usize {
        let k = ((n_blocks as f32 / 2.0).sqrt().ceil() as usize).max(2);
        k.min(n_blocks)
    }

    fn calculate_centroid(&self, embeddings: &[Vec<f32>], indices: &[usize]) -> Vec<f32> {
        if indices.is_empty() {
            return vec![0.0; embeddings.first().map(|e| e.len()).unwrap_or(384)];
        }

        let dim = embeddings.first().map(|e| e.len()).unwrap_or(384);
        let mut centroid = vec![0.0f32; dim];

        for &i in indices {
            if let Some(emb) = embeddings.get(i) {
                for (j, val) in emb.iter().enumerate() {
                    centroid[j] += val;
                }
            }
        }

        let count = indices.len() as f32;
        for val in centroid.iter_mut() {
            *val /= count;
        }

        centroid
    }

    fn calculate_coherence(&self, embeddings: &[Vec<f32>], indices: &[usize]) -> f32 {
        if indices.len() < 2 {
            return 1.0;
        }

        let mut total_similarity = 0.0f32;
        let mut count = 0usize;

        for i in 0..indices.len() {
            for j in (i + 1)..indices.len() {
                let emb_i = &embeddings[indices[i]];
                let emb_j = &embeddings[indices[j]];
                total_similarity += EmbeddingGenerator::cosine_similarity(emb_i, emb_j);
                count += 1;
            }
        }

        if count == 0 {
            return 1.0;
        }

        total_similarity / count as f32
    }

    fn calculate_vacancy(&self, cluster_size: usize) -> VacancyLevel {
        let density = cluster_size as u32;
        VacancyLevel::from_density(density, self.config.min_cluster_size as u32 * 2)
    }

    fn generate_structure_content(
        &self,
        blocks: &[&Block],
        domain: Option<&str>,
    ) -> Result<(String, String)> {
        let titles: Vec<_> = blocks.iter()
            .filter(|b| !b.title.is_empty() && b.title != "Fleeting Note")
            .map(|b| b.title.as_str())
            .collect();

        let all_keywords = self.extract_common_keywords(blocks);

        let title = if !titles.is_empty() {
            if titles.len() <= 3 {
                format!("Structure: {}", titles.join(", "))
            } else {
                format!("Structure: {}, and {} more", titles[0], titles.len() - 1)
            }
        } else {
            format!("Structure: {} topics", all_keywords.len().min(5))
        };

        let mut description = String::new();

        if let Some(d) = domain {
            description.push_str(&format!("**Domain:** {}\n\n", d));
        }

        description.push_str("## Overview\n\n");
        description.push_str(&format!(
            "This structure note aggregates {} blocks with shared themes.\n\n",
            blocks.len()
        ));

        if !all_keywords.is_empty() {
            let keywords_str = all_keywords.iter().take(7).cloned().collect::<Vec<_>>().join(", ");
            description.push_str(&format!("**Key Topics:** {}\n\n", keywords_str));
        }

        description.push_str("## Related Blocks\n\n");
        for block in blocks.iter().take(10) {
            if !block.title.is_empty() {
                description.push_str(&format!("- [[{}]]\n", block.title));
            }
        }

        if blocks.len() > 10 {
            description.push_str(&format!("\n*...and {} more*\n", blocks.len() - 10));
        }

        Ok((title, description))
    }

    fn extract_common_keywords(&self, blocks: &[&Block]) -> Vec<String> {
        let stop_words = [
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for",
            "of", "with", "by", "from", "as", "is", "was", "are", "were", "been",
            "this", "that", "these", "those", "it", "its", "be", "have", "has",
            "what", "which", "when", "where", "why", "how",
        ];

        let mut keyword_freq: HashMap<String, usize> = HashMap::new();

        for block in blocks {
            let text = format!("{} {}", block.title, block.content);
            let words: Vec<_> = text.to_lowercase()
                .split_whitespace()
                .filter(|w| w.len() > 4 && !stop_words.contains(w))
                .map(|w| w.chars().filter(|c| c.is_alphanumeric()).collect::<String>())
                .filter(|w| w.len() > 3)
                .collect();

            for word in words {
                *keyword_freq.entry(word).or_insert(0) += 1;
            }
        }

        let mut keywords: Vec<_> = keyword_freq.into_iter().collect();
        keywords.sort_by(|a, b| b.1.cmp(&a.1));

        keywords.into_iter().take(10).map(|(k, _)| k).collect()
    }

    fn extract_cluster_keywords(&self, blocks: &[&Block]) -> String {
        let keywords = self.extract_common_keywords(blocks);
        keywords.iter().take(5).cloned().collect::<Vec<_>>().join(", ")
    }

    fn generate_section_title(&self, blocks: &[&Block], section_num: usize) -> String {
        let keywords = self.extract_common_keywords(blocks);

        if !keywords.is_empty() {
            let keyword = keywords.first().unwrap();
            format!("{} ({})", Self::title_case(keyword), section_num)
        } else {
            format!("Section {}", section_num)
        }
    }

    fn title_case(s: &str) -> String {
        let mut result = String::new();
        let mut capitalize_next = true;

        for c in s.chars() {
            if c.is_whitespace() || c == '-' || c == '_' {
                capitalize_next = true;
                result.push(c);
            } else if capitalize_next {
                result.extend(c.to_uppercase());
                capitalize_next = false;
            } else {
                result.extend(c.to_lowercase());
            }
        }

        result
    }

    async fn suggest_new_parent(
        &self,
        block: &Block,
        related_blocks: &[Block],
    ) -> Result<Option<StructureSuggestion>> {
        if related_blocks.is_empty() {
            return Ok(None);
        }

        let other_blocks: Vec<_> = related_blocks.iter()
            .filter(|b| b.id != block.id)
            .cloned()
            .collect();

        if other_blocks.len() < self.config.min_cluster_size {
            return Ok(None);
        }

        let all_blocks = std::iter::once(block.clone())
            .chain(other_blocks)
            .collect::<Vec<_>>();

        let member_ids: Vec<_> = all_blocks.iter().map(|b| b.id).collect();
        let (title, description) = self.generate_structure_content(&all_blocks.iter().collect::<Vec<_>>(), None)?;
        let mut structure = Block::structure(&title);
        structure.content = description;

        let member_count = member_ids.len();
        Ok(Some(StructureSuggestion {
            structure,
            member_ids,
            confidence: 0.6,
            reasoning: format!(
                "No existing structure found. Created new structure '{}' grouping {} related blocks.",
                title,
                member_count
            ),
        }))
    }
}

impl Default for StructureGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_block(title: &str, content: &str) -> Block {
        Block::permanent(title, content)
    }

    #[test]
    fn test_structure_suggestion_creation() {
        let block = Block::structure("Test Structure");
        let suggestion = StructureSuggestion::new(block, vec![Ulid::new(), Ulid::new()], 0.85, "Test reasoning");
        assert_eq!(suggestion.confidence, 0.85);
        assert_eq!(suggestion.member_ids.len(), 2);
    }

    #[tokio::test]
    async fn test_estimate_optimal_k() {
        let generator = StructureGenerator::new();
        assert_eq!(generator.estimate_optimal_k(10), 3);
        assert_eq!(generator.estimate_optimal_k(5), 2);
    }

    #[tokio::test]
    async fn test_suggest_structure_with_few_blocks() {
        let generator = StructureGenerator::new();
        let blocks = vec![
            create_test_block("Block 1", "Content 1"),
            create_test_block("Block 2", "Content 2"),
        ];
        let suggestions = generator.suggest_structure(&blocks, None).await.unwrap();
        assert!(suggestions.is_empty());
    }
}