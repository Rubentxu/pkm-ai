//! AI integration module
//!
//! AI as an active weaver of the knowledge graph

pub mod mcp;
pub mod embeddings;
pub mod link_suggester;
pub mod ghost_detector;
pub mod structure_generator;
pub mod semantic_clustering;

// Re-export commonly used types
pub use link_suggester::LinkSuggester;
pub use ghost_detector::GhostDetector;
