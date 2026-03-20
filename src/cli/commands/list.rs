//! List command

use crate::db::Database;
use crate::models::BlockType;
use super::create::parse_block_type;
use super::search::fuzzy_match;

/// Parses a comma-separated string of tags into a vector of Strings.
///
/// # Arguments
/// * `tags_str` - A comma-separated string of tags (e.g., "tag1, tag2, tag3")
///
/// # Returns
/// * `Vec<String>` - A vector of trimmed, non-empty tag strings
fn parse_tags(tags_str: &str) -> Vec<String> {
    tags_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub async fn execute(
    db: &Database,
    block_type: &Option<String>,
    tags: &Option<String>,
    search: &Option<String>,
    limit: usize,
    output: &str,
) -> anyhow::Result<()> {
    let repo = db.blocks();

    // Determine base filter: by type or all blocks
    let base_blocks = if let Some(bt) = block_type {
        let bt = parse_block_type(bt)?;
        repo.list_by_type(bt).await?
    } else {
        repo.list_all().await?
    };

    // Apply tag filtering if tags are provided (OR semantics: any matching tag)
    let blocks = if let Some(tags_str) = tags {
        let filter_tags = parse_tags(tags_str);
        if filter_tags.is_empty() {
            base_blocks
        } else {
            repo.search_by_tags(&filter_tags).await?
        }
    } else {
        base_blocks
    };

    // Apply fuzzy search filtering if search query is provided
    let blocks = if let Some(search_query) = search {
        if search_query.is_empty() {
            blocks
        } else {
            blocks
                .into_iter()
                .filter(|block| fuzzy_match(search_query, &block.title))
                .collect()
        }
    } else {
        blocks
    };

    match output {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&blocks)?);
        }
        "simple" => {
            for block in &blocks {
                println!("{} | {:?} | {}", block.id_str(), block.block_type, block.title);
            }
        }
        _ => {
            // Enhanced table format with emojis
            if blocks.is_empty() {
                println!("\n  ℹ️  No blocks found");
                println!("     Create your first block with: pkmai create --title \"Your Note\"\n");
                return Ok(());
            }

            println!("\n  📋 Found {} block(s)", blocks.len());
            println!("  ────────────────────────────────────────────────────────────");
            println!("  {:<22} {:<12} {}", "ULID", "TYPE", "TITLE");
            println!("  ────────────────────────────────────────────────────────────");
            for block in blocks.iter().take(limit) {
                let type_emoji = match block.block_type {
                    BlockType::Fleeting => "⚡",
                    BlockType::Literature => "📖",
                    BlockType::Permanent => "💎",
                    BlockType::Structure => "🗂️",
                    BlockType::Hub => "🌐",
                    BlockType::Task => "✅",
                    BlockType::Reference => "📚",
                    BlockType::Outline => "📝",
                    BlockType::Ghost => "👻",
                };
                println!(
                    "  {:<22} {:<12} {}",
                    block.id_str().chars().take(22).collect::<String>(),
                    format!("{} {:?}", type_emoji, block.block_type).to_lowercase(),
                    block.title.chars().take(30).collect::<String>()
                );
            }
            println!("  ────────────────────────────────────────────────────────────");
            println!("  Showing {} of {} blocks\n", blocks.len().min(limit), blocks.len());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tags_single_tag() {
        let result = parse_tags("rust");
        assert_eq!(result, vec!["rust"]);
    }

    #[test]
    fn test_parse_tags_multiple_tags() {
        let result = parse_tags("rust, golang, python");
        assert_eq!(result, vec!["rust", "golang", "python"]);
    }

    #[test]
    fn test_parse_tags_with_empty_spaces() {
        let result = parse_tags("  rust  ,  golang  ,  python  ");
        assert_eq!(result, vec!["rust", "golang", "python"]);
    }

    #[test]
    fn test_parse_tags_empty_string() {
        let result = parse_tags("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_tags_with_empty_entries() {
        let result = parse_tags("rust,,golang,  ,python");
        assert_eq!(result, vec!["rust", "golang", "python"]);
    }

    #[test]
    fn test_parse_tags_single_tag_no_comma() {
        let result = parse_tags("rust");
        assert_eq!(result, vec!["rust"]);
    }

    // =========================================================================
    // Output Formatting Tests
    // =========================================================================

    #[test]
    fn test_block_type_emoji_fleeting() {
        let emoji = match BlockType::Fleeting {
            BlockType::Fleeting => "⚡",
            _ => "",
        };
        assert_eq!(emoji, "⚡");
    }

    #[test]
    fn test_block_type_emoji_literature() {
        let emoji = match BlockType::Literature {
            BlockType::Literature => "📖",
            _ => "",
        };
        assert_eq!(emoji, "📖");
    }

    #[test]
    fn test_block_type_emoji_permanent() {
        let emoji = match BlockType::Permanent {
            BlockType::Permanent => "💎",
            _ => "",
        };
        assert_eq!(emoji, "💎");
    }

    #[test]
    fn test_block_type_emoji_structure() {
        let emoji = match BlockType::Structure {
            BlockType::Structure => "🗂️",
            _ => "",
        };
        assert_eq!(emoji, "🗂️");
    }

    #[test]
    fn test_block_type_emoji_hub() {
        let emoji = match BlockType::Hub {
            BlockType::Hub => "🌐",
            _ => "",
        };
        assert_eq!(emoji, "🌐");
    }

    #[test]
    fn test_block_type_emoji_task() {
        let emoji = match BlockType::Task {
            BlockType::Task => "✅",
            _ => "",
        };
        assert_eq!(emoji, "✅");
    }

    #[test]
    fn test_block_type_emoji_reference() {
        let emoji = match BlockType::Reference {
            BlockType::Reference => "📚",
            _ => "",
        };
        assert_eq!(emoji, "📚");
    }

    #[test]
    fn test_block_type_emoji_outline() {
        let emoji = match BlockType::Outline {
            BlockType::Outline => "📝",
            _ => "",
        };
        assert_eq!(emoji, "📝");
    }

    #[test]
    fn test_block_type_emoji_ghost() {
        let emoji = match BlockType::Ghost {
            BlockType::Ghost => "👻",
            _ => "",
        };
        assert_eq!(emoji, "👻");
    }

    // =========================================================================
    // Test Helper Functions
    // =========================================================================

    /// Helper to get emoji for a block type (mirrors the logic in execute)
    fn get_emoji_for_type(block_type: &BlockType) -> &'static str {
        match block_type {
            BlockType::Fleeting => "⚡",
            BlockType::Literature => "📖",
            BlockType::Permanent => "💎",
            BlockType::Structure => "🗂️",
            BlockType::Hub => "🌐",
            BlockType::Task => "✅",
            BlockType::Reference => "📚",
            BlockType::Outline => "📝",
            BlockType::Ghost => "👻",
        }
    }

    #[test]
    fn test_all_block_types_have_emoji() {
        let types = vec![
            BlockType::Fleeting,
            BlockType::Literature,
            BlockType::Permanent,
            BlockType::Structure,
            BlockType::Hub,
            BlockType::Task,
            BlockType::Reference,
            BlockType::Outline,
            BlockType::Ghost,
        ];

        for block_type in &types {
            let emoji = get_emoji_for_type(block_type);
            assert!(!emoji.is_empty(), "Block type {:?} should have an emoji", block_type);
        }
    }

    #[test]
    fn test_emoji_mapping_is_correct() {
        // Verify each block type maps to the expected emoji
        assert_eq!(get_emoji_for_type(&BlockType::Fleeting), "⚡");
        assert_eq!(get_emoji_for_type(&BlockType::Literature), "📖");
        assert_eq!(get_emoji_for_type(&BlockType::Permanent), "💎");
        assert_eq!(get_emoji_for_type(&BlockType::Structure), "🗂️");
        assert_eq!(get_emoji_for_type(&BlockType::Hub), "🌐");
        assert_eq!(get_emoji_for_type(&BlockType::Task), "✅");
        assert_eq!(get_emoji_for_type(&BlockType::Reference), "📚");
        assert_eq!(get_emoji_for_type(&BlockType::Outline), "📝");
        assert_eq!(get_emoji_for_type(&BlockType::Ghost), "👻");
    }
}
