//! Document Synthesis Integration Tests
//!
//! Tests the complete workflow: Notes → Structure → Synthesis → Document
//!
//! **Note:** Some tests require `TocGenerator::generate()` to be implemented.
//! These are marked with `#[ignore]` and will pass once the TOC generation is complete.

use pkm_ai::db::Database;
use pkm_ai::models::{Block, BlockType, Edge, LinkType};
use pkm_ai::models::FractionalIndex;
use pkm_ai::synthesis::{OutputFormat, Synthesizer};
use tempfile::TempDir;

mod common {
    use super::*;

    /// Creates a fresh database for each test
    pub async fn create_test_db() -> anyhow::Result<(TempDir, Database)> {
        let temp_dir = TempDir::new()?;
        let db = Database::rocksdb(temp_dir.path()).await?;
        Ok((temp_dir, db))
    }
}

// ============================================================================
// Zettelkasten Workflow Tests (Working)
// ============================================================================

#[tokio::test]
async fn test_zettelkasten_note_lifecycle() -> anyhow::Result<()> {
    let (_temp_dir, db) = common::create_test_db().await?;

    // Step 1: Create a fleeting note (quick capture)
    let fleeting = Block::fleeting("Quick thought about entropy");
    db.blocks().create(fleeting.clone()).await?;
    assert_eq!(fleeting.block_type, BlockType::Fleeting);

    // Step 2: Create a literature note (from reading)
    let literature = Block::new(BlockType::Literature, "Shannon's Paper");
    db.blocks().create(literature.clone()).await?;
    assert_eq!(literature.block_type, BlockType::Literature);

    // Step 3: Create permanent notes (crystallized knowledge)
    let entropy = Block::permanent("Entropy", "Entropy measures uncertainty");
    let info_theory = Block::permanent("Information Theory", "Mathematical theory of communication");

    db.blocks().create(entropy.clone()).await?;
    db.blocks().create(info_theory.clone()).await?;

    // Step 4: Create links (Zettelkasten connections)
    let edge1 = Edge::new(literature.id, entropy.id, LinkType::Supports);
    db.edges().create(edge1.clone()).await?;

    let edge2 = Edge::new(entropy.id, info_theory.id, LinkType::Extends);
    db.edges().create(edge2.clone()).await?;

    // Verify links
    let entropy_outgoing = db.edges().outgoing_from(&entropy.id).await?;
    assert_eq!(entropy_outgoing.len(), 1);
    assert_eq!(entropy_outgoing[0].to, info_theory.id);

    let entropy_incoming = db.edges().incoming_to(&entropy.id).await?;
    assert_eq!(entropy_incoming.len(), 1);
    assert_eq!(entropy_incoming[0].from, literature.id);

    // Verify all blocks exist
    let all_blocks = db.blocks().list_all().await?;
    assert_eq!(all_blocks.len(), 4);

    Ok(())
}

#[tokio::test]
async fn test_structure_workflow() -> anyhow::Result<()> {
    let (_temp_dir, db) = common::create_test_db().await?;

    // Create a structure note (MOC - Map of Content)
    let moc = Block::structure("Research Notes Index");
    db.blocks().create(moc.clone()).await?;
    assert_eq!(moc.block_type, BlockType::Structure);

    // Create section notes
    let intro = Block::outline("Introduction").with_content("This is the introduction...");
    let methods = Block::outline("Methods").with_content("These are the research methods...");
    let results = Block::outline("Results").with_content("These are the findings...");
    let conclusion = Block::outline("Conclusion").with_content("This concludes our research...");

    db.blocks().create(intro.clone()).await?;
    db.blocks().create(methods.clone()).await?;
    db.blocks().create(results.clone()).await?;
    db.blocks().create(conclusion.clone()).await?;

    // Link sections to structure using SectionOf links with fractional indexing
    let edge1 = Edge::section_of_at(intro.id, moc.id, FractionalIndex::first());
    let edge2 = Edge::section_of_at(methods.id, moc.id, FractionalIndex::after_last(&edge1.sequence_weight));
    let edge3 = Edge::section_of_at(results.id, moc.id, FractionalIndex::after_last(&edge2.sequence_weight));
    let edge4 = Edge::section_of_at(conclusion.id, moc.id, FractionalIndex::after_last(&edge3.sequence_weight));

    db.edges().create(edge1.clone()).await?;
    db.edges().create(edge2.clone()).await?;
    db.edges().create(edge3.clone()).await?;
    db.edges().create(edge4.clone()).await?;

    // Verify structure has 4 sections (structure is dst, so query incoming_to)
    let sections: Vec<Edge> = db.edges().incoming_to(&moc.id).await?;
    assert_eq!(sections.len(), 4);

    // Verify all sections are synthesis links
    for section_edge in &sections {
        assert!(section_edge.is_synthesis_link(), "All section links should be synthesis links");
    }

    // Verify first section is Introduction
    let first_section: Vec<Edge> = db.edges().incoming_to(&moc.id).await?
        .into_iter()
        .filter(|e| e.from == intro.id)
        .collect();
    assert_eq!(first_section.len(), 1);

    Ok(())
}

// ============================================================================
// Synthesizer Creation Test (Working)
// ============================================================================

#[tokio::test]
async fn test_synthesizer_creation() -> anyhow::Result<()> {
    let (_temp_dir, db) = common::create_test_db().await?;

    let synthesizer = Synthesizer::new(&db);

    // Verify synthesizer was created (just check it's not null)
    let _ = synthesizer;

    Ok(())
}

// ============================================================================
// Tests Requiring TocGenerator Implementation (Marked as ignore)
// ============================================================================

/// Tests that require TocGenerator::generate() to be implemented
///
/// TODO: Implement TocGenerator::generate() to query sections from DB
#[tokio::test]
async fn test_synthesize_empty_structure() -> anyhow::Result<()> {
    let (_temp_dir, db) = common::create_test_db().await?;

    let empty_moc = Block::structure("Empty Document");
    db.blocks().create(empty_moc.clone()).await?;

    let synthesizer = Synthesizer::new(&db);

    let result = synthesizer.synthesize(&empty_moc.id, OutputFormat::Markdown, None).await?;

    assert_eq!(result.format, OutputFormat::Markdown);
    assert_eq!(result.blocks_used, 0);
    assert_eq!(result.blocks_total, 0);
    assert_eq!(result.title, "Empty Document");
    assert!(result.toc.sections.is_empty());

    Ok(())
}

/// TODO: Implement TocGenerator::generate() to query sections from DB
#[tokio::test]
async fn test_synthesize_with_sections() -> anyhow::Result<()> {
    let (_temp_dir, db) = common::create_test_db().await?;

    let moc = Block::structure("My Research Paper");
    db.blocks().create(moc.clone()).await?;

    let intro = Block::outline("Introduction").with_content("Introduction content");
    let methods = Block::outline("Methods").with_content("Methods content");
    let conclusion = Block::outline("Conclusion").with_content("Conclusion content");

    db.blocks().create(intro.clone()).await?;
    db.blocks().create(methods.clone()).await?;
    db.blocks().create(conclusion.clone()).await?;

    let edge1 = Edge::section_of_at(intro.id, moc.id, FractionalIndex::first());
    let edge2 = Edge::section_of_at(methods.id, moc.id, FractionalIndex::after_last(&edge1.sequence_weight));
    let edge3 = Edge::section_of_at(conclusion.id, moc.id, FractionalIndex::after_last(&edge2.sequence_weight));

    db.edges().create(edge1).await?;
    db.edges().create(edge2).await?;
    db.edges().create(edge3).await?;

    let synthesizer = Synthesizer::new(&db);
    let result = synthesizer.synthesize(&moc.id, OutputFormat::Markdown, None).await?;

    assert_eq!(result.blocks_used, 3);
    assert_eq!(result.toc.sections.len(), 3);
    assert_eq!(result.toc.sections[0].title, "Introduction");
    assert_eq!(result.toc.sections[1].title, "Methods");
    assert_eq!(result.toc.sections[2].title, "Conclusion");

    Ok(())
}

/// TODO: Implement TocGenerator::generate()
#[tokio::test]
async fn test_synthesize_html_format() -> anyhow::Result<()> {
    let (_temp_dir, db) = common::create_test_db().await?;

    let moc = Block::structure("HTML Report");
    db.blocks().create(moc.clone()).await?;

    let section = Block::outline("Summary").with_content("Key findings.");
    db.blocks().create(section.clone()).await?;

    let edge = Edge::section_of_at(section.id, moc.id, FractionalIndex::first());
    db.edges().create(edge).await?;

    let synthesizer = Synthesizer::new(&db);
    let result = synthesizer.synthesize(&moc.id, OutputFormat::Html, None).await?;

    assert_eq!(result.format, OutputFormat::Html);
    assert_eq!(result.blocks_used, 1);

    Ok(())
}

/// TODO: Implement TocGenerator::generate()
#[tokio::test]
async fn test_synthesize_preserves_section_order() -> anyhow::Result<()> {
    let (_temp_dir, db) = common::create_test_db().await?;

    let moc = Block::structure("Ordered Document");
    db.blocks().create(moc.clone()).await?;

    let d = Block::outline("D").with_content("Content D");
    let b = Block::outline("B").with_content("Content B");
    let a = Block::outline("A").with_content("Content A");
    let c = Block::outline("C").with_content("Content C");

    db.blocks().create(d.clone()).await?;
    db.blocks().create(b.clone()).await?;
    db.blocks().create(a.clone()).await?;
    db.blocks().create(c.clone()).await?;

    let edge_a = Edge::section_of_at(a.id, moc.id, FractionalIndex::first());
    let edge_b = Edge::section_of_at(b.id, moc.id, FractionalIndex::after_last(&edge_a.sequence_weight));
    let edge_c = Edge::section_of_at(c.id, moc.id, FractionalIndex::after_last(&edge_b.sequence_weight));
    let edge_d = Edge::section_of_at(d.id, moc.id, FractionalIndex::after_last(&edge_c.sequence_weight));

    db.edges().create(edge_a).await?;
    db.edges().create(edge_b).await?;
    db.edges().create(edge_c).await?;
    db.edges().create(edge_d).await?;

    let synthesizer = Synthesizer::new(&db);
    let result = synthesizer.synthesize(&moc.id, OutputFormat::Markdown, None).await?;

    assert_eq!(result.toc.sections[0].title, "A");
    assert_eq!(result.toc.sections[1].title, "B");
    assert_eq!(result.toc.sections[2].title, "C");
    assert_eq!(result.toc.sections[3].title, "D");

    Ok(())
}

/// TODO: Implement TocGenerator::generate()
#[tokio::test]
async fn test_toc_generation() -> anyhow::Result<()> {
    let (_temp_dir, db) = common::create_test_db().await?;

    let moc = Block::structure("TOC Test Document");
    db.blocks().create(moc.clone()).await?;

    let sec1 = Block::outline("Section 1").with_content("Content 1");
    let sec2 = Block::outline("Section 2").with_content("Content 2");
    let sec3 = Block::outline("Section 3").with_content("Content 3");

    db.blocks().create(sec1.clone()).await?;
    db.blocks().create(sec2.clone()).await?;
    db.blocks().create(sec3.clone()).await?;

    let e1 = Edge::section_of_at(sec1.id, moc.id, FractionalIndex::first());
    let e2 = Edge::section_of_at(sec2.id, moc.id, FractionalIndex::after_last(&e1.sequence_weight));
    let e3 = Edge::section_of_at(sec3.id, moc.id, FractionalIndex::after_last(&e2.sequence_weight));

    db.edges().create(e1).await?;
    db.edges().create(e2).await?;
    db.edges().create(e3).await?;

    let synthesizer = Synthesizer::new(&db);
    let toc = synthesizer.generate_toc(&moc.id).await?;

    assert_eq!(toc.structure_id, moc.id);
    assert_eq!(toc.sections.len(), 3);

    Ok(())
}

// ============================================================================
// Full Document Workflow Test
// ============================================================================

/// Full end-to-end test: Notes → Structure → Synthesis
///
/// Full end-to-end test: Notes → Structure → Synthesis
#[tokio::test]
async fn test_full_document_workflow() -> anyhow::Result<()> {
    let (_temp_dir, db) = common::create_test_db().await?;

    // ========================================================================
    // PHASE 1: Create Zettelkasten Notes
    // ========================================================================

    let shannon = Block::new(BlockType::Literature, "Shannon's 1948 Paper")
        .with_content("A Mathematical Theory of Communication");
    let thermal = Block::new(BlockType::Literature, "Thermodynamics Paper")
        .with_content("Statistical mechanics and entropy");

    db.blocks().create(shannon.clone()).await?;
    db.blocks().create(thermal.clone()).await?;

    let entropy_note = Block::permanent("Entropy Definition", "Entropy measures uncertainty");
    let info_note = Block::permanent("Information Content", "Information quantifies events");
    let max_ent = Block::permanent("Maximum Entropy Principle", "Maximum entropy distribution");

    db.blocks().create(entropy_note.clone()).await?;
    db.blocks().create(info_note.clone()).await?;
    db.blocks().create(max_ent.clone()).await?;

    // Create Zettelkasten links
    db.edges().create(Edge::new(shannon.id, entropy_note.id, LinkType::Supports)).await?;
    db.edges().create(Edge::new(thermal.id, max_ent.id, LinkType::Supports)).await?;
    db.edges().create(Edge::new(entropy_note.id, info_note.id, LinkType::Extends)).await?;
    db.edges().create(Edge::new(max_ent.id, info_note.id, LinkType::Related)).await?;

    // ========================================================================
    // PHASE 2: Create Document Structure
    // ========================================================================

    let doc = Block::structure("Information Theory Overview");
    db.blocks().create(doc.clone()).await?;

    let sec_intro = Block::outline("Introduction").with_content("Overview of information theory.");
    let sec_entropy = Block::outline("Entropy").with_content(entropy_note.content.as_str());
    let sec_info = Block::outline("Information").with_content(info_note.content.as_str());
    let sec_maxent = Block::outline("Maximum Entropy").with_content(max_ent.content.as_str());
    let sec_conclusion = Block::outline("Conclusion").with_content("Foundation of modern theory.");

    db.blocks().create(sec_intro.clone()).await?;
    db.blocks().create(sec_entropy.clone()).await?;
    db.blocks().create(sec_info.clone()).await?;
    db.blocks().create(sec_maxent.clone()).await?;
    db.blocks().create(sec_conclusion.clone()).await?;

    let i1 = Edge::section_of_at(sec_intro.id, doc.id, FractionalIndex::first());
    let i2 = Edge::section_of_at(sec_entropy.id, doc.id, FractionalIndex::after_last(&i1.sequence_weight));
    let i3 = Edge::section_of_at(sec_info.id, doc.id, FractionalIndex::after_last(&i2.sequence_weight));
    let i4 = Edge::section_of_at(sec_maxent.id, doc.id, FractionalIndex::after_last(&i3.sequence_weight));
    let i5 = Edge::section_of_at(sec_conclusion.id, doc.id, FractionalIndex::after_last(&i4.sequence_weight));

    db.edges().create(i1).await?;
    db.edges().create(i2).await?;
    db.edges().create(i3).await?;
    db.edges().create(i4).await?;
    db.edges().create(i5).await?;

    // ========================================================================
    // PHASE 3: Synthesize Document
    // ========================================================================

    let synthesizer = Synthesizer::new(&db);
    let result = synthesizer.synthesize(&doc.id, OutputFormat::Markdown, None).await?;

    // ========================================================================
    // VERIFICATION
    // ========================================================================

    assert_eq!(result.title, "Information Theory Overview");
    assert_eq!(result.blocks_used, 5);
    assert_eq!(result.toc.sections.len(), 5);
    assert_eq!(result.toc.sections[0].title, "Introduction");
    assert_eq!(result.toc.sections[1].title, "Entropy");
    assert_eq!(result.toc.sections[2].title, "Information");
    assert_eq!(result.toc.sections[3].title, "Maximum Entropy");
    assert_eq!(result.toc.sections[4].title, "Conclusion");

    let content = String::from_utf8_lossy(&result.content);
    assert!(!content.is_empty());
    assert!(content.len() > 100);

    // Verify Zettelkasten links still intact
    let all_edges = db.edges().list_all().await?;
    assert_eq!(all_edges.len(), 9);

    let all_blocks = db.blocks().list_all().await?;
    assert_eq!(all_blocks.len(), 11);

    Ok(())
}

// ============================================================================
// Edge Cases (Working)
// ============================================================================

#[tokio::test]
async fn test_synthesize_nonexistent_structure() -> anyhow::Result<()> {
    let (_temp_dir, db) = common::create_test_db().await?;

    let synthesizer = Synthesizer::new(&db);
    let fake_id = ulid::Ulid::new();

    // Should fail gracefully or return empty result
    let result = synthesizer.synthesize(&fake_id, OutputFormat::Markdown, None).await;

    match result {
        Ok(r) => {
            assert_eq!(r.blocks_used, 0);
        }
        Err(_) => {
            // Error is acceptable - structure doesn't exist
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_synthesis_with_special_characters() -> anyhow::Result<()> {
    let (_temp_dir, db) = common::create_test_db().await?;

    let moc = Block::structure("Special Characters Test");
    db.blocks().create(moc.clone()).await?;

    let section = Block::outline("Code & Math")
        .with_content("Testing: <html> & \"quotes\" and $variables");
    db.blocks().create(section.clone()).await?;

    let edge = Edge::section_of_at(section.id, moc.id, FractionalIndex::first());
    db.edges().create(edge).await?;

    // Verify edge was created with special characters in content
    let retrieved = db.blocks().get(&section.id).await?;
    assert!(retrieved.is_some());
    assert!(retrieved.unwrap().content.contains("quotes"));

    Ok(())
}

#[tokio::test]
async fn test_fractional_index_ordering() -> anyhow::Result<()> {
    let (_temp_dir, db) = common::create_test_db().await?;

    let moc = Block::structure("Ordering Test");
    db.blocks().create(moc.clone()).await?;

    // Create items with explicit fractional indexing
    let first = Block::outline("First").with_content("1");
    let second = Block::outline("Second").with_content("2");
    let third = Block::outline("Third").with_content("3");

    db.blocks().create(first.clone()).await?;
    db.blocks().create(second.clone()).await?;
    db.blocks().create(third.clone()).await?;

    let e1 = Edge::section_of_at(first.id, moc.id, FractionalIndex::first());
    let e2 = Edge::section_of_at(second.id, moc.id, FractionalIndex::after_last(&e1.sequence_weight));
    let e3 = Edge::section_of_at(third.id, moc.id, FractionalIndex::after_last(&e2.sequence_weight));

    db.edges().create(e1.clone()).await?;
    db.edges().create(e2.clone()).await?;
    db.edges().create(e3.clone()).await?;

    // Verify order by querying and sorting
    let sections: Vec<Edge> = db.edges().incoming_to(&moc.id).await?;
    let mut sorted = sections.clone();
    sorted.sort_by(|a, b| a.sequence_weight.cmp(&b.sequence_weight));

    // Verify sequence weights are in increasing order
    assert!(e1.sequence_weight < e2.sequence_weight);
    assert!(e2.sequence_weight < e3.sequence_weight);

    // Verify sorted order matches block titles
    assert_eq!(sorted[0].from, first.id);
    assert_eq!(sorted[1].from, second.id);
    assert_eq!(sorted[2].from, third.id);

    Ok(())
}

// ============================================================================
// SPEC Structure Test
// ============================================================================

#[tokio::test]
async fn test_spec_es_document_synthesis() -> anyhow::Result<()> {
    let (_temp_dir, db) = common::create_test_db().await?;

    // Create the main SPEC_ES structure (MOC)
    let spec_moc = Block::structure("SPEC_ES - Especificacion Tecnica PKM-AI");
    db.blocks().create(spec_moc.clone()).await?;

    // 1. Vision y Principios
    let s1 = Block::outline("1.1 Vision Central")
        .with_content("PKM-AI es un Sistema Operativo de Conocimiento para equipos que trabajan con enjambres de agentes de IA.");
    db.blocks().create(s1.clone()).await?;
    let e1 = Edge::section_of_at(s1.id, spec_moc.id, FractionalIndex::first());
    db.edges().create(e1.clone()).await?;

    let s2 = Block::outline("1.2 Principios Fundamentales")
        .with_content("Principios: Modelo de Bloques-Atomo, Columna Vertebral Estructural Primero, Separacion Semantica/Estructural.");
    db.blocks().create(s2.clone()).await?;
    let e2 = Edge::section_of_at(s2.id, spec_moc.id, FractionalIndex::after_last(&e1.sequence_weight));
    db.edges().create(e2.clone()).await?;

    let s3 = Block::outline("1.3 Relacion entre Proyectos")
        .with_content("PKM-AI y Nexus-WASM son proyectos hermanos bajo hodei-pkm.");
    db.blocks().create(s3.clone()).await?;
    let e3 = Edge::section_of_at(s3.id, spec_moc.id, FractionalIndex::after_last(&e2.sequence_weight));
    db.edges().create(e3.clone()).await?;

    // 2. Modelos de Dominio
    let s4 = Block::outline("2.1 Bloque")
        .with_content("Bloque es la unidad fundamental de informacion con id (ULID), block_type, content, properties, embedding_bloom.");
    db.blocks().create(s4.clone()).await?;
    let e4 = Edge::section_of_at(s4.id, spec_moc.id, FractionalIndex::after_last(&e3.sequence_weight));
    db.edges().create(e4.clone()).await?;

    let s5 = Block::outline("2.2 Arista")
        .with_content("Arista representa enlaces entre bloques con id, link_type, from, to, properties, sequence_weight.");
    db.blocks().create(s5.clone()).await?;
    let e5 = Edge::section_of_at(s5.id, spec_moc.id, FractionalIndex::after_last(&e4.sequence_weight));
    db.edges().create(e5.clone()).await?;

    let s6 = Block::outline("2.3 FractionalIndex")
        .with_content("CRITICO: NO usar f32. Usar indexacion fraccionaria lexicografica para evitar degradacion de precision.");
    db.blocks().create(s6.clone()).await?;
    let _e6 = Edge::section_of_at(s6.id, spec_moc.id, FractionalIndex::after_last(&e5.sequence_weight));
    db.edges().create(_e6.clone()).await?;

    // Synthesize the document
    let synthesizer = Synthesizer::new(&db);
    let result = synthesizer.synthesize(&spec_moc.id, OutputFormat::Markdown, None).await?;

    // Verify
    assert_eq!(result.title, "SPEC_ES - Especificacion Tecnica PKM-AI");
    assert_eq!(result.blocks_used, 6);
    assert_eq!(result.toc.sections.len(), 6);

    // Verify section order
    assert_eq!(result.toc.sections[0].title, "1.1 Vision Central");
    assert_eq!(result.toc.sections[1].title, "1.2 Principios Fundamentales");
    assert_eq!(result.toc.sections[2].title, "1.3 Relacion entre Proyectos");
    assert_eq!(result.toc.sections[3].title, "2.1 Bloque");
    assert_eq!(result.toc.sections[4].title, "2.2 Arista");
    assert_eq!(result.toc.sections[5].title, "2.3 FractionalIndex");

    // Verify content is not empty
    let content = String::from_utf8_lossy(&result.content);
    assert!(!content.is_empty());
    assert!(content.contains("PKM-AI"), "Content: {}", content);
    assert!(content.contains("Bloques-Atomo") || content.contains("Principios"), "Content: {}", content);

    println!("\n=== Synthesized SPEC_ES Document ===");
    println!("Title: {}", result.title);
    println!("Sections: {}", result.toc.sections.len());
    println!("Content length: {} chars", content.len());
    println!("\nTOC:");
    for (i, section) in result.toc.sections.iter().enumerate() {
        println!("  {}. {}", i + 1, section.title);
    }
    println!("\n--- Content Preview (first 500 chars) ---");
    println!("{}", &content[..content.len().min(500)]);

    Ok(())
}
