//! Commit: immutable snapshot of the knowledge structure
//!
//! A commit represents a point-in-time snapshot of the entire knowledge graph
//! including the ordered structure of blocks and their relationships.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::{AgentId, StructureSnapshot};

/// CommitId: newtype wrapper around Ulid for commit identification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommitId(Ulid);

impl CommitId {
    #[must_use]
    pub const fn new(id: Ulid) -> Self {
        Self(id)
    }

    #[must_use]
    pub const fn as_ulid(self) -> Ulid {
        self.0
    }
}

impl Default for CommitId {
    fn default() -> Self {
        Self(Ulid::new())
    }
}

impl std::fmt::Display for CommitId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Ulid> for CommitId {
    fn from(id: Ulid) -> Self {
        Self(id)
    }
}

impl From<CommitId> for Ulid {
    fn from(id: CommitId) -> Self {
        id.0
    }
}

/// Commit: an immutable snapshot of the knowledge structure
///
/// Contains a complete snapshot of the block order and edges at a specific point,
/// along with metadata about the author and commit message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    /// Unique commit identifier
    pub id: CommitId,
    /// Snapshot of the structure at this point
    pub structure_snapshot: StructureSnapshot,
    /// Parent commit IDs (empty for initial commit)
    pub parents: Vec<CommitId>,
    /// Author of the commit
    pub author: AgentId,
    /// Commit message describing the changes
    pub message: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Blocks added in this commit (for fast-forward sync)
    #[serde(default)]
    pub blocks_added: Vec<Ulid>,
    /// Blocks removed in this commit (for fast-forward sync)
    #[serde(default)]
    pub blocks_removed: Vec<Ulid>,
    /// Blocks modified in this commit (for fast-forward sync)
    #[serde(default)]
    pub blocks_modified: Vec<Ulid>,
}

impl Commit {
    /// Create a new commit
    ///
    /// # Arguments
    /// * `structure` - The structure snapshot to commit
    /// * `author` - The agent creating the commit
    /// * `message` - Description of the changes
    /// * `parents` - Parent commit IDs (empty for initial commit)
    /// * `blocks_added` - Blocks added in this commit
    /// * `blocks_removed` - Blocks removed in this commit
    /// * `blocks_modified` - Blocks modified in this commit
    #[must_use]
    pub fn new(
        structure: StructureSnapshot,
        author: AgentId,
        message: String,
        parents: Vec<CommitId>,
        blocks_added: Vec<Ulid>,
        blocks_removed: Vec<Ulid>,
        blocks_modified: Vec<Ulid>,
    ) -> Self {
        Self {
            id: CommitId::default(),
            structure_snapshot: structure,
            parents,
            author,
            message,
            created_at: Utc::now(),
            blocks_added,
            blocks_removed,
            blocks_modified,
        }
    }

    /// Create an initial commit with no parents
    #[must_use]
    pub fn initial(structure: StructureSnapshot, author: AgentId, message: String) -> Self {
        Self::new(structure, author, message, Vec::new(), Vec::new(), Vec::new(), Vec::new())
    }

    /// Check if this is an initial commit (no parents)
    #[must_use]
    pub fn is_initial(&self) -> bool {
        self.parents.is_empty()
    }

    /// Get the number of parents
    #[must_use]
    pub fn parent_count(&self) -> usize {
        self.parents.len()
    }

    /// Check if this commit is an ancestor of another commit
    ///
    /// This is a basic check - in a real implementation this would traverse
    /// the commit graph to determine ancestry.
    #[must_use]
    pub fn is_ancestor_of(&self, _other: &Commit) -> bool {
        // TODO: Implement proper ancestry check by traversing parent graph
        false
    }

    /// Get the commit ID as a string
    #[must_use]
    pub fn id_str(&self) -> String {
        self.id.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_snapshot() -> StructureSnapshot {
        StructureSnapshot {
            id: Ulid::new(),
            block_order: vec![Ulid::new(), Ulid::new()],
            edges: Vec::new(),
        }
    }

    #[test]
    fn test_commit_creation() {
        let structure = create_test_snapshot();
        let author = AgentId::new("test-author");
        let commit = Commit::new(
            structure.clone(),
            author.clone(),
            "Test commit".to_string(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );

        assert_eq!(commit.author.as_str(), "test-author");
        assert_eq!(commit.message, "Test commit");
        assert!(commit.is_initial());
        assert!(commit.parents.is_empty());
        assert!(commit.blocks_added.is_empty());
        assert!(commit.blocks_removed.is_empty());
        assert!(commit.blocks_modified.is_empty());
        assert_eq!(commit.structure_snapshot.id, structure.id);
    }

    #[test]
    fn test_commit_with_parents() {
        let structure = create_test_snapshot();
        let author = AgentId::new("test-author");
        let parent_id = CommitId::new(Ulid::new());

        let commit = Commit::new(
            structure,
            author,
            "Child commit".to_string(),
            vec![parent_id],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );

        assert!(!commit.is_initial());
        assert_eq!(commit.parent_count(), 1);
        assert_eq!(commit.parents[0], parent_id);
    }

    #[test]
    fn test_commit_id() {
        let structure = create_test_snapshot();
        let author = AgentId::new("test");
        let commit = Commit::new(
            structure,
            author,
            "msg".to_string(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );

        assert!(!commit.id_str().is_empty());
        assert_eq!(commit.id_str(), commit.id.to_string());
    }

    #[test]
    fn test_commit_id_newtype() {
        let id = CommitId::new(Ulid::new());
        let ulid: Ulid = id.into();
        let back_to_id: CommitId = ulid.into();
        assert_eq!(id, back_to_id);
    }

    #[test]
    fn test_initial_commit_has_no_parents() {
        let structure = create_test_snapshot();
        let commit = Commit::initial(structure, AgentId::new("author"), "Initial".to_string());

        assert!(commit.is_initial());
        assert!(commit.parents.is_empty());
    }

    #[test]
    fn test_commit_with_blocks_tracking() {
        let structure = create_test_snapshot();
        let author = AgentId::new("test-author");
        let block_added = Ulid::new();
        let block_modified = Ulid::new();
        let block_removed = Ulid::new();

        let commit = Commit::new(
            structure,
            author,
            "Commit with changes".to_string(),
            Vec::new(),
            vec![block_added],
            vec![block_removed],
            vec![block_modified],
        );

        assert_eq!(commit.blocks_added.len(), 1);
        assert!(commit.blocks_added.contains(&block_added));
        assert_eq!(commit.blocks_removed.len(), 1);
        assert!(commit.blocks_removed.contains(&block_removed));
        assert_eq!(commit.blocks_modified.len(), 1);
        assert!(commit.blocks_modified.contains(&block_modified));
    }

    #[test]
    fn test_commit_serialization_with_blocks() {
        let structure = create_test_snapshot();
        let author = AgentId::new("test");
        let block_added = Ulid::new();

        let commit = Commit::new(
            structure.clone(),
            author,
            "Serialized commit".to_string(),
            Vec::new(),
            vec![block_added],
            Vec::new(),
            Vec::new(),
        );

        let json = serde_json::to_string(&commit).unwrap();
        assert!(json.contains("blocks_added"));
        assert!(json.contains(&block_added.to_string()));

        let parsed: Commit = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.blocks_added.len(), 1);
        assert_eq!(parsed.blocks_added[0], block_added);
    }
}
