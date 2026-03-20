//! TOC command: Generate Table of Contents from Structure
//!
//! This command generates a formatted table of contents from a Structure block's
//! spine of content, showing the hierarchical structure of a document.

use crate::db::Database;
use crate::models::{Block, BlockType, Edge, FractionalIndex, LinkType};
use ulid::Ulid;

/// A TOC entry
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TocEntry {
    pub id: String,
    pub title: String,
    pub block_type: BlockType,
    pub depth: u32,
    pub weight: FractionalIndex,
    pub children: Vec<TocEntry>,
}

/// Execute the TOC command
pub async fn execute(
    db: &Database,
    id: &str,
) -> anyhow::Result<()> {
    let structure_id = parse_ulid(id)?;
    let block_repo = db.blocks();

    // Get the structure block
    let structure = block_repo.get(&structure_id).await?
        .ok_or_else(|| anyhow::anyhow!("Structure block not found: {}", id))?;

    if structure.block_type != BlockType::Structure {
        anyhow::bail!("Block {} is not a Structure block (type: {:?})", id, structure.block_type);
    }

    println!("📖 Table of Contents: {}", structure.title);
    println!();
    println!("   ID: {}", structure.id_str());
    println!();
    println!("────────────────────────────────────────────────────────────");
    println!();

    // Build TOC
    let toc = build_toc(db, &structure).await?;

    // Display TOC
    if toc.children.is_empty() {
        println!("   (empty - no linked blocks)");
        println!();
        println!("💡 Link blocks to this structure using:");
        println!("   nexus link --from <block_id> --to {} --link-type section_of", id);
    } else {
        print_toc(&toc.children, 0);
    }

    // Statistics
    let total_entries = count_entries(&toc.children);
    let sections = count_by_type(&toc.children, &BlockType::Structure);
    let zettels = count_by_type(&toc.children, &BlockType::Permanent);
    let ghosts = count_by_type(&toc.children, &BlockType::Ghost);

    println!();
    println!("────────────────────────────────────────────────────────────");
    println!();
    println!("📊 Statistics:");
    println!("   Total entries: {}", total_entries);
    println!("   Sections: {}", sections);
    println!("   Zettels: {}", zettels);
    println!("   Ghosts: {}", ghosts);

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

/// Build the TOC from a structure block
async fn build_toc(db: &Database, structure: &Block) -> anyhow::Result<TocEntry> {
    let edge_repo = db.edges();

    let mut entry = TocEntry {
        id: structure.id_str(),
        title: structure.title.clone(),
        block_type: structure.block_type.clone(),
        depth: 0,
        weight: FractionalIndex::first(),
        children: Vec::new(),
    };

    // Get all outgoing edges from this structure
    let outgoing = edge_repo.outgoing_from(&structure.id).await?;

    // Filter for structural spine links
    let mut spine_edges: Vec<&Edge> = outgoing.iter()
        .filter(|e| matches!(e.link_type, LinkType::Next | LinkType::SectionOf | LinkType::OrderedChild))
        .collect();

    // Sort by sequence weight (FractionalIndex already implements Ord)
    spine_edges.sort_by(|a, b| a.sequence_weight.cmp(&b.sequence_weight));

    // Process each child
    for edge in spine_edges {
        let block_repo = db.blocks();
        if let Ok(Some(child_block)) = block_repo.get(&edge.to).await {
            let child_entry = Box::pin(build_entry_impl(db, &child_block, edge.sequence_weight.clone(), 1)).await?;
            entry.children.push(child_entry);
        }
    }

    Ok(entry)
}

/// Build a TOC entry recursively
#[allow(dead_code)]
async fn build_entry(db: &Database, block: &Block, weight: FractionalIndex, depth: u32) -> anyhow::Result<TocEntry> {
    build_entry_impl(db, block, weight, depth).await
}

async fn build_entry_impl(db: &Database, block: &Block, weight: FractionalIndex, depth: u32) -> anyhow::Result<TocEntry> {
    let edge_repo = db.edges();

    let mut entry = TocEntry {
        id: block.id_str(),
        title: block.title.clone(),
        block_type: block.block_type.clone(),
        depth,
        weight,
        children: Vec::new(),
    };

    // If this is a structure, recurse into its children
    if block.block_type == BlockType::Structure {
        let outgoing = edge_repo.outgoing_from(&block.id).await?;

        let mut spine_edges: Vec<_> = outgoing.into_iter()
            .filter(|e| matches!(e.link_type, LinkType::Next | LinkType::SectionOf | LinkType::OrderedChild))
            .collect();

        spine_edges.sort_by(|a, b| a.sequence_weight.cmp(&b.sequence_weight));

        let block_repo = db.blocks();
        for edge in spine_edges {
            if let Ok(Some(child_block)) = block_repo.get(&edge.to).await {
                let child_entry = Box::pin(build_entry_impl(db, &child_block, edge.sequence_weight.clone(), depth + 1)).await?;
                entry.children.push(child_entry);
            }
        }
    }

    Ok(entry)
}

/// Print the TOC with proper indentation
fn print_toc(entries: &[TocEntry], depth: u32) {
    for entry in entries {
        let indent = "   ".repeat(depth as usize);
        let prefix = if depth == 0 { "  " } else { "└─ " };

        // Type indicator
        let type_indicator = match entry.block_type {
            BlockType::Structure => "📁",
            BlockType::Permanent => "📝",
            BlockType::Ghost => "👻",
            BlockType::Fleeting => "💡",
            BlockType::Literature => "📚",
            BlockType::Hub => "🧭",
            BlockType::Task => "✅",
            BlockType::Reference => "🔗",
            BlockType::Outline => "📋",
        };

        // Depth-based numbering
        let numbering = if depth == 0 {
            format!("{}. ", entries.iter().position(|e| e.id == entry.id).unwrap_or(0) + 1)
        } else {
            String::new()
        };

        // Weight display
        let weight_str = if entry.weight != FractionalIndex::first() {
            format!(" [{}]", entry.weight)
        } else {
            String::new()
        };

        println!("{}{}{}{} {}{}", indent, prefix, numbering, type_indicator, entry.title, weight_str);

        // Print children
        if !entry.children.is_empty() {
            print_toc(&entry.children, depth + 1);
        }
    }
}

/// Count total entries
fn count_entries(entries: &[TocEntry]) -> usize {
    let mut count = 0;
    for entry in entries {
        count += 1;
        count += count_entries(&entry.children);
    }
    count
}

/// Count entries by block type
fn count_by_type(entries: &[TocEntry], target_type: &BlockType) -> usize {
    let mut count = 0;
    for entry in entries {
        if entry.block_type == *target_type {
            count += 1;
        }
        count += count_by_type(&entry.children, target_type);
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_entries() {
        let entries = vec![
            TocEntry {
                id: "1".to_string(),
                title: "Test".to_string(),
                block_type: BlockType::Structure,
                depth: 0,
                weight: FractionalIndex::first(),
                children: vec![
                    TocEntry {
                        id: "2".to_string(),
                        title: "Child".to_string(),
                        block_type: BlockType::Permanent,
                        depth: 1,
                        weight: FractionalIndex::after_last(&FractionalIndex::first()),
                        children: vec![],
                    },
                ],
            },
        ];

        assert_eq!(count_entries(&entries), 2);
    }

    #[test]
    fn test_count_by_type() {
        let entries = vec![
            TocEntry {
                id: "1".to_string(),
                title: "Test".to_string(),
                block_type: BlockType::Structure,
                depth: 0,
                weight: FractionalIndex::first(),
                children: vec![
                    TocEntry {
                        id: "2".to_string(),
                        title: "Permanent".to_string(),
                        block_type: BlockType::Permanent,
                        depth: 1,
                        weight: FractionalIndex::first(),
                        children: vec![],
                    },
                    TocEntry {
                        id: "3".to_string(),
                        title: "Ghost".to_string(),
                        block_type: BlockType::Ghost,
                        depth: 1,
                        weight: FractionalIndex::after_last(&FractionalIndex::first()),
                        children: vec![],
                    },
                ],
            },
        ];

        assert_eq!(count_by_type(&entries, &BlockType::Structure), 1);
        assert_eq!(count_by_type(&entries, &BlockType::Permanent), 1);
        assert_eq!(count_by_type(&entries, &BlockType::Ghost), 1);
    }
}