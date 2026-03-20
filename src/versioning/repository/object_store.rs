//! ObjectStore: filesystem-based storage for versioned objects
//!
//! Stores commits, blocks, and structures as JSON files in a directory structure.

use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::versioning::{Commit, CommitId, StructureSnapshot};
use ulid::Ulid;

#[derive(Error, Debug)]
pub enum ObjectStoreError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Object not found: {0}")]
    NotFound(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

/// ObjectStore provides filesystem-based storage for versioned objects.
/// Objects are stored as JSON files in a structured directory hierarchy:
/// - objects/commits/{id}.json
/// - objects/blocks/{id}.json
/// - objects/structures/{id}.json
#[derive(Debug, Clone)]
pub struct ObjectStore {
    root: PathBuf,
    commits_path: PathBuf,
    blocks_path: PathBuf,
    structures_path: PathBuf,
}

impl ObjectStore {
    /// Create a new ObjectStore at the given root path
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        Self {
            commits_path: root.join("objects").join("commits"),
            blocks_path: root.join("objects").join("blocks"),
            structures_path: root.join("objects").join("structures"),
            root,
        }
    }

    /// Create and initialize ObjectStore at the given root path
    pub fn at(root: impl Into<PathBuf>) -> Result<Self, ObjectStoreError> {
        let store = Self::new(root);
        store.init()?;
        Ok(store)
    }

    /// Get the root path
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Initialize the directory structure
    pub fn init(&self) -> Result<(), ObjectStoreError> {
        fs::create_dir_all(&self.commits_path)?;
        fs::create_dir_all(&self.blocks_path)?;
        fs::create_dir_all(&self.structures_path)?;
        Ok(())
    }

    /// Check if the store is initialized
    #[must_use]
    pub fn is_initialized(&self) -> bool {
        self.commits_path.exists()
            && self.blocks_path.exists()
            && self.structures_path.exists()
    }

    /// Get the path for a commit file
    #[must_use]
    fn commit_path(&self, id: CommitId) -> PathBuf {
        self.commits_path.join(format!("{}.json", id))
    }

    /// Get the path for a block file
    #[allow(dead_code)]
    #[must_use]
    fn block_path(&self, id: Ulid) -> PathBuf {
        self.blocks_path.join(format!("{}.json", id))
    }

    /// Get the path for a structure file
    #[must_use]
    fn structure_path(&self, id: Ulid) -> PathBuf {
        self.structures_path.join(format!("{}.json", id))
    }

    /// Save a commit to the store
    pub fn put_commit(&self, commit: &Commit) -> Result<(), ObjectStoreError> {
        let path = self.commit_path(commit.id);
        let json = serde_json::to_string_pretty(commit)?;
        fs::write(&path, json)?;
        Ok(())
    }

    /// Load a commit from the store
    pub fn get_commit(&self, id: CommitId) -> Result<Commit, ObjectStoreError> {
        let path = self.commit_path(id);
        if !path.exists() {
            return Err(ObjectStoreError::NotFound(format!("Commit {}", id)));
        }
        let json = fs::read_to_string(&path)?;
        let commit: Commit = serde_json::from_str(&json)?;
        Ok(commit)
    }

    /// Check if a commit exists
    #[must_use]
    pub fn has_commit(&self, id: CommitId) -> bool {
        self.commit_path(id).exists()
    }

    /// Delete a commit
    pub fn delete_commit(&self, id: CommitId) -> Result<(), ObjectStoreError> {
        let path = self.commit_path(id);
        if !path.exists() {
            return Err(ObjectStoreError::NotFound(format!("Commit {}", id)));
        }
        fs::remove_file(path)?;
        Ok(())
    }

    /// Save a structure snapshot to the store
    pub fn put_structure(&self, structure: &StructureSnapshot) -> Result<(), ObjectStoreError> {
        let path = self.structure_path(structure.id);
        let json = serde_json::to_string_pretty(structure)?;
        fs::write(&path, json)?;
        Ok(())
    }

    /// Load a structure snapshot from the store
    pub fn get_structure(&self, id: Ulid) -> Result<StructureSnapshot, ObjectStoreError> {
        let path = self.structure_path(id);
        if !path.exists() {
            return Err(ObjectStoreError::NotFound(format!("Structure {}", id)));
        }
        let json = fs::read_to_string(&path)?;
        let structure: StructureSnapshot = serde_json::from_str(&json)?;
        Ok(structure)
    }

    /// Check if a structure exists
    #[must_use]
    pub fn has_structure(&self, id: Ulid) -> bool {
        self.structure_path(id).exists()
    }

    /// Delete a structure
    pub fn delete_structure(&self, id: Ulid) -> Result<(), ObjectStoreError> {
        let path = self.structure_path(id);
        if !path.exists() {
            return Err(ObjectStoreError::NotFound(format!("Structure {}", id)));
        }
        fs::remove_file(path)?;
        Ok(())
    }

    /// List all commits in the store
    pub fn list_commits(&self) -> Result<Vec<CommitId>, ObjectStoreError> {
        let mut commits = Vec::new();
        if !self.commits_path.exists() {
            return Ok(commits);
        }
        for entry in fs::read_dir(&self.commits_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json")
                && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                    && let Ok(ulid) = stem.parse::<Ulid>() {
                        commits.push(CommitId::new(ulid));
                    }
        }
        commits.sort_by_key(|id| id.as_ulid());
        Ok(commits)
    }

    /// List all structures in the store
    pub fn list_structures(&self) -> Result<Vec<Ulid>, ObjectStoreError> {
        let mut structures = Vec::new();
        if !self.structures_path.exists() {
            return Ok(structures);
        }
        for entry in fs::read_dir(&self.structures_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json")
                && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                    && let Ok(ulid) = stem.parse::<Ulid>() {
                        structures.push(ulid);
                    }
        }
        structures.sort();
        Ok(structures)
    }

    /// Get an iterator over all commits
    pub fn iter_commits(&self) -> Result<CommitIterator, ObjectStoreError> {
        CommitIterator::new(self.commits_path.clone())
    }
}

/// Iterator over commits in the store
#[derive(Debug)]
pub struct CommitIterator {
    entries: Vec<CommitId>,
}

impl CommitIterator {
    fn new(path: PathBuf) -> Result<Self, ObjectStoreError> {
        let mut entries = Vec::new();
        if !path.exists() {
            return Ok(Self { entries });
        }
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json")
                && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                    && let Ok(ulid) = stem.parse::<Ulid>() {
                        entries.push(CommitId::new(ulid));
                    }
        }
        entries.sort_by_key(|id| id.as_ulid());
        Ok(Self { entries })
    }
}

impl Iterator for CommitIterator {
    type Item = CommitId;

    fn next(&mut self) -> Option<Self::Item> {
        self.entries.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::versioning::{AgentId, EdgeSnapshot};
    use tempfile::TempDir;

    fn create_test_commit(id: CommitId) -> Commit {
        Commit {
            id,
            structure_snapshot: StructureSnapshot {
                id: Ulid::new(),
                block_order: vec![Ulid::new(), Ulid::new()],
                edges: Vec::new(),
            },
            parents: Vec::new(),
            author: AgentId::new("test"),
            message: "Test commit".to_string(),
            created_at: chrono::Utc::now(),
            blocks_added: Vec::new(),
            blocks_removed: Vec::new(),
            blocks_modified: Vec::new(),
        }
    }

    #[test]
    fn test_object_store_init() {
        let temp = TempDir::new().unwrap();
        let store = ObjectStore::new(temp.path());

        assert!(!store.is_initialized());
        store.init().unwrap();
        assert!(store.is_initialized());
    }

    #[test]
    fn test_put_and_get_commit() {
        let temp = TempDir::new().unwrap();
        let store = ObjectStore::new(temp.path());
        store.init().unwrap();

        let commit = create_test_commit(CommitId::new(Ulid::new()));
        let commit_id = commit.id;

        store.put_commit(&commit).unwrap();
        assert!(store.has_commit(commit_id));

        let loaded = store.get_commit(commit_id).unwrap();
        assert_eq!(loaded.id, commit_id);
        assert_eq!(loaded.message, commit.message);
    }

    #[test]
    fn test_get_nonexistent_commit() {
        let temp = TempDir::new().unwrap();
        let store = ObjectStore::new(temp.path());
        store.init().unwrap();

        let result = store.get_commit(CommitId::new(Ulid::new()));
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_commit() {
        let temp = TempDir::new().unwrap();
        let store = ObjectStore::new(temp.path());
        store.init().unwrap();

        let commit = create_test_commit(CommitId::new(Ulid::new()));
        let commit_id = commit.id;

        store.put_commit(&commit).unwrap();
        assert!(store.has_commit(commit_id));

        store.delete_commit(commit_id).unwrap();
        assert!(!store.has_commit(commit_id));
    }

    #[test]
    fn test_list_commits() {
        let temp = TempDir::new().unwrap();
        let store = ObjectStore::new(temp.path());
        store.init().unwrap();

        let commit1 = create_test_commit(CommitId::new(Ulid::new()));
        let commit2 = create_test_commit(CommitId::new(Ulid::new()));

        store.put_commit(&commit1).unwrap();
        store.put_commit(&commit2).unwrap();

        let commits = store.list_commits().unwrap();
        assert_eq!(commits.len(), 2);
    }

    #[test]
    fn test_commit_iterator() {
        let temp = TempDir::new().unwrap();
        let store = ObjectStore::new(temp.path());
        store.init().unwrap();

        for i in 0..5 {
            let mut commit = create_test_commit(CommitId::new(Ulid::new()));
            commit.message = format!("Commit {}", i);
            store.put_commit(&commit).unwrap();
        }

        let iter = store.iter_commits().unwrap();
        let count = iter.count();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_put_and_get_structure() {
        let temp = TempDir::new().unwrap();
        let store = ObjectStore::new(temp.path());
        store.init().unwrap();

        let structure = StructureSnapshot {
            id: Ulid::new(),
            block_order: vec![Ulid::new(), Ulid::new()],
            edges: vec![EdgeSnapshot {
                source: Ulid::new(),
                target: Ulid::new(),
                relation: "follows".to_string(),
            }],
        };
        let structure_id = structure.id;

        store.put_structure(&structure).unwrap();
        assert!(store.has_structure(structure_id));

        let loaded = store.get_structure(structure_id).unwrap();
        assert_eq!(loaded.id, structure_id);
        assert_eq!(loaded.edges.len(), 1);
    }
}
