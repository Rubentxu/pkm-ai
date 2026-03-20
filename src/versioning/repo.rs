//! VersionRepo: Git-like repository for knowledge management
//!
//! Provides a high-level interface for version control operations including
//! commits, branches, tags, and working set management.

use std::path::PathBuf;
use ulid::Ulid;

use super::repository::{ObjectStore, RefStore, WorkingSetStore, WorkingSetStoreError};
use super::{AgentId, BlockDelta, Commit, CommitId, ConflictInfo, ConflictType, View, ViewName, WorkingSet};

/// Repository context for version operations
#[allow(dead_code)]
#[derive(Debug)]
pub struct VersionRepo {
    root: PathBuf,
    pub object_store: ObjectStore,
    pub ref_store: RefStore,
    pub working_set_store: WorkingSetStore,
}

impl VersionRepo {
    /// Create a new version repository at the given root path
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        Self {
            object_store: ObjectStore::new(&root),
            ref_store: RefStore::new(&root),
            working_set_store: WorkingSetStore::new(&root),
            root,
        }
    }

    /// Initialize the repository
    pub fn init(&self) -> Result<(), VersionError> {
        self.object_store.init().map_err(VersionError::ObjectStore)?;
        self.ref_store.init().map_err(VersionError::RefStore)?;
        self.working_set_store.init().map_err(VersionError::WorkingSetStore)?;
        Ok(())
    }

    /// Get the current HEAD branch name
    pub fn get_head_branch(&self) -> Result<Option<ViewName>, VersionError> {
        self.ref_store.get_head().map_err(VersionError::RefStore)
    }

    /// Get the current HEAD commit
    pub fn get_head_commit(&self) -> Result<Option<Commit>, VersionError> {
        let head = self.get_head_branch()?;
        match head {
            Some(name) => {
                let view = self.ref_store.get_branch(&name).map_err(VersionError::RefStore)?;
                match self.object_store.get_commit(CommitId::new(view.target())) {
                    Ok(commit) => Ok(Some(commit)),
                    Err(super::repository::ObjectStoreError::NotFound(_)) => Ok(None),
                    Err(e) => Err(VersionError::ObjectStore(e)),
                }
            }
            None => Ok(None),
        }
    }

    /// Get the working set
    pub fn get_working_set(&self) -> Result<WorkingSet, VersionError> {
        match self.working_set_store.load() {
            Ok(ws) => Ok(ws),
            Err(WorkingSetStoreError::NotFound) => {
                // Return empty working set if none exists
                Ok(WorkingSet::new(AgentId::new("default")))
            }
            Err(e) => Err(VersionError::WorkingSetStore(e)),
        }
    }

    /// List all branches
    pub fn list_branches(&self) -> Result<Vec<ViewName>, VersionError> {
        self.ref_store.list_branches().map_err(VersionError::RefStore)
    }

    /// List all commits
    pub fn list_commits(&self) -> Result<Vec<CommitId>, VersionError> {
        self.object_store.list_commits().map_err(VersionError::ObjectStore)
    }

    /// Get commits as detailed list
    pub fn get_commits(&self, limit: usize) -> Result<Vec<Commit>, VersionError> {
        let mut commits = Vec::new();
        let ids = self.list_commits()?;
        for id in ids.into_iter().rev().take(limit) {
            if let Ok(commit) = self.object_store.get_commit(id) {
                commits.push(commit);
            }
        }
        Ok(commits)
    }

    /// Log all commits (newest first)
    pub fn log(&self) -> Result<Vec<Commit>, VersionError> {
        self.get_commits(100)
    }

    /// Stage a block for commit
    pub fn add_block(&self, _block_id: Ulid, delta: BlockDelta) -> Result<(), VersionError> {
        let mut ws = self.get_working_set()?;
        ws.stage_block(delta);
        self.working_set_store.save(&ws).map_err(VersionError::WorkingSetStore)?;
        Ok(())
    }

    /// Stage a block by ID (creates a Created delta automatically)
    pub fn stage(&self, block_id: &Ulid) -> Result<(), VersionError> {
        let delta = BlockDelta::Created {
            block_id: *block_id,
            title: format!("Block {}", block_id),
            content: String::new(),
            block_type: "note".to_string(),
        };
        self.add_block(*block_id, delta)
    }

    /// Stage an edge for commit
    pub fn add_edge(&self, source: Ulid, target: Ulid, relation: &str) -> Result<(), VersionError> {
        use super::EdgeDelta;
        let delta = EdgeDelta::Created {
            source,
            target,
            relation: relation.to_string(),
        };
        let mut ws = self.get_working_set()?;
        ws.stage_edge(delta);
        self.working_set_store.save(&ws).map_err(VersionError::WorkingSetStore)?;
        Ok(())
    }

    /// Unstage a block by ID (remove from staging area without discarding)
    pub fn unstage(&self, block_id: &Ulid) -> Result<(), VersionError> {
        let mut ws = self.get_working_set()?;
        ws.unstage_block(block_id);
        self.working_set_store.save(&ws).map_err(VersionError::WorkingSetStore)?;
        Ok(())
    }

    /// Discard all staged changes (clear the working set)
    pub fn discard_working_set(&self) -> Result<(), VersionError> {
        let mut ws = self.get_working_set()?;
        ws.clear();
        self.working_set_store.save(&ws).map_err(VersionError::WorkingSetStore)?;
        Ok(())
    }

    /// Create a new commit
    pub fn commit(&self, message: &str, author: AgentId) -> Result<CommitId, VersionError> {
        let ws = self.get_working_set()?;

        if ws.is_empty() {
            return Err(VersionError::NothingToCommit);
        }

        // Get current HEAD commit for parent
        let head = self.get_head_commit()?;
        let parents: Vec<CommitId> = head.map(|c| c.id).into_iter().collect();

        // Create structure snapshot from working set
        let structure_id = Ulid::new();
        let blocks_added: Vec<Ulid> = ws.staged_blocks().keys().cloned().collect();
        let blocks_removed = ws.removed_blocks().to_vec();

        let structure = super::StructureSnapshot {
            id: structure_id,
            block_order: blocks_added.clone(),
            edges: Vec::new(),
        };

        // Create commit
        let commit_id = CommitId::new(Ulid::new());
        let commit = Commit {
            id: commit_id,
            structure_snapshot: structure,
            parents,
            author,
            message: message.to_string(),
            created_at: chrono::Utc::now(),
            blocks_added,
            blocks_removed,
            blocks_modified: Vec::new(),
        };

        // Save commit
        self.object_store.put_commit(&commit).map_err(VersionError::ObjectStore)?;

        // Update HEAD branch
        if let Some(head_name) = self.get_head_branch()? {
            let mut view = self.ref_store.get_branch(&head_name).map_err(VersionError::RefStore)?;
            view.set_target(commit_id.as_ulid());
            self.ref_store.put_branch(&view).map_err(VersionError::RefStore)?;
        }

        // Clear working set
        self.working_set_store.clear().map_err(VersionError::WorkingSetStore)?;

        Ok(commit_id)
    }

    /// Get diff between commits for a block
    #[allow(dead_code)]
    pub fn get_block_diff(&self, _block_id: Ulid, _from_commit: Option<CommitId>, _to_commit: Option<CommitId>) -> Result<Option<String>, VersionError> {
        // For now, return None since we don't have block-level history yet
        Ok(None)
    }

    /// Create a new branch pointing to the current HEAD
    pub fn create_branch(&self, name: &str) -> Result<ViewName, VersionError> {
        let view_name = ViewName::new(name);

        // Check if branch already exists
        if self.ref_store.has_branch(&view_name) {
            return Err(VersionError::BranchAlreadyExists(name.to_string()));
        }

        // Get current HEAD commit target
        let target = match self.get_head_commit()? {
            Some(commit) => commit.id.as_ulid(),
            None => Ulid::new(), // Point to new commit if no HEAD exists
        };

        // Create new branch
        let view = View::branch(name, target);
        self.ref_store.put_branch(&view).map_err(VersionError::RefStore)?;

        Ok(view_name)
    }

    /// Switch HEAD to a different branch
    pub fn checkout(&self, name: &str) -> Result<ViewName, VersionError> {
        let view_name = ViewName::new(name);

        // Check if branch exists
        if !self.ref_store.has_branch(&view_name) {
            return Err(VersionError::BranchNotFound(name.to_string()));
        }

        // Set the branch as HEAD
        self.ref_store.set_head(&view_name).map_err(VersionError::RefStore)?;

        Ok(view_name)
    }

    /// Create a new branch and switch to it
    pub fn checkout_new_branch(&self, name: &str) -> Result<ViewName, VersionError> {
        let view_name = ViewName::new(name);

        // Check if branch already exists
        if self.ref_store.has_branch(&view_name) {
            return Err(VersionError::BranchAlreadyExists(name.to_string()));
        }

        // Get current HEAD commit target
        // If no HEAD commit exists, we can't create a branch based on it
        let target = match self.get_head_commit()? {
            Some(commit) => commit.id.as_ulid(),
            None => {
                return Err(VersionError::NoHeadCommit);
            }
        };

        // Create new branch and set as HEAD
        let view = View::branch_head(name, target);
        self.ref_store.put_branch(&view).map_err(VersionError::RefStore)?;

        Ok(view_name)
    }

    /// Delete a branch
    pub fn delete_branch(&self, name: &str, force: bool) -> Result<(), VersionError> {
        let view_name = ViewName::new(name);

        // Check if branch exists
        if !self.ref_store.has_branch(&view_name) {
            return Err(VersionError::BranchNotFound(name.to_string()));
        }

        // Check if it's the current HEAD
        if let Some(head) = self.get_head_branch()?
            && head == view_name {
                return Err(VersionError::CannotDeleteHead);
            }

        // Check if branch is merged (if not forcing)
        if !force {
            // For now, just warn that we can't easily determine if it's merged
            // In a real implementation, we'd check ancestry
        }

        self.ref_store.delete_branch(&view_name).map_err(VersionError::RefStore)?;
        Ok(())
    }

    /// Check if ancestor_id is an ancestor of descendant_id
    /// Traverses the commit graph by following parent links
    fn is_ancestor_of(&self, ancestor_id: CommitId, descendant_id: CommitId) -> bool {
        if ancestor_id == descendant_id {
            return false;
        }

        let mut current_id = descendant_id;

        loop {
            if current_id == ancestor_id {
                return true;
            }

            match self.object_store.get_commit(current_id) {
                Ok(commit) => {
                    if commit.parents.is_empty() {
                        return false;
                    }
                    current_id = commit.parents[0];
                }
                Err(_) => {
                    return false;
                }
            }
        }
    }

    /// Detect conflicts between two commits
    ///
    /// Analyzes blocks and edges to find modifications that conflict
    /// between the two branches being merged.
    fn detect_conflicts(
        &self,
        our_commit: &Commit,
        their_commit: &Commit,
    ) -> Result<Vec<ConflictInfo>, VersionError> {
        use std::collections::HashSet;
        let mut conflicts = Vec::new();

        // Get blocks from both commits
        let our_blocks: HashSet<Ulid> = our_commit.structure_snapshot.block_order.iter().cloned().collect();
        let their_blocks: HashSet<Ulid> = their_commit.structure_snapshot.block_order.iter().cloned().collect();

        // Blocks in common (potential conflict area)
        let common_blocks: HashSet<_> = our_blocks.intersection(&their_blocks).collect();

        for block_id in common_blocks {
            // Check if the block was modified in both branches
            let our_modified = our_commit.blocks_modified.contains(block_id);
            let their_modified = their_commit.blocks_modified.contains(block_id);

            if our_modified && their_modified {
                conflicts.push(ConflictInfo {
                    conflict_type: ConflictType::BlockModifiedBoth,
                    block_id: Some(*block_id),
                    description: format!(
                        "Block {} was modified in both branches",
                        block_id
                    ),
                });
            }

            // Check if deleted in one and modified in another
            let our_deleted = our_commit.blocks_removed.contains(block_id);
            let their_deleted = their_commit.blocks_removed.contains(block_id);

            if (our_deleted && their_modified) || (their_deleted && our_modified) {
                conflicts.push(ConflictInfo {
                    conflict_type: ConflictType::BlockDeletedVsModified,
                    block_id: Some(*block_id),
                    description: format!(
                        "Block {} was deleted in one branch but modified in another",
                        block_id
                    ),
                });
            }
        }

        // Verify edges as well
        let our_edges: HashSet<_> = our_commit.structure_snapshot.edges.iter()
            .map(|e| (e.source, e.target))
            .collect();
        let their_edges: HashSet<_> = their_commit.structure_snapshot.edges.iter()
            .map(|e| (e.source, e.target))
            .collect();

        // Edges in both but potentially different properties would be edge conflicts
        // For simplicity, we check if same edge exists in both with different relations
        for our_edge in &our_edges {
            if their_edges.contains(our_edge) {
                // Edge exists in both - check if relation differs
                let our_relation = our_commit.structure_snapshot.edges.iter()
                    .find(|e| e.source == our_edge.0 && e.target == our_edge.1)
                    .map(|e| &e.relation);
                let their_relation = their_commit.structure_snapshot.edges.iter()
                    .find(|e| e.source == our_edge.0 && e.target == our_edge.1)
                    .map(|e| &e.relation);

                if our_relation != their_relation {
                    conflicts.push(ConflictInfo {
                        conflict_type: ConflictType::EdgeConflict,
                        block_id: None,
                        description: format!(
                            "Edge from {} to {} was modified differently in each branch",
                            our_edge.0, our_edge.1
                        ),
                    });
                }
            }
        }

        Ok(conflicts)
    }

    /// Merge a branch into the current HEAD
    pub fn merge(&self, branch_name: &str, strategy: MergeStrategy) -> Result<MergeResult, VersionError> {
        let view_name = ViewName::new(branch_name);

        // Check if branch exists
        if !self.ref_store.has_branch(&view_name) {
            return Err(VersionError::BranchNotFound(branch_name.to_string()));
        }

        // Get current HEAD and target branch
        let head_commit = match self.get_head_commit()? {
            Some(c) => c,
            None => return Err(VersionError::NoHeadCommit),
        };

        let branch_view = self.ref_store.get_branch(&view_name).map_err(VersionError::RefStore)?;
        let branch_commit = self.object_store.get_commit(CommitId::new(branch_view.target()))
            .map_err(VersionError::ObjectStore)?;

        // Check if branches point to the same commit
        if branch_commit.id == head_commit.id {
            // Already merged - nothing to do
            return Ok(MergeResult::AlreadyMerged);
        }

        // Check if already merged (branch is ancestor of HEAD)
        let _branch_is_ancestor = self.is_ancestor_of(branch_commit.id, head_commit.id);

        // Check for fast-forward (HEAD is ancestor of branch)
        let head_is_ancestor = self.is_ancestor_of(head_commit.id, branch_commit.id);

        if head_is_ancestor {
            // Fast-forward: just move HEAD to branch
            let mut new_view = branch_view.clone();
            new_view.set_is_head(true);
            self.ref_store.put_branch(&new_view).map_err(VersionError::RefStore)?;

            // Unset is_head on old HEAD branch
            if let Some(head_name) = self.get_head_branch()?
                && head_name != view_name {
                    let mut old_view = self.ref_store.get_branch(&head_name).map_err(VersionError::RefStore)?;
                    old_view.set_is_head(false);
                    let _ = self.ref_store.put_branch(&old_view);
                }

            return Ok(MergeResult::FastForward);
        }

        // Detect conflicts between our HEAD and their branch
        let conflicts = self.detect_conflicts(&head_commit, &branch_commit)?;

        // Three-way merge needed
        match strategy {
            MergeStrategy::Ours => {
                // Keep our changes always - create commit with our blocks
                let merge_commit_id = CommitId::new(Ulid::new());
                let structure = super::StructureSnapshot {
                    id: Ulid::new(),
                    block_order: head_commit.structure_snapshot.block_order.clone(),
                    edges: head_commit.structure_snapshot.edges.clone(),
                };

                let merge_commit = Commit {
                    id: merge_commit_id,
                    structure_snapshot: structure,
                    parents: vec![head_commit.id, branch_commit.id],
                    author: super::AgentId::new("system"),
                    message: format!("Merge branch '{}' into {} (ours)", branch_name, self.get_head_branch()?.map(|n| n.to_string()).unwrap_or_else(|| "main".to_string())),
                    created_at: chrono::Utc::now(),
                    blocks_added: head_commit.blocks_added.clone(),
                    blocks_removed: head_commit.blocks_removed.clone(),
                    blocks_modified: head_commit.blocks_modified.clone(),
                };

                self.object_store.put_commit(&merge_commit).map_err(VersionError::ObjectStore)?;

                // Update HEAD to point to merge commit
                if let Some(head_name) = self.get_head_branch()? {
                    let mut view = self.ref_store.get_branch(&head_name).map_err(VersionError::RefStore)?;
                    view.set_target(merge_commit_id.as_ulid());
                    self.ref_store.put_branch(&view).map_err(VersionError::RefStore)?;
                }

                Ok(MergeResult::Clean { commit_id: merge_commit_id })
            }
            MergeStrategy::Theirs => {
                // Take their changes always - create commit with their blocks
                let merge_commit_id = CommitId::new(Ulid::new());
                let structure = super::StructureSnapshot {
                    id: Ulid::new(),
                    block_order: branch_commit.structure_snapshot.block_order.clone(),
                    edges: branch_commit.structure_snapshot.edges.clone(),
                };

                let merge_commit = Commit {
                    id: merge_commit_id,
                    structure_snapshot: structure,
                    parents: vec![head_commit.id, branch_commit.id],
                    author: super::AgentId::new("system"),
                    message: format!("Merge branch '{}' into {} (theirs)", branch_name, self.get_head_branch()?.map(|n| n.to_string()).unwrap_or_else(|| "main".to_string())),
                    created_at: chrono::Utc::now(),
                    blocks_added: branch_commit.blocks_added.clone(),
                    blocks_removed: branch_commit.blocks_removed.clone(),
                    blocks_modified: branch_commit.blocks_modified.clone(),
                };

                self.object_store.put_commit(&merge_commit).map_err(VersionError::ObjectStore)?;

                // Update HEAD to point to merge commit
                if let Some(head_name) = self.get_head_branch()? {
                    let mut view = self.ref_store.get_branch(&head_name).map_err(VersionError::RefStore)?;
                    view.set_target(merge_commit_id.as_ulid());
                    self.ref_store.put_branch(&view).map_err(VersionError::RefStore)?;
                }

                Ok(MergeResult::Clean { commit_id: merge_commit_id })
            }
            MergeStrategy::Merge => {
                // Real merge: detect conflicts
                // Create merge commit with combined changes
                let merge_commit_id = CommitId::new(Ulid::new());

                // Combine block orders - use ours as base, append theirs that are new
                let mut combined_blocks = head_commit.structure_snapshot.block_order.clone();
                for block_id in &branch_commit.structure_snapshot.block_order {
                    if !combined_blocks.contains(block_id) {
                        combined_blocks.push(*block_id);
                    }
                }

                // Combine edges
                let mut combined_edges = head_commit.structure_snapshot.edges.clone();
                for edge in &branch_commit.structure_snapshot.edges {
                    if !combined_edges.iter().any(|e| e.source == edge.source && e.target == edge.target) {
                        combined_edges.push(edge.clone());
                    }
                }

                let structure = super::StructureSnapshot {
                    id: Ulid::new(),
                    block_order: combined_blocks,
                    edges: combined_edges,
                };

                // Combine change tracking
                let mut combined_added = head_commit.blocks_added.clone();
                for block_id in &branch_commit.blocks_added {
                    if !combined_added.contains(block_id) {
                        combined_added.push(*block_id);
                    }
                }

                let mut combined_removed = head_commit.blocks_removed.clone();
                for block_id in &branch_commit.blocks_removed {
                    if !combined_removed.contains(block_id) {
                        combined_removed.push(*block_id);
                    }
                }

                let mut combined_modified = head_commit.blocks_modified.clone();
                for block_id in &branch_commit.blocks_modified {
                    if !combined_modified.contains(block_id) {
                        combined_modified.push(*block_id);
                    }
                }

                let merge_commit = Commit {
                    id: merge_commit_id,
                    structure_snapshot: structure,
                    parents: vec![head_commit.id, branch_commit.id],
                    author: super::AgentId::new("system"),
                    message: format!("Merge branch '{}' into {}", branch_name, self.get_head_branch()?.map(|n| n.to_string()).unwrap_or_else(|| "main".to_string())),
                    created_at: chrono::Utc::now(),
                    blocks_added: combined_added,
                    blocks_removed: combined_removed,
                    blocks_modified: combined_modified,
                };

                self.object_store.put_commit(&merge_commit).map_err(VersionError::ObjectStore)?;

                // Update HEAD to point to merge commit
                if let Some(head_name) = self.get_head_branch()? {
                    let mut view = self.ref_store.get_branch(&head_name).map_err(VersionError::RefStore)?;
                    view.set_target(merge_commit_id.as_ulid());
                    self.ref_store.put_branch(&view).map_err(VersionError::RefStore)?;
                }

                // Return result based on whether conflicts were detected
                if conflicts.is_empty() {
                    Ok(MergeResult::Clean { commit_id: merge_commit_id })
                } else {
                    Ok(MergeResult::Conflicts { commit_id: merge_commit_id, conflicts })
                }
            }
        }
    }

    /// List all branches with details
    pub fn list_branches_detailed(&self) -> Result<Vec<BranchInfo>, VersionError> {
        let branches = self.list_branches()?;
        let head = self.get_head_branch()?;
        let mut result = Vec::new();

        for name in branches {
            let view = self.ref_store.get_branch(&name).map_err(VersionError::RefStore)?;
            let is_head = head.as_ref() == Some(&name);
            result.push(BranchInfo {
                name,
                target: view.target(),
                is_head,
            });
        }

        Ok(result)
    }

    /// List all tags with details
    pub fn list_tags_detailed(&self) -> Result<Vec<TagInfo>, VersionError> {
        let tags = self.ref_store.list_tags().map_err(VersionError::RefStore)?;
        let mut result = Vec::new();

        for name in tags {
            if let Ok(view) = self.ref_store.get_tag(&name) {
                result.push(TagInfo {
                    name,
                    target: view.target(),
                    message: view.message().map(String::from),
                });
            }
        }

        Ok(result)
    }

    /// Create an annotated tag
    pub fn create_tag(&self, name: &str, commit_id: Option<CommitId>, message: Option<&str>) -> Result<ViewName, VersionError> {
        let view_name = ViewName::new(name);

        // Check if tag already exists
        if self.ref_store.has_tag(&view_name) {
            return Err(VersionError::TagAlreadyExists(name.to_string()));
        }

        // Get target commit
        let target = match commit_id {
            Some(id) => id.as_ulid(),
            None => {
                // Use current HEAD if no commit specified
                match self.get_head_commit()? {
                    Some(commit) => commit.id.as_ulid(),
                    None => return Err(VersionError::NoHeadCommit),
                }
            }
        };

        // Create tag
        let view = match message {
            Some(msg) => View::tag_with_message(name, target, msg.to_string()),
            None => View::tag(name, target),
        };
        self.ref_store.put_tag(&view).map_err(VersionError::RefStore)?;

        Ok(view_name)
    }

    /// Delete a tag
    pub fn delete_tag(&self, name: &str) -> Result<(), VersionError> {
        let view_name = ViewName::new(name);

        if !self.ref_store.has_tag(&view_name) {
            return Err(VersionError::TagNotFound(name.to_string()));
        }

        self.ref_store.delete_tag(&view_name).map_err(VersionError::RefStore)?;
        Ok(())
    }

    /// Resolve a conflict using the specified resolution strategy
    ///
    /// Creates a new commit that applies the conflict resolution to the merge commit.
    pub fn resolve_conflict(
        &self,
        commit_id: CommitId,
        conflict_id: &Ulid,
        resolution: ConflictResolution,
    ) -> Result<CommitId, VersionError> {
        // Get the commit with conflicts
        let commit = self.object_store.get_commit(commit_id)
            .map_err(VersionError::ObjectStore)?;

        // Find the conflict by block_id
        let conflict_block_id = conflict_id;

        // Create new commit with resolution applied
        let new_commit_id = CommitId::new(Ulid::new());

        let (blocks_added, blocks_removed, blocks_modified) = match resolution {
            ConflictResolution::UseOurs => {
                // Keep our version - remove their changes to the conflicting block
                let blocks_added: Vec<Ulid> = commit.blocks_added.clone();
                let blocks_removed: Vec<Ulid> = commit.blocks_removed.clone();
                let mut blocks_modified = commit.blocks_modified.clone();
                // Remove the conflict block from modified if we're using ours
                blocks_modified.retain(|b| b != conflict_block_id);
                (blocks_added, blocks_removed, blocks_modified)
            }
            ConflictResolution::UseTheirs => {
                // Take their version
                let blocks_added: Vec<Ulid> = commit.blocks_added.clone();
                let blocks_removed: Vec<Ulid> = commit.blocks_removed.clone();
                let mut blocks_modified = commit.blocks_modified.clone();
                // Remove the conflict block from modified if we're using theirs
                blocks_modified.retain(|b| b != conflict_block_id);
                (blocks_added, blocks_removed, blocks_modified)
            }
            ConflictResolution::Manual(block_id) => {
                // Use a specific block
                let blocks_added: Vec<Ulid> = commit.blocks_added.clone();
                let blocks_removed: Vec<Ulid> = commit.blocks_removed.clone();
                let mut blocks_modified = commit.blocks_modified.clone();
                blocks_modified.retain(|b| b != conflict_block_id);
                blocks_modified.push(block_id);
                (blocks_added, blocks_removed, blocks_modified)
            }
        };

        let new_commit = Commit {
            id: new_commit_id,
            structure_snapshot: commit.structure_snapshot.clone(),
            parents: commit.parents.clone(),
            author: super::AgentId::new("system"),
            message: format!("Resolved conflict for block {} using {:?}", conflict_block_id, resolution),
            created_at: chrono::Utc::now(),
            blocks_added,
            blocks_removed,
            blocks_modified,
        };

        self.object_store.put_commit(&new_commit).map_err(VersionError::ObjectStore)?;

        // Update HEAD to point to new commit
        if let Some(head_name) = self.get_head_branch()? {
            let mut view = self.ref_store.get_branch(&head_name).map_err(VersionError::RefStore)?;
            view.set_target(new_commit_id.as_ulid());
            self.ref_store.put_branch(&view).map_err(VersionError::RefStore)?;
        }

        Ok(new_commit_id)
    }
}

/// Information about a tag
#[derive(Debug)]
pub struct TagInfo {
    pub name: ViewName,
    pub target: ulid::Ulid,
    pub message: Option<String>,
}

/// Error types for version operations
#[derive(Debug, thiserror::Error)]
pub enum VersionError {
    #[error("Object store error: {0}")]
    ObjectStore(#[from] super::repository::ObjectStoreError),

    #[error("Ref store error: {0}")]
    RefStore(#[from] super::repository::RefStoreError),

    #[error("Working set store error: {0}")]
    WorkingSetStore(#[from] super::repository::WorkingSetStoreError),

    #[error("Nothing to commit")]
    NothingToCommit,

    #[error("Block not found: {0}")]
    BlockNotFound(String),

    #[error("Commit not found: {0}")]
    CommitNotFound(String),

    #[error("Branch not found: {0}")]
    BranchNotFound(String),

    #[error("Branch already exists: {0}")]
    BranchAlreadyExists(String),

    #[error("Cannot delete current HEAD branch")]
    CannotDeleteHead,

    #[error("No HEAD commit to merge from")]
    NoHeadCommit,

    #[error("Tag not found: {0}")]
    TagNotFound(String),

    #[error("Tag already exists: {0}")]
    TagAlreadyExists(String),
}

/// Merge strategy for conflict resolution
#[derive(Debug, Clone, Copy)]
pub enum MergeStrategy {
    /// Keep our changes
    Ours,
    /// Take their changes
    Theirs,
    /// Create merge commit
    Merge,
}

/// Result of a merge operation
#[derive(Debug)]
pub enum MergeResult {
    /// Merge successful with no conflicts
    Clean { commit_id: CommitId },
    /// Merge with conflicts that require manual resolution
    Conflicts { commit_id: CommitId, conflicts: Vec<ConflictInfo> },
    /// Fast-forward was successful
    FastForward,
    /// Branch was already merged (branch is ancestor of HEAD)
    AlreadyMerged,
}

/// Conflict resolution strategy
#[derive(Debug, Clone)]
pub enum ConflictResolution {
    /// Use our version (the current HEAD)
    UseOurs,
    /// Use their version (the merged branch)
    UseTheirs,
    /// Use a specific block manually
    Manual(Ulid),
}

/// Information about a branch
#[derive(Debug)]
pub struct BranchInfo {
    pub name: ViewName,
    pub target: ulid::Ulid,
    pub is_head: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use super::BlockDelta;

    fn create_test_repo(temp_dir: &TempDir) -> VersionRepo {
        let repo = VersionRepo::new(temp_dir.path());
        repo.init().unwrap();
        repo
    }

    fn create_test_block_delta(block_id: Ulid) -> BlockDelta {
        BlockDelta::Created {
            block_id,
            title: "Test Block".to_string(),
            content: "Test content".to_string(),
            block_type: "note".to_string(),
        }
    }

    #[test]
    fn test_repo_init() {
        let temp = TempDir::new().unwrap();
        let repo = create_test_repo(&temp);

        assert!(repo.object_store.is_initialized());
        assert!(repo.ref_store.is_initialized());
    }

    #[test]
    fn test_stage_and_commit() {
        let temp = TempDir::new().unwrap();
        let repo = create_test_repo(&temp);

        // Create initial branch first (needed for commit)
        repo.create_branch("main").unwrap();
        repo.checkout("main").unwrap();

        // Stage a block
        let block_id = Ulid::new();
        let delta = create_test_block_delta(block_id);
        repo.add_block(block_id, delta).unwrap();

        // Verify working set has staged block
        let ws = repo.get_working_set().unwrap();
        assert_eq!(ws.staged_blocks_count(), 1);
        assert!(ws.is_block_staged(&block_id));

        // Commit
        let commit_id = repo.commit("Initial commit", AgentId::new("test")).unwrap();
        assert!(!commit_id.to_string().is_empty());

        // Verify working set is cleared after commit
        let ws = repo.get_working_set().unwrap();
        assert!(ws.is_empty());
    }

    #[test]
    fn test_log_empty() {
        let temp = TempDir::new().unwrap();
        let repo = create_test_repo(&temp);

        let commits = repo.log().unwrap();
        assert!(commits.is_empty());
    }

    #[test]
    fn test_log_after_commit() {
        let temp = TempDir::new().unwrap();
        let repo = create_test_repo(&temp);

        repo.create_branch("main").unwrap();
        repo.checkout("main").unwrap();

        let block_id = Ulid::new();
        repo.stage(&block_id).unwrap();
        let commit_id = repo.commit("Initial commit", AgentId::new("test")).unwrap();

        let commits = repo.log().unwrap();
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].id, commit_id);
        assert_eq!(commits[0].message, "Initial commit");
    }

    #[test]
    fn test_create_branch() {
        let temp = TempDir::new().unwrap();
        let repo = create_test_repo(&temp);

        repo.create_branch("main").unwrap();
        repo.checkout("main").unwrap();

        let branch_name = repo.create_branch("feature").unwrap();
        assert_eq!(branch_name.as_str(), "feature");

        let branches = repo.list_branches().unwrap();
        assert!(branches.iter().any(|b| b.as_str() == "feature"));
    }

    #[test]
    fn test_create_branch_already_exists() {
        let temp = TempDir::new().unwrap();
        let repo = create_test_repo(&temp);

        repo.create_branch("main").unwrap();
        repo.checkout("main").unwrap();
        repo.create_branch("feature").unwrap();

        let result = repo.create_branch("feature");
        assert!(matches!(result, Err(VersionError::BranchAlreadyExists(_))));
    }

    #[test]
    fn test_checkout() {
        let temp = TempDir::new().unwrap();
        let repo = create_test_repo(&temp);

        repo.create_branch("main").unwrap();
        repo.checkout("main").unwrap();
        repo.create_branch("feature").unwrap();

        repo.checkout("feature").unwrap();

        let head = repo.get_head_branch().unwrap();
        assert_eq!(head.unwrap().as_str(), "feature");
    }

    #[test]
    fn test_checkout_nonexistent() {
        let temp = TempDir::new().unwrap();
        let repo = create_test_repo(&temp);

        let result = repo.checkout("nonexistent");
        assert!(matches!(result, Err(VersionError::BranchNotFound(_))));
    }

    #[test]
    fn test_delete_branch() {
        let temp = TempDir::new().unwrap();
        let repo = create_test_repo(&temp);

        repo.create_branch("main").unwrap();
        repo.checkout("main").unwrap();
        repo.create_branch("feature").unwrap();

        repo.delete_branch("feature", false).unwrap();

        let branches = repo.list_branches().unwrap();
        assert!(!branches.iter().any(|b| b.as_str() == "feature"));
    }

    #[test]
    fn test_delete_branch_cannot_delete_head() {
        let temp = TempDir::new().unwrap();
        let repo = create_test_repo(&temp);

        repo.create_branch("main").unwrap();
        repo.checkout("main").unwrap();

        let result = repo.delete_branch("main", false);
        assert!(matches!(result, Err(VersionError::CannotDeleteHead)));
    }

    #[test]
    fn test_create_and_delete_tag() {
        let temp = TempDir::new().unwrap();
        let repo = create_test_repo(&temp);

        repo.create_branch("main").unwrap();
        repo.checkout("main").unwrap();

        let block_id = Ulid::new();
        repo.stage(&block_id).unwrap();
        repo.commit("Initial", AgentId::new("test")).unwrap();

        let tag_name = repo.create_tag("v1.0.0", None, Some("Version 1.0.0")).unwrap();
        assert_eq!(tag_name.as_str(), "v1.0.0");

        repo.delete_tag("v1.0.0").unwrap();

        let tags = repo.list_tags_detailed().unwrap();
        assert!(!tags.iter().any(|t| t.name.as_str() == "v1.0.0"));
    }

    #[test]
    fn test_commit_nothing_to_commit() {
        let temp = TempDir::new().unwrap();
        let repo = create_test_repo(&temp);

        repo.create_branch("main").unwrap();
        repo.checkout("main").unwrap();

        let result = repo.commit("Empty commit", AgentId::new("test"));
        assert!(matches!(result, Err(VersionError::NothingToCommit)));
    }

    #[test]
    fn test_merge_fast_forward() {
        let temp = TempDir::new().unwrap();
        let repo = create_test_repo(&temp);

        // Create main and add a commit
        repo.create_branch("main").unwrap();
        repo.checkout("main").unwrap();
        let block_id = Ulid::new();
        repo.stage(&block_id).unwrap();
        repo.commit("Initial", AgentId::new("test")).unwrap();

        // Create feature branch with another commit
        repo.create_branch("feature").unwrap();
        repo.checkout("feature").unwrap();
        let block_id2 = Ulid::new();
        repo.stage(&block_id2).unwrap();
        repo.commit("Feature work", AgentId::new("test")).unwrap();

        // Switch back to main
        repo.checkout("main").unwrap();

        // Merge feature into main (should be fast-forward)
        let result = repo.merge("feature", MergeStrategy::Merge).unwrap();
        assert!(matches!(result, MergeResult::FastForward));

        // Verify main now points to feature's commit
        let head = repo.get_head_commit().unwrap().unwrap();
        assert_eq!(head.message, "Feature work");
    }

    #[test]
    fn test_merge_already_merged() {
        let temp = TempDir::new().unwrap();
        let repo = create_test_repo(&temp);

        repo.create_branch("main").unwrap();
        repo.checkout("main").unwrap();
        let block_id = Ulid::new();
        repo.stage(&block_id).unwrap();
        repo.commit("Initial", AgentId::new("test")).unwrap();

        // Create and merge feature
        repo.create_branch("feature").unwrap();
        repo.checkout("feature").unwrap();
        let block_id2 = Ulid::new();
        repo.stage(&block_id2).unwrap();
        repo.commit("Feature", AgentId::new("test")).unwrap();

        repo.checkout("main").unwrap();
        repo.merge("feature", MergeStrategy::Merge).unwrap();

        // Try to merge again - should say already merged
        let result = repo.merge("feature", MergeStrategy::Merge).unwrap();
        assert!(matches!(result, MergeResult::AlreadyMerged));
    }

    #[test]
    fn test_working_set_persistence() {
        let temp = TempDir::new().unwrap();
        let repo = create_test_repo(&temp);

        repo.create_branch("main").unwrap();
        repo.checkout("main").unwrap();

        // Stage some blocks
        let block_id1 = Ulid::new();
        let block_id2 = Ulid::new();
        repo.stage(&block_id1).unwrap();
        repo.stage(&block_id2).unwrap();

        // Create a new repo instance pointing to same directory
        let repo2 = VersionRepo::new(temp.path());
        let ws = repo2.get_working_set().unwrap();
        assert_eq!(ws.staged_blocks_count(), 2);
    }
}