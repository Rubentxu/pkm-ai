//! Remote repository connection and operations

use super::{Packfile, RemoteConfig};
use crate::versioning::repository::{ObjectStore, RefStore};
use crate::versioning::View;
use std::path::Path;
use thiserror::Error;
use ulid::Ulid;

#[derive(Error, Debug)]
pub enum RemoteError {
    #[error("Remote not found: {0}")]
    NotFound(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Repository not found at path: {0}")]
    RepoNotFound(String),

    #[error("Fast-forward required but not possible")]
    FastForwardRequired,

    #[error("Ref not found: {0}")]
    RefNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Object store error: {0}")]
    ObjectStore(String),

    #[error("Ref store error: {0}")]
    RefStore(String),
}

impl From<crate::versioning::repository::ObjectStoreError> for RemoteError {
    fn from(e: crate::versioning::repository::ObjectStoreError) -> Self {
        RemoteError::ObjectStore(e.to_string())
    }
}

impl From<crate::versioning::repository::RefStoreError> for RemoteError {
    fn from(e: crate::versioning::repository::RefStoreError) -> Self {
        RemoteError::RefStore(e.to_string())
    }
}

/// Remote repository connection
#[derive(Debug)]
pub struct Remote {
    config: RemoteConfig,
    local_repo: std::path::PathBuf,
}

impl Remote {
    /// Create a new remote connection
    pub fn new(name: impl Into<String>, local_repo: impl AsRef<Path>, remote_path: impl AsRef<Path>) -> Self {
        Self {
            config: RemoteConfig::new(name, remote_path),
            local_repo: local_repo.as_ref().to_path_buf(),
        }
    }

    /// Create from RemoteConfig
    pub fn from_config(config: RemoteConfig, local_repo: impl AsRef<Path>) -> Self {
        Self {
            local_repo: local_repo.as_ref().to_path_buf(),
            config,
        }
    }

    /// Get the remote name
    pub fn name(&self) -> &str {
        &self.config.name
    }

    /// Get the remote path
    pub fn path(&self) -> &Path {
        &self.config.path
    }

    /// Check if the remote repository exists
    pub fn exists(&self) -> bool {
        self.config.path.join(".pkm").exists()
    }

    /// Connect to the remote repository
    pub fn connect(&self) -> Result<RemoteConnection, RemoteError> {
        if !self.exists() {
            return Err(RemoteError::RepoNotFound(
                self.config.path.to_string_lossy().to_string(),
            ));
        }

        RemoteConnection::new(self.config.clone(), &self.local_repo)
    }

    /// Fetch references from remote without applying changes
    pub fn fetch(&self) -> Result<FetchResult, RemoteError> {
        let conn = self.connect()?;

        // Get all branches and tags from remote
        let remote_refs = conn.fetch_refs()?;

        Ok(FetchResult {
            updated_refs: remote_refs,
            packfile: None,
        })
    }

    /// Fetch with packfile data
    pub fn fetch_with_packfile(&self) -> Result<FetchResult, RemoteError> {
        let conn = self.connect()?;

        // Get all references
        let refs = conn.fetch_refs()?;

        // Generate packfile with objects needed
        let packfile = conn.fetch_packfile()?;

        Ok(FetchResult {
            updated_refs: refs,
            packfile: Some(packfile),
        })
    }
}

/// Connection to a remote repository
#[allow(dead_code)]
#[derive(Debug)]
pub struct RemoteConnection {
    config: RemoteConfig,
    local_repo: std::path::PathBuf,
    remote_object_store: ObjectStore,
    pub(crate) remote_ref_store: RefStore,
}

impl RemoteConnection {
    fn new(config: RemoteConfig, local_repo: &Path) -> Result<Self, RemoteError> {
        let remote_path = config.path.join(".pkm");
        let local_repo = local_repo.to_path_buf();

        let remote_object_store = ObjectStore::at(&remote_path)
            .map_err(|e| RemoteError::ConnectionFailed(e.to_string()))?;
        let remote_ref_store = RefStore::at(&remote_path)
            .map_err(|e| RemoteError::ConnectionFailed(e.to_string()))?;

        Ok(Self {
            config,
            local_repo,
            remote_object_store,
            remote_ref_store,
        })
    }

    /// Extract target from a View
    fn extract_target(view: &View) -> Ulid {
        match view {
            View::Branch { target, .. } => *target,
            View::Tag { target, .. } => *target,
        }
    }

    /// Fetch all references from remote
    pub fn fetch_refs(&self) -> Result<Vec<RemoteRef>, RemoteError> {
        let mut refs = Vec::new();

        // Fetch branches
        for branch_name in self.remote_ref_store.list_branches()? {
            if let Ok(view) = self.remote_ref_store.get_branch(&branch_name) {
                refs.push(RemoteRef {
                    name: branch_name.to_string(),
                    target: Self::extract_target(&view),
                    is_branch: true,
                });
            }
        }

        // Fetch tags
        for tag_name in self.remote_ref_store.list_tags()? {
            if let Ok(view) = self.remote_ref_store.get_tag(&tag_name) {
                refs.push(RemoteRef {
                    name: tag_name.to_string(),
                    target: Self::extract_target(&view),
                    is_branch: false,
                });
            }
        }

        Ok(refs)
    }

    /// Fetch packfile containing needed objects
    pub fn fetch_packfile(&self) -> Result<Packfile, RemoteError> {
        let commits = self.remote_object_store.list_commits()?;
        let structures = self.remote_object_store.list_structures()?;

        let mut entries = Vec::new();

        // Get all commit objects
        for commit_id in &commits {
            if let Ok(_commit) = self.remote_object_store.get_commit(*commit_id) {
                entries.push(entries.len() as u32);
            }
        }

        // Get all structure objects
        for structure_id in &structures {
            if let Ok(_structure) = self.remote_object_store.get_structure(*structure_id) {
                entries.push(structure_id.to_string().parse().unwrap_or(0));
            }
        }

        Ok(Packfile::new(entries))
    }

    /// Update a reference in the remote
    pub fn update_ref(&mut self, ref_name: &str, target: Ulid) -> Result<(), RemoteError> {
        // Update branch or tag in remote ref store
        let simple_name = ref_name.trim_start_matches("refs/heads/").trim_start_matches("refs/tags/");
        if ref_name.starts_with("refs/heads/") || !ref_name.contains('/') {
            self.remote_ref_store.set_branch(simple_name, target)?;
        } else if ref_name.starts_with("refs/tags/") {
            self.remote_ref_store.set_tag(simple_name, target)?;
        }
        Ok(())
    }
}

/// Remote reference information
#[derive(Debug, Clone)]
pub struct RemoteRef {
    pub name: String,
    pub target: Ulid,
    pub is_branch: bool,
}

/// Result of a fetch operation
#[derive(Debug)]
pub struct FetchResult {
    pub updated_refs: Vec<RemoteRef>,
    pub packfile: Option<Packfile>,
}

/// Push result
#[derive(Debug)]
pub struct PushResult {
    pub success: bool,
    pub pushed_refs: Vec<String>,
    pub fast_forward_status: Vec<super::FastForwardStatus>,
}

/// Remote manager for handling multiple remotes
#[derive(Debug, Default)]
pub struct RemoteManager {
    remotes: Vec<Remote>,
}

impl RemoteManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_remote(&mut self, remote: Remote) {
        self.remotes.push(remote);
    }

    pub fn get_remote(&self, name: &str) -> Option<&Remote> {
        self.remotes.iter().find(|r| r.name() == name)
    }

    pub fn list_remotes(&self) -> Vec<&Remote> {
        self.remotes.iter().collect()
    }

    pub fn remove_remote(&mut self, name: &str) -> bool {
        let len = self.remotes.len();
        self.remotes.retain(|r| r.name() != name);
        self.remotes.len() < len
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_remote() -> (TempDir, Remote) {
        let temp_dir = TempDir::new().unwrap();
        let local_repo = TempDir::new().unwrap();

        // Create .pkm structure in remote
        let remote_path = temp_dir.path().join(".pkm");
        std::fs::create_dir_all(remote_path.join("objects/commits")).unwrap();
        std::fs::create_dir_all(remote_path.join("refs/heads")).unwrap();

        let remote = Remote::new("origin", local_repo.path(), temp_dir.path());

        (temp_dir, remote)
    }

    #[test]
    fn test_remote_creation() {
        let (_dir, remote) = create_test_remote();

        assert_eq!(remote.name(), "origin");
        assert!(remote.exists());
    }

    #[test]
    fn test_remote_not_found() {
        let temp_local = TempDir::new().unwrap();
        let remote_path = "/nonexistent/path";
        let remote = Remote::new("test", temp_local.path(), remote_path);

        assert!(!remote.exists());
    }

    #[test]
    fn test_remote_manager() {
        let temp_dir = TempDir::new().unwrap();
        let temp_local = TempDir::new().unwrap();

        let remote_path = temp_dir.path().join(".pkm");
        std::fs::create_dir_all(&remote_path).unwrap();

        let mut manager = RemoteManager::new();

        let remote1 = Remote::new("origin", temp_local.path(), &remote_path);
        manager.add_remote(remote1);

        assert_eq!(manager.list_remotes().len(), 1);
        assert!(manager.get_remote("origin").is_some());
        assert!(manager.get_remote("upstream").is_none());

        manager.remove_remote("origin");
        assert!(manager.list_remotes().is_empty());
    }
}
