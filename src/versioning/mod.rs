//! Versioning layer with Git-like API for knowledge graph
//!
//! This module provides version control semantics for the PKM system:
//! - Commits: snapshots of the knowledge structure
//! - Views: branches and tags pointing to commits
//! - WorkingSet: staging area for atomic commits
//! - Delta: changes to blocks and edges
//! - Sync: distributed synchronization with remotes
//! - Repo: high-level repository interface

mod commit;
mod delta;
mod view;
mod working_set;
mod agent;
pub mod repository;
pub mod sync;
pub mod repo;

pub use commit::{Commit, CommitId};
pub use delta::{BlockDelta, EdgeDelta};
pub use view::{View, ViewName};
pub use working_set::{WorkingSet, WorkingSetId};
pub use agent::AgentId;
pub use sync::{Remote, RemoteConfig, SyncEngine, Packfile, FastForwardStatus};
pub use repo::{VersionRepo, VersionError, MergeStrategy, MergeResult, BranchInfo, TagInfo, ConflictResolution};

use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// StructureSnapshot: immutable snapshot of the knowledge structure at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureSnapshot {
    /// Unique identifier for this structure snapshot
    pub id: Ulid,
    /// Ordered block IDs following the FOLLOWZETTEL algorithm
    pub block_order: Vec<Ulid>,
    /// Edge relationships at snapshot time
    pub edges: Vec<EdgeSnapshot>,
}

/// Minimal edge representation for snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeSnapshot {
    pub source: Ulid,
    pub target: Ulid,
    pub relation: String,
}

/// Information about a conflict detected during merge
#[derive(Debug, Clone)]
pub struct ConflictInfo {
    /// Type of conflict
    pub conflict_type: ConflictType,
    /// Affected block ID
    pub block_id: Option<Ulid>,
    /// Description of the conflict
    pub description: String,
}

/// Types of possible conflicts
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictType {
    /// A block was modified in both branches
    BlockModifiedBoth,
    /// A block was deleted in one branch but modified in another
    BlockDeletedVsModified,
    /// An edge was modified in conflicting ways
    EdgeConflict,
    /// A structure was modified in both branches
    StructureConflict,
}

/// Repository reference to a commit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommitRef(Ulid);

impl CommitRef {
    pub const fn new(id: Ulid) -> Self {
        Self(id)
    }

    pub const fn null() -> Self {
        Self(Ulid::from_bytes([0u8; 16]))
    }

    #[must_use]
    pub const fn id(self) -> Ulid {
        self.0
    }
}

impl Default for CommitRef {
    fn default() -> Self {
        Self::null()
    }
}

impl std::fmt::Display for CommitRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_ref_default_is_null() {
        let null_ref = CommitRef::null();
        assert_eq!(null_ref.id().to_string(), "00000000000000000000000000");
    }

    #[test]
    fn structure_snapshot_has_id() {
        let snapshot = StructureSnapshot {
            id: Ulid::new(),
            block_order: Vec::new(),
            edges: Vec::new(),
        };
        assert!(!snapshot.id.to_string().is_empty());
    }
}
