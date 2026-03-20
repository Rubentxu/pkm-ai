//! DeleteBlock Command
//!
//! Command to delete a block and its associated edges.

use ulid::Ulid;
use crate::ports::block_repository::BlockRepository;
use crate::ports::edge_repository::EdgeRepository;
use crate::ports::repository_error::RepositoryResult;
use std::sync::Arc;

/// Command to delete a block
#[derive(Debug)]
pub struct DeleteBlockCommand {
    pub id: Ulid,
}

/// DeleteBlockUseCase: Application service for deleting blocks
pub struct DeleteBlockUseCase<R: BlockRepository, E: EdgeRepository> {
    block_repo: Arc<R>,
    edge_repo: Arc<E>,
}

impl<R: BlockRepository, E: EdgeRepository> DeleteBlockUseCase<R, E> {
    /// Create a new use case instance
    pub fn new(block_repo: Arc<R>, edge_repo: Arc<E>) -> Self {
        Self {
            block_repo,
            edge_repo,
        }
    }

    /// Execute the command - deletes block and all associated edges
    pub async fn execute(&self, cmd: DeleteBlockCommand) -> RepositoryResult<()> {
        // First, delete all edges involving this block
        self.edge_repo.delete_for_block(&cmd.id).await?;

        // Then delete the block
        self.block_repo.delete(&cmd.id).await?;

        Ok(())
    }
}
