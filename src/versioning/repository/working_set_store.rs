//! WorkingSetStore: filesystem-based storage for working set (staging area)
//!
//! Stores the current working set state for atomic commits.

use std::fs;
use std::path::PathBuf;
use thiserror::Error;

use crate::versioning::{WorkingSet, WorkingSetId};

#[derive(Error, Debug)]
pub enum WorkingSetStoreError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Working set not found")]
    NotFound,

    #[error("Working set ID mismatch: expected {expected}, found {found}")]
    IdMismatch { expected: WorkingSetId, found: WorkingSetId },
}

/// WorkingSetStore provides filesystem-based storage for the working set.
#[derive(Debug, Clone)]
pub struct WorkingSetStore {
    path: PathBuf,
}

impl WorkingSetStore {
    /// Create a new WorkingSetStore at the given root path
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            path: root.into(),
        }
    }

    /// Get the path to the working set file
    #[must_use]
    fn working_set_path(&self) -> PathBuf {
        self.path.join("working_set.json")
    }

    /// Initialize the store (creates the directory)
    pub fn init(&self) -> Result<(), WorkingSetStoreError> {
        fs::create_dir_all(&self.path)?;
        Ok(())
    }

    /// Check if the store has a working set
    #[must_use]
    pub fn exists(&self) -> bool {
        self.working_set_path().exists()
    }

    /// Save a working set to the store
    pub fn save(&self, working_set: &WorkingSet) -> Result<(), WorkingSetStoreError> {
        let path = self.working_set_path();
        let json = serde_json::to_string_pretty(working_set)?;
        fs::write(&path, json)?;
        Ok(())
    }

    /// Load a working set from the store
    pub fn load(&self) -> Result<WorkingSet, WorkingSetStoreError> {
        let path = self.working_set_path();
        if !path.exists() {
            return Err(WorkingSetStoreError::NotFound);
        }
        let json = fs::read_to_string(&path)?;
        let working_set: WorkingSet = serde_json::from_str(&json)?;
        Ok(working_set)
    }

    /// Load a working set from the store and verify the ID
    pub fn load_with_id(
        &self,
        expected_id: WorkingSetId,
    ) -> Result<WorkingSet, WorkingSetStoreError> {
        let working_set = self.load()?;
        if working_set.id != expected_id {
            return Err(WorkingSetStoreError::IdMismatch {
                expected: expected_id,
                found: working_set.id,
            });
        }
        Ok(working_set)
    }

    /// Delete the working set from the store
    pub fn delete(&self) -> Result<(), WorkingSetStoreError> {
        let path = self.working_set_path();
        if !path.exists() {
            return Err(WorkingSetStoreError::NotFound);
        }
        fs::remove_file(path)?;
        Ok(())
    }

    /// Clear the working set (reset to empty)
    pub fn clear(&self) -> Result<(), WorkingSetStoreError> {
        let path = self.working_set_path();
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::versioning::{AgentId, BlockDelta};
    use tempfile::TempDir;
    use ulid::Ulid;

    fn create_test_working_set(id: WorkingSetId) -> WorkingSet {
        WorkingSet::with_id(id, AgentId::new("test"))
    }

    #[test]
    fn test_working_set_store_init() {
        let temp = TempDir::new().unwrap();
        let store = WorkingSetStore::new(temp.path().join("pkm").join("working_set"));

        assert!(!store.exists());
        store.init().unwrap();
        // After init, directory should exist but not the file
        if let Some(parent) = store.path.parent() {
            assert!(parent.exists());
        }
    }

    #[test]
    fn test_save_and_load() {
        let temp = TempDir::new().unwrap();
        let store = WorkingSetStore::new(temp.path().join("pkm").join("working_set"));
        store.init().unwrap();

        let working_set = create_test_working_set(WorkingSetId::new(Ulid::new()));
        let original_id = working_set.id;

        store.save(&working_set).unwrap();
        assert!(store.exists());

        let loaded = store.load().unwrap();
        assert_eq!(loaded.id, original_id);
    }

    #[test]
    fn test_load_nonexistent() {
        let temp = TempDir::new().unwrap();
        let store = WorkingSetStore::new(temp.path().join("pkm").join("working_set"));
        store.init().unwrap();

        let result = store.load();
        assert!(result.is_err());
    }

    #[test]
    fn test_delete() {
        let temp = TempDir::new().unwrap();
        let store = WorkingSetStore::new(temp.path().join("pkm").join("working_set"));
        store.init().unwrap();

        let working_set = create_test_working_set(WorkingSetId::new(Ulid::new()));
        store.save(&working_set).unwrap();
        assert!(store.exists());

        store.delete().unwrap();
        assert!(!store.exists());
    }

    #[test]
    fn test_clear() {
        let temp = TempDir::new().unwrap();
        let store = WorkingSetStore::new(temp.path().join("pkm").join("working_set"));
        store.init().unwrap();

        let working_set = create_test_working_set(WorkingSetId::new(Ulid::new()));
        store.save(&working_set).unwrap();
        assert!(store.exists());

        store.clear().unwrap();
        assert!(!store.exists());
    }

    #[test]
    fn test_load_with_id_success() {
        let temp = TempDir::new().unwrap();
        let store = WorkingSetStore::new(temp.path().join("pkm").join("working_set"));
        store.init().unwrap();

        let working_set = create_test_working_set(WorkingSetId::new(Ulid::new()));
        let id = working_set.id;

        store.save(&working_set).unwrap();

        let loaded = store.load_with_id(id).unwrap();
        assert_eq!(loaded.id, id);
    }

    #[test]
    fn test_load_with_id_mismatch() {
        let temp = TempDir::new().unwrap();
        let store = WorkingSetStore::new(temp.path().join("pkm").join("working_set"));
        store.init().unwrap();

        let working_set = create_test_working_set(WorkingSetId::new(Ulid::new()));
        store.save(&working_set).unwrap();

        let different_id = WorkingSetId::new(Ulid::new());
        let result = store.load_with_id(different_id);
        assert!(result.is_err());

        if let Err(WorkingSetStoreError::IdMismatch { expected, found }) = result {
            assert_eq!(found, working_set.id);
            assert_eq!(expected, different_id);
        }
    }

    #[test]
    fn test_round_trip_with_staged_blocks() {
        let temp = TempDir::new().unwrap();
        let store = WorkingSetStore::new(temp.path().join("pkm").join("working_set"));
        store.init().unwrap();

        let mut working_set = create_test_working_set(WorkingSetId::new(Ulid::new()));

        // Add a staged block delta
        let block_delta = BlockDelta::Created {
            block_id: Ulid::new(),
            title: "Test Block".to_string(),
            content: "Test content".to_string(),
            block_type: "note".to_string(),
        };
        working_set.stage_block(block_delta);

        store.save(&working_set).unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.id, working_set.id);
        assert_eq!(loaded.staged_blocks().len(), 1);
    }
}