//! Synthesize command: Generate documents from the Structural Spine
//!
//! This command takes a Structure block and synthesizes a complete document
//! in various formats (PDF, HTML, Markdown, Typst) using the Synthesizer library.

use crate::db::Database;
use crate::synthesis::{OutputFormat, Synthesizer};
use ulid::Ulid;

/// Execute the synthesize command
///
/// Generates a complete document from a Structure block's spine of content.
pub async fn execute(
    db: &Database,
    id: &str,
    output: &str,
    template: &Option<String>,
    file: &Option<String>,
) -> anyhow::Result<()> {
    // Parse structure ID
    let structure_id = parse_ulid(id)?;

    // Determine output format
    let format = parse_output_format(output)?;

    // Determine template
    let template_name = template.as_deref();

    println!("📝 Synthesizing document from structure: {}", id);
    println!("   Format: {:?}", format);
    if let Some(t) = template_name {
        println!("   Template: {}", t);
    }
    if let Some(f) = file {
        println!("   Output file: {}", f);
    }
    println!();

    // Use the Synthesizer library which correctly handles SectionOf/ordered_child links
    println!("🔍 Step 1/3: Traversing structure via Synthesizer...");
    let synthesizer = Synthesizer::new(db);

    println!("📑 Step 2/3: Generating TOC and synthesizing content...");
    let result = synthesizer.synthesize(&structure_id, format, template_name).await?;

    println!("📤 Step 3/3: Rendering output...");

    // Output the result
    match format {
        OutputFormat::Markdown | OutputFormat::Html => {
            let content = String::from_utf8(result.content)?;
            output_content(&content, file)?;
        }
        OutputFormat::Pdf | OutputFormat::Typst => {
            // For PDF, the Synthesizer outputs Typst source
            let content = String::from_utf8(result.content)?;
            let output_file = file.clone().unwrap_or_else(|| format!("{}.typst", sanitize_filename(&result.title)));
            std::fs::write(&output_file, &content)?;
            println!("✅ Generated Typst source: {}", output_file);
            println!("   Compile with: typst compile {} [output.pdf]", output_file);
        }
    }

    // Print summary
    println!();
    println!("✅ Document synthesis complete!");
    println!();
    println!("📊 Statistics:");
    println!("   - Blocks used: {}", result.blocks_used);
    println!("   - Sections: {}", result.toc.sections.len());
    let total_chars = result.toc.sections.iter()
        .map(|s| s.blocks.len() * 200) // rough estimate
        .sum::<usize>();
    println!("   - Total content: ~{} characters", total_chars);

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

/// Parse output format
fn parse_output_format(s: &str) -> anyhow::Result<OutputFormat> {
    match s.to_lowercase().as_str() {
        "pdf" => Ok(OutputFormat::Pdf),
        "html" | "htm" => Ok(OutputFormat::Html),
        "markdown" | "md" => Ok(OutputFormat::Markdown),
        "typst" | "typ" => Ok(OutputFormat::Typst),
        _ => anyhow::bail!("Unknown format: {}. Use pdf, html, markdown, or typst", s),
    }
}

/// Sanitize filename
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

/// Output content to file or stdout
fn output_content(content: &str, file: &Option<String>) -> anyhow::Result<()> {
    match file {
        Some(path) => {
            std::fs::write(path, content)?;
            println!("✅ Written to: {}", path);
        }
        None => {
            println!("{}", content);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("My Document"), "My_Document");
        assert_eq!(sanitize_filename("Test/File"), "Test_File");
        assert_eq!(sanitize_filename("valid-name"), "valid-name");
    }

    #[test]
    fn test_parse_output_format() {
        assert!(matches!(parse_output_format("pdf").unwrap(), OutputFormat::Pdf));
        assert!(matches!(parse_output_format("html").unwrap(), OutputFormat::Html));
        assert!(matches!(parse_output_format("markdown").unwrap(), OutputFormat::Markdown));
        assert!(matches!(parse_output_format("typst").unwrap(), OutputFormat::Typst));
    }
}