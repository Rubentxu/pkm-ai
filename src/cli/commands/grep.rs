//! Grep command - Search block content
//!
//! Searches for text patterns in block content and titles.

use crate::db::Database;

/// Execute grep search on blocks
///
/// # Arguments
/// * `db` - Database reference
/// * `pattern` - Search pattern (regex supported)
/// * `content_only` - Only search in content, not titles
/// * `case_sensitive` - Case-sensitive search
/// * `limit` - Maximum number of results
pub async fn execute(
    db: &Database,
    pattern: &str,
    content_only: bool,
    case_sensitive: bool,
    limit: usize,
) -> anyhow::Result<()> {
    let blocks = db.blocks();
    let all_blocks = blocks.list_all().await?;

    let regex_flags = if case_sensitive { "" } else { "(?i)" };
    let regex_pattern = format!("{}{}", regex_flags, pattern);

    let re = regex::RegexBuilder::new(&regex_pattern)
        .case_insensitive(!case_sensitive)
        .build()
        .map_err(|e| anyhow::anyhow!("Invalid regex pattern: {}", e))?;

    let mut matches: Vec<MatchResult> = Vec::new();

    for block in all_blocks {
        let mut found_in_title = false;
        let mut found_in_content = false;
        let mut match_line = String::new();

        if !content_only && re.is_match(&block.title) {
            found_in_title = true;
            match_line = block.title.clone();
        }

        if re.is_match(&block.content) {
            found_in_content = true;
            // Find the matching line
            for line in block.content.lines() {
                if re.is_match(line) {
                    match_line = line.chars().take(80).collect::<String>();
                    break;
                }
            }
        }

        if found_in_title || found_in_content {
            matches.push(MatchResult {
                block_id: block.id,
                title: block.title,
                matched_in: if found_in_title && found_in_content {
                    "title+content".to_string()
                } else if found_in_title {
                    "title".to_string()
                } else {
                    "content".to_string()
                },
                match_preview: match_line,
            });
        }

        if matches.len() >= limit {
            break;
        }
    }

    // Sort by ULID timestamp (newest first)
    matches.sort_by(|a, b| b.block_id.cmp(&a.block_id));

    if matches.is_empty() {
        println!("No matches found for pattern: {}", pattern);
        return Ok(());
    }

    println!("Found {} matches for pattern: {}", matches.len(), pattern);
    println!();
    println!("┌──────────────────────────┬─────────────┬─────────────────────────────────────────────────────────────┐");
    println!("│ ULID                     │ Matched In │ Preview                                                         │");
    println!("├──────────────────────────┼─────────────┼─────────────────────────────────────────────────────────────┤");

    for m in matches.iter().take(limit) {
        let id_str = m.block_id.to_string();
        let preview = m.match_preview.chars().take(60).collect::<String>();
        println!(
            "│ {:<24} │ {:<11} │ {:<61} │",
            id_str.chars().take(24).collect::<String>(),
            m.matched_in,
            preview
        );
    }

    println!("└──────────────────────────┴─────────────┴─────────────────────────────────────────────────────────────┘");

    Ok(())
}

/// Result of a grep match
#[allow(dead_code)]
#[derive(Debug)]
struct MatchResult {
    block_id: ulid::Ulid,
    title: String,
    matched_in: String,
    match_preview: String,
}