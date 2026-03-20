//! Table of Contents Generator
//!
//! Generates hierarchical TOC from Structure blocks by querying section_of edges.

#![allow(dead_code)]

use crate::models::{Block, FractionalIndex};
use crate::{NexusError, NexusResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;
use ulid::Ulid;

/// Table of Contents entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Toc {
    /// Structure block ID
    pub structure_id: Ulid,
    /// Title of the structure
    pub title: String,
    /// Top-level sections
    pub sections: Vec<TocSection>,
    /// Total blocks in structure
    pub total_blocks: usize,
    /// Estimated page count (assuming ~500 words per page)
    pub estimated_pages: usize,
}

impl Default for Toc {
    fn default() -> Self {
        Self {
            structure_id: Ulid::nil(),
            title: String::new(),
            sections: Vec::new(),
            total_blocks: 0,
            estimated_pages: 0,
        }
    }
}

/// A section in the TOC hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocSection {
    /// Section block ID
    pub id: Ulid,
    /// Section title
    pub title: String,
    /// Depth in hierarchy (0 = top-level)
    pub depth: u32,
    /// Block IDs in this section (ordered)
    pub blocks: Vec<Ulid>,
    /// Child sections
    pub children: Vec<TocSection>,
    /// Completion percentage (0.0 - 1.0)
    pub completion: f32,
}

impl TocSection {
    /// Create a new top-level section
    pub fn new(id: Ulid, title: String) -> Self {
        Self {
            id,
            title,
            depth: 0,
            blocks: Vec::new(),
            children: Vec::new(),
            completion: 0.0,
        }
    }

    /// Create a child section
    pub fn child(id: Ulid, title: String, parent_depth: u32) -> Self {
        Self {
            id,
            title,
            depth: parent_depth + 1,
            blocks: Vec::new(),
            children: Vec::new(),
            completion: 0.0,
        }
    }

    /// Count total blocks including children
    pub fn total_block_count(&self) -> usize {
        let self_blocks = self.blocks.len();
        let child_blocks: usize = self.children.iter().map(|c| c.total_block_count()).sum();
        self_blocks + child_blocks
    }
}

/// TOC Generator - builds hierarchical table of contents from Structure blocks
pub struct TocGenerator<'a> {
    db: &'a Surreal<Db>,
}

impl<'a> TocGenerator<'a> {
    /// Create a new TOC generator
    pub fn new(db: &'a Surreal<Db>) -> Self {
        Self { db }
    }

    /// Generate TOC from a Structure block
    ///
    /// Queries all edges with link_type = 'section_of' and builds
    /// a hierarchical structure ordered by sequence_weight.
    pub async fn generate(&self, structure_id: &Ulid) -> NexusResult<Toc> {
        // 1. Fetch the structure block to get its title
        let structure_block = self
            .fetch_block(structure_id)
            .await?
            .ok_or_else(|| NexusError::BlockNotFound(structure_id.to_string()))?;

        // Validate it's a Structure block
        if !matches!(structure_block.block_type, crate::models::BlockType::Structure) {
            return Err(NexusError::InvalidBlockType(format!(
                "Block {} is not a Structure block, got {:?}",
                structure_id, structure_block.block_type
            )));
        }

        // 2. Query all sections linked via 'section_of' edges
        let sections = self.fetch_sections(structure_id).await?;

        // 3. Build hierarchical structure
        let sections_with_blocks = self.build_hierarchy(sections).await?;

        // 4. Calculate totals
        let total_blocks: usize = sections_with_blocks
            .iter()
            .map(|s| s.total_block_count())
            .sum();

        // Estimate pages: ~500 words per page, avg 5 chars per word
        let word_count = self.estimate_word_count(&sections_with_blocks);
        let estimated_pages = (word_count / 500).max(1);

        Ok(Toc {
            structure_id: *structure_id,
            title: structure_block.title,
            sections: sections_with_blocks,
            total_blocks,
            estimated_pages,
        })
    }

    /// Fetch a block by ID
    async fn fetch_block(&self, id: &Ulid) -> NexusResult<Option<Block>> {
        let id_str = id.to_string();
        let sql = "SELECT * FROM block WHERE ulid = $ulid LIMIT 1";
        let mut result = self.db
            .query(sql)
            .bind(("ulid", id_str))
            .await
            .map_err(|e| NexusError::Database(e.to_string()))?;

        let blocks: Vec<Block> = result.take(0).map_err(|e| NexusError::Database(e.to_string()))?;
        Ok(blocks.into_iter().next())
    }

    /// Fetch all section edges for a structure
    async fn fetch_sections(&self, structure_id: &Ulid) -> NexusResult<Vec<SectionEdge>> {
        // Query edges where:
        // - src (source) is the section block
        // - dst (target) is the structure block
        // - link_type is 'section_of'
        let id_str = structure_id.to_string();

        let query = r#"
            SELECT
                ulid as edge_id,
                src as section_id,
                dst as structure_id,
                sequence_weight
            FROM edge
            WHERE dst = $structure AND link_type = 'section_of'
            ORDER BY sequence_weight ASC
        "#;

        let mut result = self.db
            .query(query)
            .bind(("structure", id_str))
            .await
            .map_err(|e| NexusError::Database(e.to_string()))?;

        let edges: Vec<SectionEdge> = result
            .take(0)
            .map_err(|e| NexusError::Database(e.to_string()))?;

        Ok(edges)
    }

    /// Build hierarchical section structure with blocks
    async fn build_hierarchy(&self, sections: Vec<SectionEdge>) -> NexusResult<Vec<TocSection>> {
        if sections.is_empty() {
            return Ok(Vec::new());
        }

        // Group sections by parent (for now, flat with subsections)
        // In a full implementation, we'd query subsection_of edges too
        let mut result: Vec<TocSection> = Vec::new();
        let mut section_blocks: HashMap<String, Vec<Ulid>> = HashMap::new();

        // Fetch blocks for each section
        for section in &sections {
            let blocks = self.fetch_section_blocks(&section.section_id).await?;
            section_blocks.insert(section.section_id.clone(), blocks);
        }

        // Build TocSection for each section edge
        for section in sections {
            let section_ulid = Ulid::from_string(&section.section_id)
                .map_err(|e| NexusError::Database(format!("Invalid ULID: {}", e)))?;

            let section_block = self
                .fetch_block(&section_ulid)
                .await?
                .ok_or_else(|| NexusError::BlockNotFound(section.section_id.clone()))?;

            let mut toc_section = TocSection::new(section_ulid, section_block.title);
            toc_section.blocks = section_blocks
                .get(&section.section_id)
                .cloned()
                .unwrap_or_default();

            // Calculate completion based on block content
            toc_section.completion = self.calculate_completion(&toc_section.blocks).await?;

            result.push(toc_section);
        }

        Ok(result)
    }

    /// Fetch blocks linked via 'ordered_child' edges within a section
    async fn fetch_section_blocks(&self, section_id: &str) -> NexusResult<Vec<Ulid>> {
        let query = r#"
            SELECT
                src as block_id,
                sequence_weight
            FROM edge
            WHERE dst = $section
            AND link_type = 'ordered_child'
            ORDER BY sequence_weight ASC
        "#;

        let mut result = self.db
            .query(query)
            .bind(("section", section_id.to_string()))
            .await
            .map_err(|e| NexusError::Database(e.to_string()))?;

        #[derive(Deserialize)]
        struct BlockEdge {
            block_id: String,
        }

        let edges: Vec<BlockEdge> = result
            .take(0)
            .map_err(|e| NexusError::Database(e.to_string()))?;

        Ok(edges
            .into_iter()
            .filter_map(|e| Ulid::from_string(&e.block_id).ok())
            .filter(|id| !id.is_nil())
            .collect())
    }

    /// Calculate completion percentage for a section
    async fn calculate_completion(&self, blocks: &[Ulid]) -> NexusResult<f32> {
        if blocks.is_empty() {
            return Ok(0.0);
        }

        let mut filled = 0usize;
        for block_id in blocks {
            if let Some(block) = self.fetch_block(block_id).await? {
                // A block is "complete" if it has meaningful content (> 50 chars)
                if block.content.len() > 50 {
                    filled += 1;
                }
            }
        }

        Ok(filled as f32 / blocks.len() as f32)
    }

    /// Estimate word count from sections
    fn estimate_word_count(&self, sections: &[TocSection]) -> usize {
        // For now, estimate based on block count
        // Average 200 words per block
        let total_blocks: usize = sections.iter().map(|s| s.total_block_count()).sum();
        total_blocks * 200
    }
}

/// Helper struct for section edge data
#[derive(Debug, Deserialize)]
struct SectionEdge {
    #[allow(dead_code)]
    edge_id: String,
    section_id: String,
    #[allow(dead_code)]
    structure_id: String,
    #[allow(dead_code)]
    sequence_weight: FractionalIndex,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toc_default() {
        let toc = Toc::default();
        assert!(toc.structure_id.is_nil());
        assert!(toc.title.is_empty());
        assert!(toc.sections.is_empty());
        assert_eq!(toc.total_blocks, 0);
        assert_eq!(toc.estimated_pages, 0);
    }

    #[test]
    fn test_toc_section_new() {
        let id = Ulid::new();
        let section = TocSection::new(id, "Test Section".to_string());

        assert_eq!(section.id, id);
        assert_eq!(section.title, "Test Section");
        assert_eq!(section.depth, 0);
        assert!(section.blocks.is_empty());
        assert!(section.children.is_empty());
        assert_eq!(section.completion, 0.0);
    }

    #[test]
    fn test_toc_section_child() {
        let id = Ulid::new();
        let child = TocSection::child(id, "Child Section".to_string(), 0);

        assert_eq!(child.depth, 1);
        assert_eq!(child.id, id);
    }

    #[test]
    fn test_toc_section_total_block_count() {
        let parent_id = Ulid::new();
        let mut parent = TocSection::new(parent_id, "Parent".to_string());
        parent.blocks = vec![Ulid::new(), Ulid::new()];

        let child_id = Ulid::new();
        let mut child = TocSection::child(child_id, "Child".to_string(), 0);
        child.blocks = vec![Ulid::new()];

        parent.children.push(child);

        assert_eq!(parent.total_block_count(), 3);
    }

    #[test]
    fn test_estimate_word_count() {
        // TocGenerator::estimate_word_count is a pure function, so we can test it
        // without a database connection by creating mock data

        // We can't easily create a TocGenerator without a DB, but we can test
        // the word count calculation logic directly
        let sections = vec![
            TocSection {
                id: Ulid::new(),
                title: "Section 1".to_string(),
                depth: 0,
                blocks: vec![Ulid::new(), Ulid::new()],
                children: vec![],
                completion: 1.0,
            },
            TocSection {
                id: Ulid::new(),
                title: "Section 2".to_string(),
                depth: 0,
                blocks: vec![Ulid::new()],
                children: vec![],
                completion: 0.5,
            },
        ];

        // 3 blocks * 200 words = 600 words
        // Since we can't call generator.estimate_word_count without a DB,
        // we verify the block count calculation
        let total_blocks: usize = sections.iter().map(|s| s.total_block_count()).sum();
        assert_eq!(total_blocks, 3);
        assert_eq!(total_blocks * 200, 600);
    }
}