//! Ghost Nodes: Predictive placeholders for missing content
//!
//! Ghost nodes are AI-detected gaps in the knowledge structure.
//! They represent content that SHOULD exist but doesn't yet.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// Ghost node status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum GhostStatus {
    /// AI detected this gap
    Detected,
    /// User acknowledged the gap
    Acknowledged,
    /// User is working on filling it
    InProgress,
    /// Gap has been filled (ghost promoted to real block)
    Filled,
    /// User decided not to fill this gap
    Dismissed,
}

/// A Ghost Node representing a content gap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostNode {
    /// Ghost ID
    pub id: Ulid,

    /// What content should be here
    pub description: String,

    /// Why AI thinks this is needed
    pub ai_rationale: String,

    /// Confidence score
    pub confidence: f32,

    /// Suggested position (between which blocks)
    pub position_hint: PositionHint,

    /// Current status
    pub status: GhostStatus,

    /// Related blocks that triggered this detection
    pub trigger_blocks: Vec<Ulid>,

    /// Keywords that would belong in this content
    pub expected_keywords: Vec<String>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// If filled, reference to the new block
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filled_by: Option<Ulid>,
}

/// Position hint for where a ghost should be placed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionHint {
    /// Block before the ghost
    pub after: Option<Ulid>,

    /// Block after the ghost
    pub before: Option<Ulid>,

    /// Parent section (if applicable)
    pub parent_section: Option<Ulid>,

    /// Sequence weight (if structural)
    #[serde(default)]
    pub sequence_weight: f32,
}

#[allow(dead_code)]
impl GhostNode {
    /// Create a new ghost node
    pub fn new(description: impl Into<String>, confidence: f32) -> Self {
        Self {
            id: Ulid::new(),
            description: description.into(),
            ai_rationale: String::new(),
            confidence: confidence.clamp(0.0, 1.0),
            position_hint: PositionHint::default(),
            status: GhostStatus::Detected,
            trigger_blocks: Vec::new(),
            expected_keywords: Vec::new(),
            created_at: Utc::now(),
            filled_by: None,
        }
    }

    /// Set AI rationale
    pub fn with_rationale(mut self, rationale: impl Into<String>) -> Self {
        self.ai_rationale = rationale.into();
        self
    }

    /// Set position between two blocks
    pub fn between(mut self, after: Ulid, before: Ulid) -> Self {
        self.position_hint.after = Some(after);
        self.position_hint.before = Some(before);
        self
    }

    /// Set parent section
    pub fn in_section(mut self, section: Ulid) -> Self {
        self.position_hint.parent_section = Some(section);
        self
    }

    /// Add a trigger block
    pub fn triggered_by(mut self, block: Ulid) -> Self {
        self.trigger_blocks.push(block);
        self
    }

    /// Add expected keyword
    pub fn expecting_keyword(mut self, keyword: impl Into<String>) -> Self {
        self.expected_keywords.push(keyword.into());
        self
    }

    /// Mark as acknowledged
    pub fn acknowledge(&mut self) {
        self.status = GhostStatus::Acknowledged;
    }

    /// Mark as in progress
    pub fn start_filling(&mut self) {
        self.status = GhostStatus::InProgress;
    }

    /// Mark as filled with a real block
    pub fn fill_with(&mut self, block_id: Ulid) {
        self.status = GhostStatus::Filled;
        self.filled_by = Some(block_id);
    }

    /// Dismiss this ghost
    pub fn dismiss(&mut self) {
        self.status = GhostStatus::Dismissed;
    }

    /// Check if this ghost is actionable
    pub fn is_actionable(&self) -> bool {
        matches!(self.status, GhostStatus::Detected | GhostStatus::Acknowledged)
    }
}

impl Default for PositionHint {
    fn default() -> Self {
        Self {
            after: None,
            before: None,
            parent_section: None,
            sequence_weight: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ghost_creation() {
        let ghost = GhostNode::new("Missing explanation of Zarf principle", 0.87)
            .with_rationale("Multiple blocks reference Zarf but none explain it");

        assert_eq!(ghost.status, GhostStatus::Detected);
        assert_eq!(ghost.confidence, 0.87);
        assert!(ghost.is_actionable());
    }

    #[test]
    fn test_ghost_lifecycle() {
        let mut ghost = GhostNode::new("Test", 0.9);

        ghost.acknowledge();
        assert_eq!(ghost.status, GhostStatus::Acknowledged);

        ghost.start_filling();
        assert_eq!(ghost.status, GhostStatus::InProgress);

        let block_id = Ulid::new();
        ghost.fill_with(block_id);
        assert_eq!(ghost.status, GhostStatus::Filled);
        assert_eq!(ghost.filled_by, Some(block_id));
        assert!(!ghost.is_actionable());
    }

    #[test]
    fn test_position_hint() {
        let a = Ulid::new();
        let b = Ulid::new();
        let section = Ulid::new();

        let ghost = GhostNode::new("Test", 0.8)
            .between(a, b)
            .in_section(section);

        assert_eq!(ghost.position_hint.after, Some(a));
        assert_eq!(ghost.position_hint.before, Some(b));
        assert_eq!(ghost.position_hint.parent_section, Some(section));
    }
}
