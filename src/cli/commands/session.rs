//! Session management command
//!
//! Integrates sessions with the Zettelkasten block workflow. Sessions track blocks
//! created during their lifetime, enabling knowledge graph participation.

use crate::db::Database;
use crate::models::{Block, BlockType};
use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use ulid::Ulid;

/// Session status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionStatus {
    Active,
    Ended,
    Restored,
}

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionStatus::Active => write!(f, "active"),
            SessionStatus::Ended => write!(f, "ended"),
            SessionStatus::Restored => write!(f, "restored"),
        }
    }
}

/// Session data stored as a block
///
/// Tracks blocks created and modified during the session for Zettelkasten integration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Session {
    /// Unique session identifier
    pub id: String,
    /// Agent name (e.g., claude-code, opencode)
    pub agent: String,
    /// Project name
    pub project: String,
    /// Session start timestamp
    pub started_at: DateTime<Utc>,
    /// Session end timestamp (None if still active)
    pub ended_at: Option<DateTime<Utc>>,
    /// Working directory at session start
    pub cwd: Option<String>,
    /// Session status
    pub status: SessionStatus,
    /// Optional session description
    pub description: Option<String>,
    /// Session summary (set on end)
    pub summary: Option<String>,
    /// Checkpoint ID if created
    pub checkpoint_id: Option<String>,
    /// ULIDs of blocks created during this session
    pub created_blocks: Vec<String>,
    /// ULIDs of blocks modified during this session
    pub modified_blocks: Vec<String>,
    /// ULID of the session block itself
    pub session_block_id: Option<String>,
}

impl Session {
    /// Create a new Session with the given parameters
    pub fn new(id: String, agent: String, project: String, cwd: Option<String>) -> Self {
        Self {
            id,
            agent,
            project,
            started_at: Utc::now(),
            ended_at: None,
            cwd,
            status: SessionStatus::Active,
            description: None,
            summary: None,
            checkpoint_id: None,
            created_blocks: Vec::new(),
            modified_blocks: Vec::new(),
            session_block_id: None,
        }
    }

    /// Add a block ID to the created_blocks list
    pub fn track_created(&mut self, block_id: &Ulid) {
        self.created_blocks.push(block_id.to_string());
    }

    /// Add a block ID to the modified_blocks list
    pub fn track_modified(&mut self, block_id: &Ulid) {
        self.modified_blocks.push(block_id.to_string());
    }

    /// Get the session block ID if set
    pub fn session_block_id(&self) -> Option<&str> {
        self.session_block_id.as_deref()
    }
}

/// Session metadata stored in block metadata field (not opaque JSON)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionMetadata {
    pub session_id: String,
    pub agent: String,
    pub project: String,
    pub started_at: DateTime<Utc>,
    pub description: Option<String>,
}

/// Get database path for a project: ~/.pkmai/{project}/data.db
fn get_project_db_path(project: &str) -> anyhow::Result<PathBuf> {
    let mut path = dirs::home_dir().context("Cannot find home directory")?;
    path.push(".pkmai");
    path.push(project);
    std::fs::create_dir_all(&path).context("Failed to create .pkmai directory")?;
    path.push("data.db");
    Ok(path)
}

/// Execute session command
pub async fn execute(
    command: &crate::cli::SessionCommands,
) -> anyhow::Result<()> {
    match command {
        crate::cli::SessionCommands::Start {
            agent,
            project,
            session_id,
            cwd,
            description,
        } => {
            let id = session_id.clone().unwrap_or_else(|| Ulid::new().to_string());
            start_session(&id, agent, project, cwd.as_deref(), description.as_deref()).await
        }
        crate::cli::SessionCommands::End {
            session_id,
            project,
            auto_summary,
            summary,
            promote_to,
        } => {
            end_session(session_id, project.as_deref(), *auto_summary, summary.as_deref(), promote_to.as_deref()).await
        }
        crate::cli::SessionCommands::List { limit, project } => {
            list_sessions(*limit, project.as_deref()).await
        }
        crate::cli::SessionCommands::Restore { session_id, project } => {
            restore_session(session_id, project.as_deref()).await
        }
        crate::cli::SessionCommands::Checkpoint { session_id, project } => {
            create_checkpoint(session_id, project.as_deref()).await
        }
        crate::cli::SessionCommands::Capture {
            session_id,
            project,
            content,
        } => {
            capture_block(session_id, project.as_deref(), content).await
        }
        crate::cli::SessionCommands::Blocks {
            session_id,
            project,
        } => {
            list_session_blocks(session_id, project.as_deref()).await
        }
    }
}

/// Start a new session
///
/// Creates a session metadata block with proper structure (not opaque JSON).
/// Returns the session ID for tracking.
async fn start_session(
    id: &str,
    agent: &str,
    project: &str,
    cwd: Option<&str>,
    description: Option<&str>,
) -> anyhow::Result<()> {
    let db_path = get_project_db_path(project)?;
    let db = Database::rocksdb(&db_path).await?;

    let mut session = Session::new(
        id.to_string(),
        agent.to_string(),
        project.to_string(),
        cwd.map(|s| s.to_string()),
    );
    session.description = description.map(|s| s.to_string());

    let now = Utc::now();
    let block_id = Ulid::new();
    session.session_block_id = Some(block_id.to_string());

    // Create session metadata for the block's metadata field (not opaque JSON)
    let session_metadata = SessionMetadata {
        session_id: id.to_string(),
        agent: agent.to_string(),
        project: project.to_string(),
        started_at: now,
        description: session.description.clone(),
    };

    // Store session as structured block with proper metadata
    let content = serde_json::to_string_pretty(&session)
        .context("Failed to serialize session")?;

    let mut metadata = HashMap::new();
    metadata.insert(
        "session".to_string(),
        serde_json::to_value(&session_metadata).context("Failed to serialize session metadata")?,
    );
    metadata.insert(
        "session_id".to_string(),
        serde_json::Value::String(id.to_string()),
    );
    metadata.insert(
        "session_status".to_string(),
        serde_json::Value::String("active".to_string()),
    );

    let block = Block {
        id: block_id,
        block_type: BlockType::Task,
        title: format!("session/{}", id),
        content,
        tags: vec!["session".to_string(), "session-active".to_string()],
        metadata,
        created_at: now,
        updated_at: now,
        version: 1,
        ai_confidence: None,
        semantic_centroid: None,
    };

    db.blocks().create(block).await.context("Failed to create session block")?;

    println!("Session started: {}", id);
    println!("  Agent: {}", agent);
    println!("  Project: {}", project);
    println!("  Database: {}", db_path.display());
    if let Some(desc) = description {
        println!("  Description: {}", desc);
    }

    Ok(())
}

/// End a session
///
/// Accepts `promote_to` parameter to transition blocks created during session
/// to a higher order type (literature or permanent).
async fn end_session(
    session_id: &str,
    project: Option<&str>,
    _auto_summary: Option<bool>,
    summary: Option<&str>,
    promote_to: Option<&str>,
) -> anyhow::Result<()> {
    let project = project.unwrap_or("default");
    let db_path = get_project_db_path(project)?;
    let db = Database::rocksdb(&db_path).await?;

    // Find the session block
    let block = find_session_block(&db, session_id)
        .await
        .context("Session not found")?;

    // Parse existing session
    let mut session: Session = serde_json::from_str(&block.content)
        .context("Failed to parse session JSON")?;

    session.ended_at = Some(Utc::now());
    session.status = SessionStatus::Ended;
    session.summary = summary.map(|s| s.to_string());

    // If promote_to is specified, transition blocks created during session
    if let Some(target_type) = promote_to {
        let target_block_type = match target_type.to_lowercase().as_str() {
            "literature" => BlockType::Literature,
            "permanent" => BlockType::Permanent,
            _ => anyhow::bail!("Invalid promote_to type: {}. Use 'literature' or 'permanent'.", target_type),
        };

        // Promote each block created during session
        for block_ulid in &session.created_blocks {
            if let Ok(ulid) = block_ulid.parse::<Ulid>() {
                if let Ok(Some(mut block)) = db.blocks().get(&ulid).await {
                    block.block_type = target_block_type.clone();
                    block.updated_at = Utc::now();
                    block.version += 1;

                    // Add session tags to track origin
                    block.tags.push(format!("from-session/{}", session_id));
                    if !block.tags.contains(&"promoted".to_string()) {
                        block.tags.push("promoted".to_string());
                    }

                    let _ = db.blocks().update(block).await;
                }
            }
        }

        println!("Promoted {} blocks to {}", session.created_blocks.len(), target_type);
    }

    // Update the block
    let content = serde_json::to_string_pretty(&session)
        .context("Failed to serialize session")?;

    let mut updated_block = block;
    updated_block.content = content;
    updated_block.updated_at = Utc::now();
    // Update tags to reflect ended status
    updated_block.tags = vec!["session".to_string(), "session-ended".to_string()];
    // Update metadata
    if let Some(status) = updated_block.metadata.get_mut("session_status") {
        *status = serde_json::Value::String("ended".to_string());
    }

    db.blocks().update(updated_block).await.context("Failed to update session block")?;

    println!("Session ended: {}", session_id);
    if let Some(summary) = &session.summary {
        println!("  Summary: {}", summary);
    }
    println!("  Blocks created: {}", session.created_blocks.len());
    println!("  Blocks modified: {}", session.modified_blocks.len());

    Ok(())
}

/// Find a session block by session ID
async fn find_session_block(db: &Database, session_id: &str) -> anyhow::Result<Block> {
    let blocks = db.blocks()
        .search_content(&format!("session/{}", session_id))
        .await
        .context("Failed to search for session")?;

    blocks
        .into_iter()
        .find(|b| b.title == format!("session/{}", session_id))
        .context("Session not found")
}

/// Capture a block linked to an active session
///
/// Creates a fleeting block with session_id in metadata for tracking.
async fn capture_block(
    session_id: &str,
    project: Option<&str>,
    content: &str,
) -> anyhow::Result<()> {
    let project = project.unwrap_or("default");
    let db_path = get_project_db_path(project)?;
    let db = Database::rocksdb(&db_path).await?;

    // Find and validate session is active
    let session_block = find_session_block(&db, session_id).await?;
    let session: Session = serde_json::from_str(&session_block.content)
        .context("Failed to parse session JSON")?;

    if matches!(session.status, SessionStatus::Ended) {
        anyhow::bail!("Cannot capture to ended session. Start a new session first.");
    }

    let now = Utc::now();
    let block_id = Ulid::new();

    // Create fleeting block with session tracking
    let mut metadata = HashMap::new();
    metadata.insert(
        "session_id".to_string(),
        serde_json::Value::String(session_id.to_string()),
    );
    metadata.insert(
        "captured_at".to_string(),
        serde_json::Value::String(now.to_rfc3339()),
    );

    let block = Block {
        id: block_id,
        block_type: BlockType::Fleeting,
        title: format!("capture/{}", block_id),
        content: content.to_string(),
        tags: vec![
            "session-capture".to_string(),
            format!("session/{}", session_id),
        ],
        metadata,
        created_at: now,
        updated_at: now,
        version: 1,
        ai_confidence: None,
        semantic_centroid: None,
    };

    db.blocks().create(block).await.context("Failed to create capture block")?;

    // Update session's created_blocks tracking
    let mut updated_session = session;
    updated_session.track_created(&block_id);

    let updated_content = serde_json::to_string_pretty(&updated_session)
        .context("Failed to serialize session")?;

    let mut updated_block = session_block;
    updated_block.content = updated_content;
    updated_block.updated_at = now;
    db.blocks().update(updated_block).await.context("Failed to update session block")?;

    println!("Capture created: {}", block_id);
    println!("  Session: {}", session_id);
    println!("  Project: {}", project);

    Ok(())
}

/// List blocks created during a session
async fn list_session_blocks(
    session_id: &str,
    project: Option<&str>,
) -> anyhow::Result<()> {
    let project = project.unwrap_or("default");
    let db_path = get_project_db_path(project)?;

    // Check if database exists
    if !db_path.exists() {
        println!("No sessions found (project database does not exist: {})", db_path.display());
        return Ok(());
    }

    let db = Database::rocksdb(&db_path).await?;

    // Find session block
    let session_block = match find_session_block(&db, session_id).await {
        Ok(block) => block,
        Err(_) => {
            println!("Session not found: {}", session_id);
            return Ok(());
        }
    };

    let session: Session = serde_json::from_str(&session_block.content)
        .context("Failed to parse session JSON")?;

    println!("Session: {}", session_id);
    println!("  Status: {}", session.status);
    println!("  Created blocks: {}", session.created_blocks.len());
    println!("  Modified blocks: {}", session.modified_blocks.len());
    println!();

    if session.created_blocks.is_empty() && session.modified_blocks.is_empty() {
        println!("No blocks tracked for this session.");
        return Ok(());
    }

    // List created blocks
    if !session.created_blocks.is_empty() {
        println!("Created blocks:");
        println!("{:<26} {:<15} {:<40}", "ULID", "Type", "Title");
        println!("{}", "-".repeat(85));

        for block_ulid in &session.created_blocks {
            if let Ok(ulid) = block_ulid.parse::<Ulid>() {
                if let Ok(Some(block)) = db.blocks().get(&ulid).await {
                    let type_str = format!("{:?}", block.block_type);
                    let title = if block.title.len() > 40 {
                        format!("{}...", &block.title[..37])
                    } else {
                        block.title.clone()
                    };
                    println!("{:<26} {:<15} {:<40}", block_ulid, type_str, title);
                } else {
                    println!("{:<26} {:<15} {:<40}", block_ulid, "MISSING", "(block not found)");
                }
            }
        }
    }

    // List modified blocks
    if !session.modified_blocks.is_empty() {
        println!();
        println!("Modified blocks:");
        println!("{:<26} {:<15} {:<40}", "ULID", "Type", "Title");
        println!("{}", "-".repeat(85));

        for block_ulid in &session.modified_blocks {
            if let Ok(ulid) = block_ulid.parse::<Ulid>() {
                if let Ok(Some(block)) = db.blocks().get(&ulid).await {
                    let type_str = format!("{:?}", block.block_type);
                    let title = if block.title.len() > 40 {
                        format!("{}...", &block.title[..37])
                    } else {
                        block.title.clone()
                    };
                    println!("{:<26} {:<15} {:<40}", block_ulid, type_str, title);
                } else {
                    println!("{:<26} {:<15} {:<40}", block_ulid, "MISSING", "(block not found)");
                }
            }
        }
    }

    Ok(())
}

/// List all sessions
async fn list_sessions(limit: usize, project: Option<&str>) -> anyhow::Result<()> {
    let project = project.unwrap_or("default");
    let db_path = get_project_db_path(project)?;

    // Check if database exists
    if !db_path.exists() {
        println!("No sessions found (project database does not exist: {})", db_path.display());
        return Ok(());
    }

    let db = Database::rocksdb(&db_path).await?;

    let blocks = db.blocks()
        .search_by_tags(&["session".to_string()])
        .await
        .context("Failed to search for sessions")?;

    if blocks.is_empty() {
        println!("No sessions found for project: {}", project);
        return Ok(());
    }

    // Limit results
    let blocks: Vec<_> = blocks.into_iter().take(limit).collect();

    println!("Sessions for project '{}':", project);
    println!("{:<26} {:<15} {:<20} {:<10} {:<10}", "ID", "Agent", "Project", "Status", "Blocks");
    println!("{}", "-".repeat(90));

    for block in blocks {
        if let Ok(session) = serde_json::from_str::<Session>(&block.content) {
            let block_count = session.created_blocks.len() + session.modified_blocks.len();
            println!(
                "{:<26} {:<15} {:<20} {:<10} {:<10}",
                session.id,
                session.agent,
                session.project,
                session.status,
                block_count
            );
        }
    }

    Ok(())
}

/// Restore a session
async fn restore_session(session_id: &str, project: Option<&str>) -> anyhow::Result<()> {
    let project = project.unwrap_or("default");
    let db_path = get_project_db_path(project)?;
    let db = Database::rocksdb(&db_path).await?;

    // Find the session block
    let block = find_session_block(&db, session_id)
        .await
        .context("Session not found")?;

    // Parse session
    let session: Session = serde_json::from_str(&block.content)
        .context("Failed to parse session JSON")?;

    // Update status to Restored
    let mut updated_session = session.clone();
    updated_session.status = SessionStatus::Restored;

    let content = serde_json::to_string_pretty(&updated_session)
        .context("Failed to serialize session")?;

    let mut updated_block = block;
    updated_block.content = content;
    updated_block.updated_at = Utc::now();
    updated_block.tags = vec!["session".to_string(), "session-restored".to_string()];
    if let Some(status) = updated_block.metadata.get_mut("session_status") {
        *status = serde_json::Value::String("restored".to_string());
    }

    db.blocks().update(updated_block).await.context("Failed to update session block")?;

    println!("Session restored: {}", session_id);
    println!("  Agent: {}", session.agent);
    println!("  Project: {}", session.project);
    println!("  Started: {}", session.started_at);
    println!("  Blocks created: {}", session.created_blocks.len());
    println!("  Blocks modified: {}", session.modified_blocks.len());

    Ok(())
}

/// Create a checkpoint for a session
async fn create_checkpoint(session_id: &str, project: Option<&str>) -> anyhow::Result<()> {
    let project = project.unwrap_or("default");
    let db_path = get_project_db_path(project)?;
    let db = Database::rocksdb(&db_path).await?;

    // Find the session block
    let block = find_session_block(&db, session_id)
        .await
        .context("Session not found")?;

    // Parse session
    let session: Session = serde_json::from_str(&block.content)
        .context("Failed to parse session JSON")?;

    // Generate checkpoint ID
    let checkpoint_id = Ulid::new().to_string();

    // Update session with checkpoint
    let mut updated_session = session.clone();
    updated_session.checkpoint_id = Some(checkpoint_id.clone());

    let content = serde_json::to_string_pretty(&updated_session)
        .context("Failed to serialize session")?;

    let mut updated_block = block;
    updated_block.content = content;
    updated_block.updated_at = Utc::now();

    db.blocks().update(updated_block).await.context("Failed to update session block")?;

    println!("Checkpoint created: {}", checkpoint_id);
    println!("  For session: {}", session_id);
    println!("  Blocks tracked: {} created, {} modified",
        session.created_blocks.len(), session.modified_blocks.len());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_new() {
        let session = Session::new(
            "test-id".to_string(),
            "claude".to_string(),
            "test-project".to_string(),
            Some("/tmp".to_string()),
        );

        assert_eq!(session.id, "test-id");
        assert_eq!(session.agent, "claude");
        assert_eq!(session.project, "test-project");
        assert_eq!(session.status, SessionStatus::Active);
        assert!(session.created_blocks.is_empty());
        assert!(session.modified_blocks.is_empty());
    }

    #[test]
    fn test_session_track_blocks() {
        let mut session = Session::new(
            "test-id".to_string(),
            "claude".to_string(),
            "test-project".to_string(),
            None,
        );

        let block_id = Ulid::new();
        session.track_created(&block_id);
        assert_eq!(session.created_blocks.len(), 1);
        assert_eq!(session.created_blocks[0], block_id.to_string());

        let block_id2 = Ulid::new();
        session.track_modified(&block_id2);
        assert_eq!(session.modified_blocks.len(), 1);
        assert_eq!(session.modified_blocks[0], block_id2.to_string());
    }

    #[test]
    fn test_session_serialization() {
        let mut session = Session::new(
            "test-id".to_string(),
            "claude".to_string(),
            "test-project".to_string(),
            None,
        );
        session.description = Some("Test session".to_string());
        session.session_block_id = Some(Ulid::new().to_string());

        let json = serde_json::to_string_pretty(&session).unwrap();
        let parsed: Session = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, session.id);
        assert_eq!(parsed.agent, session.agent);
        assert_eq!(parsed.project, session.project);
        assert_eq!(parsed.description, session.description);
    }
}
