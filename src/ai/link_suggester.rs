//! Link Suggester: AI-powered link suggestions based on semantic similarity
//!
//! This module suggests links between blocks based on semantic similarity
//! of their embeddings. It analyzes both outgoing and incoming link possibilities.

use crate::ai::embeddings::{EmbeddingGenerator, DEFAULT_SIMILARITY_THRESHOLD};
use crate::models::{Block, LinkType};
use anyhow::Result;
use ulid::Ulid;
use std::collections::HashSet;

/// A link suggestion with confidence score and reasoning
#[derive(Debug, Clone)]
pub struct LinkSuggestion {
    /// Target block ID
    pub target_id: Ulid,
    /// Suggested link type
    pub link_type: LinkType,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Human-readable reason for the suggestion
    pub reason: String,
}

#[allow(dead_code)]
impl LinkSuggestion {
    /// Create a new link suggestion
    pub fn new(target_id: Ulid, link_type: LinkType, confidence: f32, reason: impl Into<String>) -> Self {
        Self {
            target_id,
            link_type,
            confidence: confidence.clamp(0.0, 1.0),
            reason: reason.into(),
        }
    }
}

/// Link suggester using semantic similarity
///
/// This struct provides link suggestion functionality based on semantic similarity.
/// It works with in-memory block data rather than requiring database access.
#[derive(Debug, Clone)]
pub struct LinkSuggester {
    embeddings: EmbeddingGenerator,
    /// Minimum similarity threshold for suggestions
    threshold: f32,
    /// Maximum number of suggestions to return
    max_suggestions: usize,
}

#[allow(dead_code)]
impl LinkSuggester {
    /// Create a new link suggester
    pub fn new() -> Self {
        Self {
            embeddings: EmbeddingGenerator::new(),
            threshold: DEFAULT_SIMILARITY_THRESHOLD,
            max_suggestions: 10,
        }
    }

    /// Create with custom configuration
    pub fn with_config(threshold: f32, max_suggestions: usize) -> Self {
        Self {
            embeddings: EmbeddingGenerator::new(),
            threshold: threshold.clamp(0.0, 1.0),
            max_suggestions: max_suggestions.max(1),
        }
    }

    /// Suggest outgoing links for a block
    ///
    /// Analyzes the block's content and finds semantically similar blocks
    /// that could be good candidates for outgoing links.
    ///
    /// # Arguments
    /// * `block` - The source block to suggest links from
    /// * `candidates` - All candidate blocks to consider for linking
    /// * `exclude_ids` - Block IDs to exclude from suggestions (e.g., already linked)
    pub async fn suggest_outgoing(
        &self,
        block: &Block,
        candidates: &[Block],
        exclude_ids: Option<&HashSet<Ulid>>,
    ) -> Result<Vec<LinkSuggestion>> {
        // Generate embedding for the source block
        let source_embedding = self.embeddings.embed(block).await?;

        let mut suggestions = Vec::new();

        for candidate in candidates {
            // Skip self and excluded blocks
            if candidate.id == block.id {
                continue;
            }
            if let Some(excluded) = exclude_ids && excluded.contains(&candidate.id) {
                continue;
            }

            let candidate_embedding = self.embeddings.embed(candidate).await?;
            let similarity = EmbeddingGenerator::cosine_similarity(&source_embedding, &candidate_embedding);

            if similarity >= self.threshold {
                let link_type = self.predict_link_type(block, candidate, similarity);
                let reason = self.generate_reason(block, candidate, similarity);

                suggestions.push(LinkSuggestion {
                    target_id: candidate.id,
                    link_type,
                    confidence: similarity,
                    reason,
                });
            }
        }

        // Sort by confidence and limit results
        suggestions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        suggestions.truncate(self.max_suggestions);

        Ok(suggestions)
    }

    /// Suggest incoming links for a block
    ///
    /// Finds blocks that would benefit from linking TO this block.
    ///
    /// # Arguments
    /// * `block` - The target block to suggest links to
    /// * `candidates` - All candidate blocks that could link to the target
    /// * `exclude_ids` - Block IDs to exclude from suggestions
    pub async fn suggest_incoming(
        &self,
        block: &Block,
        candidates: &[Block],
        exclude_ids: Option<&HashSet<Ulid>>,
    ) -> Result<Vec<LinkSuggestion>> {
        // For incoming links, we essentially do the same as outgoing
        // but the perspective is reversed - we're looking for blocks
        // that this block could be a good target for
        let target_embedding = self.embeddings.embed(block).await?;

        let mut suggestions = Vec::new();

        for candidate in candidates {
            // Skip self and excluded blocks
            if candidate.id == block.id {
                continue;
            }
            if let Some(excluded) = exclude_ids && excluded.contains(&candidate.id) {
                continue;
            }

            let candidate_embedding = self.embeddings.embed(candidate).await?;
            let similarity = EmbeddingGenerator::cosine_similarity(&target_embedding, &candidate_embedding);

            if similarity >= self.threshold {
                // For incoming suggestions, suggest the candidate should link TO this block
                let link_type = self.predict_link_type(candidate, block, similarity);
                let reason = format!(
                    "Block '{}' could benefit from linking to '{}' (similarity: {:.2})",
                    candidate.title, block.title, similarity
                );

                suggestions.push(LinkSuggestion {
                    target_id: candidate.id,
                    link_type,
                    confidence: similarity,
                    reason,
                });
            }
        }

        suggestions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        suggestions.truncate(self.max_suggestions);

        Ok(suggestions)
    }

    /// Suggest bidirectional links (both directions)
    pub async fn suggest_bidirectional(
        &self,
        block: &Block,
        candidates: &[Block],
        exclude_ids: Option<&HashSet<Ulid>>,
    ) -> Result<Vec<LinkSuggestion>> {
        let mut outgoing = self.suggest_outgoing(block, candidates, exclude_ids).await?;

        // Also check incoming as potential outgoing from other blocks' perspective
        let incoming = self.suggest_incoming(block, candidates, exclude_ids).await?;

        // Dedupe and combine
        let mut seen = HashSet::new();
        outgoing.retain(|s| seen.insert(s.target_id));

        for inc in incoming {
            if seen.insert(inc.target_id) {
                outgoing.push(inc);
            }
        }

        outgoing.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        outgoing.truncate(self.max_suggestions);

        Ok(outgoing)
    }

    /// Filter candidates to likely candidates for linking
    pub fn filter_candidates(&self, blocks: &[Block]) -> Vec<Block> {
        blocks.iter()
            .filter(|b| {
                matches!(b.block_type,
                    crate::models::BlockType::Permanent |
                    crate::models::BlockType::Literature |
                    crate::models::BlockType::Reference |
                    crate::models::BlockType::Structure
                ) && !b.content.is_empty()
            })
            .cloned()
            .collect()
    }

    /// Predict the most appropriate link type based on content similarity
    fn predict_link_type(&self, source: &Block, target: &Block, similarity: f32) -> LinkType {
        use crate::models::BlockType;

        // High similarity with extension keywords
        let source_lower = source.content.to_lowercase();
        let target_lower = target.content.to_lowercase();

        // Check for extension patterns
        if self.contains_extension_pattern(&source_lower) && self.contains_extension_pattern(&target_lower) && similarity > 0.8 {
            return LinkType::Extends;
        }

        // Check for refinement patterns
        if (self.contains_refinement_pattern(&source_lower) || self.contains_refinement_pattern(&target_lower)) && similarity >= 0.7 {
            return LinkType::Refines;
        }

        // Check for contradiction patterns
        if self.contains_contradiction_pattern(&source_lower) || self.contains_contradiction_pattern(&target_lower) {
            return LinkType::Contradicts;
        }

        // Check for question patterns
        if self.contains_question_pattern(&source_lower) && similarity > 0.5 {
            return LinkType::Questions;
        }

        // Check for support patterns
        if self.contains_support_pattern(&source_lower) && self.contains_support_pattern(&target_lower) {
            return LinkType::Supports;
        }

        // Check for reference patterns
        if self.contains_reference_pattern(&source_lower) || self.contains_reference_pattern(&target_lower) {
            return LinkType::References;
        }

        // Check block type relationships
        match (&source.block_type, &target.block_type) {
            (BlockType::Structure, _) => LinkType::SectionOf,
            (_, BlockType::Structure) => LinkType::Contains,
            (BlockType::Hub, _) => LinkType::Contains,
            (_, BlockType::Hub) => LinkType::SectionOf,
            _ => {
                // Default based on similarity threshold
                if similarity > 0.7 {
                    LinkType::Related
                } else {
                    LinkType::SimilarTo
                }
            }
        }
    }

    /// Check if text contains extension patterns
    fn contains_extension_pattern(&self, text: &str) -> bool {
        let patterns = [
            "builds on", "extends", "expands", "continues", "based on",
            "following", "after", "subsequent", "later", "next",
            "adds to", "develops", "improves", "enhances",
        ];
        patterns.iter().any(|p| text.contains(p))
    }

    /// Check if text contains refinement patterns
    fn contains_refinement_pattern(&self, text: &str) -> bool {
        let patterns = [
            "specifically", "in particular", "more precisely",
            "refines", "clarifies", "specifies", "details",
            "elaborates", "expands on", "example",
        ];
        patterns.iter().any(|p| text.contains(p))
    }

    /// Check if text contains contradiction patterns
    fn contains_contradiction_pattern(&self, text: &str) -> bool {
        let patterns = [
            "however", "but", "although", "despite", "contrary",
            "instead", "rather", "whereas", "while", "unlike",
            "different from", "opposite", "contrast",
        ];
        patterns.iter().any(|p| text.contains(p))
    }

    /// Check if text contains question patterns
    fn contains_question_pattern(&self, text: &str) -> bool {
        text.contains('?') ||
        text.contains("why") ||
        text.contains("how") ||
        text.contains("what if") ||
        text.contains("wonder")
    }

    /// Check if text contains support patterns
    fn contains_support_pattern(&self, text: &str) -> bool {
        let patterns = [
            "evidence", "study", "research", "shows", "demonstrates",
            "proves", "confirms", "supports", "according to",
            "data", "results", "findings",
        ];
        patterns.iter().any(|p| text.contains(p))
    }

    /// Check if text contains reference patterns
    fn contains_reference_pattern(&self, text: &str) -> bool {
        let patterns = [
            "according to", "cited", "reference", "see also",
            "related to", "mentioned in", "as described in",
            "quoted", "stated in", "from",
        ];
        patterns.iter().any(|p| text.contains(p))
    }

    /// Generate a human-readable reason for the suggestion
    fn generate_reason(&self, source: &Block, target: &Block, similarity: f32) -> String {
        let mut reasons = Vec::new();

        // Title similarity
        if !source.title.is_empty() && !target.title.is_empty() {
            let source_lower = source.title.to_lowercase();
            let target_lower = target.title.to_lowercase();
            let source_words: Vec<&str> = source_lower.split_whitespace().collect();
            let target_words: std::collections::HashSet<&str> = target_lower.split_whitespace().collect();

            // Find intersection but preserve original case from source
            let intersection: Vec<&str> = source_words.iter()
                .filter(|w| target_words.contains(*w))
                .copied()
                .collect();

            if !intersection.is_empty() {
                // Map back to original case from source title
                let source_originals: Vec<&str> = source.title.split_whitespace().collect();
                let shared_with_case: Vec<String> = intersection.iter()
                    .map(|lower_word| {
                        source_originals.iter()
                            .find(|orig| orig.to_lowercase() == **lower_word)
                            .map(|s| (*s).to_string())
                            .unwrap_or_else(|| (*lower_word).to_string())
                    })
                    .collect();
                reasons.push(format!("Shared title words: {}", shared_with_case.join(", ")));
            }
        }

        // Tag overlap
        if !source.tags.is_empty() && !target.tags.is_empty() {
            let source_tags: std::collections::HashSet<_> =
                source.tags.iter().map(|t| t.to_lowercase()).collect();
            let target_tags: std::collections::HashSet<_> =
                target.tags.iter().map(|t| t.to_lowercase()).collect();

            let overlap: Vec<String> = source_tags.intersection(&target_tags).map(|s| (*s).to_string()).collect();
            if !overlap.is_empty() {
                reasons.push(format!("Shared tags: {}", overlap.join(", ")));
            }
        }

        // Keyword extraction
        let source_keywords = self.extract_keywords(&source.content);
        let target_keywords = self.extract_keywords(&target.content);
        let common_keywords: Vec<String> = source_keywords.intersection(&target_keywords).map(|s| (*s).to_string()).collect();

        if !common_keywords.is_empty() {
            let keyword_slice: Vec<&str> = common_keywords.iter().take(5).map(|s| s.as_str()).collect();
            reasons.push(format!("Common concepts: {}", keyword_slice.join(", ")));
        }

        // Format the reason
        if reasons.is_empty() {
            format!("Semantic similarity: {:.0}%", similarity * 100.0)
        } else {
            format!("{:.0}% similar - {}", similarity * 100.0, reasons.join("; "))
        }
    }

    /// Extract significant keywords from text
    fn extract_keywords(&self, text: &str) -> std::collections::HashSet<String> {
        let stop_words = [
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for",
            "of", "with", "by", "from", "as", "is", "was", "are", "were", "been",
            "be", "have", "has", "had", "do", "does", "did", "will", "would", "could",
            "should", "may", "might", "must", "can", "this", "that", "these", "those",
            "i", "you", "he", "she", "it", "we", "they", "what", "which", "who",
            "when", "where", "why", "how", "all", "each", "every", "both", "few",
            "more", "most", "other", "some", "such", "no", "nor", "not", "only",
            "own", "same", "so", "than", "too", "very", "just", "also", "now",
        ];

        text.to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() > 4 && !stop_words.contains(w))
            .map(|w| w.chars().filter(|c| c.is_alphanumeric()).collect::<String>())
            .filter(|w| w.len() > 3)
            .collect()
    }
}

impl Default for LinkSuggester {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_block(title: &str, content: &str, block_type: crate::models::BlockType) -> Block {
        let mut block = Block::new(block_type, title);
        block.content = content.to_string();
        block
    }

    #[test]
    fn test_link_suggestion_creation() {
        let suggestion = LinkSuggestion::new(
            Ulid::new(),
            LinkType::Extends,
            0.85,
            "Test reason",
        );

        assert_eq!(suggestion.confidence, 0.85);
        assert_eq!(suggestion.link_type, LinkType::Extends);
        assert_eq!(suggestion.reason, "Test reason");
    }

    #[test]
    fn test_link_suggestion_confidence_clamping() {
        // Test that confidence is clamped to 0.0-1.0
        let suggestion_high = LinkSuggestion::new(Ulid::new(), LinkType::Related, 1.5, "High");
        assert_eq!(suggestion_high.confidence, 1.0);

        let suggestion_low = LinkSuggestion::new(Ulid::new(), LinkType::Related, -0.5, "Low");
        assert_eq!(suggestion_low.confidence, 0.0);
    }

    #[test]
    fn test_predict_link_type() {
        let suggester = LinkSuggester::new();

        // Test extension pattern
        let source = create_test_block(
            "Base Concept",
            "This builds on previous work",
            crate::models::BlockType::Permanent,
        );
        let target = create_test_block(
            "Extended Concept",
            "This extends the base concept",
            crate::models::BlockType::Permanent,
        );

        let link_type = suggester.predict_link_type(&source, &target, 0.9);
        assert_eq!(link_type, LinkType::Extends);

        // Test refinement pattern
        let source = create_test_block(
            "Main Idea",
            "The concept is important for various reasons",
            crate::models::BlockType::Permanent,
        );
        let target = create_test_block(
            "Refined Idea",
            "Specifically, this elaborates on the details",
            crate::models::BlockType::Permanent,
        );

        let link_type = suggester.predict_link_type(&source, &target, 0.7);
        assert_eq!(link_type, LinkType::Refines);

        // Test contradiction pattern
        let source = create_test_block(
            "Statement A",
            "However, this contradicts earlier claims",
            crate::models::BlockType::Permanent,
        );
        let target = create_test_block(
            "Statement B",
            "The traditional view suggests otherwise",
            crate::models::BlockType::Permanent,
        );

        let link_type = suggester.predict_link_type(&source, &target, 0.6);
        assert_eq!(link_type, LinkType::Contradicts);
    }

    #[test]
    fn test_extract_keywords() {
        let suggester = LinkSuggester::new();

        let text = "The quick brown fox jumps over the lazy dog in the forest";
        let keywords = suggester.extract_keywords(text);

        assert!(keywords.contains(&"quick".to_string()));
        assert!(keywords.contains(&"brown".to_string()));
        assert!(keywords.contains(&"forest".to_string()));
        assert!(!keywords.contains(&"the".to_string())); // Stop word
        assert!(!keywords.contains(&"over".to_string())); // Stop word
    }

    #[test]
    fn test_generate_reason() {
        let suggester = LinkSuggester::new();

        let source = create_test_block(
            "Rust Programming",
            "Rust is a systems programming language",
            crate::models::BlockType::Permanent,
        );
        let target = create_test_block(
            "Rust Performance",
            "Rust offers excellent performance and safety",
            crate::models::BlockType::Permanent,
        );

        let reason = suggester.generate_reason(&source, &target, 0.75);

        assert!(reason.contains("Rust")); // From title
        assert!(reason.contains("75%")); // From similarity
    }

    #[test]
    fn test_link_suggester_config() {
        let suggester = LinkSuggester::with_config(0.8, 5);

        // Threshold should be set
        assert_eq!(suggester.threshold, 0.8);
        assert_eq!(suggester.max_suggestions, 5);
    }

    #[tokio::test]
    async fn test_suggest_outgoing() {
        let suggester = LinkSuggester::new();

        let blocks = vec![
            create_test_block("Rust", "Rust is a programming language", crate::models::BlockType::Permanent),
            create_test_block("Python", "Python is also a programming language", crate::models::BlockType::Permanent),
            create_test_block("Java", "Java is a compiled language", crate::models::BlockType::Permanent),
        ];

        let suggestions = suggester.suggest_outgoing(&blocks[0], &blocks[1..], None).await.unwrap();

        // Should find Python as similar (both programming languages)
        assert!(!suggestions.is_empty());
    }

    #[tokio::test]
    async fn test_exclude_ids() {
        let suggester = LinkSuggester::new();

        let blocks = vec![
            create_test_block("Rust", "Rust is a programming language", crate::models::BlockType::Permanent),
            create_test_block("Python", "Python is also a programming language", crate::models::BlockType::Permanent),
        ];

        let exclude = vec![blocks[1].id].into_iter().collect::<HashSet<_>>();

        let suggestions = suggester.suggest_outgoing(&blocks[0], &[blocks[1].clone()], Some(&exclude)).await.unwrap();

        // Should have no suggestions since Python is excluded
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_filter_candidates() {
        let suggester = LinkSuggester::new();

        let blocks = vec![
            create_test_block("Permanent", "Content", crate::models::BlockType::Permanent),
            create_test_block("Fleeting", "Content", crate::models::BlockType::Fleeting),
            create_test_block("Empty", "", crate::models::BlockType::Permanent),
            create_test_block("Structure", "Content", crate::models::BlockType::Structure),
        ];

        let filtered = suggester.filter_candidates(&blocks);

        // Should only include Permanent and Structure (not Fleeting or Empty)
        assert_eq!(filtered.len(), 2);
    }
}