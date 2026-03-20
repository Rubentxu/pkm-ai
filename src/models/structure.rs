//! Structure Model for Git-like API
//!
//! Represents a named collection of blocks with deterministic ordering via spine_order.
//! The Structure model provides the Git-like structure for organizing knowledge
//! following the Folgezettel digital principles.

use crate::models::FractionalIndex;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ulid::Ulid;

/// A named structure that organizes blocks with deterministic ordering.
///
/// Structure provides:
/// - Name-based identification
/// - Root blocks as entry points
/// - Block tree for hierarchical relationships
/// - Spine order for deterministic sequential traversal using FractionalIndex
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Structure {
    /// Unique identifier
    pub id: Ulid,
    /// Human-readable name
    pub name: String,
    /// Entry point block IDs
    pub root_blocks: Vec<Ulid>,
    /// Hierarchical block relationships: parent -> children
    pub block_tree: HashMap<Ulid, Vec<Ulid>>,
    /// Sequential ordering with fractional indexing for deterministic traversal
    pub spine_order: Vec<(Ulid, FractionalIndex)>,
    /// Arbitrary properties for extensibility
    pub properties: HashMap<String, serde_json::Value>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last modification timestamp
    pub updated_at: DateTime<Utc>,
}

#[allow(dead_code)]
impl Structure {
    /// Create a new empty structure with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new(),
            name: name.into(),
            root_blocks: Vec::new(),
            block_tree: HashMap::new(),
            spine_order: Vec::new(),
            properties: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a structure with a single root block.
    pub fn with_root(name: impl Into<String>, root_id: Ulid) -> Self {
        let mut structure = Self::new(name);
        structure.root_blocks.push(root_id);
        structure
    }

    /// Add a block to the structure at the end of the spine.
    pub fn push_block(&mut self, block_id: Ulid) {
        let position = if let Some(last) = self.spine_order.last() {
            FractionalIndex::after_last(&last.1)
        } else {
            FractionalIndex::first()
        };
        self.spine_order.push((block_id, position));
        self.updated_at = Utc::now();
    }

    /// Insert a block between two existing positions in the spine.
    pub fn insert_block_between(
        &mut self,
        block_id: Ulid,
        before: &FractionalIndex,
        after: &FractionalIndex,
    ) {
        let position = FractionalIndex::between(before, after);
        self.spine_order.push((block_id, position));
        self.sort_spine();
        self.updated_at = Utc::now();
    }

    /// Sort the spine to maintain deterministic ordering.
    pub fn sort_spine(&mut self) {
        self.spine_order.sort_by(|a, b| a.1.cmp(&b.1));
    }

    /// Get the position of a block in the spine.
    pub fn position_of(&self, block_id: &Ulid) -> Option<&FractionalIndex> {
        self.spine_order
            .iter()
            .find(|(id, _)| id == block_id)
            .map(|(_, pos)| pos)
    }

    /// Get all blocks in spine order.
    pub fn spine_blocks(&self) -> Vec<Ulid> {
        self.spine_order.iter().map(|(id, _)| *id).collect()
    }

    /// Add a child relationship to the block tree.
    pub fn add_child(&mut self, parent_id: Ulid, child_id: Ulid) {
        self.block_tree
            .entry(parent_id)
            .or_default()
            .push(child_id);
        self.updated_at = Utc::now();
    }

    /// Get children of a block.
    pub fn children_of(&self, parent_id: &Ulid) -> Vec<Ulid> {
        self.block_tree.get(parent_id).cloned().unwrap_or_default()
    }

    /// Get the root blocks.
    pub fn roots(&self) -> &[Ulid] {
        &self.root_blocks
    }

    /// Check if a block exists in this structure.
    pub fn contains_block(&self, block_id: &Ulid) -> bool {
        self.spine_order.iter().any(|(id, _)| id == block_id)
    }

    /// Get the number of blocks in the structure.
    pub fn len(&self) -> usize {
        self.spine_order.len()
    }

    /// Check if the structure is empty.
    pub fn is_empty(&self) -> bool {
        self.spine_order.is_empty()
    }

    /// Set a property.
    pub fn set_property(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.properties.insert(key.into(), value);
        self.updated_at = Utc::now();
    }

    /// Get a property.
    pub fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.properties.get(key)
    }
}

impl Default for Structure {
    fn default() -> Self {
        Self::new("Untitled")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_structure() {
        let structure = Structure::new("Test Structure");
        assert_eq!(structure.name, "Test Structure");
        assert!(structure.is_empty());
    }

    #[test]
    fn test_push_block() {
        let mut structure = Structure::new("Test");
        let block_id = Ulid::new();

        structure.push_block(block_id);

        assert_eq!(structure.len(), 1);
        assert!(structure.contains_block(&block_id));
    }

    #[test]
    fn test_spine_ordering() {
        let mut structure = Structure::new("Test");
        let block1 = Ulid::new();
        let block2 = Ulid::new();
        let block3 = Ulid::new();

        structure.push_block(block1);
        structure.push_block(block2);
        structure.push_block(block3);

        // Verify order is maintained
        let spine = structure.spine_blocks();
        assert_eq!(spine, vec![block1, block2, block3]);
    }

    #[test]
    fn test_insert_between() {
        let mut structure = Structure::new("Test");
        let block1 = Ulid::new();
        let block2 = Ulid::new();
        let block_between = Ulid::new();

        structure.push_block(block1);
        structure.push_block(block2);

        // Insert between block1 and block2
        // Clone positions first to avoid borrow conflict
        let (pos1, pos2) = {
            let p1 = structure.position_of(&block1).cloned();
            let p2 = structure.position_of(&block2).cloned();
            match (p1, p2) {
                (Some(a), Some(b)) => (a, b),
                _ => return,
            }
        };
        structure.insert_block_between(block_between, &pos1, &pos2);

        let spine = structure.spine_blocks();
        assert_eq!(spine.len(), 3);
        assert!(spine.contains(&block_between));
    }

    #[test]
    fn test_children() {
        let mut structure = Structure::new("Test");
        let parent = Ulid::new();
        let child = Ulid::new();

        structure.add_child(parent, child);

        assert_eq!(structure.children_of(&parent), vec![child]);
        assert!(structure.children_of(&Ulid::new()).is_empty());
    }

    #[test]
    fn test_properties() {
        let mut structure = Structure::new("Test");
        structure.set_property("description", serde_json::json!("A test structure"));

        assert_eq!(
            structure.get_property("description"),
            Some(&serde_json::json!("A test structure"))
        );
    }
}
