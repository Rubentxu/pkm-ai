//! Structural Spine module
//!
//! The backbone of ordered knowledge (Folgezettel digital)
//!
//! This module provides:
//! - Spine traversal (deterministic traversal following 'next' links)
//! - Gravity hooks (semantic attraction calculation)
//! - Structural linting (validation and issue detection)
//! - Semantic re-balancing (auto-organization by density)

#![allow(dead_code)]

pub mod traversal;
pub mod gravity_hooks;
pub mod linting;
pub mod rebalancing;

// Re-export main types for convenience
pub use traversal::{
    SpineTraversal, SpineTraversalResult,
};
pub use gravity_hooks::{
    GravityCalculator, GravityMatch,
};
pub use linting::{
    StructuralLinter, LintIssue,
};
pub use rebalancing::{
    SpineRebalancer, RebalanceReport, DensityAnalysis,
};

/// Spine engine - main entry point for spine operations
pub struct SpineEngine<'a> {
    db: &'a crate::db::Database,
}

impl<'a> SpineEngine<'a> {
    /// Create a new SpineEngine
    pub fn new(db: &'a crate::db::Database) -> Self {
        Self { db }
    }

    /// Get the traversal engine
    pub fn traversal(&self) -> SpineTraversal<'a> {
        SpineTraversal::new(self.db)
    }

    /// Get the gravity calculator
    pub fn gravity(&self) -> GravityCalculator<'a> {
        GravityCalculator::new(self.db)
    }

    /// Get the structural linter
    pub fn linter(&self) -> StructuralLinter<'a> {
        StructuralLinter::new(self.db)
    }

    /// Get the rebalancer
    pub fn rebalancer(&self) -> SpineRebalancer<'a> {
        SpineRebalancer::new(self.db)
    }

    /// Traverse the spine
    pub async fn traverse(&self, from: Option<ulid::Ulid>, depth: u32) -> crate::NexusResult<SpineTraversalResult> {
        self.traversal().traverse_from(from, depth).await
    }

    /// Check gravity for a block
    pub async fn check_gravity(&self, block_id: &ulid::Ulid) -> crate::NexusResult<Vec<GravityMatch>> {
        self.gravity().calculate_gravity_all(*block_id).await
    }

    /// Run structural lint
    pub async fn lint(&self) -> crate::NexusResult<Vec<LintIssue>> {
        self.linter().lint().await
    }

    /// Analyze density
    pub async fn analyze_density(&self) -> crate::NexusResult<DensityAnalysis> {
        self.rebalancer().analyze().await
    }

    /// Rebalance all sections
    pub async fn rebalance(&self) -> crate::NexusResult<RebalanceReport> {
        self.rebalancer().rebalance_all().await
    }
}

impl<'a> Default for SpineEngine<'a> {
    fn default() -> Self {
        panic!("SpineEngine requires a Database reference")
    }
}
