//! Test doubles module
//!
//! Provides mock implementations for testing repository abstractions.

pub mod mock_block_repo;
pub mod mock_edge_repo;

pub use mock_block_repo::MockBlockRepository;
pub use mock_edge_repo::MockEdgeRepository;