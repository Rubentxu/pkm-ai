//! Repository traits
//!
//! Defines the port (trait) abstractions for repository operations.

use crate::models::{Block, BlockType, Edge, LinkType};
use crate::NexusError;
use async_trait::async_trait;
use ulid::Ulid;

/// Block repository trait for abstracting storage operations
#[allow(dead_code)]
#[async_trait]
pub trait BlockRepositoryTrait: Send + Sync {
    /// Create a new block
    async fn create(&self, block: Block) -> Result<(), NexusError>;

    /// Get a block by ID
    async fn get(&self, id: &Ulid) -> Result<Option<Block>, NexusError>;

    /// Update a block
    async fn update(&self, block: Block) -> Result<(), NexusError>;

    /// Delete a block
    async fn delete(&self, id: &Ulid) -> Result<(), NexusError>;

    /// List blocks by type
    async fn list_by_type(&self, block_type: BlockType) -> Result<Vec<Block>, NexusError>;

    /// Search blocks by tags
    async fn search_by_tags(&self, tags: &[String]) -> Result<Vec<Block>, NexusError>;

    /// Full-text search in content
    async fn search_content(&self, query: &str) -> Result<Vec<Block>, NexusError>;

    /// List all blocks
    async fn list_all(&self) -> Result<Vec<Block>, NexusError>;
}

/// Edge repository trait for abstracting storage operations
#[allow(dead_code)]
#[async_trait]
pub trait EdgeRepositoryTrait: Send + Sync {
    /// Create a new edge
    async fn create(&self, edge: Edge) -> Result<(), NexusError>;

    /// Get an edge by ID
    async fn get(&self, id: &Ulid) -> Result<Option<Edge>, NexusError>;

    /// Delete an edge by ID
    async fn delete(&self, id: &Ulid) -> Result<(), NexusError>;

    /// Delete a specific edge between source and target blocks
    async fn delete_by_source_target(&self, source: &Ulid, target: &Ulid) -> Result<(), NexusError>;

    /// Delete all edges originating from a block
    async fn delete_all_from(&self, source: &Ulid) -> Result<(), NexusError>;

    /// Delete all edges targeting a block
    async fn delete_all_to(&self, target: &Ulid) -> Result<(), NexusError>;

    /// Delete all edges associated with a block (both incoming and outgoing)
    async fn delete_for_block(&self, block_id: &Ulid) -> Result<(), NexusError>;

    /// Get edges by link type
    async fn list_by_type(&self, link_type: LinkType) -> Result<Vec<Edge>, NexusError>;

    /// Get outgoing edges from a block
    async fn outgoing_from(&self, block_id: &Ulid) -> Result<Vec<Edge>, NexusError>;

    /// Get incoming edges to a block
    async fn incoming_to(&self, block_id: &Ulid) -> Result<Vec<Edge>, NexusError>;

    /// List all edges
    async fn list_all(&self) -> Result<Vec<Edge>, NexusError>;

    /// Update an edge
    async fn update(&self, edge: Edge) -> Result<(), NexusError>;
}
