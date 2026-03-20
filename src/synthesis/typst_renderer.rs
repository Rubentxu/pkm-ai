//! Typst Renderer: Generate PDF/HTML/Markdown from documents
//!
//! Renders synthesized content to various output formats.
//! PDF generation requires the typst crate to be enabled.
//! Falls back to Markdown/HTML when typst is unavailable.

#![allow(dead_code)]

use crate::{NexusError, NexusResult};

/// Output format for document rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum RenderFormat {
    /// PDF format (via Typst)
    Pdf,
    /// HTML format
    Html,
    /// Markdown format (fallback)
    #[default]
    Markdown,
}


/// Renderer for document output
#[derive(Debug, Clone)]
pub struct TypstRenderer {
    /// HTML CSS styles
    styles: String,
}

impl Default for TypstRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl TypstRenderer {
    /// Create a new renderer with default styles
    pub fn new() -> Self {
        Self {
            styles: Self::default_styles(),
        }
    }

    /// Create with custom CSS styles
    pub fn with_styles(styles: impl Into<String>) -> Self {
        Self {
            styles: styles.into(),
        }
    }

    /// Render document to PDF
    ///
    /// This requires the typst crate to be uncommented in Cargo.toml.
    /// Returns an error if typst is not available.
    pub async fn render_pdf(&self, content: &str) -> NexusResult<Vec<u8>> {
        #[cfg(feature = "typst")]
        {
            self.render_pdf_typst(content).await
        }

        #[cfg(not(feature = "typst"))]
        {
            let _ = content; // Suppress unused warning
            Err(NexusError::Synthesis(
                "PDF rendering requires the 'typst' feature to be enabled in Cargo.toml".to_string(),
            ))
        }
    }

    /// Internal PDF rendering with typst
    #[cfg(feature = "typst")]
    async fn render_pdf_typst(&self, content: &str) -> NexusResult<Vec<u8>> {
        use typst::World;

        // Compile the Typst content
        let world = TypstWorld::new(content);
        let document = typst::compile(&world)
            .map_err(|e| NexusError::Synthesis(format!("Typst compilation failed: {}", e)))?;

        // Export to PDF
        let pdf_bytes = typst::export::pdf(&document);
        Ok(pdf_bytes)
    }

    /// Render document to HTML
    pub async fn render_html(&self, content: &str) -> NexusResult<String> {
        let html = self.convert_markdown_to_html(content);
        let full_html = self.wrap_in_html_document(&html);
        Ok(full_html)
    }

    /// Render document to Markdown
    ///
    /// This is the fallback when other formats are unavailable.
    pub fn render_markdown(&self, content: &str) -> String {
        content.to_string()
    }

    /// Render to a specific format
    pub async fn render(&self, content: &str, format: RenderFormat) -> NexusResult<Vec<u8>> {
        match format {
            RenderFormat::Pdf => {
                let bytes = self.render_pdf(content).await?;
                Ok(bytes)
            }
            RenderFormat::Html => {
                let html = self.render_html(content).await?;
                Ok(html.into_bytes())
            }
            RenderFormat::Markdown => Ok(self.render_markdown(content).into_bytes()),
        }
    }

    /// Convert Markdown content to HTML
    fn convert_markdown_to_html(&self, markdown: &str) -> String {
        // Use the markdown crate for basic parsing
        
        markdown::to_html(markdown)
    }

    /// Wrap HTML content in a full HTML document with styles
    fn wrap_in_html_document(&self, body_content: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Document</title>
    <style>
{styles}
    </style>
</head>
<body>
{body}
</body>
</html>"#,
            styles = self.styles,
            body = body_content
        )
    }

    /// Default CSS styles for HTML output
    fn default_styles() -> String {
        r#"
        :root {
            --bg-color: #ffffff;
            --text-color: #333333;
            --heading-color: #1a1a1a;
            --link-color: #0066cc;
            --code-bg: #f5f5f5;
            --border-color: #dddddd;
        }

        @media (prefers-color-scheme: dark) {
            :root {
                --bg-color: #1a1a1a;
                --text-color: #e0e0e0;
                --heading-color: #ffffff;
                --link-color: #66b3ff;
                --code-bg: #2d2d2d;
                --border-color: #444444;
            }
        }

        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
            line-height: 1.6;
            max-width: 800px;
            margin: 0 auto;
            padding: 2rem;
            background-color: var(--bg-color);
            color: var(--text-color);
        }

        h1, h2, h3, h4, h5, h6 {
            color: var(--heading-color);
            margin-top: 1.5em;
            margin-bottom: 0.5em;
        }

        h1 { font-size: 2em; border-bottom: 2px solid var(--border-color); padding-bottom: 0.3em; }
        h2 { font-size: 1.5em; border-bottom: 1px solid var(--border-color); padding-bottom: 0.2em; }
        h3 { font-size: 1.25em; }

        a {
            color: var(--link-color);
            text-decoration: none;
        }

        a:hover {
            text-decoration: underline;
        }

        code {
            background-color: var(--code-bg);
            padding: 0.2em 0.4em;
            border-radius: 3px;
            font-family: 'Fira Code', 'Consolas', monospace;
            font-size: 0.9em;
        }

        pre {
            background-color: var(--code-bg);
            padding: 1em;
            border-radius: 5px;
            overflow-x: auto;
        }

        pre code {
            background: none;
            padding: 0;
        }

        blockquote {
            border-left: 4px solid var(--link-color);
            margin: 1em 0;
            padding-left: 1em;
            color: #666;
        }

        table {
            border-collapse: collapse;
            width: 100%;
            margin: 1em 0;
        }

        th, td {
            border: 1px solid var(--border-color);
            padding: 0.5em;
            text-align: left;
        }

        th {
            background-color: var(--code-bg);
        }

        .bibliography {
            margin-top: 2em;
            padding-top: 1em;
            border-top: 1px solid var(--border-color);
        }

        .toc {
            background-color: var(--code-bg);
            padding: 1em;
            border-radius: 5px;
            margin: 1em 0;
        }

        .section-content {
            margin: 1em 0;
        }
        "#
        .to_string()
    }
}

/// Typst world implementation for compilation
#[cfg(feature = "typst")]
struct TypstWorld {
    source: String,
}

#[cfg(feature = "typst")]
impl TypstWorld {
    fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
        }
    }
}

#[cfg(feature = "typst")]
impl typst::World for TypstWorld {
    fn library(&self) -> &typst::Library {
        &typst::Library::default()
    }

    fn main(&self) -> &str {
        &self.source
    }

    fn source(&self, _id: typst::iso::Path) -> Option<&str> {
        Some(&self.source)
    }

    fn file(&self, _id: &typst::iso::Path) -> Option<Result<Vec<u8>, std::io::Error>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_format_default() {
        let format = RenderFormat::default();
        assert_eq!(format, RenderFormat::Markdown);
    }

    #[test]
    fn test_typst_renderer_new() {
        let renderer = TypstRenderer::new();
        assert!(!renderer.styles.is_empty());
    }

    #[test]
    fn test_render_markdown() {
        let renderer = TypstRenderer::new();
        let input = "# Hello\n\nThis is **bold**.";
        let output = renderer.render_markdown(input);
        assert_eq!(output, input);
    }

    #[tokio::test]
    async fn test_render_html() {
        let renderer = TypstRenderer::new();
        let input = "# Hello\n\nThis is **bold**.";
        let output = renderer.render_html(input).await.unwrap();

        assert!(output.contains("<!DOCTYPE html>"));
        assert!(output.contains("<h1>Hello</h1>"));
        assert!(output.contains("This is <strong>bold</strong>"));
    }

    #[tokio::test]
    async fn test_render_markdown_async() {
        let renderer = TypstRenderer::new();
        let input = "# Test";
        let output = renderer.render(input, RenderFormat::Markdown).await.unwrap();
        assert_eq!(output, input.as_bytes());
    }

    #[tokio::test]
    async fn test_render_pdf_unavailable() {
        let renderer = TypstRenderer::new();
        let result = renderer.render_pdf("# Test").await;
        #[cfg(not(feature = "typst"))]
        assert!(result.is_err());
    }

    #[test]
    fn test_convert_markdown_to_html() {
        let renderer = TypstRenderer::new();

        // Headers
        let html = renderer.convert_markdown_to_html("# Header");
        assert!(html.contains("<h1>Header</h1>"));

        // Bold
        let html = renderer.convert_markdown_to_html("**bold**");
        assert!(html.contains("<strong>bold</strong>"));

        // Links
        let html = renderer.convert_markdown_to_html("[link](https://example.com)");
        assert!(html.contains("<a href=\"https://example.com\">link</a>"));
    }

    #[test]
    fn test_wrap_in_html_document() {
        let renderer = TypstRenderer::new();
        let body = "<h1>Test</h1>";
        let html = renderer.wrap_in_html_document(body);

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<h1>Test</h1>"));
        assert!(html.contains("<style>"));
    }

    #[test]
    fn test_default_styles_contains_common_rules() {
        let renderer = TypstRenderer::new();
        assert!(renderer.styles.contains("body {"));
        assert!(renderer.styles.contains("h1 {"));
        assert!(renderer.styles.contains("--bg-color:"));
    }
}