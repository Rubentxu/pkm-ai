//! Template Engine for Document Synthesis
//!
//! Loads and renders templates for document generation.
//! Supports Markdown, HTML, and Typst template formats.

#![allow(dead_code)]

use crate::{NexusError, NexusResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::Path;

/// Template format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum TemplateFormat {
    /// Markdown format
    #[default]
    Markdown,
    /// HTML format
    Html,
    /// Typst format (for PDF generation)
    Typst,
}


/// A document template
#[derive(Debug, Clone)]
pub struct Template {
    /// Template name
    pub name: String,
    /// Template format
    pub format: TemplateFormat,
    /// Template content with placeholders
    pub content: String,
    /// Template description
    pub description: String,
}

/// Data for template rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisData {
    /// Document title
    pub title: String,
    /// Author name
    pub author: String,
    /// Document date
    pub date: String,
    /// Abstract/summary
    pub abstract_: String,
    /// Table of contents as JSON string
    pub toc_json: String,
    /// Sections content
    pub sections: Vec<SectionData>,
    /// Bibliography entries
    pub bibliography: Vec<BibliographyEntry>,
    /// Custom metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// A section in the document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionData {
    /// Section title
    pub title: String,
    /// Section depth (1 = h1, 2 = h2, etc.)
    pub depth: u32,
    /// Section content (Markdown/HTML)
    pub content: String,
    /// Section ID for linking
    pub id: String,
}

/// Bibliography entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BibliographyEntry {
    /// Citation key
    pub key: String,
    /// Entry title
    pub title: String,
    /// Authors
    pub authors: Vec<String>,
    /// Year
    pub year: u16,
    /// Source/journal
    pub source: Option<String>,
    /// URL if available
    pub url: Option<String>,
}

/// Template Engine for rendering documents
pub struct TemplateEngine {
    /// Loaded templates by name
    templates: HashMap<String, Template>,
    /// Default template directory
    template_dir: Option<String>,
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateEngine {
    /// Create a new template engine
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            template_dir: None,
        }
    }

    /// Create with a template directory
    pub fn with_template_dir(template_dir: impl Into<String>) -> Self {
        let template_dir = template_dir.into();
        let mut engine = Self::new();
        engine.template_dir = Some(template_dir);
        engine
    }

    /// Load a template by name
    ///
    /// First searches in the template directory, then falls back
    /// to built-in templates. Returns an owned Template.
    pub fn load(&mut self, name: &str) -> NexusResult<Template> {
        // Check if already loaded
        if let Some(template) = self.templates.get(name) {
            return Ok(template.clone());
        }

        // Try to load from filesystem
        if let Some(ref dir) = self.template_dir {
            let path = Path::new(dir).join(format!("{}.tmpl", name));
            if path.exists() {
                let content = fs::read_to_string(&path)
                    .map_err(NexusError::Io)?;
                let format = Self::detect_format(&content);
                let description = Self::extract_description(&content)
                    .unwrap_or_else(|| name.to_string());

                let template = Template {
                    name: name.to_string(),
                    format,
                    content,
                    description,
                };
                self.templates.insert(name.to_string(), template.clone());
                return Ok(template);
            }
        }

        // Fall back to built-in templates
        let template = Self::get_builtin_template(name)?;
        self.templates.insert(name.to_string(), template.clone());
        Ok(template)
    }

    /// Get a template without caching
    pub fn get(&self, name: &str) -> NexusResult<Template> {
        if let Some(template) = self.templates.get(name) {
            return Ok(template.clone());
        }
        Self::get_builtin_template(name)
    }

    /// Render content with template and data
    pub fn render(&self, template: &Template, data: &SynthesisData) -> NexusResult<String> {
        match template.format {
            TemplateFormat::Markdown => self.render_markdown(template, data),
            TemplateFormat::Html => self.render_html(template, data),
            TemplateFormat::Typst => self.render_typst(template, data),
        }
    }

    /// Render to Markdown
    fn render_markdown(&self, template: &Template, data: &SynthesisData) -> NexusResult<String> {
        let mut output = template.content.clone();

        // Replace placeholders
        output = self.replace_placeholders(&output, data);

        Ok(output)
    }

    /// Render to HTML
    fn render_html(&self, template: &Template, data: &SynthesisData) -> NexusResult<String> {
        let mut output = template.content.clone();
        output = self.replace_placeholders(&output, data);

        // Convert Markdown to HTML if needed
        output = self.markdown_to_html(&output);

        Ok(output)
    }

    /// Render to Typst
    fn render_typst(&self, template: &Template, data: &SynthesisData) -> NexusResult<String> {
        let mut output = template.content.clone();
        output = self.replace_placeholders(&output, data);

        // Convert Markdown sections to Typst format
        output = self.markdown_to_typst(&output, data);

        Ok(output)
    }

    /// Replace placeholders in template
    fn replace_placeholders(&self, content: &str, data: &SynthesisData) -> String {
        let mut result = content.to_string();

        // Simple placeholders
        result = result.replace("{{title}}", &data.title);
        result = result.replace("{{author}}", &data.author);
        result = result.replace("{{date}}", &data.date);
        result = result.replace("{{abstract}}", &data.abstract_);
        result = result.replace("{{toc}}", &data.toc_json);

        // Sections
        let sections_html = self.render_sections_html(&data.sections);
        result = result.replace("{{sections}}", &sections_html);

        // Bibliography
        let bibliography_html = self.render_bibliography_html(&data.bibliography);
        result = result.replace("{{bibliography}}", &bibliography_html);

        // Custom metadata
        for (key, value) in &data.metadata {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        result
    }

    /// Render sections as HTML
    fn render_sections_html(&self, sections: &[SectionData]) -> String {
        let mut html = String::new();
        for section in sections {
            let heading = match section.depth {
                1 => format!("<h1 id=\"{}\">{}</h1>", section.id, section.title),
                2 => format!("<h2 id=\"{}\">{}</h2>", section.id, section.title),
                3 => format!("<h3 id=\"{}\">{}</h3>", section.id, section.title),
                _ => format!("<h{} id=\"{}\">{}</h{}>", section.depth, section.id, section.title, section.depth),
            };
            html.push_str(&heading);
            html.push_str("<div class=\"section-content\">\n");
            html.push_str(&section.content);
            html.push_str("\n</div>\n");
        }
        html
    }

    /// Render bibliography as HTML
    fn render_bibliography_html(&self, bibliography: &[BibliographyEntry]) -> String {
        if bibliography.is_empty() {
            return String::new();
        }

        let mut html = String::from("<section id=\"bibliography\" class=\"bibliography\">\n");
        html.push_str("<h2>References</h2>\n<ol>\n");

        for entry in bibliography {
            let authors = entry.authors.join(", ");
            let citation = format!(
                "{} ({}). {}. {}",
                authors,
                entry.year,
                entry.title,
                entry.source.as_deref().unwrap_or("")
            );
            html.push_str(&format!(
                "<li id=\"{}\">{}</li>\n",
                entry.key, citation
            ));
        }

        html.push_str("</ol>\n</section>\n");
        html
    }

    /// Convert Markdown to basic HTML
    fn markdown_to_html(&self, content: &str) -> String {
        let mut html = content.to_string();

        // Headers
        html = html.replace("# ", "<h1>");
        html = html.replace("## ", "<h2>");
        html = html.replace("### ", "<h3>");
        // Note: This is a simplified conversion. A real implementation
        // would use a proper Markdown parser.

        html
    }

    /// Convert Markdown sections to Typst format
    fn markdown_to_typst(&self, content: &str, data: &SynthesisData) -> String {
        let mut typst = String::new();

        // Write title
        writeln!(typst, "#let document_title = \"{}\"", data.title).unwrap();
        writeln!(typst, "#let document_author = \"{}\"", data.author).unwrap();
        writeln!(typst, "#let document_date = \"{}\"", data.date).unwrap();

        // Parse sections from content
        let mut in_section = false;

        for line in content.lines() {
            if line.starts_with("# ") {
                if in_section {
                    writeln!(typst, ")").unwrap();
                }
                let current_title = line.trim_start_matches("# ").to_string();
                writeln!(
                    typst,
                    "#heading(level: 1)[{}]",
                    Self::escape_typst_string(&current_title)
                )
                .unwrap();
                writeln!(typst, "#set text(size: 11pt)").unwrap();
                writeln!(typst, "#parbreak()").unwrap();
                in_section = true;
            } else if in_section {
                let escaped = Self::escape_typst_string(line);
                writeln!(typst, "{} #parbreak()", escaped).unwrap();
            }
        }

        if in_section {
            writeln!(typst, ")").unwrap();
        }

        typst
    }

    /// Escape string for Typst
    fn escape_typst_string(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('#', "\\#")
            .replace('*', "\\*")
            .replace('_', "\\_")
    }

    /// Detect template format from content
    fn detect_format(content: &str) -> TemplateFormat {
        let lower = content.to_lowercase();
        if lower.contains("<!doctype html>") || lower.contains("<html") {
            TemplateFormat::Html
        } else if lower.contains("#let ") || lower.contains("#set ") {
            TemplateFormat::Typst
        } else {
            TemplateFormat::Markdown
        }
    }

    /// Extract description from template
    fn extract_description(content: &str) -> Option<String> {
        content
            .lines()
            .find(|l| l.starts_with("description:"))
            .map(|l| l.trim_start_matches("description:").trim().to_string())
    }

    /// Get built-in template
    fn get_builtin_template(name: &str) -> NexusResult<Template> {
        match name {
            "technical-whitepaper" => Ok(Self::technical_whitepaper_template()),
            "academic-paper" => Ok(Self::academic_paper_template()),
            "tutorial" => Ok(Self::tutorial_template()),
            "reference-manual" => Ok(Self::reference_manual_template()),
            "default" | "basic" => Ok(Self::default_template()),
            _ => Err(NexusError::Synthesis(format!(
                "Unknown template: '{}'",
                name
            ))),
        }
    }

    /// Built-in: Technical Whitepaper template
    fn technical_whitepaper_template() -> Template {
        Template {
            name: "technical-whitepaper".to_string(),
            format: TemplateFormat::Markdown,
            description: "Technical whitepaper with abstract, sections, and references".to_string(),
            content: r#"# Technical Whitepaper

# {{title}}

**Author:** {{author}}
**Date:** {{date}}

## Abstract

{{abstract}}

## Table of Contents

{{toc}}

{{sections}}

{{bibliography}}
"#
            .to_string(),
        }
    }

    /// Built-in: Academic Paper template
    fn academic_paper_template() -> Template {
        Template {
            name: "academic-paper".to_string(),
            format: TemplateFormat::Markdown,
            description: "Academic paper with structured abstract and citations".to_string(),
            content: r#"# {{title}}

**{{author}}**
**{{date}}**

---

## Abstract

{{abstract}}

**Keywords:** {{keywords}}

---

{{sections}}

## References

{{bibliography}}
"#
            .to_string(),
        }
    }

    /// Built-in: Tutorial template
    fn tutorial_template() -> Template {
        Template {
            name: "tutorial".to_string(),
            format: TemplateFormat::Markdown,
            description: "Step-by-step tutorial with clear headings".to_string(),
            content: r#"# {{title}}

*Tutorial*
**Author:** {{author}}
**Prerequisites:** {{prerequisites}}

---

{{toc}}

{{sections}}

## Summary

{{summary}}
"#
            .to_string(),
        }
    }

    /// Built-in: Reference Manual template
    fn reference_manual_template() -> Template {
        Template {
            name: "reference-manual".to_string(),
            format: TemplateFormat::Markdown,
            description: "Reference manual with detailed sections".to_string(),
            content: r#"# {{title}}

**Version:** {{version}}
**Last Updated:** {{date}}

---

## Overview

{{abstract}}

{{sections}}

## Appendix

{{appendix}}
"#
            .to_string(),
        }
    }

    /// Built-in: Default/Basic template
    fn default_template() -> Template {
        Template {
            name: "default".to_string(),
            format: TemplateFormat::Markdown,
            description: "Simple document template".to_string(),
            content: r#"# {{title}}

{{author}} - {{date}}

---

{{sections}}
"#
            .to_string(),
        }
    }

    /// List available built-in templates
    pub fn list_templates() -> Vec<String> {
        vec![
            "technical-whitepaper".to_string(),
            "academic-paper".to_string(),
            "tutorial".to_string(),
            "reference-manual".to_string(),
            "default".to_string(),
        ]
    }

    /// Check if template exists
    pub fn exists(&self, name: &str) -> bool {
        if self.templates.contains_key(name) {
            true
        } else {
            Self::get_builtin_template(name).is_ok()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_data() -> SynthesisData {
        SynthesisData {
            title: "Test Document".to_string(),
            author: "Test Author".to_string(),
            date: "2026-03-19".to_string(),
            abstract_: "This is a test abstract.".to_string(),
            toc_json: "1. Section 1\n2. Section 2".to_string(),
            sections: vec![
                SectionData {
                    title: "Introduction".to_string(),
                    depth: 1,
                    content: "This is the introduction.".to_string(),
                    id: "intro".to_string(),
                },
                SectionData {
                    title: "Methods".to_string(),
                    depth: 1,
                    content: "These are the methods.".to_string(),
                    id: "methods".to_string(),
                },
            ],
            bibliography: vec![
                BibliographyEntry {
                    key: "smith2020".to_string(),
                    title: "Test Paper".to_string(),
                    authors: vec!["Smith, J.".to_string()],
                    year: 2020,
                    source: Some("Test Journal".to_string()),
                    url: None,
                },
            ],
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_template_engine_new() {
        let engine = TemplateEngine::new();
        assert!(engine.templates.is_empty());
    }

    #[test]
    fn test_load_builtin_template() {
        let mut engine = TemplateEngine::new();
        let template = engine.load("technical-whitepaper").unwrap();
        assert_eq!(template.name, "technical-whitepaper");
        assert_eq!(template.format, TemplateFormat::Markdown);
    }

    #[test]
    fn test_load_unknown_template() {
        let mut engine = TemplateEngine::new();
        let result = engine.load("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_render_markdown() {
        let engine = TemplateEngine::new();
        let data = create_test_data();

        // Use get() instead of load() to avoid borrow conflict
        let template = engine.get("default").unwrap();
        let result = engine.render(&template, &data).unwrap();
        assert!(result.contains("Test Document"));
        assert!(result.contains("Test Author"));
    }

    #[test]
    fn test_replace_placeholders() {
        let engine = TemplateEngine::new();
        let data = create_test_data();

        let template = engine.get("default").unwrap();
        let result = engine.render(&template, &data).unwrap();

        assert!(result.contains("Test Document"));
        assert!(result.contains("Test Author"));
        assert!(result.contains("2026-03-19"));
    }

    #[test]
    fn test_list_templates() {
        let templates = TemplateEngine::list_templates();
        assert!(templates.contains(&"technical-whitepaper".to_string()));
        assert!(templates.contains(&"default".to_string()));
    }

    #[test]
    fn test_detect_format() {
        assert_eq!(
            TemplateEngine::detect_format("<!DOCTYPE html>"),
            TemplateFormat::Html
        );
        assert_eq!(
            TemplateEngine::detect_format("#let x = 1"),
            TemplateFormat::Typst
        );
        assert_eq!(
            TemplateEngine::detect_format("# Heading"),
            TemplateFormat::Markdown
        );
    }

    #[test]
    fn test_escape_typst_string() {
        let _engine = TemplateEngine::new();
        assert_eq!(
            TemplateEngine::escape_typst_string("Hello \"World\""),
            "Hello \\\"World\\\""
        );
        assert_eq!(
            TemplateEngine::escape_typst_string("Line\\nBreak"),
            "Line\\\\nBreak"
        );
    }

    #[test]
    fn test_exists() {
        let engine = TemplateEngine::new();
        assert!(engine.exists("default"));
        assert!(!engine.exists("nonexistent"));
    }

    #[test]
    fn test_template_formats() {
        let engine = TemplateEngine::new();

        let md = engine.get("default").unwrap();
        assert_eq!(md.format, TemplateFormat::Markdown);

        let tp = engine.get("technical-whitepaper").unwrap();
        assert_eq!(tp.format, TemplateFormat::Markdown);
    }
}