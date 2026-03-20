//! Ghost command: Manage Ghost Nodes (predictive placeholders)
//!
//! Ghost Nodes are predictive placeholders that represent anticipated content.
//! They help maintain structural integrity and guide content creation.

use crate::cli::GhostCommands;
use crate::db::Database;
use crate::models::BlockType;
use chrono::{DateTime, Utc};
use ulid::Ulid;

/// Execute ghost commands
pub async fn execute(
    db: &Database,
    command: &GhostCommands,
) -> anyhow::Result<()> {
    match command {
        GhostCommands::List => {
            list_ghost_nodes(db).await?;
        }
        GhostCommands::Show { id } => {
            show_ghost_node(db, id).await?;
        }
        GhostCommands::Fill { id, content } => {
            fill_ghost_node(db, id, content).await?;
        }
        GhostCommands::Dismiss { id } => {
            dismiss_ghost_node(db, id).await?;
        }
    }

    Ok(())
}

/// Parse a ULID from string
fn parse_ulid(s: &str) -> anyhow::Result<Ulid> {
    if let Ok(ulid) = s.parse::<Ulid>() {
        return Ok(ulid);
    }
    if let Some(inner) = s.strip_prefix("block:")
        && let Ok(ulid) = inner.parse::<Ulid>() {
        return Ok(ulid);
    }
    if let Some(stripped) = s.strip_prefix("ghost_") {
        // Handle ghost_X aliases - try to parse the numeric part
        if let Ok(ulid) = stripped.parse::<u128>() {
            return Ok(Ulid::from(u128::MAX - ulid));
        }
    }
    anyhow::bail!("Invalid ULID format: {}", s)
}

/// List all ghost nodes
async fn list_ghost_nodes(db: &Database) -> anyhow::Result<()> {
    let block_repo = db.blocks();

    // Get all ghost blocks
    let ghosts = block_repo.list_by_type(BlockType::Ghost).await?;

    if ghosts.is_empty() {
        println!("👻 Ghost Nodes:");
        println!();
        println!("   No ghost nodes found. AI will detect missing content.");
        println!();
        println!("💡 Create a ghost node with:");
        println!("   nexus create --block-type ghost --title \"Topic Name\"");
        return Ok(())    }

    println!("👻 Ghost Nodes (Placeholders):");
    println!();

    // Calculate statistics
    let empty_count = ghosts.iter().filter(|b| b.content.is_empty()).count();
    let draft_count = ghosts.iter().filter(|b| !b.content.is_empty() && b.content.len() < 100).count();
    let high_priority: usize = ghosts.iter().filter(|b| {
        b.metadata.get("priority")
            .and_then(|v| v.as_str())
            .map(|s| s == "high")
            .unwrap_or(false)
    }).count();

    for (i, ghost) in ghosts.iter().enumerate() {
        let completion_status = if ghost.content.is_empty() {
            "empty (0% complete)"
        } else if ghost.content.len() < 100 {
            "draft (10% complete)"
        } else {
            "partial"
        };

        let priority = ghost.metadata.get("priority")
            .and_then(|v| v.as_str())
            .unwrap_or("medium");

        let age = calculate_age(&ghost.created_at);

        println!("{}. [{}] {}", i + 1, ghost.id_str().chars().take(8).collect::<String>(), ghost.title);
        println!("   Priority: {}", priority);
        println!("   Status: {} ⚠️", completion_status);
        println!("   Created: {} (age: {})", ghost.created_at.format("%Y-%m-%d"), age);
        println!();
    }

    println!("📊 Summary:");
    println!("   - Total Ghosts: {}", ghosts.len());
    println!("   - High priority: {}", high_priority);
    println!("   - Empty: {}", empty_count);
    println!("   - Draft: {}", draft_count);

    println!();
    println!("💡 Run `nexus ghost show <id>` for details");
    println!("💡 Run `nexus ghost fill <id> --content \"...\"` to activate");

    Ok(())
}

/// Show details of a specific ghost node
async fn show_ghost_node(db: &Database, id: &str) -> anyhow::Result<()> {
    let ghost_id = parse_ulid(id)?;
    let block_repo = db.blocks();

    let ghost = block_repo.get(&ghost_id).await?
        .ok_or_else(|| anyhow::anyhow!("Ghost node not found: {}", id))?;

    if ghost.block_type != BlockType::Ghost {
        anyhow::bail!("Block {} is not a ghost node (type: {:?})", id, ghost.block_type);
    }

    let priority = ghost.metadata.get("priority")
        .and_then(|v| v.as_str())
        .unwrap_or("medium");

    let intent = ghost.metadata.get("intent")
        .and_then(|v| v.as_str())
        .unwrap_or("Not specified");

    let keywords = ghost.metadata.get("keywords")
        .and_then(|v| v.as_str())
        .unwrap_or("None");

    let age = calculate_age(&ghost.created_at);
    let completion = if ghost.content.is_empty() {
        "empty (0%)"
    } else if ghost.content.len() < 100 {
        "draft (~10%)"
    } else {
        "partial"
    };

    println!("👻 Ghost Node: {}", ghost.title);
    println!();
    println!("   ID: {}", ghost.id_str());
    println!("   Priority: {}", priority);
    println!("   Completion: {}", completion);
    println!("   Age: {} (created {})", age, ghost.created_at.format("%Y-%m-%d"));
    println!();
    println!("   Intent: {}", intent);
    println!("   Keywords: {}", keywords);
    println!();

    if !ghost.content.is_empty() {
        println!("   Content Preview:");
        println!("   ────────────────────────────────────────");
        let preview = ghost.content.chars().take(200).collect::<String>();
        println!("   {}", preview);
        if ghost.content.len() > 200 {
            println!("   ...");
        }
        println!("   ────────────────────────────────────────");
    } else {
        println!("   Content: (empty)");
    }

    println!();
    println!("💡 To fill this ghost: nexus ghost fill {} --content \"Your content here\"", id);
    println!("💡 To dismiss this ghost: nexus ghost dismiss {}", id);

    Ok(())
}

/// Fill a ghost node with real content (activate it)
async fn fill_ghost_node(db: &Database, id: &str, content: &str) -> anyhow::Result<()> {
    let ghost_id = parse_ulid(id)?;
    let block_repo = db.blocks();

    let ghost = block_repo.get(&ghost_id).await?
        .ok_or_else(|| anyhow::anyhow!("Ghost node not found: {}", id))?;

    if ghost.block_type != BlockType::Ghost {
        anyhow::bail!("Block {} is not a ghost node (type: {:?})", id, ghost.block_type);
    }

    // Convert ghost to permanent
    let mut activated_block = ghost.clone();
    activated_block.block_type = BlockType::Permanent;
    activated_block.content = content.to_string();
    activated_block.updated_at = Utc::now();

    // Preserve metadata
    if let Some(title) = ghost.metadata.get("original_title").and_then(|v| v.as_str()) {
        activated_block.title = title.to_string();
    }

    // Update the block
    block_repo.update(activated_block.clone()).await?;

    println!("✅ Ghost Node activated!");
    println!();
    println!("   Old ID: ghost_{}", ghost_id.to_string().chars().take(8).collect::<String>());
    println!("   New ID: {}", activated_block.id_str());
    println!("   Title: {}", activated_block.title);
    println!("   Type: {:?}", activated_block.block_type);
    println!("   Status: stable");
    println!();
    println!("📊 Section coverage increased!");

    Ok(())
}

/// Dismiss a ghost node (remove it)
async fn dismiss_ghost_node(db: &Database, id: &str) -> anyhow::Result<()> {
    let ghost_id = parse_ulid(id)?;
    let block_repo = db.blocks();

    let ghost = block_repo.get(&ghost_id).await?
        .ok_or_else(|| anyhow::anyhow!("Ghost node not found: {}", id))?;

    if ghost.block_type != BlockType::Ghost {
        anyhow::bail!("Block {} is not a ghost node (type: {:?})", id, ghost.block_type);
    }

    // Delete the ghost block
    block_repo.delete(&ghost_id).await?;

    println!("🗑️  Ghost Node dismissed!");
    println!();
    println!("   ID: {}", id);
    println!("   Title: {}", ghost.title);
    println!();
    println!("💡 The placeholder has been removed from the structure.");

    Ok(())
}

/// Calculate age string from creation date
fn calculate_age(created: &DateTime<Utc>) -> chrono::Duration {
    Utc::now() - *created
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Block;

    #[test]
    fn test_parse_ulid_formats() {
        // Valid ULID
        let ulid = parse_ulid("01ARZ3NDEKTSV4RRFFQ69G5FAV").unwrap();
        assert_eq!(ulid.to_string(), "01ARZ3NDEKTSV4RRFFQ69G5FAV");

        // With block: prefix
        let ulid = parse_ulid("block:01ARZ3NDEKTSV4RRFFQ69G5FAV").unwrap();
        assert_eq!(ulid.to_string(), "01ARZ3NDEKTSV4RRFFQ69G5FAV");

        // Invalid format should fail
        assert!(parse_ulid("invalid").is_err());
    }

    #[test]
    fn test_ghost_block_creation() {
        let ghost = Block::new(BlockType::Ghost, "Test Ghost");
        assert_eq!(ghost.block_type, BlockType::Ghost);
        assert_eq!(ghost.title, "Test Ghost");
    }
}