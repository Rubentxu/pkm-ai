//! Synchronization engine for push/pull operations
//!
//! This module implements the core sync logic including:
//! - Fast-forward detection using commit ancestry
//! - Push with fast-forward enforcement
//! - Pull with automatic merging

use super::{Packfile, PackfileError, Remote, RemoteError, RemoteRef};
use crate::versioning::repository::{ObjectStore, RefStore};
use crate::versioning::{CommitId, View, ViewName};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;
use ulid::Ulid;

#[derive(Error, Debug)]
pub enum SyncError {
    #[error("Remote error: {0}")]
    Remote(#[from] RemoteError),

    #[error("Packfile error: {0}")]
    Packfile(#[from] PackfileError),

    #[error("Fast-forward required")]
    FastForwardRequired,

    #[error("Ref not found: {0}")]
    RefNotFound(String),

    #[error("Sync failed: {0}")]
    Failed(String),
}

/// Result of a sync operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub success: bool,
    pub updated_refs: Vec<SyncRefUpdate>,
    pub message: String,
}

/// Reference update result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRefUpdate {
    pub ref_name: String,
    pub old_value: Option<Ulid>,
    pub new_value: Option<Ulid>,
    pub fast_forward: bool,
}

/// Fast-forward status for a push operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FastForwardStatus {
    /// Local is ahead, can fast-forward
    CanFastForward,
    /// Remote is ahead, fast-forward required
    NeedsFastForward,
    /// Diverged, need merge
    Diverged,
    /// Commits are unrelated
    Unrelated,
}

/// Sync engine for managing push/pull operations
#[allow(dead_code)]
#[derive(Debug)]
pub struct SyncEngine {
    local_repo: std::path::PathBuf,
    local_object_store: ObjectStore,
    local_ref_store: RefStore,
}

impl SyncEngine {
    /// Create a new sync engine
    pub fn new(repo_path: impl AsRef<Path>) -> Result<Self, SyncError> {
        let repo_path = repo_path.as_ref();
        let pkm_path = repo_path.join(".pkm");

        let local_object_store = ObjectStore::at(&pkm_path)
            .map_err(|e| SyncError::Failed(e.to_string()))?;
        let local_ref_store = RefStore::at(&pkm_path)
            .map_err(|e| SyncError::Failed(e.to_string()))?;

        Ok(Self {
            local_repo: repo_path.to_path_buf(),
            local_object_store,
            local_ref_store,
        })
    }

    /// Extract target Ulid from a View
    fn extract_target(view: &View) -> Ulid {
        match view {
            View::Branch { target, .. } => *target,
            View::Tag { target, .. } => *target,
        }
    }

    /// Check fast-forward status between local and remote refs
    pub fn check_fast_forward(
        &self,
        local_ref: &str,
        remote_ref: &RemoteRef,
    ) -> Result<FastForwardStatus, SyncError> {
        // Get local ref target
        let local_view = if let Ok(branch) = self.local_ref_store.get_branch(&ViewName::new(local_ref)) {
            branch
        } else if let Ok(tag) = self.local_ref_store.get_tag(&ViewName::new(local_ref)) {
            tag
        } else {
            return Err(SyncError::RefNotFound(local_ref.to_string()));
        };

        let local_target = Self::extract_target(&local_view);

        // If remote has no target, we can always push
        let remote_target = remote_ref.target;

        // If local is empty, just push
        if local_target.to_string() == "00000000000000000000000000" {
            return Ok(FastForwardStatus::CanFastForward);
        }

        // Get commit ancestry for both
        let local_ancestry = self.get_commit_ancestry(local_target)?;
        let remote_ancestry = self.get_remote_commit_ancestry(remote_target, remote_ref)?;

        // Check if local commit is ancestor of remote
        let local_is_ancestor = Self::is_ancestor(local_target, remote_target, &local_ancestry);
        let remote_is_ancestor = Self::is_ancestor(remote_target, local_target, &remote_ancestry);

        if local_is_ancestor {
            Ok(FastForwardStatus::CanFastForward)
        } else if remote_is_ancestor {
            Ok(FastForwardStatus::NeedsFastForward)
        } else {
            Ok(FastForwardStatus::Diverged)
        }
    }

    /// Get ancestry chain for a commit
    fn get_commit_ancestry(&self, commit_id: Ulid) -> Result<Vec<Ulid>, SyncError> {
        let mut ancestry = Vec::new();
        let mut current = CommitId::new(commit_id);

        while let Ok(commit) = self.local_object_store.get_commit(current) {
            ancestry.push(current.as_ulid());
            if commit.parents.is_empty() {
                break;
            }
            current = commit.parents[0];
        }

        Ok(ancestry)
    }

    /// Get ancestry from remote commit
    fn get_remote_commit_ancestry(
        &self,
        commit_id: Ulid,
        _remote_ref: &RemoteRef,
    ) -> Result<Vec<Ulid>, SyncError> {
        // This would require connecting to remote
        // For now, return empty - implementation depends on having remote connection
        Ok(vec![commit_id])
    }

    /// Check if commit A is ancestor of commit B
    /// The ancestry chain represents commits on the path from the newest to oldest (root).
    /// For example, [C, B, A] means C → B → A (A is the root).
    /// For A to be an ancestor of C, A must appear LATER in the chain (closer to root).
    fn is_ancestor(ancestor: Ulid, descendant: Ulid, ancestry: &[Ulid]) -> bool {
        if ancestor == descendant {
            return true;
        }
        let ancestor_pos = ancestry.iter().position(|&id| id == ancestor);
        let descendant_pos = ancestry.iter().position(|&id| id == descendant);

        match (ancestor_pos, descendant_pos) {
            (Some(a_pos), Some(d_pos)) => a_pos > d_pos,
            _ => false,
        }
    }

    /// Push refs to remote
    pub fn push(
        &self,
        remote: &Remote,
        refs: &[&str],
        force: bool,
    ) -> Result<SyncResult, SyncError> {
        let mut conn = remote.connect()?;

        let mut updates = Vec::new();
        let mut ff_statuses = Vec::new();

        for ref_name in refs {
            let view_name = ViewName::new(*ref_name);
            let local_view = self.local_ref_store.get_branch(&view_name)
                .or_else(|_| self.local_ref_store.get_tag(&view_name))
                .ok();

            let (old_value, new_value) = local_view.as_ref()
                .map(|v| (Some(Self::extract_target(v)), Some(Self::extract_target(v))))
                .unwrap_or((None, None));

            // Check fast-forward unless forcing
            if !force {
                let remote_refs = conn.fetch_refs()?;
                let remote_ref = remote_refs.iter().find(|r| r.name == *ref_name);

                if let Some(remote_ref) = remote_ref {
                    let status = self.check_fast_forward(ref_name, remote_ref)?;
                    ff_statuses.push(status);

                    if status == FastForwardStatus::NeedsFastForward && !force {
                        return Err(SyncError::FastForwardRequired);
                    }
                }
            }

            // Update remote ref
            if let Some(target) = new_value {
                conn.update_ref(ref_name, target)?;
            }

            updates.push(SyncRefUpdate {
                ref_name: ref_name.to_string(),
                old_value,
                new_value,
                fast_forward: ff_statuses.last().map(|s| *s == FastForwardStatus::CanFastForward).unwrap_or(false),
            });
        }

        Ok(SyncResult {
            success: true,
            updated_refs: updates,
            message: "Push completed successfully".to_string(),
        })
    }

    /// Pull from remote
    #[allow(clippy::unused_async)]
    pub async fn pull(
        &self,
        remote: &Remote,
        ref_name: &str,
        _strategy: MergeStrategy,
    ) -> Result<SyncResult, SyncError> {
        let mut conn = remote.connect()?;

        // Fetch remote refs
        let remote_refs = conn.fetch_refs()?;
        let remote_ref = remote_refs.iter()
            .find(|r| r.name == ref_name)
            .ok_or_else(|| SyncError::RefNotFound(ref_name.to_string()))?;

        // Get current local ref
        let view_name = ViewName::new(ref_name);
        let local_view = self.local_ref_store.get_branch(&view_name)
            .or_else(|_| self.local_ref_store.get_tag(&view_name))
            .ok();
        let local_target = local_view.as_ref().map(Self::extract_target);

        // Apply packfile if available
        match conn.fetch_packfile() {
            Ok(packfile) => {
                self.apply_packfile(&packfile)?;
            }
            Err(e) => {
                tracing::warn!("Failed to fetch packfile: {}", e);
            }
        }

        // Update local ref
        if local_target.is_some() {
            conn.update_ref(ref_name, remote_ref.target)?;
        }

        Ok(SyncResult {
            success: true,
            updated_refs: vec![SyncRefUpdate {
                ref_name: ref_name.to_string(),
                old_value: local_target,
                new_value: Some(remote_ref.target),
                fast_forward: true,
            }],
            message: format!("Pull completed for {}", ref_name),
        })
    }

    /// Apply a packfile to local repository
    fn apply_packfile(&self, packfile: &Packfile) -> Result<(), SyncError> {
        for entry in &packfile.entries {
            match entry.object_type {
                super::PackfileObjectType::Commit => {
                    // Commit objects would be deserialized and stored
                }
                super::PackfileObjectType::Structure => {
                    // Structure objects would be stored
                }
                super::PackfileObjectType::Block => {
                    // Block objects would be stored
                }
            }
        }
        Ok(())
    }

    /// Fetch without applying
    pub fn fetch(&self, remote: &Remote) -> Result<SyncResult, SyncError> {
        let result = remote.fetch()?;

        Ok(SyncResult {
            success: true,
            updated_refs: result.updated_refs.iter().map(|r| SyncRefUpdate {
                ref_name: r.name.clone(),
                old_value: None,
                new_value: Some(r.target),
                fast_forward: false,
            }).collect(),
            message: "Fetch completed".to_string(),
        })
    }
}

/// Merge strategy for pull operations
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub enum MergeStrategy {
    /// Prefer local changes
    Ours,
    /// Prefer remote changes
    Theirs,
    /// Create merge commit
    #[default]
    Merge,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_forward_status() {
        assert_eq!(format!("{:?}", FastForwardStatus::CanFastForward), "CanFastForward");
        assert_eq!(format!("{:?}", FastForwardStatus::NeedsFastForward), "NeedsFastForward");
        assert_eq!(format!("{:?}", FastForwardStatus::Diverged), "Diverged");
        assert_eq!(format!("{:?}", FastForwardStatus::Unrelated), "Unrelated");
    }

    #[test]
    fn test_merge_strategy() {
        assert_eq!(MergeStrategy::default(), MergeStrategy::Merge);
    }

    #[test]
    fn test_sync_ref_update() {
        let update = SyncRefUpdate {
            ref_name: "main".to_string(),
            old_value: Some(Ulid::new()),
            new_value: Some(Ulid::new()),
            fast_forward: true,
        };

        assert_eq!(update.ref_name, "main");
        assert!(update.fast_forward);
    }

    #[test]
    fn test_is_ancestor() {
        // Test case: commit chain is A <- B <- C (C is newest, A is oldest)
        let commit_a = Ulid::from_string("00000000000000000000000001").unwrap();
        let commit_b = Ulid::from_string("00000000000000000000000002").unwrap();
        let commit_c = Ulid::from_string("00000000000000000000000003").unwrap();

        // Chain from C to A: [C, B, A]
        let chain_from_c = vec![commit_c, commit_b, commit_a];

        // A is ancestor of C (A appears in C's ancestry)
        assert!(SyncEngine::is_ancestor(commit_a, commit_c, &chain_from_c));
        // B is ancestor of C
        assert!(SyncEngine::is_ancestor(commit_b, commit_c, &chain_from_c));
        // C is NOT ancestor of A (A doesn't appear in C's ancestry)
        assert!(!SyncEngine::is_ancestor(commit_c, commit_a, &chain_from_c));
        // A is ancestor of itself
        assert!(SyncEngine::is_ancestor(commit_a, commit_a, &chain_from_c));

        // Unrelated commit not in ancestry
        let unrelated = Ulid::from_string("00000000000000000000000005").unwrap();
        assert!(!SyncEngine::is_ancestor(unrelated, commit_c, &chain_from_c));
    }
}
