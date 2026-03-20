//! Synchronization layer for distributed knowledge graph
//!
//! This module provides Git-like sync semantics:
//! - Remote: represents a remote repository connection
//! - Packfile: differential sync format for efficient transfer
//! - Fast-forward detection using commit ancestry

mod remote;
mod packfile;
mod sync_engine;

pub use remote::{Remote, RemoteError, RemoteConnection, RemoteRef, FetchResult, PushResult, RemoteManager};
pub use packfile::{Packfile, PackfileEntry, PackfileError, PackfileObjectType};
pub use sync_engine::{SyncEngine, SyncResult, SyncError, FastForwardStatus, SyncRefUpdate, MergeStrategy};

use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// Commit reference with ancestry information for sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteCommitRef {
    pub commit_id: Ulid,
    pub parent_ids: Vec<Ulid>,
    pub is_ancestor: bool,
}

impl RemoteCommitRef {
    pub fn new(commit_id: Ulid, parent_ids: Vec<Ulid>) -> Self {
        Self {
            commit_id,
            parent_ids,
            is_ancestor: false,
        }
    }

    /// Check if this commit is an ancestor of another commit
    pub fn is_ancestor_of(&self, other: &RemoteCommitRef, ancestry: &[RemoteCommitRef]) -> bool {
        if self.commit_id == other.commit_id {
            return true;
        }

        let mut to_visit = other.parent_ids.clone();
        let mut visited = std::collections::HashSet::new();

        while let Some(parent_id) = to_visit.pop() {
            if visited.contains(&parent_id) {
                continue;
            }
            visited.insert(parent_id);

            if parent_id == self.commit_id {
                return true;
            }

            if let Some(parent) = ancestry.iter().find(|r| r.commit_id == parent_id) {
                to_visit.extend(parent.parent_ids.clone());
            }
        }

        false
    }
}

/// Remote repository configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConfig {
    pub name: String,
    pub path: std::path::PathBuf,
    pub fetch_url: Option<String>,
    pub push_url: Option<String>,
}

impl RemoteConfig {
    pub fn new(name: impl Into<String>, path: impl AsRef<std::path::Path>) -> Self {
        Self {
            name: name.into(),
            path: path.as_ref().to_path_buf(),
            fetch_url: None,
            push_url: None,
        }
    }

    pub fn with_fetch_url(mut self, url: impl Into<String>) -> Self {
        self.fetch_url = Some(url.into());
        self
    }

    pub fn with_push_url(mut self, url: impl Into<String>) -> Self {
        self.push_url = Some(url.into());
        self
    }
}

/// Shallow clone tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShallowInfo {
    /// Commits that are shallow (don't have full ancestry)
    pub shallow_commits: Vec<Ulid>,
    /// Depth limit for shallow clones
    pub depth: Option<usize>,
}

impl ShallowInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_depth(mut self, depth: usize) -> Self {
        self.depth = Some(depth);
        self
    }

    pub fn add_shallow_commit(&mut self, commit_id: Ulid) {
        if !self.shallow_commits.contains(&commit_id) {
            self.shallow_commits.push(commit_id);
        }
    }

    pub fn is_shallow(&self, commit_id: &Ulid) -> bool {
        self.shallow_commits.contains(commit_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_commit_ref_ancestry() {
        // Create a simple commit chain: A <- B <- C
        let commit_a = RemoteCommitRef::new(Ulid::from_string("00000000000000000000000001").unwrap(), vec![]);
        let commit_b = RemoteCommitRef::new(Ulid::from_string("00000000000000000000000002").unwrap(), vec![commit_a.commit_id]);
        let commit_c = RemoteCommitRef::new(Ulid::from_string("00000000000000000000000003").unwrap(), vec![commit_b.commit_id]);

        let ancestry = vec![commit_a.clone(), commit_b.clone(), commit_c.clone()];

        // C should be ancestor of itself
        assert!(commit_c.is_ancestor_of(&commit_c, &ancestry));

        // A should be ancestor of C
        assert!(commit_a.is_ancestor_of(&commit_c, &ancestry));

        // C should not be ancestor of A
        assert!(!commit_c.is_ancestor_of(&commit_a, &ancestry));
    }

    #[test]
    fn test_shallow_info() {
        let mut shallow = ShallowInfo::new();
        let commit_id = Ulid::new();

        assert!(!shallow.is_shallow(&commit_id));

        shallow.add_shallow_commit(commit_id);
        assert!(shallow.is_shallow(&commit_id));

        let other_id = Ulid::new();
        assert!(!shallow.is_shallow(&other_id));
    }
}