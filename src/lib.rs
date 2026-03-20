//! Nexus-Grafo: High-performance PKM with Zettelkasten + SurrealDB + AI
//!
//! A Personal Knowledge Management system treating ORDER and STRUCTURE as first-class citizens.

pub mod models;
pub mod db;
pub mod ai;
pub mod synthesis;
pub mod spine;
pub mod versioning;
pub mod tui;

// Re-export versioning types for convenient access
pub use versioning::repository::{ObjectStore, RefStore, WorkingSetStore};
pub use versioning::{AgentId, BlockDelta, Commit, CommitId, View, ViewName, WorkingSet, WorkingSetId};

pub mod prelude {
    pub use anyhow::{Context, Result};
    pub use serde::{Deserialize, Serialize};
    pub use surrealdb::sql::Thing;
    pub use ulid::Ulid;
    pub use crate::{NexusError, NexusResult};
}

pub use prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum NexusError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Block not found: {0}")]
    BlockNotFound(String),

    #[error("Invalid block type: {0}")]
    InvalidBlockType(String),

    #[error("Invalid edge type: {0}")]
    InvalidEdgeType(String),

    #[error("Circular reference detected: {0} -> {1}")]
    CircularReference(String, String),

    #[error("Structural lint failed: {0}")]
    StructuralLint(String),

    #[error("Synthesis error: {0}")]
    Synthesis(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type NexusResult<T> = Result<T, NexusError>;