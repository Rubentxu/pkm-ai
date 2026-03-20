//! Gravity Check command: Validate semantic placement of blocks
//!
//! This command checks if a block is in the correct section based on
//! semantic gravity - how well the block's content matches section boundaries.

use crate::db::Database;
use crate::models::{Block, BlockType, SmartSection};
use ulid::Ulid;

/// A section candidate for gravity check
#[derive(Debug, Clone)]
pub struct SectionCandidate {
    pub block: Block,
    pub affinity: f32,
    pub density: u32,
    pub vacancy: String,
}

/// Execute gravity check for a specific block
pub async fn execute(
    db: &Database,
    id: &str,
    threshold: f32,
) -> anyhow::Result<()> {
    let block_id = parse_ulid(id)?;
    let block_repo = db.blocks();

    // Get the block to check
    let block = block_repo.get(&block_id).await?
        .ok_or_else(|| anyhow::anyhow!("Block not found: {}", id))?;

    println!("🌌 Gravity Check for block: {}", block.title);
    println!("   ID: {}", block.id_str());
    println!("   Type: {:?}", block.block_type);
    println!();

    // Find all structure sections
    let structures = block_repo.list_by_type(BlockType::Structure).await?;

    if structures.is_empty() {
        println!("⚠️  No structure sections found in database.");
        println!("   Create a Structure block first with `nexus create -t structure`");
        return Ok(());
    }

    // Calculate gravity for each section
    let mut candidates: Vec<SectionCandidate> = Vec::new();

    for structure in &structures {
        let section = build_smart_section(structure);
        let affinity = calculate_affinity(&block, &section);

        candidates.push(SectionCandidate {
            block: structure.clone(),
            affinity,
            density: section.density,
            vacancy: format!("{:?}", section.vacancy).to_lowercase(),
        });
    }

    // Sort by affinity
    candidates.sort_by(|a, b| b.affinity.partial_cmp(&a.affinity).unwrap_or(std::cmp::Ordering::Equal));

    // Find current section
    let current_section = find_current_section(db, &block_id).await?;

    // Display current location
    if let Some(current) = &current_section {
        println!("📍 Current Location:");
        println!("   Section: [{}]", current.block.title);
        println!("   Affinity: {:.2}", current.affinity);
        println!();
    } else {
        println!("📍 Current Location:");
        println!("   Section: (none - this block may be an orphan)");
        println!();
    }

    // Display alternative sections
    let top_candidates: Vec<_> = candidates.iter().take(5).collect();

    if top_candidates.len() > 1 {
        println!("🔗 Alternative Sections (Top {}):", top_candidates.len());
        println!();

        for (i, candidate) in top_candidates.iter().enumerate() {
            let is_current = current_section.as_ref()
                .map(|c| c.block.id == candidate.block.id)
                .unwrap_or(false);

            let marker = if is_current { "(current)" } else { "" };
            let improvement = current_section.as_ref()
                .map(|c| candidate.affinity - c.affinity)
                .unwrap_or(0.0);

            let arrow = if improvement > threshold {
                " ⭐"
            } else if improvement > 0.0 {
                " (+)"
            } else {
                ""
            };

            println!("  {}. [{}] {}", i + 1, candidate.block.title, marker);
            println!("     Affinity: {:.2}{}", candidate.affinity, arrow);

            if improvement > threshold {
                println!("     Reason: Higher affinity than current location!");
            } else if improvement > 0.0 {
                println!("     Reason: +{:.2} higher than current", improvement);
            }

            println!("     Density: {} blocks ({})", candidate.density, candidate.vacancy);
            println!();
        }
    }

    // Generate recommendation
    if let Some(current) = &current_section {
        let best = &candidates[0];

        if best.affinity - current.affinity > threshold {
            println!("⚠️  Recommendation:");
            println!();
            println!("   This block has HIGHER affinity with [{}] ({:.2})", best.block.title, best.affinity);
            println!("   than its current section ({:.2})", current.affinity);
            println!();
            println!("   💡 Consider moving to [{}] for better semantic placement.", best.block.title);
            println!();
            println!("   To move: nexus reorder {} --to-section {}", id, best.block.id_str().chars().take(8).collect::<String>());
        } else if best.affinity > 0.7 {
            println!("✅ The block is well-placed in its current section.");
        } else {
            println!("ℹ️  This block has moderate affinity with all sections.");
            println!("   It may be a general-purpose or transitional note.");
        }
    } else {
        // Block is not in any section
        if let Some(best) = candidates.first() {
            println!("💡 This block is not linked to any section.");
            println!();
            if best.affinity > 0.5 {
                println!("   Suggested section: [{}] (affinity: {:.2})", best.block.title, best.affinity);
                println!();
                println!("   To link: nexus link --from {} --to {} --link-type section_of", id, best.block.id_str().chars().take(8).collect::<String>());
            } else {
                println!("   No section has strong affinity with this block.");
                println!("   Consider creating a new section or linking manually.");
            }
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
    anyhow::bail!("Invalid ULID format: {}", s)
}

/// Build a smart section from a structure block
fn build_smart_section(structure: &Block) -> SmartSection {
    let mut section = SmartSection::new(&structure.title);

    // Extract intent from title
    section.intent = structure.title.clone();

    // Extract keywords from content or metadata
    if !structure.content.is_empty() {
        // Use content words as keywords
        let words: Vec<String> = structure.content
            .split_whitespace()
            .take(20)
            .map(|w| w.to_lowercase().chars().filter(|c| c.is_alphanumeric()).collect::<String>())
            .filter(|w: &String| w.len() > 3)
            .collect();
        section.keywords = words;
    }

    // Extract keywords from tags
    for tag in &structure.tags {
        section.keywords.push(tag.to_lowercase());
    }

    // Extract boundary constraints from metadata
    if let Some(constraints) = structure.metadata.get("boundary_constraints")
        && let Some(constraints_arr) = constraints.as_array() {
            for c in constraints_arr {
                if let Some(s) = c.as_str() {
                    section.boundary_constraints.push(s.to_string());
                }
            }
        }

    // Set density (would need edge query in production)
    section.density = 0;
    section.vacancy = crate::models::VacancyLevel::Empty;

    section
}

/// Calculate semantic affinity between a block and a section
fn calculate_affinity(block: &Block, section: &SmartSection) -> f32 {
    let mut score: f32 = 0.0;
    let mut weight: f32 = 0.0;

    // Title keyword matching
    let block_title_words: Vec<String> = block.title
        .to_lowercase()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    for keyword in &section.keywords {
        weight += 1.0;
        if block_title_words.iter().any(|w| w.contains(&keyword.to_lowercase())) {
            score += 1.0;
        }
    }

    // Content keyword matching
    let block_content_words: Vec<String> = block.content
        .to_lowercase()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    for keyword in &section.keywords {
        if block_content_words.iter().any(|w| w.contains(&keyword.to_lowercase())) {
            score += 0.5;
        }
    }

    // Tag matching
    for tag in &block.tags {
        if section.keywords.iter().any(|k| tag.to_lowercase().contains(&k.to_lowercase())) {
            score += 2.0;
            weight += 2.0;
        }
    }

    // Boundary constraint matching
    for constraint in &section.boundary_constraints {
        let constraint_lower = constraint.to_lowercase();
        if block.content.to_lowercase().contains(&constraint_lower)
            || block.title.to_lowercase().contains(&constraint_lower) {
            score += 1.5;
            weight += 1.5;
        }
    }

    if weight == 0.0 {
        // Default affinity if no matching criteria
        return 0.3;
    }

    (score / weight).min(1.0)
}

/// Find which section a block currently belongs to
async fn find_current_section(db: &Database, block_id: &Ulid) -> anyhow::Result<Option<SectionCandidate>> {
    let edge_repo = db.edges();

    // Get incoming edges to this block
    let incoming = edge_repo.incoming_to(block_id).await?;

    // Find section_of or next edges from a structure
    for edge in incoming {
        if matches!(edge.link_type, crate::models::LinkType::SectionOf | crate::models::LinkType::Contains) {
            let block_repo = db.blocks();
            if let Ok(Some(structure)) = block_repo.get(&edge.from).await
                && structure.block_type == BlockType::Structure {
                    let section = build_smart_section(&structure);
                    let affinity = calculate_affinity_by_id(db, block_id, &section).await?;

                    return Ok(Some(SectionCandidate {
                        block: structure,
                        affinity,
                        density: section.density,
                        vacancy: format!("{:?}", section.vacancy).to_lowercase(),
                    }));
                }
        }
    }

    Ok(None)
}

/// Calculate affinity by fetching block content
async fn calculate_affinity_by_id(db: &Database, block_id: &Ulid, section: &SmartSection) -> anyhow::Result<f32> {
    let block_repo = db.blocks();
    if let Some(block) = block_repo.get(block_id).await? {
        Ok(calculate_affinity(&block, section))
    } else {
        Ok(0.3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ulid() {
        let ulid = parse_ulid("01ARZ3NDEKTSV4RRFFQ69G5FAV").unwrap();
        assert_eq!(ulid.to_string(), "01ARZ3NDEKTSV4RRFFQ69G5FAV");
    }

    #[test]
    fn test_affinity_calculation() {
        let section = SmartSection::new("Rust Programming")
            .with_boundary("rust")
            .with_boundary("ownership");

        let block = Block::permanent("Rust Ownership", "Rust ownership is important");

        let affinity = calculate_affinity(&block, &section);
        assert!(affinity > 0.5, "Expected affinity > 0.5, got {}", affinity);
    }
}