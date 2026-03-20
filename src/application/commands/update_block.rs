//! UpdateBlock Command
//!
//! Command to update an existing block.

use ulid::Ulid;
use crate::domain::models::Block;
use crate::ports::block_repository::BlockRepository;
use crate::ports::repository_error::RepositoryResult;
use std::sync::Arc;

/// Command to update a block
#[derive(Debug)]
pub struct UpdateBlockCommand {
    pub id: Ulid,
    pub title: Option<String>,
    pub content: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// UpdateBlockUseCase: Application service for updating blocks
pub struct UpdateBlockUseCase<R: BlockRepository> {
    block_repo: Arc<R>,
}

impl<R: BlockRepository> UpdateBlockUseCase<R> {
    /// Create a new use case instance
    pub fn new(block_repo: Arc<R>) -> Self {
        Self { block_repo }
    }

    /// Execute the command
    pub async fn execute(&self, cmd: UpdateBlockCommand) -> RepositoryResult<()> {
        let mut block = self
            .block_repo
            .get(&cmd.id)
            .await?
            .ok_or_else(|| crate::ports::repository_error::RepositoryError::BlockNotFound(cmd.id.to_string()))?;

        if let Some(title) = cmd.title {
            block.title = title;
        }

        if let Some(content) = cmd.content {
            block.content = content;
        }

        if let Some(tags) = cmd.tags {
            block.tags = tags;
        }

        block.touch();
        self.block_repo.update(&block).await?;

        Ok(())
    }
}
