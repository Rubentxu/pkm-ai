//! TraverseSpine Query
//!
//! Query to traverse the document spine.

use ulid::Ulid;
use crate::domain::models::SpineNode;
use crate::ports::graph_query_port::GraphQueryPort;
use crate::ports::repository_error::RepositoryResult;
use std::sync::Arc;

/// Query to traverse the spine
#[derive(Debug)]
pub struct TraverseSpineQuery {
    pub root_id: Ulid,
    pub max_depth: u32,
}

/// TraverseSpineUseCase: Application service for traversing spines
pub struct TraverseSpineUseCase<G: GraphQueryPort> {
    graph_query: Arc<G>,
}

impl<G: GraphQueryPort> TraverseSpineUseCase<G> {
    /// Create a new use case instance
    pub fn new(graph_query: Arc<G>) -> Self {
        Self { graph_query }
    }

    /// Execute the query
    pub async fn execute(&self, query: TraverseSpineQuery) -> RepositoryResult<Option<SpineNode>> {
        self.graph_query
            .traverse_spine(&query.root_id, query.max_depth)
            .await
    }
}
