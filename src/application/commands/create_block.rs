//! CreateBlock Command
//!
//! Command to create a new block.

use ulid::Ulid;
use crate::domain::models::{Block, BlockType};
use crate::ports::block_repository::BlockRepository;
use crate::ports::repository_error::RepositoryResult;
use std::sync::Arc;

/// Command to create a new block
#[derive(Debug)]
pub struct CreateBlockCommand {
    pub block_type: BlockType,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
}

/// CreateBlockUseCase: Application service for creating blocks
pub struct CreateBlockUseCase<R: BlockRepository> {
    block_repo: Arc<R>,
}

impl<R: BlockRepository> CreateBlockUseCase<R> {
    /// Create a new use case instance
    pub fn new(block_repo: Arc<R>) -> Self {
        Self { block_repo }
    }

    /// Execute the command
    pub async fn execute(&self, cmd: CreateBlockCommand) -> RepositoryResult<Ulid> {
        let mut block = Block::new(cmd.block_type, cmd.title);
        block.content = cmd.content;
        block.tags = cmd.tags;

        self.block_repo.create(block.clone()).await?;

        Ok(block.id)
    }
}
