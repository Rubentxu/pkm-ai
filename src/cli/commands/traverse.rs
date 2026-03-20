//! Traverse command: Walk the Structural Spine in deterministic order
//!
//! This command traverses the Structural Spine (Folgezettel digital) and displays
//! blocks in their deterministic sequence order based on sequence_weight.

use crate::db::Database;
use crate::models::{Block, BlockType, Edge, FractionalIndex, LinkType};
use ulid::Ulid;

/// TraversalResult represents a node in the spine traversal
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TraversalNode {
    pub block: Block,
    pub depth: u32,
    pub children: Vec<TraversalNode>,
    pub sequence_weight: FractionalIndex,
}

/// Output format for traversal
#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(Default)]
pub enum TraverseFormat {
    #[default]
    Tree,
    Json,
    Markdown,
    Simple,
}


impl std::str::FromStr for TraverseFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "tree" => Ok(Self::Tree),
            "json" => Ok(Self::Json),
            "markdown" | "md" => Ok(Self::Markdown),
            "simple" => Ok(Self::Simple),
            _ => Err(format!("Unknown format: {}. Use tree, json, markdown, or simple", s)),
        }
    }
}

/// Execute the traverse command
///
/// Traverses the Structural Spine from a starting point (or root) and displays
/// blocks in deterministic order based on sequence_weight.
pub async fn execute(
    db: &Database,
    from: &Option<String>,
    depth: u32,
    link_type: &Option<String>,
    content: bool,
) -> anyhow::Result<()> {
    let format = TraverseFormat::Tree;

    // Parse starting ID or find root structures
    let start_id = if let Some(id_str) = from {
        Some(parse_ulid(id_str)?)
    } else {
        None
    };

    // Query blocks to traverse
    let nodes = if let Some(start) = start_id {
        traverse_from_block(db, start, depth).await?
    } else {
        traverse_from_root(db, depth).await?
    };

    if nodes.is_empty() {
        println!("📭 No blocks found in the structural spine.");
        println!("   Create a Structure block and link blocks with 'nexus link --link-type next'");
        return Ok(());
    }

    // Calculate statistics
    let total_blocks = count_blocks(&nodes);
    let sections = nodes.iter().filter(|n| n.block.block_type == BlockType::Structure).count();
    let zettels = nodes.iter().filter(|n| n.block.block_type == BlockType::Permanent).count();
    let ghosts = nodes.iter().filter(|n| n.block.block_type == BlockType::Ghost).count();

    match format {
        TraverseFormat::Tree => {
            print_tree(&nodes, 0, content);
        }
        TraverseFormat::Json => {
            print_json(&nodes, content)?;
        }
        TraverseFormat::Markdown => {
            print_markdown(&nodes, 0, content);
        }
        TraverseFormat::Simple => {
            print_simple(&nodes, content);
        }
    }

    // Print statistics
    println!();
    println!("📊 Statistics:");
    println!("   Total blocks: {}", total_blocks);
    println!("   Sections: {}", sections);
    println!("   Zettels: {}", zettels);
    println!("   Ghosts: {} ({}% of content)", ghosts, if total_blocks > 0 { (ghosts * 100) / total_blocks } else { 0 });

    // Check for orphans if requested
    if link_type.as_ref().map(|s| s == "orphans").unwrap_or(false) {
        print_orphans(db).await?;
    }

    Ok(())
}

/// Parse a ULID from string, handling various formats
fn parse_ulid(s: &str) -> anyhow::Result<Ulid> {
    // Try direct parsing first
    if let Ok(ulid) = s.parse::<Ulid>() {
        return Ok(ulid);
    }

    // Try with record prefix (e.g., "block:01ABCD...")
    if let Some(inner) = s.strip_prefix("block:")
        && let Ok(ulid) = inner.parse::<Ulid>() {
        return Ok(ulid);
    }

    anyhow::bail!("Invalid ULID format: {}", s)
}

/// Traverse from a specific block
async fn traverse_from_block(db: &Database, start_id: Ulid, max_depth: u32) -> anyhow::Result<Vec<TraversalNode>> {
    let block_repo = db.blocks();
    let _edge_repo = db.edges();

    // Get the starting block
    let start_block = block_repo.get(&start_id).await?
        .ok_or_else(|| anyhow::anyhow!("Block not found: {}", start_id))?;

    // Build traversal tree
    let mut nodes = Vec::new();
    traverse_recursive(db, &start_block, 0, max_depth, &mut nodes).await?;

    Ok(nodes)
}

/// Traverse from root structure blocks
async fn traverse_from_root(db: &Database, max_depth: u32) -> anyhow::Result<Vec<TraversalNode>> {
    let block_repo = db.blocks();

    // Find all root-level structure blocks (no incoming "section_of" edges)
    // These are typically Structure, Hub, or Outline blocks at the top level
    let all_blocks = block_repo.list_by_type(BlockType::Structure).await?;

    let mut nodes = Vec::new();

    // For each structure, get its spine children
    for structure_block in all_blocks {
        let structure_nodes = traverse_from_block(db, structure_block.id, max_depth).await?;
        nodes.extend(structure_nodes);
    }

    Ok(nodes)
}

/// Recursively traverse the spine
async fn traverse_recursive(
    db: &Database,
    block: &Block,
    current_depth: u32,
    max_depth: u32,
    nodes: &mut Vec<TraversalNode>,
) -> anyhow::Result<()> {
    traverse_recursive_impl(db, block, current_depth, max_depth, nodes).await
}

async fn traverse_recursive_impl(
    db: &Database,
    block: &Block,
    current_depth: u32,
    max_depth: u32,
    nodes: &mut Vec<TraversalNode>,
) -> anyhow::Result<()> {
    if current_depth >= max_depth && max_depth > 0 {
        return Ok(());
    }

    let edge_repo = db.edges();

    // Get outgoing "next" edges ordered by sequence_weight
    let outgoing = edge_repo.outgoing_from(&block.id).await?;

    // Filter for structural spine links (next, next_sibling)
    let spine_edges: Vec<&Edge> = outgoing.iter()
        .filter(|e| matches!(e.link_type, LinkType::Next | LinkType::NextSibling))
        .collect();

    // Sort by sequence_weight (FractionalIndex already implements Ord)
    let mut sorted_edges: Vec<_> = spine_edges.into_iter().collect();
    sorted_edges.sort_by(|a, b| a.sequence_weight.cmp(&b.sequence_weight));

    let block_repo = db.blocks();

    for edge in sorted_edges {
        if let Ok(Some(child_block)) = block_repo.get(&edge.to).await {
            let child_nodes = if child_block.block_type == BlockType::Structure {
                // For structures, recurse to get their children
                let mut children = Vec::new();
                Box::pin(traverse_recursive_impl(db, &child_block, current_depth + 1, max_depth, &mut children)).await?;
                children
            } else {
                Vec::new()
            };

            nodes.push(TraversalNode {
                block: child_block,
                depth: current_depth,
                children: child_nodes,
                sequence_weight: edge.sequence_weight.clone(),
            });

            // If this is a structure, we've already recursed, no need to add children separately
        }
    }

    Ok(())
}

/// Count total blocks in traversal
fn count_blocks(nodes: &[TraversalNode]) -> usize {
    let mut count = 0;
    for node in nodes {
        count += 1;
        count += count_blocks(&node.children);
    }
    count
}

/// Print traversal as a tree
fn print_tree(nodes: &[TraversalNode], indent: u32, show_content: bool) {
    for node in nodes {
        let prefix = "  ".repeat(indent as usize);
        let block = &node.block;

        // Determine block type indicator
        let type_indicator = match block.block_type {
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

        // Status indicator
        let status = if block.content.is_empty() {
            "empty ⚠️"
        } else {
            "stable ✅"
        };

        println!("{}{} [{}] {} ({})", prefix, type_indicator, block.id_str().chars().take(8).collect::<String>(), block.title, status);

        // Show sequence weight (if not the first/default position)
        if node.sequence_weight != FractionalIndex::first() {
            println!("{}   └─ Weight: {}", prefix, node.sequence_weight);
        }

        // Show content preview if requested
        if show_content && !block.content.is_empty() {
            let preview = block.content.chars().take(100).collect::<String>();
            println!("{}   └─ \"{}\"", prefix, preview);
        }

        // Recurse into children
        if !node.children.is_empty() {
            print_tree(&node.children, indent + 1, show_content);
        }
    }
}

/// Print traversal as JSON
fn print_json(nodes: &[TraversalNode], show_content: bool) -> anyhow::Result<()> {
    #[derive(serde::Serialize)]
    struct NodeJson {
        id: String,
        title: String,
        block_type: String,
        weight: String,
        content_preview: Option<String>,
        children: Vec<NodeJson>,
    }

    fn convert_node(node: &TraversalNode, show_content: bool) -> NodeJson {
        NodeJson {
            id: node.block.id_str(),
            title: node.block.title.clone(),
            block_type: format!("{:?}", node.block.block_type).to_lowercase(),
            weight: node.sequence_weight.to_string(),
            content_preview: if show_content && !node.block.content.is_empty() {
                Some(node.block.content.chars().take(100).collect())
            } else {
                None
            },
            children: node.children.iter().map(|c| convert_node(c, show_content)).collect(),
        }
    }

    let json_nodes: Vec<NodeJson> = nodes.iter().map(|n| convert_node(n, show_content)).collect();
    println!("{}", serde_json::to_string_pretty(&json_nodes)?);
    Ok(())
}

/// Print traversal as Markdown
fn print_markdown(nodes: &[TraversalNode], indent: u32, show_content: bool) {
    for node in nodes {
        let prefix = "  ".repeat(indent as usize);
        let block = &node.block;

        // Markdown heading based on depth
        let heading = match indent {
            0 => "#",
            1 => "##",
            2 => "###",
            _ => "####",
        };

        println!("{} {} {}", prefix, heading, block.title);

        if node.sequence_weight != FractionalIndex::first() {
            println!("{} - Weight: {}", prefix, node.sequence_weight);
        }

        if show_content && !block.content.is_empty() {
            println!();
            println!("{}", block.content);
            println!();
        }

        if !node.children.is_empty() {
            print_markdown(&node.children, indent + 1, show_content);
        }
    }
}

/// Print traversal as simple list
fn print_simple(nodes: &[TraversalNode], show_content: bool) {
    for node in nodes {
        let block = &node.block;
        println!("[{}] {} | {:?}", block.id_str().chars().take(8).collect::<String>(), block.title, block.block_type);

        if show_content && !block.content.is_empty() {
            let preview = block.content.chars().take(60).collect::<String>();
            println!("       \"{}\"", preview);
        }

        if !node.children.is_empty() {
            print_simple(&node.children, show_content);
        }
    }
}

/// Print orphan blocks (blocks without incoming NEXT edges)
async fn print_orphans(db: &Database) -> anyhow::Result<()> {
    let block_repo = db.blocks();
    let edge_repo = db.edges();

    // Get all blocks
    let structures = block_repo.list_by_type(BlockType::Structure).await?;
    let permanents = block_repo.list_by_type(BlockType::Permanent).await?;

    let mut all_blocks: Vec<Block> = structures;
    all_blocks.extend(permanents);

    println!();
    println!("⚠️  Orphan Blocks (no NEXT link):");
    println!();

    let mut orphan_count = 0;
    for block in &all_blocks {
        let incoming = edge_repo.incoming_to(&block.id).await?;
        let has_next_incoming = incoming.iter().any(|e| matches!(e.link_type, LinkType::Next | LinkType::NextSibling));

        if !has_next_incoming && orphan_count < 10 {
            orphan_count += 1;
            println!("  {}. [{}] {}", orphan_count, block.id_str().chars().take(8).collect::<String>(), block.title);
            println!("     Status: {:?}", block.block_type);
            if block.content.is_empty() {
                println!("     Content: empty ⚠️");
            }
            println!();
        }
    }

    if orphan_count == 0 {
        println!("   No orphan blocks found. All blocks are linked in the spine.");
    } else {
        println!("💡 Run `nexus link --auto-locate` to fix automatically");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_traverse_format_parsing() {
        assert_eq!(TraverseFormat::Tree, "tree".parse().unwrap());
        assert_eq!(TraverseFormat::Json, "json".parse().unwrap());
        assert_eq!(TraverseFormat::Markdown, "markdown".parse().unwrap());
        assert_eq!(TraverseFormat::Simple, "simple".parse().unwrap());
        assert_eq!(TraverseFormat::Markdown, "md".parse().unwrap());
        assert!("unknown".parse::<TraverseFormat>().is_err());
    }

    #[test]
    fn test_count_blocks() {
        let nodes = vec![
            TraversalNode {
                block: Block::structure("Test"),
                depth: 0,
                children: vec![
                    TraversalNode {
                        block: Block::permanent("Child 1", "Content"),
                        depth: 1,
                        children: vec![],
                        sequence_weight: FractionalIndex::first(),
                    },
                ],
                sequence_weight: FractionalIndex::first(),
            },
        ];

        assert_eq!(count_blocks(&nodes), 2);
    }
}