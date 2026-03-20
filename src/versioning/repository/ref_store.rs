//! RefStore: filesystem-based storage for branches and tags
//!
//! Stores views (branches and tags) as single-line files in a structured directory hierarchy:
//! - refs/heads/{name} - branch refs (contains target ULID)
//! - refs/tags/{name} - tag refs (contains target ULID + optional message)

use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::versioning::{View, ViewName};
use ulid::Ulid;

#[derive(Error, Debug)]
pub enum RefStoreError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("View not found: {0}")]
    NotFound(String),

    #[error("Invalid view name: {0}")]
    InvalidName(String),
}

/// RefStore provides filesystem-based storage for views (branches and tags).
#[derive(Debug, Clone)]
pub struct RefStore {
    root: PathBuf,
    heads_path: PathBuf,
    tags_path: PathBuf,
}

impl RefStore {
    /// Create a new RefStore at the given root path
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        Self {
            heads_path: root.join("refs").join("heads"),
            tags_path: root.join("refs").join("tags"),
            root,
        }
    }

    /// Create and initialize RefStore at the given root path
    pub fn at(root: impl Into<PathBuf>) -> Result<Self, RefStoreError> {
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
    pub fn init(&self) -> Result<(), RefStoreError> {
        fs::create_dir_all(&self.heads_path)?;
        fs::create_dir_all(&self.tags_path)?;
        Ok(())
    }

    /// Check if the store is initialized
    #[must_use]
    pub fn is_initialized(&self) -> bool {
        self.heads_path.exists() && self.tags_path.exists()
    }

    /// Get the path for a branch ref file
    #[must_use]
    fn branch_path(&self, name: &ViewName) -> PathBuf {
        self.heads_path.join(name.as_str())
    }

    /// Get the path for a tag ref file
    #[must_use]
    fn tag_path(&self, name: &ViewName) -> PathBuf {
        self.tags_path.join(name.as_str())
    }

    /// Save a branch view to the store
    pub fn put_branch(&self, view: &View) -> Result<(), RefStoreError> {
        match view {
            View::Branch { name, target: _, is_head: _ } => {
                let path = self.branch_path(name);
                let content = serde_json::to_string(view)?;
                fs::write(&path, content)?;
                Ok(())
            }
            View::Tag { .. } => Err(RefStoreError::InvalidName(
                "Cannot save a Tag as a branch".to_string(),
            )),
        }
    }

    /// Load a branch view from the store
    pub fn get_branch(&self, name: &ViewName) -> Result<View, RefStoreError> {
        let path = self.branch_path(name);
        if !path.exists() {
            return Err(RefStoreError::NotFound(format!("Branch {}", name)));
        }
        let content = fs::read_to_string(&path)?;
        let view: View = serde_json::from_str(&content)?;
        match &view {
            View::Branch { .. } => Ok(view),
            View::Tag { .. } => Err(RefStoreError::InvalidName(
                "Stored view is a Tag, not a Branch".to_string(),
            )),
        }
    }

    /// Check if a branch exists
    #[must_use]
    pub fn has_branch(&self, name: &ViewName) -> bool {
        self.branch_path(name).exists()
    }

    /// Delete a branch
    pub fn delete_branch(&self, name: &ViewName) -> Result<(), RefStoreError> {
        let path = self.branch_path(name);
        if !path.exists() {
            return Err(RefStoreError::NotFound(format!("Branch {}", name)));
        }
        fs::remove_file(path)?;
        Ok(())
    }

    /// Set a branch to point to a specific target
    pub fn set_branch(&self, name: &str, target: Ulid) -> Result<(), RefStoreError> {
        let view = View::branch(name, target);
        self.put_branch(&view)
    }

    /// Save a tag view to the store
    pub fn put_tag(&self, view: &View) -> Result<(), RefStoreError> {
        match view {
            View::Tag { name, target: _, message: _ } => {
                let path = self.tag_path(name);
                let content = serde_json::to_string(view)?;
                fs::write(&path, content)?;
                Ok(())
            }
            View::Branch { .. } => Err(RefStoreError::InvalidName(
                "Cannot save a Branch as a tag".to_string(),
            )),
        }
    }

    /// Load a tag view from the store
    pub fn get_tag(&self, name: &ViewName) -> Result<View, RefStoreError> {
        let path = self.tag_path(name);
        if !path.exists() {
            return Err(RefStoreError::NotFound(format!("Tag {}", name)));
        }
        let content = fs::read_to_string(&path)?;
        let view: View = serde_json::from_str(&content)?;
        match &view {
            View::Tag { .. } => Ok(view),
            View::Branch { .. } => Err(RefStoreError::InvalidName(
                "Stored view is a Branch, not a Tag".to_string(),
            )),
        }
    }

    /// Check if a tag exists
    #[must_use]
    pub fn has_tag(&self, name: &ViewName) -> bool {
        self.tag_path(name).exists()
    }

    /// Delete a tag
    pub fn delete_tag(&self, name: &ViewName) -> Result<(), RefStoreError> {
        let path = self.tag_path(name);
        if !path.exists() {
            return Err(RefStoreError::NotFound(format!("Tag {}", name)));
        }
        fs::remove_file(path)?;
        Ok(())
    }

    /// Set a tag to point to a specific target
    pub fn set_tag(&self, name: &str, target: Ulid) -> Result<(), RefStoreError> {
        let view = View::tag(name, target);
        self.put_tag(&view)
    }

    /// List all branches in the store
    pub fn list_branches(&self) -> Result<Vec<ViewName>, RefStoreError> {
        let mut branches = Vec::new();
        if !self.heads_path.exists() {
            return Ok(branches);
        }
        for entry in fs::read_dir(&self.heads_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file()
                && let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    branches.push(ViewName::new(name));
                }
        }
        branches.sort_by_key(|n| n.as_str().to_string());
        Ok(branches)
    }

    /// List all tags in the store
    pub fn list_tags(&self) -> Result<Vec<ViewName>, RefStoreError> {
        let mut tags = Vec::new();
        if !self.tags_path.exists() {
            return Ok(tags);
        }
        for entry in fs::read_dir(&self.tags_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file()
                && let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    tags.push(ViewName::new(name));
                }
        }
        tags.sort_by_key(|n| n.as_str().to_string());
        Ok(tags)
    }

    /// Get an iterator over all branches
    pub fn iter_branches(&self) -> Result<BranchIterator, RefStoreError> {
        BranchIterator::new(self.heads_path.clone())
    }

    /// Get an iterator over all tags
    pub fn iter_tags(&self) -> Result<TagIterator, RefStoreError> {
        TagIterator::new(self.tags_path.clone())
    }

    /// Get the current HEAD branch name
    pub fn get_head(&self) -> Result<Option<ViewName>, RefStoreError> {
        let branches = self.list_branches()?;
        for name in branches {
            if let Ok(view) = self.get_branch(&name)
                && view.is_head() {
                    return Ok(Some(name));
                }
        }
        Ok(None)
    }

    /// Set a branch as HEAD
    pub fn set_head(&self, name: &ViewName) -> Result<(), RefStoreError> {
        // First, unset all other branches as HEAD
        let branches = self.list_branches()?;
        for branch_name in branches {
            if let Ok(mut view) = self.get_branch(&branch_name)
                && view.is_head() {
                    view.set_is_head(false);
                    self.put_branch(&view)?;
                }
        }

        // Now set the new HEAD
        let mut view = self.get_branch(name)?;
        view.set_is_head(true);
        self.put_branch(&view)?;
        Ok(())
    }
}

/// Iterator over branches in the store
#[derive(Debug)]
pub struct BranchIterator {
    entries: Vec<ViewName>,
}

impl BranchIterator {
    fn new(path: PathBuf) -> Result<Self, RefStoreError> {
        let mut entries = Vec::new();
        if !path.exists() {
            return Ok(Self { entries });
        }
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file()
                && let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    entries.push(ViewName::new(name));
                }
        }
        entries.sort_by_key(|n| n.as_str().to_string());
        Ok(Self { entries })
    }
}

impl Iterator for BranchIterator {
    type Item = ViewName;

    fn next(&mut self) -> Option<Self::Item> {
        self.entries.pop()
    }
}

/// Iterator over tags in the store
#[derive(Debug)]
pub struct TagIterator {
    entries: Vec<ViewName>,
}

impl TagIterator {
    fn new(path: PathBuf) -> Result<Self, RefStoreError> {
        let mut entries = Vec::new();
        if !path.exists() {
            return Ok(Self { entries });
        }
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file()
                && let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    entries.push(ViewName::new(name));
                }
        }
        entries.sort_by_key(|n| n.as_str().to_string());
        Ok(Self { entries })
    }
}

impl Iterator for TagIterator {
    type Item = ViewName;

    fn next(&mut self) -> Option<Self::Item> {
        self.entries.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use ulid::Ulid;

    #[test]
    fn test_ref_store_init() {
        let temp = TempDir::new().unwrap();
        let store = RefStore::new(temp.path());

        assert!(!store.is_initialized());
        store.init().unwrap();
        assert!(store.is_initialized());
    }

    #[test]
    fn test_put_and_get_branch() {
        let temp = TempDir::new().unwrap();
        let store = RefStore::new(temp.path());
        store.init().unwrap();

        let target = Ulid::new();
        let view = View::branch_head("main", target);
        let view_name = ViewName::new("main");

        store.put_branch(&view).unwrap();
        assert!(store.has_branch(&view_name));

        let loaded = store.get_branch(&view_name).unwrap();
        assert_eq!(loaded.target(), target);
        assert!(loaded.is_head());
    }

    #[test]
    fn test_delete_branch() {
        let temp = TempDir::new().unwrap();
        let store = RefStore::new(temp.path());
        store.init().unwrap();

        let target = Ulid::new();
        let view = View::branch("main", target);
        let view_name = ViewName::new("main");

        store.put_branch(&view).unwrap();
        assert!(store.has_branch(&view_name));

        store.delete_branch(&view_name).unwrap();
        assert!(!store.has_branch(&view_name));
    }

    #[test]
    fn test_get_nonexistent_branch() {
        let temp = TempDir::new().unwrap();
        let store = RefStore::new(temp.path());
        store.init().unwrap();

        let result = store.get_branch(&ViewName::new("nonexistent"));
        assert!(result.is_err());
    }

    #[test]
    fn test_put_and_get_tag() {
        let temp = TempDir::new().unwrap();
        let store = RefStore::new(temp.path());
        store.init().unwrap();

        let target = Ulid::new();
        let view = View::tag_with_message("v1.0.0", target, "Release 1.0.0".to_string());
        let view_name = ViewName::new("v1.0.0");

        store.put_tag(&view).unwrap();
        assert!(store.has_tag(&view_name));

        let loaded = store.get_tag(&view_name).unwrap();
        assert_eq!(loaded.target(), target);
        assert_eq!(loaded.message(), Some("Release 1.0.0"));
    }

    #[test]
    fn test_delete_tag() {
        let temp = TempDir::new().unwrap();
        let store = RefStore::new(temp.path());
        store.init().unwrap();

        let target = Ulid::new();
        let view = View::tag("v1.0.0", target);
        let view_name = ViewName::new("v1.0.0");

        store.put_tag(&view).unwrap();
        assert!(store.has_tag(&view_name));

        store.delete_tag(&view_name).unwrap();
        assert!(!store.has_tag(&view_name));
    }

    #[test]
    fn test_list_branches() {
        let temp = TempDir::new().unwrap();
        let store = RefStore::new(temp.path());
        store.init().unwrap();

        store
            .put_branch(&View::branch("main", Ulid::new()))
            .unwrap();
        store
            .put_branch(&View::branch("feature-a", Ulid::new()))
            .unwrap();
        store
            .put_branch(&View::branch("feature-b", Ulid::new()))
            .unwrap();

        let branches = store.list_branches().unwrap();
        assert_eq!(branches.len(), 3);
    }

    #[test]
    fn test_list_tags() {
        let temp = TempDir::new().unwrap();
        let store = RefStore::new(temp.path());
        store.init().unwrap();

        store.put_tag(&View::tag("v1.0.0", Ulid::new())).unwrap();
        store.put_tag(&View::tag("v1.1.0", Ulid::new())).unwrap();

        let tags = store.list_tags().unwrap();
        assert_eq!(tags.len(), 2);
    }

    #[test]
    fn test_set_head() {
        let temp = TempDir::new().unwrap();
        let store = RefStore::new(temp.path());
        store.init().unwrap();

        // Create multiple branches
        store.put_branch(&View::branch("main", Ulid::new())).unwrap();
        store.put_branch(&View::branch("develop", Ulid::new())).unwrap();

        // Set main as HEAD
        store.set_head(&ViewName::new("main")).unwrap();

        // Verify main is HEAD
        let main_view = store.get_branch(&ViewName::new("main")).unwrap();
        assert!(main_view.is_head());

        // Verify develop is not HEAD
        let develop_view = store.get_branch(&ViewName::new("develop")).unwrap();
        assert!(!develop_view.is_head());

        // Set develop as HEAD
        store.set_head(&ViewName::new("develop")).unwrap();

        // Verify develop is now HEAD
        let develop_view = store.get_branch(&ViewName::new("develop")).unwrap();
        assert!(develop_view.is_head());

        // Verify main is no longer HEAD
        let main_view = store.get_branch(&ViewName::new("main")).unwrap();
        assert!(!main_view.is_head());
    }

    #[test]
    fn test_get_head() {
        let temp = TempDir::new().unwrap();
        let store = RefStore::new(temp.path());
        store.init().unwrap();

        // No HEAD initially
        assert!(store.get_head().unwrap().is_none());

        // Create branch and set as HEAD
        store.put_branch(&View::branch_head("main", Ulid::new())).unwrap();

        let head = store.get_head().unwrap();
        assert!(head.is_some());
        assert_eq!(head.unwrap().as_str(), "main");
    }

    #[test]
    fn test_branch_iterator() {
        let temp = TempDir::new().unwrap();
        let store = RefStore::new(temp.path());
        store.init().unwrap();

        for i in 0..5 {
            let view = View::branch(format!("branch-{}", i), Ulid::new());
            store.put_branch(&view).unwrap();
        }

        let iter = store.iter_branches().unwrap();
        let count = iter.count();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_tag_iterator() {
        let temp = TempDir::new().unwrap();
        let store = RefStore::new(temp.path());
        store.init().unwrap();

        for i in 0..3 {
            let view = View::tag(format!("v1.{}.0", i), Ulid::new());
            store.put_tag(&view).unwrap();
        }

        let iter = store.iter_tags().unwrap();
        let count = iter.count();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_cannot_save_tag_as_branch() {
        let temp = TempDir::new().unwrap();
        let store = RefStore::new(temp.path());
        store.init().unwrap();

        let tag = View::tag("v1.0.0", Ulid::new());
        let result = store.put_branch(&tag);
        assert!(result.is_err());
    }

    #[test]
    fn test_cannot_save_branch_as_tag() {
        let temp = TempDir::new().unwrap();
        let store = RefStore::new(temp.path());
        store.init().unwrap();

        let branch = View::branch("main", Ulid::new());
        let result = store.put_tag(&branch);
        assert!(result.is_err());
    }
}