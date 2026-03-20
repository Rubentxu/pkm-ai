//! LinkBlocks Command
//!
//! Command to create an edge between two blocks.

use ulid::Ulid;
use crate::domain::models::{Edge, LinkType};
use crate::ports::edge_repository::EdgeRepository;
use crate::ports::graph_query_port::GraphQueryPort;
use crate::ports::repository_error::RepositoryResult;
use std::sync::Arc;

/// Command to link two blocks
#[derive(Debug)]
pub struct LinkBlocksCommand {
    pub from_id: Ulid,
    pub to_id: Ulid,
    pub link_type: LinkType,
}

/// LinkBlocksUseCase: Application service for linking blocks
pub struct LinkBlocksUseCase<E: EdgeRepository, G: GraphQueryPort> {
    edge_repo: Arc<E>,
    graph_query: Arc<G>,
}

impl<E: EdgeRepository, G: GraphQueryPort> LinkBlocksUseCase<E, G> {
    /// Create a new use case instance
    pub fn new(edge_repo: Arc<E>, graph_query: Arc<G>) -> Self {
        Self {
            edge_repo,
            graph_query,
        }
    }

    /// Execute the command
    pub async fn execute(&self, cmd: LinkBlocksCommand) -> RepositoryResult<Ulid> {
        // Check if this would create a cycle (for structural links)
        if cmd.link_type.is_structural() {
            let would_cycle = self.graph_query.would_create_cycle(&cmd.from_id, &cmd.to_id).await?;
            if would_cycle {
                return Err(crate::ports::repository_error::RepositoryError::ValidationError(
                    format!("Adding {} edge from {} to {} would create a cycle", cmd.link_type, cmd.from_id, cmd.to_id)
                ));
            }
        }

        // Create the edge
        let edge = Edge::new(cmd.from_id, cmd.to_id, cmd.link_type);
        self.edge_repo.create(edge.clone()).await?;

        Ok(edge.id)
    }
}
