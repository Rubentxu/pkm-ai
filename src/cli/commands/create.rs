//! Create command

use crate::ai::embeddings::EmbeddingGenerator;
use crate::ai::link_suggester::LinkSuggester;
use crate::db::Database;
use crate::models::{Block, BlockType, SmartSection};
use crate::{NexusError, NexusResult};
use crossterm::{
    execute,
    style::Stylize,
    terminal::{Clear, ClearType},
};
use std::collections::HashSet;
use std::io::{self, Write};

/// Similarity threshold for duplicate detection
const DUPLICATE_THRESHOLD: f32 = 0.9;

/// Gravity affinity threshold for suggesting location
const GRAVITY_THRESHOLD: f32 = 0.3;

/// High similarity threshold for auto-asking about existing notes
const HIGH_SIMILARITY_THRESHOLD: f32 = 0.95;

/// AI Pre-flight suggestions for a new block
#[derive(Debug, Clone, Default)]
pub struct PreflightSuggestions {
    /// Similar blocks that might be duplicates (similarity > 0.9)
    pub duplicates: Vec<(Block, f32)>,
    /// Suggested location based on gravity check
    pub location: Option<(Block, f32)>,
    /// Suggested tags based on similar notes
    pub suggested_tags: Vec<String>,
    /// Suggested links based on semantic similarity
    pub suggested_links: Vec<(Block, f32)>,
}

/// Print pre-flight suggestions in a user-friendly format
pub fn print_suggestions(suggestions: &PreflightSuggestions) {
    println!();
    println!("🤖 AI Pre-Flight:");
    println!();

    // Duplicates
    if !suggestions.duplicates.is_empty() {
        println!("⚠️  Similar notes found (possible duplicates):");
        for (block, sim) in &suggestions.duplicates {
            println!("   - \"{}\" ({:.2})", block.title, sim);
        }
        println!();
    }

    // Suggested location
    if let Some((block, affinity)) = &suggestions.location {
        println!("📍 Suggested location: \"{}\" (affinity: {:.2})", block.title, affinity);
        println!();
    }

    // Suggested tags
    if !suggestions.suggested_tags.is_empty() {
        println!("🏷️  Suggested tags: {}", suggestions.suggested_tags.join(", "));
        println!();
    }

    // Suggested links
    if !suggestions.suggested_links.is_empty() {
        println!("🔗 Suggested links: {} notes", suggestions.suggested_links.len());
        for (block, confidence) in suggestions.suggested_links.iter().take(5) {
            println!("   - \"{}\" ({:.2})", block.title, confidence);
        }
        if suggestions.suggested_links.len() > 5 {
            println!("   ... and {} more", suggestions.suggested_links.len() - 5);
        }
        println!();
    }
}

/// AI pre-flight check for a new block
///
/// Performs:
/// - Duplicate detection (title similarity > 0.9)
/// - Location suggestion (gravity check against structures)
/// - Tag suggestions (based on similar blocks)
/// - Link suggestions (semantic similarity)
pub async fn ai_preflight_check(
    db: &Database,
    title: &str,
    content: &Option<String>,
) -> anyhow::Result<PreflightSuggestions> {
    let mut suggestions = PreflightSuggestions::default();

    // Build candidate text from title and content
    let candidate_text = match content {
        Some(c) if !c.is_empty() => format!("{} {}", title, c),
        _ => title.to_string(),
    };

    let block_repo = db.blocks();
    let all_blocks = block_repo.list_all().await.unwrap_or_default();

    if all_blocks.is_empty() {
        return Ok(suggestions);
    }

    // 1. Find duplicates using embeddings similarity
    let embeddings = EmbeddingGenerator::new();
    let candidate_emb = embeddings.embed_text(&candidate_text).await?;

    for block in &all_blocks {
        // Check title similarity first (fast path)
        let title_sim = calculate_title_similarity(title, &block.title);
        if title_sim >= DUPLICATE_THRESHOLD {
            suggestions.duplicates.push((block.clone(), title_sim));
            continue;
        }

        // Check full embedding similarity for content blocks
        if !block.content.is_empty()
            && let Ok(block_emb) = embeddings.embed(block).await
        {
            let sim = EmbeddingGenerator::cosine_similarity(&candidate_emb, &block_emb);
            if sim >= DUPLICATE_THRESHOLD {
                suggestions.duplicates.push((block.clone(), sim));
            }
        }
    }

    // Sort duplicates by similarity descending
    suggestions.duplicates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // 2. Gravity check - find best structure for this content
    let structures = block_repo.list_by_type(BlockType::Structure).await.unwrap_or_default();

    if !structures.is_empty() {
        let mut best_affinity = GRAVITY_THRESHOLD;
        let mut best_structure: Option<Block> = None;

        for structure in &structures {
            let section = build_smart_section(structure);
            let affinity = calculate_gravity_affinity(title, content, &section);

            if affinity > best_affinity {
                best_affinity = affinity;
                best_structure = Some(structure.clone());
            }
        }

        if let Some(structure) = best_structure {
            suggestions.location = Some((structure, best_affinity));
        }
    }

    // 3. Suggest tags based on similar blocks
    let mut tag_counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();

    for block in &all_blocks {
        // Use embedding similarity to find similar blocks
        if let Ok(block_emb) = embeddings.embed(block).await {
            let sim = EmbeddingGenerator::cosine_similarity(&candidate_emb, &block_emb);
            if sim > 0.6 {
                // Collect tags from similar blocks
                for tag in &block.tags {
                    let normalized = tag.to_lowercase().trim().to_string();
                    if !normalized.is_empty() {
                        *tag_counts.entry(normalized).or_insert(0) += 1;
                    }
                }
            }
        }
    }

    // Sort tags by frequency and take top 5
    let mut sorted_tags: Vec<_> = tag_counts.into_iter().collect();
    sorted_tags.sort_by(|a, b| b.1.cmp(&a.1));
    suggestions.suggested_tags = sorted_tags
        .into_iter()
        .take(5)
        .map(|(tag, _)| tag)
        .collect();

    // 4. Suggest links using LinkSuggester
    let temp_block = Block::new(BlockType::Permanent, title);
    let mut temp_block = temp_block;
    if let Some(c) = content {
        temp_block = temp_block.with_content(c.clone());
    }

    let link_suggester = LinkSuggester::new();
    let candidates: Vec<_> = all_blocks
        .iter()
        .filter(|b| b.id != temp_block.id)
        .cloned()
        .collect();

    if !candidates.is_empty() {
        let exclude_ids: HashSet<ulid::Ulid> = HashSet::new();
        if let Ok(links) = link_suggester
            .suggest_outgoing(&temp_block, &candidates, Some(&exclude_ids))
            .await
        {
            for link in links.into_iter().take(5) {
                if let Some(block) = all_blocks.iter().find(|b| b.id == link.target_id) {
                    suggestions.suggested_links.push((block.clone(), link.confidence));
                }
            }
        }
    }

    Ok(suggestions)
}

/// Calculate title similarity using simple word overlap
fn calculate_title_similarity(title1: &str, title2: &str) -> f32 {
    let words1: std::collections::HashSet<String> = title1
        .to_lowercase()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    let words2: std::collections::HashSet<String> = title2
        .to_lowercase()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    if words1.is_empty() || words2.is_empty() {
        return 0.0;
    }

    let intersection = words1.intersection(&words2).count();
    let union = words1.union(&words2).count();

    intersection as f32 / union as f32
}

/// Build a smart section from a structure block (simplified from gravity_check)
fn build_smart_section(structure: &Block) -> SmartSection {
    let mut section = SmartSection::new(&structure.title);
    section.intent = structure.title.clone();

    // Extract keywords from content
    if !structure.content.is_empty() {
        let words: Vec<String> = structure
            .content
            .split_whitespace()
            .take(20)
            .map(|w| {
                w.to_lowercase()
                    .chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>()
            })
            .filter(|w| w.len() > 3)
            .collect();
        section.keywords = words;
    }

    // Extract keywords from tags
    for tag in &structure.tags {
        section.keywords.push(tag.to_lowercase());
    }

    section.density = 0;
    section.vacancy = crate::models::VacancyLevel::Empty;

    section
}

/// Calculate gravity affinity between a block candidate and a section
fn calculate_gravity_affinity(title: &str, content: &Option<String>, section: &SmartSection) -> f32 {
    let mut score: f32 = 0.0;
    let mut weight: f32 = 0.0;

    // Title keyword matching
    let title_words: Vec<String> = title
        .to_lowercase()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    for keyword in &section.keywords {
        weight += 1.0;
        if title_words.iter().any(|w| w.contains(&keyword.to_lowercase())) {
            score += 2.0; // Title matches are more important
        }
    }

    // Content keyword matching
    if let Some(c) = content {
        let content_words: Vec<String> = c
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        for keyword in &section.keywords {
            if content_words.iter().any(|w| w.contains(&keyword.to_lowercase())) {
                score += 1.0;
            }
        }
    }

    if weight == 0.0 {
        return GRAVITY_THRESHOLD;
    }

    (score / weight).min(1.0)
}

pub async fn execute(
    db: &Database,
    block_type: &str,
    title: &str,
    content: &Option<String>,
    tags: &Option<String>,
    verbose: bool,
    auto_stage: bool,
    interactive: bool,
) -> anyhow::Result<()> {
    let block_type_str = block_type.to_string();

    // Auto-detect block type if not explicitly provided
    let detected_type = if block_type_str.is_empty() || block_type_str == "permanent" {
        detect_block_type(title, content)
    } else {
        None
    };

    let block_type = if let Some(dt) = detected_type {
        if verbose {
            println!("🔍 Auto-detected type: {:?}", dt);
        }
        dt
    } else {
        parse_block_type(block_type)?
    };

    // Run AI pre-flight check
    let suggestions = ai_preflight_check(db, title, content).await?;

    // Handle interactive mode
    if interactive {
        return run_interactive_mode(db, title, content, tags, &suggestions, block_type, auto_stage).await;
    }

    // Show pre-flight suggestions if verbose mode is enabled
    if verbose {
        print_suggestions(&suggestions);
    }

    let mut block = Block::new(block_type, title);

    if let Some(content) = content {
        block = block.with_content(content);
    }

    if let Some(tags_str) = tags {
        let tags: Vec<String> = tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        for tag in tags {
            block = block.with_tag(tag);
        }
    }

    let repo = db.blocks();
    repo.create(block.clone()).await?;

    println!("Created block: {}", block.id);
    println!("   Type: {:?}", block.block_type);
    println!("   Title: {}", block.title);
    if !block.tags.is_empty() {
        println!("   Tags: {}", block.tags.join(", "));
    }
    println!("   ULID: {}", block.id_str());

    // Auto-stage the block if requested
    if auto_stage {
        if let Err(e) = stage_block(block.id).await {
            eprintln!("Warning: Failed to stage block: {}", e);
        } else {
            println!("   Staged for commit");
        }
    }

    Ok(())
}

pub fn parse_block_type(s: &str) -> NexusResult<BlockType> {
    match s.to_lowercase().as_str() {
        "fleeting" | "f" => Ok(BlockType::Fleeting),
        "literature" | "l" => Ok(BlockType::Literature),
        "permanent" | "p" => Ok(BlockType::Permanent),
        "structure" | "s" | "index" | "moc" => Ok(BlockType::Structure),
        "hub" | "h" => Ok(BlockType::Hub),
        "task" | "t" => Ok(BlockType::Task),
        "reference" | "r" => Ok(BlockType::Reference),
        "outline" | "o" => Ok(BlockType::Outline),
        "ghost" | "g" => Ok(BlockType::Ghost),
        _ => Err(NexusError::InvalidBlockType(s.to_string())),
    }
}

/// Stage a block in the version repository
async fn stage_block(block_id: ulid::Ulid) -> anyhow::Result<()> {
    use pkm_ai::versioning::VersionRepo;

    let repo_path = resolve_repo_path();
    let version_repo = VersionRepo::new(&repo_path);
    version_repo.init()?;
    version_repo.stage(&block_id)?;
    Ok(())
}

/// Resolve the repository path (same logic as in version.rs)
fn resolve_repo_path() -> std::path::PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join(".pkm")
}

// ============================================================================
// Smart Type Detection
// ============================================================================

/// Keywords for auto-detecting block type from content
const TASK_KEYWORDS: &[&str] = &[
    "todo", "task", "tarea", "hacer", "pending", "pending", "action",
    "deadline", "due", "complete", "finish", "implement", "fix", "bug",
    "refactor", "add", "create", "update", "delete", "remove",
];

const REFERENCE_KEYWORDS: &[&str] = &[
    "quote", "cita", "reference", "referencia", "book", "libro",
    "article", "artículo", "paper", "paper", "source", "fuente",
    "chapter", "capítulo", "page", "página", "author", "autor",
];

const LITERATURE_KEYWORDS: &[&str] = &[
    "idea", "note", "nota", "thought", "pensamiento", "insight",
    "observation", "observation", "brainstorm", "collection",
];

const STRUCTURE_KEYWORDS: &[&str] = &[
    "index", "structure", "estructura", "moc", "map", "overview",
    "summary", "resumen", "toc", "table of contents", "directory",
];

/// Detect block type based on title and content keywords
fn detect_block_type(title: &str, content: &Option<String>) -> Option<BlockType> {
    let text = if let Some(c) = content {
        format!("{} {}", title, c).to_lowercase()
    } else {
        title.to_lowercase()
    };

    let mut task_score = 0;
    let mut reference_score = 0;
    let mut literature_score = 0;
    let mut structure_score = 0;

    for kw in TASK_KEYWORDS {
        if text.contains(kw) {
            task_score += 1;
        }
    }

    for kw in REFERENCE_KEYWORDS {
        if text.contains(kw) {
            reference_score += 1;
        }
    }

    for kw in LITERATURE_KEYWORDS {
        if text.contains(kw) {
            literature_score += 1;
        }
    }

    for kw in STRUCTURE_KEYWORDS {
        if text.contains(kw) {
            structure_score += 1;
        }
    }

    // Also check for TODO prefix
    let title_lower = title.to_lowercase();
    if title_lower.starts_with("todo:")
        || title_lower.starts_with("todo ")
        || title_lower.starts_with("[ ]")
    {
        task_score += 3;
    }

    // Check for book/reference patterns
    if title_lower.contains("chapter")
        || title_lower.contains("page ")
        || title_lower.contains("page:")
    {
        reference_score += 2;
    }

    let max_score = task_score.max(reference_score).max(literature_score).max(structure_score);

    if max_score == 0 {
        return None;
    }

    if task_score == max_score {
        Some(BlockType::Task)
    } else if reference_score == max_score {
        Some(BlockType::Reference)
    } else if structure_score == max_score {
        Some(BlockType::Structure)
    } else if literature_score == max_score {
        Some(BlockType::Literature)
    } else {
        None
    }
}

// ============================================================================
// Interactive Mode
// ============================================================================

/// User choice in interactive mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractiveChoice {
    /// Use existing duplicate block
    Yes,
    /// Create new block anyway
    No,
    /// Edit before creating
    Edit,
    /// Abort operation
    Abort,
}

/// Run interactive pre-flight mode
async fn run_interactive_mode(
    db: &Database,
    title: &str,
    content: &Option<String>,
    tags: &Option<String>,
    suggestions: &PreflightSuggestions,
    block_type: BlockType,
    auto_stage: bool,
) -> anyhow::Result<()> {
    // Check if terminal is interactive, fallback to non-interactive if not
    if !is_interactive_terminal() {
        eprintln!("Warning: Non-interactive terminal detected, running in non-interactive mode");
        return create_block(db, title, content, tags, block_type, auto_stage).await;
    }

    // Clear screen and show header
    clear_screen()?;
    print_interactive_header();

    // Show AI pre-flight suggestions with colors
    print_colored_suggestions(suggestions);

    // Check for high similarity duplicates (>0.95)
    let high_sim_duplicates: Vec<_> = suggestions
        .duplicates
        .iter()
        .filter(|(_, sim)| *sim >= HIGH_SIMILARITY_THRESHOLD)
        .collect();

    if !high_sim_duplicates.is_empty() {
        // Ask about using existing note
        println!();
        println!("{}", format!("⚠️  Similar note found: \"{}\" ({:.2})", high_sim_duplicates[0].0.title, high_sim_duplicates[0].1).yellow());
        println!();
        println!("{}", "[y]es (use existing) / [n]o (create new) / [e]dit / [a]bort".cyan());

        let choice = prompt_choice()?;

        match choice {
            InteractiveChoice::Yes => {
                println!("\n✅ Using existing note: {}", high_sim_duplicates[0].0.id_str());
                return Ok(());
            }
            InteractiveChoice::Abort => {
                println!("\n❌ Aborted by user");
                return Ok(());
            }
            InteractiveChoice::Edit => {
                // Fall through to editing
            }
            InteractiveChoice::No => {
                // Continue with creation
            }
        }
    }

    // Ask for each suggestion category
    if !suggestions.duplicates.is_empty() && high_sim_duplicates.is_empty() {
        println!();
        println!("{}", "📋  Similar notes found: ".cyan());
        for (block, sim) in suggestions.duplicates.iter().take(3) {
            print!("   \"{}\" ({:.2}) ", block.title, sim);
        }
        println!();
        println!("{}", "[y]es (accept suggestions) / [n]o (ignore) / [e]dit / [a]bort".cyan());

        let choice = prompt_choice()?;

        match choice {
            InteractiveChoice::Yes => {
                // Merge suggestions into block creation
                println!("\n✅ Applying AI suggestions");
            }
            InteractiveChoice::Abort => {
                println!("\n❌ Aborted by user");
                return Ok(());
            }
            InteractiveChoice::Edit => {
                // Let user edit
            }
            InteractiveChoice::No => {
                // Ignore suggestions, continue
            }
        }
    }

    // Show location suggestion
    if let Some((block, affinity)) = &suggestions.location {
        println!();
        println!("{}", format!("📍  Suggested location: \"{}\" (affinity: {:.2})", block.title, affinity).cyan());
        println!("{}", "[y]es (accept) / [n]o (ignore)".cyan());

        let choice = prompt_choice()?;
        if choice == InteractiveChoice::Yes {
            println!("   → Will place under: {}", block.title);
        }
    }

    // Show suggested tags
    if !suggestions.suggested_tags.is_empty() {
        println!();
        println!("{}", format!("🏷️  Suggested tags: {}", suggestions.suggested_tags.join(", ")).yellow());
        println!("{}", "[y]es (accept) / [n]o (ignore)".cyan());

        let choice = prompt_choice()?;
        if choice == InteractiveChoice::Yes {
            println!("   → Will apply tags: {}", suggestions.suggested_tags.join(", "));
        }
    }

    // Create the block
    println!();
    create_block(db, title, content, tags, block_type, auto_stage).await
}

/// Check if running in an interactive terminal
fn is_interactive_terminal() -> bool {
    atty::is(atty::Stream::Stdin) && atty::is(atty::Stream::Stdout)
}

/// Clear the terminal screen
fn clear_screen() -> io::Result<()> {
    execute!(io::stdout(), Clear(ClearType::All))?;
    Ok(())
}

/// Print interactive mode header
fn print_interactive_header() {
    println!();
    println!("{}", "═".repeat(60).dim());
    println!("{}", "  🤖  AI Pre-Flight Check - Interactive Mode".bold().cyan());
    println!("{}", "═".repeat(60).dim());
    println!();
}

/// Print suggestions with colors
fn print_colored_suggestions(suggestions: &PreflightSuggestions) {
    // Duplicates
    if !suggestions.duplicates.is_empty() {
        println!("{}", "⚠️  Similar notes found (possible duplicates):".yellow().bold());
        for (block, sim) in &suggestions.duplicates {
            let similarity_color = if *sim >= 0.95 {
                "🔴"
            } else if *sim >= 0.9 {
                "🟡"
            } else {
                "🟢"
            };
            println!("   {} \"{}\" ({:.2})", similarity_color, block.title, sim);
        }
        println!();
    }

    // Suggested location
    if let Some((block, affinity)) = &suggestions.location {
        println!("{}", "📍  Suggested location:".cyan().bold());
        println!("   → \"{}\" (affinity: {:.2})", block.title, affinity);
        println!();
    }

    // Suggested tags
    if !suggestions.suggested_tags.is_empty() {
        println!("{}", "🏷️  Suggested tags:".magenta().bold());
        println!("   {}", suggestions.suggested_tags.join(", ").yellow());
        println!();
    }

    // Suggested links
    if !suggestions.suggested_links.is_empty() {
        println!("{}", "🔗  Suggested links:".green().bold());
        for (block, confidence) in suggestions.suggested_links.iter().take(5) {
            println!("   → \"{}\" ({:.2})", block.title, confidence);
        }
        if suggestions.suggested_links.len() > 5 {
            println!("   ... and {} more", suggestions.suggested_links.len() - 5);
        }
        println!();
    }
}

/// Prompt user for choice and parse input
fn prompt_choice() -> io::Result<InteractiveChoice> {
    print!("> ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let choice = input.trim().to_lowercase();

    match choice.as_str() {
        "y" | "yes" | "s" | "si" | "yep" => Ok(InteractiveChoice::Yes),
        "n" | "no" | "nope" => Ok(InteractiveChoice::No),
        "e" | "edit" | "ed" => Ok(InteractiveChoice::Edit),
        "a" | "abort" | "cancel" | "q" | "quit" => Ok(InteractiveChoice::Abort),
        _ => {
            println!("Invalid choice. Please enter: y/n/e/a");
            prompt_choice()
        }
    }
}

/// Create the block (helper function)
async fn create_block(
    db: &Database,
    title: &str,
    content: &Option<String>,
    tags: &Option<String>,
    block_type: BlockType,
    auto_stage: bool,
) -> anyhow::Result<()> {
    let mut block = Block::new(block_type, title);

    if let Some(c) = content {
        block = block.with_content(c);
    }

    if let Some(tags_str) = tags {
        let tag_list: Vec<String> = tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        for tag in tag_list {
            block = block.with_tag(tag);
        }
    }

    let repo = db.blocks();
    repo.create(block.clone()).await?;

    println!("✅ Created block: {}", block.id);
    println!("   Type: {:?}", block.block_type);
    println!("   Title: {}", block.title);
    if !block.tags.is_empty() {
        println!("   Tags: {}", block.tags.join(", "));
    }
    println!("   ULID: {}", block.id_str());

    // Auto-stage the block if requested
    if auto_stage {
        if let Err(e) = stage_block(block.id).await {
            eprintln!("Warning: Failed to stage block: {}", e);
        } else {
            println!("   Staged for commit");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_block_type_fleeting() {
        assert_eq!(parse_block_type("fleeting").unwrap(), BlockType::Fleeting);
        assert_eq!(parse_block_type("F").unwrap(), BlockType::Fleeting);
        assert_eq!(parse_block_type("f").unwrap(), BlockType::Fleeting);
        assert_eq!(parse_block_type("FLEETING").unwrap(), BlockType::Fleeting);
    }

    #[test]
    fn test_parse_block_type_literature() {
        assert_eq!(parse_block_type("literature").unwrap(), BlockType::Literature);
        assert_eq!(parse_block_type("L").unwrap(), BlockType::Literature);
        assert_eq!(parse_block_type("l").unwrap(), BlockType::Literature);
    }

    #[test]
    fn test_parse_block_type_permanent() {
        assert_eq!(parse_block_type("permanent").unwrap(), BlockType::Permanent);
        assert_eq!(parse_block_type("P").unwrap(), BlockType::Permanent);
        assert_eq!(parse_block_type("p").unwrap(), BlockType::Permanent);
    }

    #[test]
    fn test_parse_block_type_structure() {
        assert_eq!(parse_block_type("structure").unwrap(), BlockType::Structure);
        assert_eq!(parse_block_type("S").unwrap(), BlockType::Structure);
        assert_eq!(parse_block_type("s").unwrap(), BlockType::Structure);
        assert_eq!(parse_block_type("index").unwrap(), BlockType::Structure);
        assert_eq!(parse_block_type("moc").unwrap(), BlockType::Structure);
    }

    #[test]
    fn test_parse_block_type_hub() {
        assert_eq!(parse_block_type("hub").unwrap(), BlockType::Hub);
        assert_eq!(parse_block_type("H").unwrap(), BlockType::Hub);
        assert_eq!(parse_block_type("h").unwrap(), BlockType::Hub);
    }

    #[test]
    fn test_parse_block_type_task() {
        assert_eq!(parse_block_type("task").unwrap(), BlockType::Task);
        assert_eq!(parse_block_type("T").unwrap(), BlockType::Task);
        assert_eq!(parse_block_type("t").unwrap(), BlockType::Task);
    }

    #[test]
    fn test_parse_block_type_reference() {
        assert_eq!(parse_block_type("reference").unwrap(), BlockType::Reference);
        assert_eq!(parse_block_type("R").unwrap(), BlockType::Reference);
        assert_eq!(parse_block_type("r").unwrap(), BlockType::Reference);
    }

    #[test]
    fn test_parse_block_type_outline() {
        assert_eq!(parse_block_type("outline").unwrap(), BlockType::Outline);
        assert_eq!(parse_block_type("O").unwrap(), BlockType::Outline);
        assert_eq!(parse_block_type("o").unwrap(), BlockType::Outline);
    }

    #[test]
    fn test_parse_block_type_ghost() {
        assert_eq!(parse_block_type("ghost").unwrap(), BlockType::Ghost);
        assert_eq!(parse_block_type("G").unwrap(), BlockType::Ghost);
        assert_eq!(parse_block_type("g").unwrap(), BlockType::Ghost);
    }

    #[test]
    fn test_parse_block_type_invalid() {
        let result = parse_block_type("invalid");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NexusError::InvalidBlockType(_)));
    }

    #[test]
    fn test_parse_block_type_case_insensitive() {
        assert_eq!(parse_block_type("Permanent").unwrap(), BlockType::Permanent);
        assert_eq!(parse_block_type("PERMANENT").unwrap(), BlockType::Permanent);
        assert_eq!(parse_block_type("PeRmAnEnT").unwrap(), BlockType::Permanent);
    }

    #[test]
    fn test_calculate_title_similarity_exact() {
        let sim = calculate_title_similarity("Rust Ownership", "Rust Ownership");
        assert!((sim - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_title_similarity_partial() {
        let sim = calculate_title_similarity("Rust Ownership", "Rust Ownership Model");
        assert!(sim > 0.5); // Should be similar but not exact
    }

    #[test]
    fn test_calculate_title_similarity_different() {
        let sim = calculate_title_similarity("Rust Ownership", "Python Programming");
        assert!(sim < 0.5); // Should be different
    }

    #[test]
    fn test_calculate_title_similarity_empty() {
        let sim = calculate_title_similarity("", "Rust Ownership");
        assert!(sim.abs() < 0.01);
    }

    #[test]
    fn test_preflight_suggestions_default() {
        let suggestions = PreflightSuggestions::default();
        assert!(suggestions.duplicates.is_empty());
        assert!(suggestions.location.is_none());
        assert!(suggestions.suggested_tags.is_empty());
        assert!(suggestions.suggested_links.is_empty());
    }

    #[test]
    fn test_resolve_repo_path_ends_with_pkm() {
        let path = resolve_repo_path();
        assert!(path.to_string_lossy().ends_with(".pkm"));
    }

    // =========================================================================
    // Smart Type Detection Tests
    // =========================================================================

    #[test]
    fn test_detect_block_type_task_from_title() {
        // Test TODO prefix detection
        assert_eq!(detect_block_type("TODO: implement auth", &None), Some(BlockType::Task));
        assert_eq!(detect_block_type("TODO fix bug", &None), Some(BlockType::Task));
        assert_eq!(detect_block_type("[ ] Complete task", &None), Some(BlockType::Task));
    }

    #[test]
    fn test_detect_block_type_task_from_content() {
        // Test task keywords in content
        assert_eq!(
            detect_block_type("My Note", &Some("I need to fix this bug soon".to_string())),
            Some(BlockType::Task)
        );
        assert_eq!(
            detect_block_type("Notes", &Some("Implement login feature".to_string())),
            Some(BlockType::Task)
        );
    }

    #[test]
    fn test_detect_block_type_reference() {
        // Test reference keywords
        assert_eq!(
            detect_block_type("Chapter 1", &Some("Book quote here".to_string())),
            Some(BlockType::Reference)
        );
        assert_eq!(
            detect_block_type("My Notes", &Some("Author: Someone. Source: Book title".to_string())),
            Some(BlockType::Reference)
        );
    }

    #[test]
    fn test_detect_block_type_structure() {
        // Test structure keywords
        assert_eq!(
            detect_block_type("Index", &None),
            Some(BlockType::Structure)
        );
        assert_eq!(
            detect_block_type("MOC Overview", &None),
            Some(BlockType::Structure)
        );
        assert_eq!(
            detect_block_type("Table of Contents", &None),
            Some(BlockType::Structure)
        );
    }

    #[test]
    fn test_detect_block_type_literature() {
        // Test literature keywords
        assert_eq!(
            detect_block_type("My Note", &Some("Interesting idea: something".to_string())),
            Some(BlockType::Literature)
        );
        assert_eq!(
            detect_block_type("Observation", &Some("I noticed that...".to_string())),
            Some(BlockType::Literature)
        );
    }

    #[test]
    fn test_detect_block_type_no_match() {
        // No keywords should return None
        assert_eq!(detect_block_type("Random Title", &None), None);
        assert_eq!(
            detect_block_type("Title", &Some("Some random content without keywords".to_string())),
            None
        );
    }

    #[test]
    fn test_detect_block_type_priority() {
        // Task title should have higher priority than reference keywords in content
        assert_eq!(
            detect_block_type("TODO: implement auth", &Some("book reference".to_string())),
            Some(BlockType::Task)
        );
    }

    // =========================================================================
    // Interactive Choice Tests
    // =========================================================================

    #[test]
    fn test_interactive_choice_values() {
        assert_eq!(InteractiveChoice::Yes, InteractiveChoice::Yes);
        assert_eq!(InteractiveChoice::No, InteractiveChoice::No);
        assert_eq!(InteractiveChoice::Edit, InteractiveChoice::Edit);
        assert_eq!(InteractiveChoice::Abort, InteractiveChoice::Abort);
    }

    #[test]
    fn test_interactive_choice_different() {
        assert_ne!(InteractiveChoice::Yes, InteractiveChoice::No);
        assert_ne!(InteractiveChoice::Edit, InteractiveChoice::Abort);
    }

    // =========================================================================
    // Preflight Suggestions Tests
    // =========================================================================

    #[test]
    fn test_preflight_suggestions_with_duplicates() {
        let block = Block::new(BlockType::Permanent, "Test Block");
        let mut suggestions = PreflightSuggestions::default();
        suggestions.duplicates.push((block.clone(), 0.95));

        assert_eq!(suggestions.duplicates.len(), 1);
        assert_eq!(suggestions.duplicates[0].0.title, "Test Block");
    }

    #[test]
    fn test_preflight_suggestions_with_location() {
        let block = Block::new(BlockType::Structure, "Parent Structure");
        let mut suggestions = PreflightSuggestions::default();
        suggestions.location = Some((block, 0.75));

        assert!(suggestions.location.is_some());
        assert_eq!(suggestions.location.as_ref().unwrap().0.title, "Parent Structure");
    }

    #[test]
    fn test_preflight_suggestions_with_tags() {
        let mut suggestions = PreflightSuggestions::default();
        suggestions.suggested_tags = vec!["rust".to_string(), "programming".to_string()];

        assert_eq!(suggestions.suggested_tags.len(), 2);
        assert!(suggestions.suggested_tags.contains(&"rust".to_string()));
    }
}