//! Nexus-Grafo: High-performance PKM with Zettelkasten + SurrealDB + AI
//!
//! A Personal Knowledge Management system treating ORDER and STRUCTURE as first-class citizens.
//! Features:
//! - Block-Atom Model with ULID
//! - Structural Spine (Folgezettel digital)
//! - Smart Sections with semantic centroids
//! - Document Synthesis as Priority #1
//! - AI as active weaver of the knowledge graph
//! - Typst for professional document generation

use clap::Parser;
use cli::Cli;
use thiserror::Error;

mod db;
mod models;
mod cli;
mod ai;
mod synthesis;
mod spine;
mod tui;
mod versioning;

#[derive(Debug, Error)]
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging only if verbose mode is enabled
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive(tracing::Level::INFO.into())
            )
            .try_init()
            .ok();
    }

    cli.execute().await
}
