//! Architect command: Interactive TUI for structural editing
//!
//! This command launches an interactive TUI for visualizing and editing
//! the knowledge graph structure.

use crate::db::Database;

/// Execute the architect command - launches the interactive TUI
pub async fn execute(db: &Database) -> anyhow::Result<()> {
    // Launch the interactive TUI
    crate::tui::launch(db).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_architect_info() {
        // Basic test to ensure the module compiles
        assert!(true);
    }
}