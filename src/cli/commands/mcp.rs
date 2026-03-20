//! MCP server command

use crate::ai::mcp::NexusMcpServer;
use crate::db::Database;
use anyhow::Context;

/// Get database path from config or default
fn get_db_path() -> anyhow::Result<std::path::PathBuf> {
    if let Some(config) = crate::cli::config::Config::find() {
        return Ok(config.database_path());
    }

    // Default path
    let mut path = dirs::home_dir().context("Cannot find home directory")?;
    path.push(".pkmai");
    std::fs::create_dir_all(&path).context("Failed to create .pkmai directory")?;
    path.push("data.db");
    Ok(path)
}

/// Execute MCP command - entry point for clap
pub async fn execute_serve() -> anyhow::Result<()> {
    serve_mcp().await
}

/// Start the MCP server
async fn serve_mcp() -> anyhow::Result<()> {
    let db_path = get_db_path()?;

    eprintln!("Starting MCP server with database: {}", db_path.display());

    let db = Database::rocksdb(&db_path).await?;
    let server = NexusMcpServer::new(db);

    eprintln!("MCP server ready on stdio");
    eprintln!("Waiting for MCP requests...");

    server.run().await
}
