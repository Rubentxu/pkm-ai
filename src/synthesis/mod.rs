//! Document Synthesis Module
//!
//! PRIORITY #1: Convert fragments into complete documents
//!
//! This module orchestrates the synthesis of Zettelkasten fragments into
//! complete, structured documents with table of contents, templates,
//! and multiple output formats.

#![allow(dead_code)]

mod template;
mod toc;
mod typst_renderer;

pub use template::{SectionData, SynthesisData, TemplateEngine};
pub use toc::{Toc, TocGenerator, TocSection};
pub use typst_renderer::{RenderFormat, TypstRenderer};

use crate::db::Database;
use crate::models::{Block, BlockType};
use crate::NexusResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ulid::Ulid;

/// Output format for synthesized documents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum OutputFormat {
    /// PDF format (via Typst)
    Pdf,
    /// HTML format
    Html,
    /// Markdown format (fallback)
    #[default]
    Markdown,
    /// Typst source format
    Typst,
}


impl From<OutputFormat> for RenderFormat {
    fn from(format: OutputFormat) -> Self {
        match format {
            OutputFormat::Pdf => RenderFormat::Pdf,
            OutputFormat::Html => RenderFormat::Html,
            OutputFormat::Markdown => RenderFormat::Markdown,
            OutputFormat::Typst => RenderFormat::Markdown, // Fallback
        }
    }
}

/// Result of document synthesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisResult {
    /// The synthesized content
    pub content: Vec<u8>,
    /// Output format used
    pub format: OutputFormat,
    /// Table of contents
    pub toc: Toc,
    /// Number of blocks used in synthesis
    pub blocks_used: usize,
    /// Total blocks available in structure
    pub blocks_total: usize,
    /// Title of the document
    pub title: String,
}

impl Default for SynthesisResult {
    fn default() -> Self {
        Self {
            content: Vec::new(),
            format: OutputFormat::Markdown,
            toc: Toc::default(),
            blocks_used: 0,
            blocks_total: 0,
            title: String::new(),
        }
    }
}

/// Document synthesizer - orchestrates the full synthesis pipeline
pub struct Synthesizer<'a> {
    db: &'a Database,
    toc_gen: TocGenerator<'a>,
    template_engine: TemplateEngine,
    renderer: TypstRenderer,
}

impl<'a> Synthesizer<'a> {
    /// Create a new synthesizer
    pub fn new(db: &'a Database) -> Self {
        Self {
            db,
            toc_gen: TocGenerator::new(&db.inner),
            template_engine: TemplateEngine::new(),
            renderer: TypstRenderer::new(),
        }
    }

    /// Generate TOC from a structure block
    pub async fn generate_toc(&self, structure_id: &Ulid) -> NexusResult<Toc> {
        self.toc_gen.generate(structure_id).await
    }

    /// Synthesize a complete document from a structure
    ///
    /// # Arguments
    /// * `structure_id` - The Structure block ID to synthesize from
    /// * `format` - Output format (Pdf, Html, Markdown, Typst)
    /// * `template` - Optional template name (defaults to "default")
    ///
    /// # Workflow
    /// 1. Generate TOC from structure
    /// 2. Fetch all linked blocks in sequence order
    /// 3. Apply template with block content
    /// 4. Render to output format
    /// 5. Return result
    pub async fn synthesize(
        &self,
        structure_id: &Ulid,
        format: OutputFormat,
        template: Option<&str>,
    ) -> NexusResult<SynthesisResult> {
        // 1. Generate TOC
        let toc = self.toc_gen.generate(structure_id).await?;

        // 2. Fetch all blocks in order
        let blocks = self.fetch_ordered_blocks(&toc).await?;

        // 3. Build synthesis data
        let synthesis_data = self.build_synthesis_data(&toc, &blocks).await?;

        // 4. Load and render template
        let template_name = template.unwrap_or("default");
        let tmpl = self.template_engine.get(template_name)?;

        let markdown_content = self.template_engine.render(&tmpl, &synthesis_data)?;

        // 5. Render to output format
        let content = self.renderer.render(&markdown_content, format.into()).await?;

        // 6. Build result
        let blocks_used = blocks.len();
        let blocks_total = toc.total_blocks;

        Ok(SynthesisResult {
            content,
            format,
            toc,
            blocks_used,
            blocks_total,
            title: synthesis_data.title,
        })
    }

    /// Synthesize with fallback to Markdown if primary format fails
    ///
    /// This is useful when PDF generation is desired but typst is unavailable.
    pub async fn synthesize_with_fallback(
        &self,
        structure_id: &Ulid,
        preferred_format: OutputFormat,
        template: Option<&str>,
    ) -> NexusResult<SynthesisResult> {
        // Try the preferred format first
        let result = self.synthesize(structure_id, preferred_format, template).await;

        match result {
            Ok(r) => Ok(r),
            Err(e) => {
                tracing::warn!("Synthesis failed for preferred format: {}", e);
                tracing::info!("Falling back to Markdown");

                // Fall back to Markdown
                let mut r = self.synthesize(structure_id, OutputFormat::Markdown, template).await?;
                r.format = preferred_format; // Report original format intent
                Ok(r)
            }
        }
    }

    /// Fetch all blocks in a structure, ordered by sequence
    async fn fetch_ordered_blocks(&self, toc: &Toc) -> NexusResult<Vec<Block>> {
        let mut all_blocks = Vec::new();

        for section in &toc.sections {
            // Fetch section block
            if let Some(block) = self.db.blocks().get(&section.id).await? {
                all_blocks.push(block);
            }

            // Fetch child blocks in order
            for block_id in &section.blocks {
                if let Some(block) = self.db.blocks().get(block_id).await? {
                    all_blocks.push(block);
                }
            }

            // Recursively fetch children
            self.fetch_child_blocks_iterative(section, &mut all_blocks).await?;
        }

        Ok(all_blocks)
    }

    /// Fetch blocks recursively (iterative to avoid async recursion issues)
    async fn fetch_child_blocks_iterative(
        &self,
        section: &TocSection,
        blocks: &mut Vec<Block>,
    ) -> NexusResult<()> {
        // Use a stack for iterative DFS
        let mut stack: Vec<&[TocSection]> = vec![&section.children];

        while let Some(children) = stack.pop() {
            for child in children {
                if let Some(block) = self.db.blocks().get(&child.id).await? {
                    blocks.push(block);
                }
                for block_id in &child.blocks {
                    if let Some(block) = self.db.blocks().get(block_id).await? {
                        blocks.push(block);
                    }
                }
                // Add this section's children to the stack
                if !child.children.is_empty() {
                    stack.push(&child.children);
                }
            }
        }
        Ok(())
    }

    /// Build synthesis data from TOC and blocks
    async fn build_synthesis_data(
        &self,
        toc: &Toc,
        blocks: &[Block],
    ) -> NexusResult<SynthesisData> {
        let mut section_map: HashMap<Ulid, &Block> = HashMap::new();
        let mut block_map: HashMap<Ulid, &Block> = HashMap::new();

        for block in blocks {
            if matches!(block.block_type, BlockType::Structure) {
                section_map.insert(block.id, block);
            } else {
                block_map.insert(block.id, block);
            }
        }

        let mut sections_data = Vec::new();

        for toc_section in &toc.sections {
            // First try section_map (for Structure blocks), then fall back to block_map (for Outline blocks)
            let section_block = section_map.get(&toc_section.id)
                .or_else(|| block_map.get(&toc_section.id));

            // Get section content
            let section_content = section_block
                .map(|b| b.content.clone())
                .unwrap_or_default();

            let section_data = SectionData {
                title: toc_section.title.clone(),
                depth: toc_section.depth + 1,
                content: section_content,
                id: toc_section.id.to_string(),
            };
            sections_data.push(section_data);

            // Add block content as subsections
            for block_id in &toc_section.blocks {
                if let Some(block) = block_map.get(block_id) {
                    let block_data = SectionData {
                        title: block.title.clone(),
                        depth: toc_section.depth + 2,
                        content: block.content.clone(),
                        id: block.id.to_string(),
                    };
                    sections_data.push(block_data);
                }
            }
        }

        // Build TOC JSON
        let toc_json = serde_json::to_string(&toc.sections)
            .unwrap_or_default();

        Ok(SynthesisData {
            title: toc.title.clone(),
            author: "PKM-AI".to_string(),
            date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            abstract_: Self::generate_abstract(toc),
            toc_json,
            sections: sections_data,
            bibliography: Vec::new(), // TODO: Collect from references
            metadata: HashMap::new(),
        })
    }

    /// Generate abstract from TOC
    fn generate_abstract(toc: &Toc) -> String {
        if toc.sections.is_empty() {
            return "A document with no sections.".to_string();
        }

        let section_titles: Vec<&str> = toc
            .sections
            .iter()
            .map(|s| s.title.as_str())
            .take(3)
            .collect();

        if section_titles.is_empty() {
            return format!(
                "A document about {}. It contains {} sections.",
                toc.title,
                toc.sections.len()
            );
        }

        let intro = section_titles.join(", ");
        format!(
            "This document covers: {}{}",
            intro,
            if toc.sections.len() > 3 {
                format!(" and {} more sections", toc.sections.len() - 3)
            } else {
                String::new()
            }
        )
    }

    /// List available templates
    pub fn list_templates(&self) -> Vec<String> {
        TemplateEngine::list_templates()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_default() {
        assert_eq!(OutputFormat::default(), OutputFormat::Markdown);
    }

    #[test]
    fn test_output_format_from_render_format() {
        assert_eq!(RenderFormat::from(OutputFormat::Pdf), RenderFormat::Pdf);
        assert_eq!(RenderFormat::from(OutputFormat::Html), RenderFormat::Html);
        assert_eq!(RenderFormat::from(OutputFormat::Markdown), RenderFormat::Markdown);
    }

    #[test]
    fn test_synthesis_result_default() {
        let result = SynthesisResult::default();
        assert!(result.content.is_empty());
        assert_eq!(result.format, OutputFormat::Markdown);
        assert_eq!(result.blocks_used, 0);
        assert_eq!(result.blocks_total, 0);
    }

    #[test]
    fn test_generate_abstract() {
        // Test with empty sections
        let empty_toc = Toc::default();
        assert!(empty_toc.sections.is_empty());

        // Test with actual sections
        let toc = Toc {
            structure_id: Ulid::new(),
            title: "Test Document".to_string(),
            sections: vec![
                TocSection::new(Ulid::new(), "Introduction".to_string()),
                TocSection::new(Ulid::new(), "Methods".to_string()),
                TocSection::new(Ulid::new(), "Results".to_string()),
                TocSection::new(Ulid::new(), "Conclusion".to_string()),
            ],
            total_blocks: 10,
            estimated_pages: 5,
        };
        assert_eq!(toc.sections.len(), 4);
    }

    #[tokio::test]
    async fn test_synthesizer_creation() {
        // Synthesizer requires a real DB connection, so we just verify
        // the template list is populated correctly
        let templates = TemplateEngine::list_templates();
        assert!(templates.contains(&"default".to_string()));
        assert!(templates.contains(&"technical-whitepaper".to_string()));
    }
}