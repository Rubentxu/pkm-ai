//! Delta: changes to blocks and edges
//!
//! Represents atomic operations that modify the knowledge graph.

use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// BlockDelta: represents a change to a block
///
/// Variants cover the lifecycle of a block: creation, modification, deletion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BlockDelta {
    /// A new block was created
    Created {
        /// The block data
        block_id: Ulid,
        title: String,
        content: String,
        block_type: String,
    },
    /// An existing block was modified
    Modified {
        /// Block identifier
        block_id: Ulid,
        /// Previous title for diff
        old_title: String,
        /// New title
        new_title: String,
        /// Previous content for diff
        old_content: String,
        /// New content
        new_content: String,
    },
    /// A block was deleted
    Deleted {
        /// Block identifier
        block_id: Ulid,
        /// Title at deletion time (for undo)
        title: String,
    },
    /// A block was reorganized (changed position in spine)
    Reorganized {
        /// Block identifier
        block_id: Ulid,
        /// Previous position indicator (ULID of block before)
        old_predecessor: Option<Ulid>,
        /// New position indicator (ULID of block before)
        new_predecessor: Option<Ulid>,
    },
    /// A tag was added to a block
    TagAdded {
        /// Block identifier
        block_id: Ulid,
        /// The tag that was added
        tag: String,
    },
    /// A tag was removed from a block
    TagRemoved {
        /// Block identifier
        block_id: Ulid,
        /// The tag that was removed
        tag: String,
    },
}

impl BlockDelta {
    /// Get the block ID affected by this delta
    #[must_use]
    pub fn block_id(&self) -> Ulid {
        match self {
            Self::Created { block_id, .. } => *block_id,
            Self::Modified { block_id, .. } => *block_id,
            Self::Deleted { block_id, .. } => *block_id,
            Self::Reorganized { block_id, .. } => *block_id,
            Self::TagAdded { block_id, .. } => *block_id,
            Self::TagRemoved { block_id, .. } => *block_id,
        }
    }

    /// Check if this delta represents a creation
    #[must_use]
    pub fn is_creation(&self) -> bool {
        matches!(self, Self::Created { .. })
    }

    /// Check if this delta represents a deletion
    #[must_use]
    pub fn is_deletion(&self) -> bool {
        matches!(self, Self::Deleted { .. })
    }

    /// Check if this delta represents a modification
    #[must_use]
    pub fn is_modification(&self) -> bool {
        matches!(self, Self::Modified { .. })
    }
}

/// EdgeDelta: represents a change to an edge
///
/// Variants cover edge lifecycle: creation, modification, deletion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EdgeDelta {
    /// A new edge was created
    Created {
        /// Source block ID
        source: Ulid,
        /// Target block ID
        target: Ulid,
        /// Edge relation type
        relation: String,
    },
    /// An edge was modified
    Modified {
        /// Source block ID
        source: Ulid,
        /// Target block ID
        target: Ulid,
        /// Previous relation
        old_relation: String,
        /// New relation
        new_relation: String,
    },
    /// An edge was deleted
    Deleted {
        /// Source block ID
        source: Ulid,
        /// Target block ID
        target: Ulid,
        /// Relation at deletion time
        relation: String,
    },
}

impl EdgeDelta {
    /// Get the source block ID
    #[must_use]
    pub fn source(&self) -> Ulid {
        match self {
            Self::Created { source, .. } => *source,
            Self::Modified { source, .. } => *source,
            Self::Deleted { source, .. } => *source,
        }
    }

    /// Get the target block ID
    #[must_use]
    pub fn target(&self) -> Ulid {
        match self {
            Self::Created { target, .. } => *target,
            Self::Modified { target, .. } => *target,
            Self::Deleted { target, .. } => *target,
        }
    }

    /// Check if this delta represents a creation
    #[must_use]
    pub fn is_creation(&self) -> bool {
        matches!(self, Self::Created { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_delta_creation() {
        let block_id = Ulid::new();
        let delta = BlockDelta::Created {
            block_id,
            title: "Test".to_string(),
            content: "Content".to_string(),
            block_type: "note".to_string(),
        };

        assert_eq!(delta.block_id(), block_id);
        assert!(delta.is_creation());
        assert!(!delta.is_deletion());
    }

    #[test]
    fn test_block_delta_modification() {
        let block_id = Ulid::new();
        let delta = BlockDelta::Modified {
            block_id,
            old_title: "Old".to_string(),
            new_title: "New".to_string(),
            old_content: "Old content".to_string(),
            new_content: "New content".to_string(),
        };

        assert_eq!(delta.block_id(), block_id);
        assert!(!delta.is_creation());
        assert!(delta.is_modification());
    }

    #[test]
    fn test_block_delta_deletion() {
        let block_id = Ulid::new();
        let delta = BlockDelta::Deleted {
            block_id,
            title: "Deleted".to_string(),
        };

        assert_eq!(delta.block_id(), block_id);
        assert!(delta.is_deletion());
    }

    #[test]
    fn test_edge_delta_creation() {
        let source = Ulid::new();
        let target = Ulid::new();
        let delta = EdgeDelta::Created {
            source,
            target,
            relation: "follows".to_string(),
        };

        assert_eq!(delta.source(), source);
        assert_eq!(delta.target(), target);
        assert!(delta.is_creation());
    }

    #[test]
    fn test_edge_delta_serialization() {
        let source = Ulid::new();
        let target = Ulid::new();
        let delta = EdgeDelta::Created {
            source,
            target,
            relation: "follows".to_string(),
        };

        let json = serde_json::to_string(&delta).unwrap();
        assert!(json.contains("\"type\":\"created\""));
        assert!(json.contains("\"relation\":\"follows\""));

        let parsed: EdgeDelta = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.source(), source);
        assert_eq!(parsed.target(), target);
    }
}
