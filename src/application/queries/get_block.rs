//! GetBlock Query
//!
//! Query to retrieve a block by ID.

use ulid::Ulid;
use crate::domain::models::Block;
use crate::ports::block_repository::BlockRepository;
use crate::ports::repository_error::RepositoryResult;
use std::sync::Arc;

/// Query to get a block
#[derive(Debug)]
pub struct GetBlockQuery {
    pub id: Ulid,
}

/// GetBlockUseCase: Application service for retrieving blocks
pub struct GetBlockUseCase<R: BlockRepository> {
    block_repo: Arc<R>,
}

impl<R: BlockRepository> GetBlockUseCase<R> {
    /// Create a new use case instance
    pub fn new(block_repo: Arc<R>) -> Self {
        Self { block_repo }
    }

    /// Execute the query
    pub async fn execute(&self, query: GetBlockQuery) -> RepositoryResult<Option<Block>> {
        self.block_repo.get(&query.id).await
    }
}
