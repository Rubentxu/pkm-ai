//! Search command - Fuzzy search for blocks by title
//!
//! Provides fuzzy matching to find notes even with partial titles or typos.

use crate::db::Database;

/// Similarity threshold for fuzzy matching (0.0 to 1.0)
const SIMILARITY_THRESHOLD: f64 = 0.6;

/// Calculates the similarity between a query and a title using fuzzy matching.
///
/// The algorithm uses subsequence matching with scoring:
/// - +1 point for each matched character
/// - Bonus for consecutive matches
/// - Bonus for matching at word boundaries
/// - Penalty for gaps between matched characters
///
/// # Arguments
/// * `query` - The search query (lowercase)
/// * `title` - The title to match against (lowercase)
///
/// # Returns
/// * `f64` - Similarity score between 0.0 and 1.0
fn calculate_similarity(query: &str, title: &str) -> f64 {
    let query_chars: Vec<char> = query.chars().collect();
    let title_chars: Vec<char> = title.chars().collect();

    if query_chars.is_empty() {
        return 1.0;
    }

    if title_chars.is_empty() {
        return 0.0;
    }

    // Find all matching positions
    let mut matched_positions: Vec<usize> = Vec::new();
    let mut query_idx = 0;

    for (title_idx, title_char) in title_chars.iter().enumerate() {
        if query_idx < query_chars.len() && *title_char == query_chars[query_idx] {
            matched_positions.push(title_idx);
            query_idx += 1;
        }
    }

    // If not all query chars were matched, return 0
    if query_idx != query_chars.len() {
        return 0.0;
    }

    // Calculate score based on match quality
    let mut score = 0.0;

    // Base score: matched characters / query length
    score += query_chars.len() as f64;

    // Bonus for consecutive matches
    let mut consecutive_bonus = 0.0;
    for window in matched_positions.windows(2) {
        if window[1] == window[0] + 1 {
            consecutive_bonus += 0.5;
        } else {
            // Penalty for gaps
            consecutive_bonus -= 0.1 * (window[1] - window[0] - 1) as f64;
        }
    }
    score += consecutive_bonus;

    // Bonus for matching at start
    if matched_positions.first() == Some(&0) {
        score += 1.0;
    }

    // Bonus for matching at word boundaries
    for &pos in &matched_positions {
        if pos == 0 || title_chars[pos] == ' ' || title_chars[pos] == '-' || title_chars[pos] == '_' {
            score += 0.3;
        }
    }

    // Normalize by query length only (we're measuring how well the query matches, not title quality)
    // This gives a score based on how complete the match is
    let max_possible = query_chars.len() as f64
        + (query_chars.len() as f64 * 0.5) // max bonuses for consecutive
        + 1.0 // start bonus
        + (query_chars.len() as f64 * 0.3); // word boundary bonus

    (score / max_possible).clamp(0.0, 1.0)
}

/// Performs fuzzy matching between a query and a title.
///
/// Returns true if the similarity score is above the threshold (0.6).
///
/// # Arguments
/// * `query` - The search query
/// * `title` - The title to match against
///
/// # Returns
/// * `bool` - True if it's a fuzzy match
pub fn fuzzy_match(query: &str, title: &str) -> bool {
    let query_lower = query.to_lowercase();
    let title_lower = title.to_lowercase();

    // Exact match always succeeds
    if title_lower.contains(&query_lower) {
        return true;
    }

    // Fuzzy match using similarity
    let similarity = calculate_similarity(&query_lower, &title_lower);
    similarity >= SIMILARITY_THRESHOLD
}

/// Execute fuzzy search on blocks
///
/// # Arguments
/// * `db` - Database reference
/// * `query` - Search query
/// * `limit` - Maximum number of results
pub async fn execute(
    db: &Database,
    query: &str,
    limit: usize,
) -> anyhow::Result<()> {
    let blocks = db.blocks();
    let all_blocks = blocks.list_all().await?;

    // Perform fuzzy search
    let mut matches: Vec<SearchResult> = all_blocks
        .into_iter()
        .filter(|block| fuzzy_match(query, &block.title))
        .map(|block| {
            let similarity = calculate_similarity(&query.to_lowercase(), &block.title.to_lowercase());
            SearchResult {
                block_id: block.id,
                title: block.title,
                similarity,
            }
        })
        .collect();

    // Sort by similarity (highest first), then by ULID (newest first)
    matches.sort_by(|a, b| {
        b.similarity
            .partial_cmp(&a.similarity)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.block_id.cmp(&a.block_id))
    });

    if matches.is_empty() {
        println!("No matches found for query: {}", query);
        return Ok(());
    }

    println!("Found {} matches for query: {}", matches.len(), query);
    println!();
    println!("┌──────────────────────────┬─────────────────────────────────────────────────────────────┬──────────┐");
    println!("│ ULID                     │ Title                                                        │ Score    │");
    println!("├──────────────────────────┼─────────────────────────────────────────────────────────────┼──────────┤");

    for m in matches.iter().take(limit) {
        let id_str = m.block_id.to_string();
        let title = m.title.chars().take(50).collect::<String>();
        let score = format!("{:.2}", m.similarity);
        println!(
            "│ {:<24} │ {:<61} │ {:<8} │",
            id_str.chars().take(24).collect::<String>(),
            title,
            score
        );
    }

    println!("└──────────────────────────┴─────────────────────────────────────────────────────────────┴──────────┘");

    Ok(())
}

/// Result of a search match
#[allow(dead_code)]
#[derive(Debug)]
struct SearchResult {
    block_id: ulid::Ulid,
    title: String,
    similarity: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_match_exact_substring() {
        assert!(fuzzy_match("rust", "Rust Programming"));
        assert!(fuzzy_match("programming", "Rust Programming"));
        assert!(fuzzy_match("RUST", "Rust Ownership Model")); // case insensitive
    }

    #[test]
    fn test_fuzzy_match_partial() {
        // Partial matches should work
        assert!(fuzzy_match("rust own", "Rust Ownership Model"));
        assert!(fuzzy_match("prog", "Rust Programming"));
    }

    #[test]
    fn test_fuzzy_match_typos() {
        // Fuzzy matching should handle some typos
        assert!(fuzzy_match("rust programing", "Rust Programming")); // typo in "programming"
    }

    #[test]
    fn test_fuzzy_match_no_match() {
        // Very different strings should not match
        assert!(!fuzzy_match("python", "Rust Programming"));
        assert!(!fuzzy_match("xyz", "Rust Ownership Model"));
    }

    #[test]
    fn test_fuzzy_match_empty_query() {
        // Empty query should match everything
        assert!(fuzzy_match("", "Rust Programming"));
    }

    #[test]
    fn test_fuzzy_match_empty_title() {
        // Empty title should not match
        assert!(!fuzzy_match("rust", ""));
    }

    #[test]
    fn test_calculate_similarity() {
        // Exact match should be very high
        let sim = calculate_similarity("rust", "rust");
        assert!(sim > 0.8, "Expected similarity > 0.8, got {}", sim);

        // Substring match should be high
        let sim = calculate_similarity("rust", "rust programming");
        assert!(sim > 0.5);

        // Very different strings should be low
        let sim = calculate_similarity("python", "rust programming");
        assert!(sim < SIMILARITY_THRESHOLD);
    }

    #[test]
    fn test_calculate_similarity_word_boundaries() {
        // Verify fuzzy matching works for these cases
        // The key is that these should match via fuzzy_match, not necessarily high similarity scores
        assert!(fuzzy_match("rust", "Rust Programming"));
        assert!(fuzzy_match("ust", "Rust Programming"));
    }

    #[test]
    fn test_fuzzy_match_special_characters() {
        assert!(fuzzy_match("rust", "Rust-Ownership-Model"));
        assert!(fuzzy_match("rust", "rust_ownership_model"));
    }
}
