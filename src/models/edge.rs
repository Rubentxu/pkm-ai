//! Edge Model: Typed relationships between Blocks
//!
//! Edges represent the connections in the knowledge graph.
//! Supports sequence_weight for flexible ordering (Folgezettel digital).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::models::FractionalIndex;

/// Link types in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum LinkType {
    // Classic Zettelkasten links
    Extends,       // Elaborates on idea
    Refines,       // Improves precision
    Contradicts,   // Opposes or challenges
    Questions,     // Raises questions
    Supports,      // Provides evidence
    References,    // Cites or mentions
    Related,       // General association

    // Similarity
    SimilarTo,

    // Structural links (Document Synthesis)
    SectionOf,       // This block is section of Structure Note
    SubsectionOf,    // This block is subsection
    OrderedChild,    // Child with explicit order
    Next,            // Structural Spine: deterministic sequence
    NextSibling,     // Next sibling in sequence
    FirstChild,      // First child of parent

    // Hierarchy
    Contains,
    Parent,

    // AI-suggested links
    AiSuggested,
}

/// An edge connecting two blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    /// Edge ID (ULID) (stored as 'ulid' in DB)
    #[serde(rename = "ulid")]
    pub id: Ulid,

    /// Source block (stored as 'src' in DB)
    #[serde(rename = "src")]
    pub from: Ulid,

    /// Target block (stored as 'dst' in DB)
    #[serde(rename = "dst")]
    pub to: Ulid,

    /// Link type
    pub link_type: LinkType,

    /// Sequence weight for flexible ordering using FractionalIndex
    /// Uses lexicographic strings for never-degrading order
    #[serde(default)]
    pub sequence_weight: FractionalIndex,

    /// Context where this link was created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,

    /// AI justification for suggested links
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_justification: Option<String>,

    /// Confidence score (for AI-suggested links)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Is this link verified by human?
    #[serde(default)]
    pub verified: bool,
}

#[allow(dead_code)]
impl Edge {
    /// Create a new edge
    pub fn new(from: Ulid, to: Ulid, link_type: LinkType) -> Self {
        Self {
            id: Ulid::new(),
            from,
            to,
            link_type,
            sequence_weight: FractionalIndex::first(),
            context: None,
            ai_justification: None,
            confidence: None,
            created_at: Utc::now(),
            verified: false,
        }
    }

    /// Create a structural spine link (Next) with a position between two existing indices
    pub fn next_in_sequence_between(from: Ulid, to: Ulid, before: &FractionalIndex, after: &FractionalIndex) -> Self {
        let mut edge = Self::new(from, to, LinkType::Next);
        edge.sequence_weight = FractionalIndex::between(before, after);
        edge
    }

    /// Create a structural spine link (Next) at the end of a sequence
    pub fn next_in_sequence_after(from: Ulid, to: Ulid, last: &FractionalIndex) -> Self {
        let mut edge = Self::new(from, to, LinkType::Next);
        edge.sequence_weight = FractionalIndex::after_last(last);
        edge
    }

    /// Create a structural spine link (Next) as the first in a sequence
    pub fn next_in_sequence_first(from: Ulid, to: Ulid) -> Self {
        let mut edge = Self::new(from, to, LinkType::Next);
        edge.sequence_weight = FractionalIndex::first();
        edge
    }

    /// Create a section link with explicit position
    pub fn section_of_at(block: Ulid, structure: Ulid, position: FractionalIndex) -> Self {
        let mut edge = Self::new(block, structure, LinkType::SectionOf);
        edge.sequence_weight = position;
        edge
    }

    /// Set sequence weight using a FractionalIndex
    pub fn with_weight(mut self, weight: FractionalIndex) -> Self {
        self.sequence_weight = weight;
        self
    }

    /// Set AI justification
    pub fn with_ai_justification(mut self, justification: impl Into<String>) -> Self {
        self.ai_justification = Some(justification.into());
        self
    }

    /// Set confidence
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = Some(confidence.clamp(0.0, 1.0));
        self
    }

    /// Mark as verified by human
    pub fn verified(mut self) -> Self {
        self.verified = true;
        self
    }

    /// Check if this is a structural spine link
    pub fn is_structural_spine(&self) -> bool {
        matches!(self.link_type, LinkType::Next | LinkType::NextSibling)
    }

    /// Check if this is a synthesis link
    pub fn is_synthesis_link(&self) -> bool {
        matches!(
            self.link_type,
            LinkType::SectionOf | LinkType::SubsectionOf | LinkType::OrderedChild
        )
    }

    /// Get the ULID as string
    pub fn id_str(&self) -> String {
        self.id.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_creation() {
        let from = Ulid::new();
        let to = Ulid::new();
        let edge = Edge::new(from, to, LinkType::Extends);

        assert_eq!(edge.from, from);
        assert_eq!(edge.to, to);
        assert_eq!(edge.link_type, LinkType::Extends);
        assert_eq!(edge.sequence_weight, FractionalIndex::first());
    }

    #[test]
    fn test_sequence_weight() {
        let from = Ulid::new();
        let to = Ulid::new();
        let first = FractionalIndex::first();
        let second = FractionalIndex::after_last(&first);
        let edge = Edge::next_in_sequence_between(from, to, &first, &second);

        assert!(edge.is_structural_spine());
        assert!(edge.sequence_weight > first);
        assert!(edge.sequence_weight < second);
    }

    #[test]
    fn test_section_link() {
        let block = Ulid::new();
        let structure = Ulid::new();
        let position = FractionalIndex::between(&FractionalIndex::first(), &FractionalIndex::after_last(&FractionalIndex::first()));
        let edge = Edge::section_of_at(block, structure, position);

        assert_eq!(edge.link_type, LinkType::SectionOf);
        assert!(edge.is_synthesis_link());
    }
}
