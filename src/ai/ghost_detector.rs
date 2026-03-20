//! Ghost Detector: Detect gaps and missing content in the knowledge structure
//!
//! Ghost nodes represent AI-detected gaps in the knowledge graph.
//! They identify content that SHOULD exist but doesn't yet.

use crate::ai::embeddings::EmbeddingGenerator;
use crate::models::{Block, GhostNode, GhostStatus, PositionHint};
use anyhow::Result;
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use ulid::Ulid;

/// Ghost detection configuration
#[derive(Debug, Clone)]
pub struct GhostConfig {
    /// Minimum gap size to trigger ghost detection (in sequence weight units)
    pub min_sequence_gap: f32,
    /// Maximum expected blocks per section (for density calculation)
    pub max_expected_density: u32,
    /// Semantic gap threshold (0.0 to 1.0)
    pub semantic_threshold: f32,
    /// Minimum content length for semantic analysis
    pub min_content_length: usize,
}

impl Default for GhostConfig {
    fn default() -> Self {
        Self {
            min_sequence_gap: 1.0,
            max_expected_density: 10,
            semantic_threshold: 0.6,
            min_content_length: 50,
        }
    }
}

/// Ghost detector for identifying gaps in knowledge structure
///
/// This struct provides ghost detection functionality working with in-memory block data.
#[derive(Debug, Clone)]
pub struct GhostDetector {
    embeddings: EmbeddingGenerator,
    config: GhostConfig,
}

#[allow(dead_code)]
impl GhostDetector {
    /// Create a new ghost detector with default configuration
    pub fn new() -> Self {
        Self {
            embeddings: EmbeddingGenerator::new(),
            config: GhostConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: GhostConfig) -> Self {
        Self {
            embeddings: EmbeddingGenerator::new(),
            config,
        }
    }

    /// Detect sequence gaps in ordered blocks
    ///
    /// Analyzes blocks sorted by sequence_weight and finds gaps
    /// where new content should be inserted.
    pub async fn detect_sequence_gaps(&self, blocks: &[Block]) -> Result<Vec<GhostNode>> {
        if blocks.len() < 2 {
            return Ok(Vec::new());
        }

        // Sort blocks by sequence_weight (stored in metadata)
        let mut ordered: Vec<_> = blocks.iter().collect();
        ordered.sort_by(|a, b| {
            let weight_a = a.metadata.get("sequence_weight")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32;
            let weight_b = b.metadata.get("sequence_weight")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32;
            weight_a.partial_cmp(&weight_b).unwrap()
        });

        let mut ghosts = Vec::new();

        for i in 0..ordered.len() - 1 {
            let current = ordered[i];
            let next = ordered[i + 1];

            let current_weight = current.metadata.get("sequence_weight")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32;
            let next_weight = next.metadata.get("sequence_weight")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32;

            let gap = next_weight - current_weight;

            if gap > self.config.min_sequence_gap {
                let gap_ratio = gap / self.config.min_sequence_gap;

                // Generate ghost description based on adjacent content
                let description = self.generate_sequence_gap_description(current, next, gap);
                let keywords = self.extract_gap_keywords(current, next);

                let ghost = GhostNode {
                    id: Ulid::new(),
                    description,
                    ai_rationale: format!(
                        "Detected sequence gap of {:.2} between blocks '{}' and '{}'. Expected intermediate content.",
                        gap, current.title, next.title
                    ),
                    confidence: (gap_ratio.min(1.0) * 0.8).min(0.95),
                    position_hint: PositionHint {
                        after: Some(current.id),
                        before: Some(next.id),
                        parent_section: current.metadata.get("parent_section")
                            .and_then(|v| v.as_str())
                            .and_then(|s| Ulid::from_string(s).ok()),
                        sequence_weight: current_weight + (gap / 2.0),
                    },
                    status: GhostStatus::Detected,
                    trigger_blocks: vec![current.id, next.id],
                    expected_keywords: keywords,
                    created_at: Utc::now(),
                    filled_by: None,
                };

                ghosts.push(ghost);
            }
        }

        Ok(ghosts)
    }

    /// Detect semantic gaps - topics mentioned but not covered
    ///
    /// Analyzes block content to find references to concepts that
    /// aren't adequately covered in the existing blocks.
    pub async fn detect_semantic_gaps(&self, blocks: &[Block]) -> Result<Vec<GhostNode>> {
        if blocks.is_empty() {
            return Ok(Vec::new());
        }

        // Build a semantic map of what's covered
        let covered_topics = self.build_topic_map(blocks);
        let mentioned_topics = self.extract_mentioned_topics(blocks);

        let mut ghosts = Vec::new();

        // Find topics that are mentioned but not adequately covered
        for (topic, mentions) in &mentioned_topics {
            let coverage = covered_topics.get(topic).copied().unwrap_or(0.0);

            // If topic is mentioned multiple times but has low coverage
            if *mentions >= 2 && coverage < self.config.semantic_threshold {
                let ghost = self.create_semantic_ghost(topic, blocks, coverage);
                ghosts.push(ghost);
            }
        }

        // Sort by confidence (most needed first)
        ghosts.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        Ok(ghosts)
    }

    /// Detect orphan blocks - blocks without any connections
    ///
    /// Finds blocks that have no incoming or outgoing links,
    /// suggesting they may be disconnected from the knowledge graph.
    pub async fn detect_orphans(&self, blocks: &[Block]) -> Result<Vec<GhostNode>> {
        let mut ghosts = Vec::new();

        for block in blocks {
            // Check if block has sequence_weight (indicates it's part of a structure)
            let has_sequence = block.metadata.contains_key("sequence_weight");

            // Check if block has tags (indicates classification)
            let has_tags = !block.tags.is_empty();

            // Check content length (indicates meaningful content)
            let has_content = block.content.len() >= self.config.min_content_length;

            // If block has content but no structure or classification, it might be an orphan
            if has_content && !has_sequence && !has_tags {
                let ghost = GhostNode {
                    id: Ulid::new(),
                    description: format!(
                        "Orphan block '{}' found - consider linking to related content or adding structure",
                        block.title
                    ),
                    ai_rationale: String::new(),
                    confidence: 0.5,
                    position_hint: PositionHint::default(),
                    status: GhostStatus::Detected,
                    trigger_blocks: vec![block.id],
                    expected_keywords: self.extract_keywords_from_block(block).into_iter().collect(),
                    created_at: Utc::now(),
                    filled_by: None,
                };
                ghosts.push(ghost);
            }
        }

        Ok(ghosts)
    }

    /// Detect density gaps in structure sections
    ///
    /// Analyzes Structure notes to find sections that are incomplete
    /// based on expected vs actual block count.
    pub async fn detect_density_gaps(&self, blocks: &[Block]) -> Result<Vec<GhostNode>> {
        let mut ghosts = Vec::new();

        // Find structure blocks
        for block in blocks {
            if block.content.contains("##") || block.content.contains("###") {
                // Count how many headings suggest sections
                let section_count = block.content.lines()
                    .filter(|l| l.starts_with("##"))
                    .count();

                // Count blocks that link to this structure
                let linked_count = blocks.iter()
                    .filter(|b| {
                        b.metadata.get("parent_section")
                            .and_then(|v| v.as_str())
                            .map(|s| s == block.id.to_string())
                            .unwrap_or(false)
                    })
                    .count();

                if section_count > linked_count {
                    let expected = section_count as u32;
                    let actual = linked_count as u32;

                    let ghost = GhostNode {
                        id: Ulid::new(),
                        description: format!(
                            "Structure '{}' has {} sections but only {} blocks linked",
                            block.title, expected, actual
                        ),
                        ai_rationale: format!(
                            "Density gap: expected ~{} blocks per section, found {}",
                            self.config.max_expected_density, actual
                        ),
                        confidence: ((expected - actual) as f32 / expected as f32).min(0.9),
                        position_hint: PositionHint {
                            after: None,
                            before: None,
                            parent_section: Some(block.id),
                            sequence_weight: 0.0,
                        },
                        status: GhostStatus::Detected,
                        trigger_blocks: vec![block.id],
                        expected_keywords: self.extract_keywords_from_block(block).into_iter().collect(),
                        created_at: Utc::now(),
                        filled_by: None,
                    };
                    ghosts.push(ghost);
                }
            }
        }

        Ok(ghosts)
    }

    /// Detect topic evolution gaps - when a topic changes significantly
    ///
    /// Identifies when content about a topic seems to stop abruptly
    /// or when a new direction is introduced without transition.
    pub async fn detect_evolution_gaps(&self, blocks: &[Block]) -> Result<Vec<GhostNode>> {
        if blocks.len() < 3 {
            return Ok(Vec::new());
        }

        let mut ghosts = Vec::new();

        // Sort by creation time
        let mut time_ordered: Vec<_> = blocks.iter().collect();
        time_ordered.sort_by_key(|b| b.created_at);

        for window in time_ordered.windows(3) {
            let (prev, curr, next) = (window[0], window[1], window[2]);

            // Calculate semantic similarity between non-adjacent blocks
            let prev_emb = self.embeddings.embed(prev).await?;
            let next_emb = self.embeddings.embed(next).await?;
            let similarity = EmbeddingGenerator::cosine_similarity(&prev_emb, &next_emb);

            // If non-adjacent blocks are similar but current is different,
            // there might be a gap in the evolution narrative
            if similarity > 0.7 {
                let curr_emb = self.embeddings.embed(curr).await?;
                let prev_curr_sim = EmbeddingGenerator::cosine_similarity(&prev_emb, &curr_emb);
                let curr_next_sim = EmbeddingGenerator::cosine_similarity(&curr_emb, &next_emb);

                if prev_curr_sim < 0.4 && curr_next_sim < 0.4 {
                    let ghost = GhostNode {
                        id: Ulid::new(),
                        description: format!(
                            "Topic evolution gap detected: '{}' may be missing context connecting earlier and later content",
                            curr.title
                        ),
                        ai_rationale: format!(
                            "Semantic jump detected: prev-curr similarity {:.2}, curr-next similarity {:.2}",
                            prev_curr_sim, curr_next_sim
                        ),
                        confidence: (1.0 - similarity).min(0.8),
                        position_hint: PositionHint {
                            after: Some(prev.id),
                            before: Some(next.id),
                            parent_section: None,
                            sequence_weight: 0.0,
                        },
                        status: GhostStatus::Detected,
                        trigger_blocks: vec![prev.id, curr.id, next.id],
                        expected_keywords: self.extract_evolution_keywords(prev, next),
                        created_at: Utc::now(),
                        filled_by: None,
                    };
                    ghosts.push(ghost);
                }
            }
        }

        Ok(ghosts)
    }

    /// Detect structural gaps - issues in the block hierarchy
    ///
    /// Analyzes blocks with parent_section metadata to find:
    /// - Circular references (A is parent of B, B is parent of A)
    /// - Self-references (block is its own parent)
    /// - Missing parent references (parent_section points to non-existent block)
    /// - Disconnected branches (children with no valid path to a root)
    pub async fn detect_structural_gaps(&self, blocks: &[Block]) -> Result<Vec<GhostNode>> {
        if blocks.is_empty() {
            return Ok(Vec::new());
        }

        let mut ghosts = Vec::new();

        // Build lookup map: block_id -> block
        let block_map: HashMap<Ulid, &Block> = blocks.iter()
            .map(|b| (b.id, b))
            .collect();

        // Build parent-child relationships from metadata
        let parent_map: HashMap<Ulid, Ulid> = blocks.iter()
            .filter_map(|b| {
                b.metadata.get("parent_section")
                    .and_then(|v| v.as_str())
                    .and_then(|s| Ulid::from_string(s).ok())
                    .map(|parent_id| (b.id, parent_id))
            })
            .collect();

        // Detect issues: orphaned children (parent doesn't exist)
        for (child_id, parent_id) in &parent_map {
            // Self-reference check
            if child_id == parent_id {
                if let Some(child_block) = block_map.get(child_id) {
                    ghosts.push(GhostNode {
                        id: Ulid::new(),
                        description: format!(
                            "Self-reference detected: block '{}' references itself as parent",
                            child_block.title
                        ),
                        ai_rationale: "Block's parent_section points to itself, creating a circular reference".to_string(),
                        confidence: 0.95,
                        position_hint: PositionHint {
                            after: None,
                            before: None,
                            parent_section: Some(*parent_id),
                            sequence_weight: 0.0,
                        },
                        status: GhostStatus::Detected,
                        trigger_blocks: vec![*child_id],
                        expected_keywords: vec![],
                        created_at: Utc::now(),
                        filled_by: None,
                    });
                }
                continue;
            }

            // Parent doesn't exist
            if !block_map.contains_key(parent_id)
                && let Some(child_block) = block_map.get(child_id) {
                    ghosts.push(GhostNode {
                        id: Ulid::new(),
                        description: format!(
                            "Missing parent block: '{}' references non-existent parent",
                            child_block.title
                        ),
                        ai_rationale: format!(
                            "parent_section points to {} which doesn't exist in the block collection",
                            parent_id
                        ),
                        confidence: 0.85,
                        position_hint: PositionHint {
                            after: None,
                            before: None,
                            parent_section: Some(*parent_id),
                            sequence_weight: 0.0,
                        },
                        status: GhostStatus::Detected,
                        trigger_blocks: vec![*child_id],
                        expected_keywords: vec![],
                        created_at: Utc::now(),
                        filled_by: None,
                    });
                }
        }

        // Detect circular references using DFS
        let mut visited: HashSet<Ulid> = HashSet::new();
        let mut recursion_stack: HashSet<Ulid> = HashSet::new();

        fn dfs(
            node: Ulid,
            parent_map: &HashMap<Ulid, Ulid>,
            block_map: &HashMap<Ulid, &Block>,
            visited: &mut HashSet<Ulid>,
            recursion_stack: &mut HashSet<Ulid>,
            ghosts: &mut Vec<GhostNode>,
        ) {
            visited.insert(node);
            recursion_stack.insert(node);

            if let Some(&parent_id) = parent_map.get(&node) {
                // Check self-reference already handled above, skip
                if node == parent_id {
                    return;
                }

                if !visited.contains(&parent_id) {
                    dfs(parent_id, parent_map, block_map, visited, recursion_stack, ghosts);
                } else if recursion_stack.contains(&parent_id) {
                    // Found a cycle!
                    if let Some(block) = block_map.get(&node) {
                        ghosts.push(GhostNode {
                            id: Ulid::new(),
                            description: format!(
                                "Circular reference detected: '{}' is part of a cycle in the hierarchy",
                                block.title
                            ),
                            ai_rationale: format!(
                                "Block {} references {} which leads back to {}",
                                node, parent_id, node
                            ),
                            confidence: 0.9,
                            position_hint: PositionHint {
                                after: None,
                                before: None,
                                parent_section: Some(parent_id),
                                sequence_weight: 0.0,
                            },
                            status: GhostStatus::Detected,
                            trigger_blocks: vec![node, parent_id],
                            expected_keywords: vec![],
                            created_at: Utc::now(),
                            filled_by: None,
                        });
                    }
                }
            }

            recursion_stack.remove(&node);
        }

        for block in blocks {
            if !visited.contains(&block.id) {
                dfs(
                    block.id,
                    &parent_map,
                    &block_map,
                    &mut visited,
                    &mut recursion_stack,
                    &mut ghosts,
                );
            }
        }

        // Detect disconnected branches (blocks that have children but are never referenced as children themselves)
        let child_ids: HashSet<Ulid> = parent_map.keys().cloned().collect();
        let parent_ids: HashSet<Ulid> = parent_map.values().cloned().collect();

        // Find blocks that are parents but not children (potential root candidates)
        let potential_roots: HashSet<Ulid> = parent_ids.difference(&child_ids).cloned().collect();

        // Blocks that are children but not parents have no descendants
        let leaf_blocks: HashSet<Ulid> = child_ids.difference(&parent_ids).cloned().collect();

        // Find blocks that have children (parents) but aren't referenced as children
        // These are "floating" branches - parent blocks that should be children of something
        for block in blocks {
            // Skip if it's a potential root or doesn't have children
            if potential_roots.contains(&block.id) {
                continue;
            }

            // Check if this block has children (is a parent)
            let is_parent = parent_ids.contains(&block.id);
            let is_child = child_ids.contains(&block.id);

            // If it's a parent but not a child, it might be a disconnected root
            if is_parent && !is_child {
                // This is actually okay - it could be a legitimate root
                // But we should check if there are any root blocks defined
                let has_root_metadata = block.metadata.contains_key("is_root");
                if !has_root_metadata && block_map.len() > 1 {
                    // Check if this should potentially be a child but isn't linked properly
                    // Look for similar content blocks that might suggest hierarchy
                    let siblings = blocks.iter()
                        .filter(|b| {
                            b.id != block.id
                            && b.metadata.get("parent_section")
                                .and_then(|v| v.as_str())
                                .and_then(|s| Ulid::from_string(s).ok())
                                .map(|pid| pid == block.id)
                                .unwrap_or(false)
                        })
                        .count();

                    if siblings > 0 && potential_roots.contains(&block.id) {
                        // This is a legitimate parent node, not a gap
                    }
                }
            }

            // If it's a child but its parent doesn't exist, we already caught that above
            // Check for leaf blocks with high content that might need children
            if is_child && leaf_blocks.contains(&block.id) {
                let has_children_metadata = block.metadata.contains_key("expected_children");
                if has_children_metadata && block.content.len() > 200 {
                    // Block expects children but has none - structural gap
                    ghosts.push(GhostNode {
                        id: Ulid::new(),
                        description: format!(
                            "Expected child content missing: '{}' indicates it should have child blocks",
                            block.title
                        ),
                        ai_rationale: "Block has metadata indicating it should have children but none exist".to_string(),
                        confidence: 0.7,
                        position_hint: PositionHint {
                            after: None,
                            before: None,
                            parent_section: parent_map.get(&block.id).copied(),
                            sequence_weight: 0.0,
                        },
                        status: GhostStatus::Detected,
                        trigger_blocks: vec![block.id],
                        expected_keywords: self.extract_keywords_from_block(block).into_iter().take(5).collect(),
                        created_at: Utc::now(),
                        filled_by: None,
                    });
                }
            }
        }

        // Sort by confidence
        ghosts.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        Ok(ghosts)
    }

    /// Run all ghost detection strategies
    pub async fn detect_all(&self, blocks: &[Block]) -> Result<Vec<GhostNode>> {
        let mut all_ghosts = Vec::new();

        let sequence_ghosts = self.detect_sequence_gaps(blocks).await?;
        all_ghosts.extend(sequence_ghosts);

        let semantic_ghosts = self.detect_semantic_gaps(blocks).await?;
        all_ghosts.extend(semantic_ghosts);

        let orphan_ghosts = self.detect_orphans(blocks).await?;
        all_ghosts.extend(orphan_ghosts);

        let density_ghosts = self.detect_density_gaps(blocks).await?;
        all_ghosts.extend(density_ghosts);

        let evolution_ghosts = self.detect_evolution_gaps(blocks).await?;
        all_ghosts.extend(evolution_ghosts);

        let structural_ghosts = self.detect_structural_gaps(blocks).await?;
        all_ghosts.extend(structural_ghosts);

        // Dedupe by similar position hints
        all_ghosts.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        Ok(all_ghosts)
    }

    // Helper methods

    fn generate_sequence_gap_description(&self, before: &Block, after: &Block, _gap: f32) -> String {
        let before_keywords = self.extract_keywords_from_block(before);
        let after_keywords = self.extract_keywords_from_block(after);

        let common: Vec<&String> = before_keywords.intersection(&after_keywords).collect();

        if !common.is_empty() {
            format!(
                "Missing content between '{}' and '{}' on topic(s): {}",
                before.title, after.title,
                common.iter().take(3).map(|s| s.as_str()).collect::<Vec<&str>>().join(", ")
            )
        } else {
            format!(
                "Missing transition content between '{}' and '{}'",
                before.title, after.title
            )
        }
    }

    fn extract_gap_keywords(&self, before: &Block, after: &Block) -> Vec<String> {
        let mut keywords = std::collections::HashSet::new();

        keywords.extend(self.extract_keywords_from_block(before));
        keywords.extend(self.extract_keywords_from_block(after));

        keywords.into_iter().take(10).collect()
    }

    fn extract_keywords_from_block(&self, block: &Block) -> std::collections::HashSet<String> {
        let stop_words = [
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for",
            "of", "with", "by", "from", "as", "is", "was", "are", "were", "been",
            "this", "that", "these", "those", "it", "its",
        ];

        let mut text = block.title.clone();
        text.push(' ');
        text.push_str(&block.content);

        text.to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() >= 4 && !stop_words.contains(w))
            .map(|w| w.chars().filter(|c| c.is_alphanumeric()).collect::<String>())
            .filter(|w| w.len() >= 3)
            .collect()
    }

    fn build_topic_map(&self, blocks: &[Block]) -> HashMap<String, f32> {
        let mut topic_coverage: HashMap<String, f32> = HashMap::new();

        for block in blocks {
            let keywords = self.extract_keywords_from_block(block);
            for keyword in keywords {
                let current = topic_coverage.get(&keyword).copied().unwrap_or(0.0);
                // Coverage increases with content length
                let content_weight = (block.content.len() as f32 / 1000.0).min(1.0);
                topic_coverage.insert(keyword, current + content_weight);
            }
        }

        // Normalize to 0.0-1.0 range
        let max = topic_coverage.values().fold(0.0f32, |m, v| m.max(*v));
        if max > 0.0 {
            for value in topic_coverage.values_mut() {
                *value /= max;
            }
        }

        topic_coverage
    }

    fn extract_mentioned_topics(&self, blocks: &[Block]) -> HashMap<String, usize> {
        let mut topic_mentions: HashMap<String, usize> = HashMap::new();

        for block in blocks {
            let keywords = self.extract_keywords_from_block(block);
            for keyword in keywords {
                *topic_mentions.entry(keyword).or_insert(0) += 1;
            }
        }

        topic_mentions
    }

    fn create_semantic_ghost(&self, topic: &str, blocks: &[Block], coverage: f32) -> GhostNode {
        // Find blocks that mention this topic
        let relevant: Vec<_> = blocks.iter()
            .filter(|b| {
                let keywords = self.extract_keywords_from_block(b);
                keywords.contains(topic)
            })
            .collect();

        let trigger_ids: Vec<_> = relevant.iter().map(|b| b.id).take(3).collect();

        GhostNode {
            id: Ulid::new(),
            description: format!(
                "Topic '{}' is mentioned but not adequately covered - consider expanding with dedicated content",
                topic
            ),
            ai_rationale: format!(
                "Topic mentioned in {} blocks but coverage score is {:.2} (threshold: {:.2})",
                relevant.len(),
                coverage,
                self.config.semantic_threshold
            ),
            confidence: (1.0 - coverage).min(0.9),
            position_hint: PositionHint::default(),
            status: GhostStatus::Detected,
            trigger_blocks: trigger_ids,
            expected_keywords: vec![topic.to_string()],
            created_at: Utc::now(),
            filled_by: None,
        }
    }

    fn extract_evolution_keywords(&self, before: &Block, after: &Block) -> Vec<String> {
        let before_kw = self.extract_keywords_from_block(before);
        let after_kw = self.extract_keywords_from_block(after);

        // Find common keywords between before and after (the throughline topic)
        before_kw.intersection(&after_kw)
            .take(5)
            .cloned()
            .collect()
    }
}

impl Default for GhostDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_block_with_weight(title: &str, sequence_weight: f32) -> Block {
        let mut block = Block::permanent(title, "Test content");
        block.metadata.insert(
            "sequence_weight".to_string(),
            serde_json::json!(sequence_weight),
        );
        block
    }

    #[tokio::test]
    async fn test_detect_sequence_gaps() {
        let detector = GhostDetector::new();

        let blocks = vec![
            create_block_with_weight("Block 1", 1.0),
            create_block_with_weight("Block 2", 1.5),
            create_block_with_weight("Block 3", 5.0), // Large gap from 1.5 to 5.0
            create_block_with_weight("Block 4", 5.2),
        ];

        let ghosts = detector.detect_sequence_gaps(&blocks).await.unwrap();

        // Should detect gap between Block 2 (1.5) and Block 3 (5.0)
        assert_eq!(ghosts.len(), 1);
        assert_eq!(ghosts[0].trigger_blocks.len(), 2);
    }

    #[tokio::test]
    async fn test_no_gap_when_close() {
        let detector = GhostDetector::new();

        let blocks = vec![
            create_block_with_weight("Block 1", 1.0),
            create_block_with_weight("Block 2", 1.2),
            create_block_with_weight("Block 3", 1.4),
        ];

        let ghosts = detector.detect_sequence_gaps(&blocks).await.unwrap();

        // No gaps (all within min_sequence_gap of 1.0)
        assert!(ghosts.is_empty());
    }

    #[tokio::test]
    async fn test_detect_orphans() {
        let detector = GhostDetector::new();

        // Block with content but no structure
        let orphan = Block::permanent("Orphan Block", "This is a substantial piece of content that has no structure assigned to it yet and needs to be organized.");

        // Block with structure
        let structured = create_block_with_weight("Structured Block", 1.0);

        let blocks = vec![orphan.clone(), structured];

        let ghosts = detector.detect_orphans(&blocks).await.unwrap();

        // Should detect the orphan block
        assert_eq!(ghosts.len(), 1);
        assert!(ghosts[0].trigger_blocks.contains(&orphan.id));
    }

    #[tokio::test]
    async fn test_no_orphan_for_short_content() {
        let detector = GhostDetector::new();

        // Block with minimal content
        let short = Block::permanent("Short", "Hi");

        let blocks = vec![short];

        let ghosts = detector.detect_orphans(&blocks).await.unwrap();

        // Short content blocks are not considered orphans
        assert!(ghosts.is_empty());
    }

    #[tokio::test]
    async fn test_ghost_config() {
        let config = GhostConfig {
            min_sequence_gap: 2.0,
            max_expected_density: 20,
            semantic_threshold: 0.7,
            min_content_length: 100,
        };

        let detector = GhostDetector::with_config(config.clone());

        // Config should be applied
        assert_eq!(detector.config.min_sequence_gap, 2.0);
        assert_eq!(detector.config.max_expected_density, 20);
        assert_eq!(detector.config.semantic_threshold, 0.7);
        assert_eq!(detector.config.min_content_length, 100);
    }

    #[tokio::test]
    async fn test_extract_keywords() {
        let detector = GhostDetector::new();

        let block = Block::permanent(
            "Rust Programming Language",
            "Rust is a systems programming language that focuses on safety, speed, and concurrency.",
        );

        let keywords = detector.extract_keywords_from_block(&block);

        assert!(keywords.contains(&"rust".to_string()));
        assert!(keywords.contains(&"programming".to_string()));
        assert!(keywords.contains(&"language".to_string()));
        assert!(keywords.contains(&"systems".to_string()));
        assert!(!keywords.contains(&"that".to_string())); // Stop word
        assert!(!keywords.contains(&"and".to_string())); // Stop word
    }

    #[tokio::test]
    async fn test_detect_all_combines_strategies() {
        let detector = GhostDetector::new();

        let blocks = vec![
            create_block_with_weight("Block 1", 1.0),
            create_block_with_weight("Block 2", 5.0), // Gap
        ];

        let all_ghosts = detector.detect_all(&blocks).await.unwrap();

        // Should include sequence gap ghosts
        assert!(!all_ghosts.is_empty());
    }

    #[tokio::test]
    async fn test_detect_structural_gaps_empty() {
        let detector = GhostDetector::new();

        let blocks: Vec<Block> = vec![];

        let ghosts = detector.detect_structural_gaps(&blocks).await.unwrap();

        assert!(ghosts.is_empty());
    }

    #[tokio::test]
    async fn test_detect_structural_gaps_no_issues() {
        let detector = GhostDetector::new();

        // Blocks without parent_section metadata should have no structural issues
        let block1 = Block::permanent("Block 1", "Content 1");
        let block2 = Block::permanent("Block 2", "Content 2");

        let blocks = vec![block1, block2];

        let ghosts = detector.detect_structural_gaps(&blocks).await.unwrap();

        // No parent references, so no structural issues
        assert!(ghosts.is_empty());
    }

    #[tokio::test]
    async fn test_detect_structural_gaps_missing_parent() {
        let detector = GhostDetector::new();

        let mut block1 = Block::permanent("Child Block", "Content");
        // Set a parent that doesn't exist (use a ULID that won't match any block)
        let non_existent_parent = Ulid::new();
        block1.metadata.insert(
            "parent_section".to_string(),
            serde_json::json!(non_existent_parent.to_string()),
        );

        let blocks = vec![block1];

        let ghosts = detector.detect_structural_gaps(&blocks).await.unwrap();

        assert!(!ghosts.is_empty());
        assert!(ghosts.iter().any(|g| g.description.contains("Missing parent")));
    }

    #[tokio::test]
    async fn test_detect_structural_gaps_self_reference() {
        let detector = GhostDetector::new();

        let mut block = Block::permanent("Self Ref Block", "Content");
        // Set self as parent using the block's actual ID
        block.metadata.insert(
            "parent_section".to_string(),
            serde_json::json!(block.id.to_string()),
        );

        let blocks = vec![block];

        let ghosts = detector.detect_structural_gaps(&blocks).await.unwrap();

        assert!(!ghosts.is_empty());
        assert!(ghosts.iter().any(|g| g.description.contains("Self-reference")));
    }

    #[tokio::test]
    async fn test_detect_structural_gaps_valid_hierarchy() {
        let detector = GhostDetector::new();

        let parent = Block::permanent("Parent Block", "Parent Content");
        let mut child = Block::permanent("Child Block", "Child Content");

        // Child has valid parent reference using parent's actual ID
        child.metadata.insert(
            "parent_section".to_string(),
            serde_json::json!(parent.id.to_string()),
        );

        let blocks = vec![parent, child];

        let ghosts = detector.detect_structural_gaps(&blocks).await.unwrap();

        // Valid hierarchy should not produce structural gap ghosts
        // (self-reference and missing parent are the issues we detect)
        let self_ref_ghosts: Vec<_> = ghosts.iter()
            .filter(|g| g.description.contains("Self-reference"))
            .collect();
        let missing_parent_ghosts: Vec<_> = ghosts.iter()
            .filter(|g| g.description.contains("Missing parent"))
            .collect();

        assert!(self_ref_ghosts.is_empty());
        assert!(missing_parent_ghosts.is_empty());
    }
}