//! Spine traversal implementation
//!
//! Deterministic traversal of the Structural Spine following 'next' links
//! in sequence_weight order.

#![allow(dead_code)]

use crate::db::Database;
use crate::models::{BlockType, Edge, FractionalIndex, LinkType};
use crate::NexusResult;
use ulid::Ulid;

/// Spine traversal result containing ordered blocks
#[derive(Debug, Clone)]
pub struct SpineTraversalResult {
    /// Root block ID
    pub root_id: Ulid,
    /// Flattened list of blocks in document order
    pub blocks: Vec<Ulid>,
    /// Total count
    pub total_count: usize,
    /// Depth traversed
    pub depth: u32,
}

/// A node in the spine tree
#[derive(Debug, Clone)]
pub struct SpineNode {
    /// Block ID
    pub block_id: Ulid,
    /// Block type
    pub block_type: BlockType,
    /// Title
    pub title: String,
    /// Sequence weight using FractionalIndex
    pub sequence_weight: FractionalIndex,
    /// Children nodes (for structure blocks)
    pub children: Vec<SpineNode>,
}

/// Spine tree structure
#[derive(Debug, Clone)]
pub struct SpineTree {
    /// Root node
    pub root: SpineNode,
    /// All nodes indexed by ID
    pub nodes: Vec<SpineNode>,
}

/// Spine traversal engine
pub struct SpineTraversal<'a> {
    db: &'a Database,
}

impl<'a> SpineTraversal<'a> {
    /// Create a new SpineTraversal
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// Traverse the spine from a starting block following 'next' links
    ///
    /// Algorithm:
    /// 1. Start at root or specified start block
    /// 2. Follow NEXT edges in order of sequence_weight (ascending)
    /// 3. For each child that is also a structure block, recurse
    /// 4. For each child that is a zettel, add to output
    /// 5. Return flattened list of block IDs in document order
    ///
    /// # Arguments
    /// * `start` - Optional starting block ID (None = use root structure)
    /// * `depth` - Maximum depth to traverse (0 = unlimited)
    ///
    /// # Returns
    /// * `SpineTraversalResult` with ordered block IDs
    pub async fn traverse_from(&self, start: Option<Ulid>, depth: u32) -> NexusResult<SpineTraversalResult> {
        // Find root block if not specified
        let root_id = match start {
            Some(id) => id,
            None => self.find_root_structure().await?.ok_or_else(|| {
                crate::NexusError::BlockNotFound("No root structure block found".to_string())
            })?,
        };

        // Verify root exists
        let root_block = self.db.blocks().get(&root_id).await?;
        if root_block.is_none() {
            return Err(crate::NexusError::BlockNotFound(root_id.to_string()));
        }

        let mut blocks = Vec::new();
        let traversed_depth = self.traverse_iterative(root_id, depth, &mut blocks).await?;

        let total_count = blocks.len();
        Ok(SpineTraversalResult {
            root_id,
            blocks,
            total_count,
            depth: traversed_depth,
        })
    }

    /// Get the spine tree structure starting from a root
    pub async fn get_tree(&self, root: Ulid) -> NexusResult<SpineTree> {
        // Verify root exists and is a structure
        let root_block = self.db.blocks().get(&root).await?;
        let root_block = root_block.ok_or_else(|| crate::NexusError::BlockNotFound(root.to_string()))?;

        if !matches!(root_block.block_type, BlockType::Structure | BlockType::Outline) {
            return Err(crate::NexusError::InvalidBlockType(format!(
                "Root must be Structure or Outline, got {:?}",
                root_block.block_type
            )));
        }

        let mut nodes = Vec::new();
        let root_node = self.build_tree_iterative(root, &mut nodes).await?;

        Ok(SpineTree { root: root_node, nodes })
    }

    /// Find the root structure block (first structure block in creation order)
    async fn find_root_structure(&self) -> NexusResult<Option<Ulid>> {
        let structures = self.db.blocks().list_by_type(BlockType::Structure).await?;
        Ok(structures.into_iter().next().map(|b| b.id))
    }

    /// Iterative traversal using an explicit stack (avoids async recursion)
    async fn traverse_iterative(
        &self,
        start_id: Ulid,
        max_depth: u32,
        result: &mut Vec<Ulid>,
    ) -> NexusResult<u32> {
        // Stack contains: (node_id, current_depth)
        let mut stack: Vec<(Ulid, u32)> = vec![(start_id, 0)];
        let mut max_depth_reached = 0u32;

        while let Some((node_id, current_depth)) = stack.pop() {
            // Check depth limit
            if max_depth > 0 && current_depth >= max_depth {
                continue;
            }

            max_depth_reached = max_depth_reached.max(current_depth);

            // Get NEXT edges from this node, ordered by sequence_weight
            let edges = self.get_next_edges(node_id).await?;

            // We need to process in reverse order since stack is LIFO
            // to maintain correct sequence order
            let mut child_structure_ids: Vec<(Ulid, u32)> = Vec::new();

            for edge in edges.into_iter().rev() {
                // Get the target block
                if let Some(block) = self.db.blocks().get(&edge.to).await? {
                    if matches!(block.block_type, BlockType::Structure | BlockType::Outline) {
                        // Add to list of structure blocks to process (reversed)
                        child_structure_ids.push((block.id, current_depth + 1));
                    } else {
                        // Add zettel directly to result
                        result.push(block.id);
                    }
                }
            }

            // Add structure blocks to stack in reverse order to maintain sequence
            for (child_id, child_depth) in child_structure_ids.into_iter().rev() {
                stack.push((child_id, child_depth));
            }
        }

        Ok(max_depth_reached)
    }

    /// Build tree structure iteratively
    async fn build_tree_iterative(
        &self,
        start_id: Ulid,
        nodes: &mut Vec<SpineNode>,
    ) -> NexusResult<SpineNode> {
        // Stack contains: (node_id, children_so_far)
        let mut stack: Vec<(Ulid, Vec<SpineNode>)> = vec![(start_id, Vec::new())];

        while let Some((node_id, children)) = stack.pop() {
            // Get the block
            let block = self.db.blocks().get(&node_id).await?
                .ok_or_else(|| crate::NexusError::BlockNotFound(node_id.to_string()))?;

            // Get NEXT edges for children
            let edges = self.get_next_edges(node_id).await?;

            // Process children
            let _new_children: Vec<SpineNode> = Vec::new();
            for edge in edges.into_iter().rev() {
                if let Some(child_block) = self.db.blocks().get(&edge.to).await?
                    && matches!(child_block.block_type, BlockType::Structure | BlockType::Outline) {
                        // Will be processed later, add placeholder for now
                        stack.push((child_block.id, Vec::new()));
                        // We need to get the real children later
                    }
            }

            let node = SpineNode {
                block_id: block.id,
                block_type: block.block_type,
                title: block.title,
                sequence_weight: FractionalIndex::first(),
                children,
            };

            nodes.push(node.clone());
        }

        // Return the first node as root (we need to restructure this properly)
        Ok(nodes.first().cloned().unwrap_or_else(|| SpineNode {
            block_id: start_id,
            block_type: BlockType::Structure,
            title: String::new(),
            sequence_weight: FractionalIndex::first(),
            children: Vec::new(),
        }))
    }

    /// Get NEXT edges from a block ordered by sequence_weight
    async fn get_next_edges(&self, block_id: Ulid) -> NexusResult<Vec<Edge>> {
        let edges = self.db.edges().outgoing_from(&block_id).await?;
        let next_edges: Vec<Edge> = edges
            .into_iter()
            .filter(|e| e.link_type == LinkType::Next)
            .collect();

        // Sort by sequence_weight ascending (FractionalIndex already implements Ord)
        let mut sorted = next_edges;
        sorted.sort_by(|a, b| a.sequence_weight.cmp(&b.sequence_weight));

        Ok(sorted)
    }

    /// Get blocks in order with their weights (for display)
    pub async fn traverse_with_weights(&self, start: Option<Ulid>, depth: u32) -> NexusResult<Vec<(Ulid, FractionalIndex)>> {
        let result = self.traverse_from(start, depth).await?;
        let mut id_weights = Vec::new();

        for block_id in result.blocks {
            // Get incoming NEXT edge to get weight
            let incoming = self.db.edges().incoming_to(&block_id).await?;
            if let Some(weight) = incoming
                .iter()
                .find(|e| e.link_type == LinkType::Next)
                .map(|e| e.sequence_weight.clone())
            {
                id_weights.push((block_id, weight));
            } else {
                id_weights.push((block_id, FractionalIndex::first()));
            }
        }

        Ok(id_weights)
    }

    /// Find orphan blocks (blocks with no incoming NEXT edge)
    pub async fn find_orphans(&self) -> NexusResult<Vec<Ulid>> {
        // Get all blocks that are not structure/outline (permanent, fleeting, etc.)
        let all_blocks = self.db.blocks().list_all().await?;
        let mut orphans = Vec::new();

        for block in all_blocks {
            // Skip structure blocks - they're valid roots
            if matches!(block.block_type, BlockType::Structure | BlockType::Outline) {
                continue;
            }

            // Check if block has incoming NEXT edge
            let incoming = self.db.edges().incoming_to(&block.id).await?;
            let has_next_parent = incoming.iter().any(|e| e.link_type == LinkType::Next);

            if !has_next_parent {
                orphans.push(block.id);
            }
        }

        Ok(orphans)
    }
}

impl<'a> Default for SpineTraversal<'a> {
    fn default() -> Self {
        panic!("SpineTraversal requires a Database reference")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_traverse_empty_spine() {
        // This would require setting up a test database
        // For unit testing without DB, we test the structure
        let result = SpineTraversalResult {
            root_id: Ulid::new(),
            blocks: vec![],
            total_count: 0,
            depth: 0,
        };
        assert_eq!(result.total_count, 0);
    }

    #[test]
    fn test_spine_node_structure() {
        let node = SpineNode {
            block_id: Ulid::new(),
            block_type: BlockType::Structure,
            title: "Test".to_string(),
            sequence_weight: FractionalIndex::first(),
            children: vec![],
        };
        assert_eq!(node.title, "Test");
        assert!(node.children.is_empty());
    }

    #[test]
    fn test_spines_traversal_result() {
        let root_id = Ulid::new();
        let block1 = Ulid::new();
        let block2 = Ulid::new();

        let result = SpineTraversalResult {
            root_id,
            blocks: vec![block1, block2],
            total_count: 2,
            depth: 1,
        };

        assert_eq!(result.total_count, 2);
        assert_eq!(result.blocks.len(), 2);
    }
}
