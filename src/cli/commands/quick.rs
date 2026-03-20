//! Quick command: create + stage + commit in one command
//!
//! Ideal for rapid capture like Zettelkasten quick notes.

use std::path::PathBuf;

use crate::db::Database;
use crate::models::{Block, BlockType};
use pkm_ai::versioning::{AgentId, BlockDelta, VersionRepo, VersionError};

use super::create::parse_block_type;

/// Execute the quick command: create + stage + commit
pub async fn execute(
    db: &Database,
    content: &str,
    block_type: &Option<String>,
    tags: &Option<String>,
) -> anyhow::Result<()> {
    // Determine block type (default: fleeting)
    let block_type = match block_type {
        Some(t) => parse_block_type(t)?,
        None => BlockType::Fleeting,
    };

    // Generate a title from the first 50 chars of content
    let title = if content.len() > 50 {
        format!("{}...", &content[..47])
    } else {
        content.to_string()
    };

    // Create the block
    let mut block = Block::new(block_type, &title);
    block = block.with_content(content);

    // Add tags if provided
    if let Some(tags_str) = tags {
        let tag_list: Vec<String> = tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        for tag in tag_list {
            block = block.with_tag(tag);
        }
    }

    // Save block to database
    let repo = db.blocks();
    repo.create(block.clone()).await?;

    // Get the ULID for staging
    let block_ulid = block.id;

    // Initialize version repo and stage the block
    let version_repo = get_version_repo()?;
    version_repo.init()?;

    // Ensure main branch exists
    if !version_repo.ref_store.has_branch(&"main".into()) {
        version_repo.create_branch("main")?;
        version_repo.checkout("main")?;
    }

    // Create block delta for staging
    let delta = BlockDelta::Created {
        block_id: block_ulid,
        title: block.title.clone(),
        content: block.content.clone(),
        block_type: format!("{:?}", block.block_type).to_lowercase(),
    };

    // Stage the block
    version_repo.add_block(block_ulid, delta)?;

    // Generate commit message (first 50 chars of content)
    let commit_message = if content.len() > 50 {
        format!("Quick capture: {}", &content[..47])
    } else {
        format!("Quick capture: {}", content)
    };

    // Auto-commit
    let author = AgentId::new("user");
    match version_repo.commit(&commit_message, author) {
        Ok(commit_id) => {
            println!("Created and committed block: {}", block_ulid);
            println!("   Type: {:?}", block.block_type);
            if !block.tags.is_empty() {
                println!("   Tags: {}", block.tags.join(", "));
            }
            println!("   Commit: {}", commit_id);
        }
        Err(VersionError::NothingToCommit) => {
            // This shouldn't happen since we just staged a block
            println!("Created block: {} (nothing to commit)", block_ulid);
        }
        Err(e) => {
            return Err(anyhow::anyhow!("Failed to commit: {}", e));
        }
    }

    Ok(())
}

/// Get the version repository path
fn get_version_repo() -> anyhow::Result<VersionRepo> {
    let root = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".pkm");
    Ok(VersionRepo::new(&root))
}

#[cfg(test)]
mod tests {
    
    
    

    #[test]
    fn test_title_truncation_long() {
        let content = "This is a very long content that exceeds fifty characters and should be truncated";
        let title = if content.len() > 50 {
            format!("{}...", &content[..47])
        } else {
            content.to_string()
        };
        assert_eq!(title.len(), 50);
        assert!(title.ends_with("..."));
    }

    #[test]
    fn test_title_truncation_short() {
        let content = "Short content";
        let title = if content.len() > 50 {
            format!("{}...", &content[..47])
        } else {
            content.to_string()
        };
        assert_eq!(title, "Short content");
    }

    #[test]
    fn test_commit_message_truncation() {
        // Content longer than 50 chars - should use first 47 chars
        let long_content = "This is a very long content that exceeds fifty characters and should be truncated";
        let commit_message = if long_content.len() > 50 {
            format!("Quick capture: {}", &long_content[..47])
        } else {
            format!("Quick capture: {}", long_content)
        };
        assert!(commit_message.starts_with("Quick capture: "));
        // Verify the message contains the truncated content (47 chars)
        assert!(commit_message.contains(&long_content[..47]));
        // And doesn't have "..." suffix (that's only for title, not commit message)
        assert!(!commit_message.ends_with("..."));
    }

    #[test]
    fn test_commit_message_short_content() {
        // Content <= 50 chars - should use full content
        let short_content = "Short note";
        let commit_message = if short_content.len() > 50 {
            format!("Quick capture: {}", &short_content[..47])
        } else {
            format!("Quick capture: {}", short_content)
        };
        assert_eq!(commit_message, "Quick capture: Short note");
    }
}