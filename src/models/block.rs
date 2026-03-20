//! Block Model: The atomic unit of knowledge in Nexus-Grafo
//!
//! Every piece of content is a Block with a ULID identifier.
//! Blocks can be: Fleeting, Literature, Permanent, Structure, Hub, Task, Reference, Outline, Ghost

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ulid::Ulid;

/// Block types in the Zettelkasten system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BlockType {
    /// Quick capture, raw ideas
    Fleeting,
    /// Notes from sources
    Literature,
    /// Crystallized knowledge (atomic)
    Permanent,
    /// Index/MOC (Map of Content)
    Structure,
    /// Entry point to domain
    Hub,
    /// Actionable items
    Task,
    /// External references
    Reference,
    /// Skeletons for documents (TOC structure)
    Outline,
    /// Predictive placeholder
    Ghost,
}

/// A Block in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// ULID: Sortable chronologically + unique (stored as 'ulid' in DB)
    #[serde(rename = "ulid")]
    pub id: Ulid,

    /// Block type
    pub block_type: BlockType,

    /// Title (for Structure/Hub/Permanent)
    pub title: String,

    /// Content (Markdown)
    pub content: String,

    /// Tags for classification
    pub tags: Vec<String>,

    /// Metadata (flexible key-value)
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last modification
    pub updated_at: DateTime<Utc>,

    /// Version for versioning support
    #[serde(default)]
    pub version: u32,

    /// AI confidence score (for AI-generated blocks)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_confidence: Option<f32>,

    /// Semantic centroid (for smart sections)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_centroid: Option<Vec<f32>>,
}

#[allow(dead_code)]
impl Block {
    /// Create a new block with auto-generated ULID
    pub fn new(block_type: BlockType, title: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new(),
            block_type,
            title: title.into(),
            content: String::new(),
            tags: Vec::new(),
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
            version: 1,
            ai_confidence: None,
            semantic_centroid: None,
        }
    }

    /// Create a fleeting note (quick capture)
    pub fn fleeting(content: impl Into<String>) -> Self {
        let mut block = Self::new(BlockType::Fleeting, "Fleeting Note");
        block.content = content.into();
        block
    }

    /// Create a permanent note (crystallized knowledge)
    pub fn permanent(title: impl Into<String>, content: impl Into<String>) -> Self {
        let mut block = Self::new(BlockType::Permanent, title);
        block.content = content.into();
        block
    }

    /// Create a structure note (index/MOC)
    pub fn structure(title: impl Into<String>) -> Self {
        Self::new(BlockType::Structure, title)
    }

    /// Create an outline (document skeleton)
    pub fn outline(title: impl Into<String>) -> Self {
        Self::new(BlockType::Outline, title)
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Set content
    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self.updated_at = Utc::now();
        self
    }

    /// Set AI confidence
    pub fn with_ai_confidence(mut self, confidence: f32) -> Self {
        self.ai_confidence = Some(confidence.clamp(0.0, 1.0));
        self
    }

    /// Get the ULID as string (sortable)
    pub fn id_str(&self) -> String {
        self.id.to_string()
    }

    /// Get creation timestamp from ULID
    pub fn ulid_timestamp(&self) -> DateTime<Utc> {
        DateTime::<Utc>::from(self.id.datetime())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_creation() {
        let block = Block::permanent("Test", "Content");
        assert_eq!(block.block_type, BlockType::Permanent);
        assert_eq!(block.title, "Test");
        assert_eq!(block.content, "Content");
    }

    #[test]
    fn test_ulid_sortable() {
        let b1 = Block::fleeting("First");
        std::thread::sleep(std::time::Duration::from_millis(10));
        let b2 = Block::fleeting("Second");

        assert!(b1.id < b2.id, "ULIDs should be chronologically sortable");
    }

    #[test]
    fn test_block_builder() {
        let block = Block::permanent("Test", "Content")
            .with_tag("rust")
            .with_tag("pkm")
            .with_ai_confidence(0.95);

        assert_eq!(block.tags, vec!["rust", "pkm"]);
        assert_eq!(block.ai_confidence, Some(0.95));
    }
}
