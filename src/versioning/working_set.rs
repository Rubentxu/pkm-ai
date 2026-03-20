//! WorkingSet: staging area for atomic commits
//!
//! Tracks staged changes (blocks and edges) before they are committed.
//! Similar to Git's staging area (index).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use ulid::Ulid;

use super::{AgentId, BlockDelta, EdgeDelta};

/// WorkingSetId: newtype wrapper around Ulid for working set identification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkingSetId(Ulid);

impl WorkingSetId {
    #[must_use]
    pub const fn new(id: Ulid) -> Self {
        Self(id)
    }

    #[must_use]
    pub const fn as_ulid(self) -> Ulid {
        self.0
    }
}

impl Default for WorkingSetId {
    fn default() -> Self {
        Self(Ulid::new())
    }
}

impl std::fmt::Display for WorkingSetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Ulid> for WorkingSetId {
    fn from(id: Ulid) -> Self {
        Self(id)
    }
}

impl From<WorkingSetId> for Ulid {
    fn from(id: WorkingSetId) -> Self {
        id.0
    }
}

/// Operation: a recorded change in the working set
///
/// Each operation is timestamped and contains a delta describing the change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    /// Unique operation identifier
    pub id: Ulid,
    /// The delta that was applied
    pub delta: OperationDelta,
    /// When the operation was recorded
    pub timestamp: DateTime<Utc>,
}

impl Operation {
    /// Create a new operation with an auto-generated ID
    pub fn new(delta: OperationDelta) -> Self {
        Self {
            id: Ulid::new(),
            delta,
            timestamp: Utc::now(),
        }
    }
}

/// OperationDelta: wrapper for either BlockDelta or EdgeDelta
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum OperationDelta {
    Block(BlockDelta),
    Edge(EdgeDelta),
}

impl From<BlockDelta> for OperationDelta {
    fn from(delta: BlockDelta) -> Self {
        Self::Block(delta)
    }
}

impl From<EdgeDelta> for OperationDelta {
    fn from(delta: EdgeDelta) -> Self {
        Self::Edge(delta)
    }
}

/// WorkingSet: staging area for knowledge changes
///
/// Tracks staged blocks and edges along with an operation log for replay.
/// The operation log enables reverting or replaying changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingSet {
    /// Unique working set identifier
    pub id: WorkingSetId,
    /// Agent owning this working set
    author: AgentId,
    /// Staged block changes (block_id -> delta)
    staged_blocks: BTreeMap<Ulid, BlockDelta>,
    /// Staged edge changes ((source, target) -> delta)
    staged_edges: BTreeMap<(Ulid, Ulid), EdgeDelta>,
    /// Removed blocks pending commit
    #[serde(default)]
    removed_blocks: Vec<Ulid>,
    /// Removed edges pending commit
    #[serde(default)]
    removed_edges: Vec<(Ulid, Ulid)>,
    /// Operation log for replay/undo
    operations: Vec<Operation>,
    /// When the working set was created
    created_at: DateTime<Utc>,
    /// Last modification time
    updated_at: DateTime<Utc>,
}

impl WorkingSet {
    /// Create a new empty working set with auto-generated ID
    #[must_use]
    pub fn new(author: AgentId) -> Self {
        let now = Utc::now();
        Self {
            id: WorkingSetId::default(),
            author,
            staged_blocks: BTreeMap::new(),
            staged_edges: BTreeMap::new(),
            removed_blocks: Vec::new(),
            removed_edges: Vec::new(),
            operations: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a new working set with a specific ID
    #[must_use]
    pub fn with_id(id: WorkingSetId, author: AgentId) -> Self {
        let now = Utc::now();
        Self {
            id,
            author,
            staged_blocks: BTreeMap::new(),
            staged_edges: BTreeMap::new(),
            removed_blocks: Vec::new(),
            removed_edges: Vec::new(),
            operations: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Get the working set ID
    #[must_use]
    pub const fn id(&self) -> WorkingSetId {
        self.id
    }

    /// Stage a block delta
    pub fn stage_block(&mut self, delta: BlockDelta) {
        let block_id = delta.block_id();
        self.staged_blocks.insert(block_id, delta.clone());
        self.operations.push(Operation::new(delta.into()));
        self.updated_at = Utc::now();
    }

    /// Unstage a block by ID
    ///
    /// Returns the undelted delta if one was staged
    pub fn unstage_block(&mut self, block_id: &Ulid) -> Option<BlockDelta> {
        let removed = self.staged_blocks.remove(block_id);
        if removed.is_some() {
            self.updated_at = Utc::now();
        }
        removed
    }

    /// Stage an edge delta
    pub fn stage_edge(&mut self, delta: EdgeDelta) {
        let key = (delta.source(), delta.target());
        self.staged_edges.insert(key, delta.clone());
        self.operations.push(Operation::new(delta.into()));
        self.updated_at = Utc::now();
    }

    /// Unstage an edge by source and target
    ///
    /// Returns the undelted delta if one was staged
    pub fn unstage_edge(&mut self, source: &Ulid, target: &Ulid) -> Option<EdgeDelta> {
        let key = (*source, *target);
        let removed = self.staged_edges.remove(&key);
        if removed.is_some() {
            self.updated_at = Utc::now();
        }
        removed
    }

    /// Mark a block as removed (pending deletion)
    pub fn mark_block_removed(&mut self, block_id: Ulid) {
        if !self.removed_blocks.contains(&block_id) {
            self.removed_blocks.push(block_id);
            self.updated_at = Utc::now();
        }
    }

    /// Unmark a block as removed
    ///
    /// Returns true if the block was removed from the list
    pub fn unmark_block_removed(&mut self, block_id: &Ulid) -> bool {
        let initial_len = self.removed_blocks.len();
        self.removed_blocks.retain(|id| id != block_id);
        if self.removed_blocks.len() != initial_len {
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// Mark an edge as removed (pending deletion)
    pub fn mark_edge_removed(&mut self, source: Ulid, target: Ulid) {
        let edge = (source, target);
        if !self.removed_edges.contains(&edge) {
            self.removed_edges.push(edge);
            self.updated_at = Utc::now();
        }
    }

    /// Unmark an edge as removed
    ///
    /// Returns true if the edge was removed from the list
    pub fn unmark_edge_removed(&mut self, source: &Ulid, target: &Ulid) -> bool {
        let edge = (*source, *target);
        let initial_len = self.removed_edges.len();
        self.removed_edges.retain(|e| e != &edge);
        if self.removed_edges.len() != initial_len {
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// Check if there are any staged changes
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.staged_blocks.is_empty()
            && self.staged_edges.is_empty()
            && self.removed_blocks.is_empty()
            && self.removed_edges.is_empty()
    }

    /// Get the number of staged blocks
    #[must_use]
    pub fn staged_blocks_count(&self) -> usize {
        self.staged_blocks.len()
    }

    /// Get the number of staged edges
    #[must_use]
    pub fn staged_edges_count(&self) -> usize {
        self.staged_edges.len()
    }

    /// Get the number of removed blocks
    #[must_use]
    pub fn removed_blocks_count(&self) -> usize {
        self.removed_blocks.len()
    }

    /// Get the number of removed edges
    #[must_use]
    pub fn removed_edges_count(&self) -> usize {
        self.removed_edges.len()
    }

    /// Get all staged block deltas
    #[must_use]
    pub fn staged_blocks(&self) -> &BTreeMap<Ulid, BlockDelta> {
        &self.staged_blocks
    }

    /// Get all staged edge deltas
    #[must_use]
    pub fn staged_edges(&self) -> &BTreeMap<(Ulid, Ulid), EdgeDelta> {
        &self.staged_edges
    }

    /// Get removed blocks
    #[must_use]
    pub fn removed_blocks(&self) -> &[Ulid] {
        &self.removed_blocks
    }

    /// Get removed edges
    #[must_use]
    pub fn removed_edges(&self) -> &[(Ulid, Ulid)] {
        &self.removed_edges
    }

    /// Get the operation log
    #[must_use]
    pub fn operations(&self) -> &[Operation] {
        &self.operations
    }

    /// Get the author
    #[must_use]
    pub const fn author(&self) -> &AgentId {
        &self.author
    }

    /// Get creation timestamp
    #[must_use]
    pub const fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Get last update timestamp
    #[must_use]
    pub const fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Clear all staged changes
    pub fn clear(&mut self) {
        self.staged_blocks.clear();
        self.staged_edges.clear();
        self.removed_blocks.clear();
        self.removed_edges.clear();
        self.updated_at = Utc::now();
    }

    /// Get a staged block delta by ID
    #[must_use]
    pub fn get_staged_block(&self, block_id: &Ulid) -> Option<&BlockDelta> {
        self.staged_blocks.get(block_id)
    }

    /// Get a staged edge delta by source and target
    #[must_use]
    pub fn get_staged_edge(&self, source: &Ulid, target: &Ulid) -> Option<&EdgeDelta> {
        self.staged_edges.get(&(*source, *target))
    }

    /// Check if a block is staged
    #[must_use]
    pub fn is_block_staged(&self, block_id: &Ulid) -> bool {
        self.staged_blocks.contains_key(block_id)
    }

    /// Check if a block is marked for removal
    #[must_use]
    pub fn is_block_removed(&self, block_id: &Ulid) -> bool {
        self.removed_blocks.contains(block_id)
    }

    /// Replay operations to reconstruct deltas
    ///
    /// Returns an iterator over all operation deltas in order
    pub fn replay_operations(&self) -> impl Iterator<Item = &OperationDelta> {
        self.operations.iter().map(|op| &op.delta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_block_delta() -> BlockDelta {
        BlockDelta::Created {
            block_id: Ulid::new(),
            title: "Test Block".to_string(),
            content: "Test content".to_string(),
            block_type: "note".to_string(),
        }
    }

    fn create_test_edge_delta() -> EdgeDelta {
        EdgeDelta::Created {
            source: Ulid::new(),
            target: Ulid::new(),
            relation: "follows".to_string(),
        }
    }

    #[test]
    fn test_working_set_creation() {
        let author = AgentId::new("test-author");
        let ws = WorkingSet::new(author.clone());

        assert_eq!(ws.author().as_str(), "test-author");
        assert!(ws.is_empty());
        assert_eq!(ws.staged_blocks_count(), 0);
        assert_eq!(ws.staged_edges_count(), 0);
        assert_eq!(ws.removed_blocks_count(), 0);
        assert_eq!(ws.removed_edges_count(), 0);
    }

    #[test]
    fn test_working_set_with_id() {
        let author = AgentId::new("test-author");
        let id = WorkingSetId::new(Ulid::new());
        let ws = WorkingSet::with_id(id, author.clone());

        assert_eq!(ws.id(), id);
        assert_eq!(ws.author().as_str(), "test-author");
    }

    #[test]
    fn test_stage_block() {
        let author = AgentId::new("test");
        let mut ws = WorkingSet::new(author);
        let delta = create_test_block_delta();
        let block_id = delta.block_id();

        ws.stage_block(delta);

        assert!(!ws.is_empty());
        assert_eq!(ws.staged_blocks_count(), 1);
        assert!(ws.is_block_staged(&block_id));
        assert_eq!(ws.get_staged_block(&block_id).unwrap().block_id(), block_id);
    }

    #[test]
    fn test_unstage_block() {
        let author = AgentId::new("test");
        let mut ws = WorkingSet::new(author);
        let delta = create_test_block_delta();
        let block_id = delta.block_id();

        ws.stage_block(delta);
        let removed = ws.unstage_block(&block_id);

        assert!(removed.is_some());
        assert!(ws.is_empty());
        assert!(!ws.is_block_staged(&block_id));
    }

    #[test]
    fn test_stage_edge() {
        let author = AgentId::new("test");
        let mut ws = WorkingSet::new(author);
        let delta = create_test_edge_delta();
        let source = delta.source();
        let target = delta.target();

        ws.stage_edge(delta);

        assert_eq!(ws.staged_edges_count(), 1);
        assert!(ws.get_staged_edge(&source, &target).is_some());
    }

    #[test]
    fn test_operations_log() {
        let author = AgentId::new("test");
        let mut ws = WorkingSet::new(author);

        ws.stage_block(create_test_block_delta());
        ws.stage_edge(create_test_edge_delta());

        assert_eq!(ws.operations().len(), 2);
    }

    #[test]
    fn test_clear_working_set() {
        let author = AgentId::new("test");
        let mut ws = WorkingSet::new(author);

        ws.stage_block(create_test_block_delta());
        ws.stage_edge(create_test_edge_delta());

        ws.clear();

        assert!(ws.is_empty());
        assert_eq!(ws.staged_blocks_count(), 0);
        assert_eq!(ws.staged_edges_count(), 0);
        assert_eq!(ws.removed_blocks_count(), 0);
        assert_eq!(ws.removed_edges_count(), 0);
    }

    #[test]
    fn test_replay_operations() {
        let author = AgentId::new("test");
        let mut ws = WorkingSet::new(author);

        ws.stage_block(create_test_block_delta());
        ws.stage_edge(create_test_edge_delta());

        let deltas: Vec<_> = ws.replay_operations().collect();
        assert_eq!(deltas.len(), 2);
    }

    #[test]
    fn test_mark_block_removed() {
        let author = AgentId::new("test");
        let mut ws = WorkingSet::new(author);
        let block_id = Ulid::new();

        ws.mark_block_removed(block_id);

        assert!(ws.is_block_removed(&block_id));
        assert_eq!(ws.removed_blocks_count(), 1);
    }

    #[test]
    fn test_unmark_block_removed() {
        let author = AgentId::new("test");
        let mut ws = WorkingSet::new(author);
        let block_id = Ulid::new();

        ws.mark_block_removed(block_id);
        let removed = ws.unmark_block_removed(&block_id);

        assert!(removed);
        assert!(!ws.is_block_removed(&block_id));
        assert_eq!(ws.removed_blocks_count(), 0);
    }

    #[test]
    fn test_mark_edge_removed() {
        let author = AgentId::new("test");
        let mut ws = WorkingSet::new(author);
        let source = Ulid::new();
        let target = Ulid::new();

        ws.mark_edge_removed(source, target);

        assert!(ws.removed_edges().contains(&(source, target)));
        assert_eq!(ws.removed_edges_count(), 1);
    }

    #[test]
    fn test_removed_blocks_no_duplicates() {
        let author = AgentId::new("test");
        let mut ws = WorkingSet::new(author);
        let block_id = Ulid::new();

        ws.mark_block_removed(block_id);
        ws.mark_block_removed(block_id); // Duplicate

        assert_eq!(ws.removed_blocks_count(), 1);
    }

    #[test]
    fn test_removed_edges_no_duplicates() {
        let author = AgentId::new("test");
        let mut ws = WorkingSet::new(author);
        let source = Ulid::new();
        let target = Ulid::new();

        ws.mark_edge_removed(source, target);
        ws.mark_edge_removed(source, target); // Duplicate

        assert_eq!(ws.removed_edges_count(), 1);
    }
}
