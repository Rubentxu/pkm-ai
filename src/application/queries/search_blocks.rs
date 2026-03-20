//! SearchBlocks Query
//!
//! Query to search for blocks.

use crate::domain::models::Block;
use crate::ports::block_repository::BlockRepository;
use crate::ports::repository_error::RepositoryResult;
use std::sync::Arc;

/// Query to search blocks
#[derive(Debug)]
pub struct SearchBlocksQuery {
    pub query: String,
}

/// SearchBlocksUseCase: Application service for searching blocks
pub struct SearchBlocksUseCase<R: BlockRepository> {
    block_repo: Arc<R>,
}

impl<R: BlockRepository> SearchBlocksUseCase<R> {
    /// Create a new use case instance
    pub fn new(block_repo: Arc<R>) -> Self {
        Self { block_repo }
    }

    /// Execute the query
    pub async fn execute(&self, query: SearchBlocksQuery) -> RepositoryResult<Vec<Block>> {
        self.block_repo.search(&query.query).await
    }
}
