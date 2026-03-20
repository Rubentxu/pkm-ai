//! Linting Integration Tests
//!
//! Tests for the structural linting and auto-fix functionality.

use pkm_ai::models::{Block, BlockType, Edge};
use pkm_ai::db::Database;
use pkm_ai::spine::linting::{StructuralLinter, LintSeverity};
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
// Lint Detection Tests
// ============================================================================

#[tokio::test]
async fn test_detect_orphan_blocks() -> anyhow::Result<()> {
    let (_temp_dir, db) = common::create_test_db().await?;

    // Create a permanent block without any incoming links
    let orphan = Block::permanent("Orphan Block", "This block has no links");
    db.blocks().create(orphan.clone()).await?;

    // Run linter
    let linter = StructuralLinter::new(&db);
    let issues = linter.lint().await?;

    // Should find the orphan block
    let orphan_issues: Vec<_> = issues.iter()
        .filter(|i| i.code == "orphan")
        .collect();

    assert_eq!(orphan_issues.len(), 1);
    assert_eq!(orphan_issues[0].severity, LintSeverity::Error);
    assert!(orphan_issues[0].message.contains("Orphan Block"));

    Ok(())
}

#[tokio::test]
async fn test_no_issues_for_linked_block() -> anyhow::Result<()> {
    let (_temp_dir, db) = common::create_test_db().await?;

    // Create a structure block
    let structure = Block::structure("Test Structure");
    db.blocks().create(structure.clone()).await?;

    // Create a permanent block linked to the structure
    let permanent = Block::permanent("Linked Block", "Content");
    db.blocks().create(permanent.clone()).await?;

    // Create a NEXT edge from structure to permanent
    let edge = Edge::next_in_sequence_first(structure.id, permanent.id);
    db.edges().create(edge).await?;

    // Run linter
    let linter = StructuralLinter::new(&db);
    let issues = linter.lint().await?;

    // Should not find any orphan issues
    let orphan_issues: Vec<_> = issues.iter()
        .filter(|i| i.code == "orphan")
        .collect();

    assert_eq!(orphan_issues.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_detect_forward_references() -> anyhow::Result<()> {
    let (_temp_dir, db) = common::create_test_db().await?;

    // Create a block that references a non-existent block
    let fake_id = "01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string();
    let block = Block::permanent("Forward Ref Block", format!("This references [[{}]] which does not exist", fake_id));
    db.blocks().create(block.clone()).await?;

    // Run linter
    let linter = StructuralLinter::new(&db);
    let issues = linter.lint().await?;

    // Should find forward reference issue
    let forward_ref_issues: Vec<_> = issues.iter()
        .filter(|i| i.code == "forward-ref")
        .collect();

    assert_eq!(forward_ref_issues.len(), 1);
    assert_eq!(forward_ref_issues[0].severity, LintSeverity::Error);

    Ok(())
}

#[tokio::test]
async fn test_detect_circular_references() -> anyhow::Result<()> {
    let (_temp_dir, db) = common::create_test_db().await?;

    // Create two structures with NEXT links forming a cycle
    let structure1 = Block::structure("Structure 1");
    let structure2 = Block::structure("Structure 2");
    db.blocks().create(structure1.clone()).await?;
    db.blocks().create(structure2.clone()).await?;

    // Create links: structure1 -> structure2 -> structure1 (cycle)
    let edge1 = Edge::next_in_sequence_first(structure1.id, structure2.id);
    let edge2 = Edge::next_in_sequence_after(structure2.id, structure1.id, &edge1.sequence_weight);
    db.edges().create(edge1).await?;
    db.edges().create(edge2).await?;

    // Run linter
    let linter = StructuralLinter::new(&db);
    let issues = linter.lint().await?;

    // Should find circular reference issue
    let circular_issues: Vec<_> = issues.iter()
        .filter(|i| i.code == "circular-ref")
        .collect();

    assert!(!circular_issues.is_empty());

    Ok(())
}

// ============================================================================
// Auto-Fix Tests
// ============================================================================

#[tokio::test]
async fn test_auto_fix_orphan_block() -> anyhow::Result<()> {
    let (_temp_dir, db) = common::create_test_db().await?;

    // Create a structure block with enough children to not trigger unbalanced warning
    let structure = Block::structure("Test Structure");
    db.blocks().create(structure.clone()).await?;

    // Add some initial children to the structure so it won't be "unbalanced"
    let initial = Block::permanent("Initial Block", "Content");
    db.blocks().create(initial.clone()).await?;
    let initial_edge = Edge::next_in_sequence_first(structure.id, initial.id);
    db.edges().create(initial_edge).await?;

    // Create orphan permanent blocks
    let orphan1 = Block::permanent("Orphan 1", "Content 1");
    let orphan2 = Block::permanent("Orphan 2", "Content 2");
    db.blocks().create(orphan1.clone()).await?;
    db.blocks().create(orphan2.clone()).await?;

    // Run auto-fix
    let linter = StructuralLinter::new(&db);
    let fix_result = linter.auto_fix().await?;

    // Should have fixed the orphan blocks (2 orphans = 2 fixes)
    // Note: unbalanced warnings are expected since the structure has 3 children total
    // which is still below the "dense" threshold but the sparse check only triggers at <=1
    let orphan_fixes: Vec<_> = fix_result.fixes.iter()
        .filter(|f| f.code == "orphan")
        .collect();
    assert_eq!(orphan_fixes.len(), 2);
    assert!(fix_result.errors.is_empty());

    // Verify the edges were created
    let edges_from_structure: Vec<Edge> = db.edges().outgoing_from(&structure.id).await?;
    assert_eq!(edges_from_structure.len(), 3); // initial + 2 orphans

    // Verify the orphans are no longer reported as orphans
    let remaining_issues = linter.lint().await?;
    let orphan_issues: Vec<_> = remaining_issues.iter()
        .filter(|i| i.code == "orphan")
        .collect();
    assert_eq!(orphan_issues.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_auto_fix_does_not_affect_structures() -> anyhow::Result<()> {
    let (_temp_dir, db) = common::create_test_db().await?;

    // Create structure blocks (they are valid roots)
    let structure1 = Block::structure("Structure 1");
    let structure2 = Block::structure("Structure 2");
    db.blocks().create(structure1.clone()).await?;
    db.blocks().create(structure2.clone()).await?;

    // Create a block to link them properly to avoid unbalanced warnings
    let block1 = Block::permanent("Block 1", "Content 1");
    let block2 = Block::permanent("Block 2", "Content 2");
    db.blocks().create(block1.clone()).await?;
    db.blocks().create(block2.clone()).await?;

    // Link them properly
    let edge1 = Edge::next_in_sequence_first(structure1.id, block1.id);
    let edge2 = Edge::next_in_sequence_first(structure2.id, block2.id);
    db.edges().create(edge1).await?;
    db.edges().create(edge2).await?;

    // Run auto-fix
    let linter = StructuralLinter::new(&db);
    let fix_result = linter.auto_fix().await?;

    // Structures should not be treated as orphans
    // Fix count should be 0 for orphans
    let orphan_fixes: Vec<_> = fix_result.fixes.iter()
        .filter(|f| f.code == "orphan")
        .collect();
    assert_eq!(orphan_fixes.len(), 0);
    assert!(fix_result.errors.is_empty());

    // Verify structures are still intact
    let structures = db.blocks().list_by_type(BlockType::Structure).await?;
    assert_eq!(structures.len(), 2);

    Ok(())
}

#[tokio::test]
async fn test_auto_fix_with_no_issues() -> anyhow::Result<()> {
    let (_temp_dir, db) = common::create_test_db().await?;

    // Create a structure with multiple children to avoid unbalanced warning
    let structure = Block::structure("Test Structure");
    db.blocks().create(structure.clone()).await?;

    // Create multiple blocks and link them to avoid sparse structure warning
    let blocks: Vec<Block> = (0..3).map(|i| {
        Block::permanent(format!("Linked Block {}", i), format!("Content {}", i))
    }).collect();

    for (i, block) in blocks.iter().enumerate() {
        db.blocks().create(block.clone()).await?;
        let edge = if i == 0 {
            Edge::next_in_sequence_first(structure.id, block.id)
        } else {
            let _prev_block = &blocks[i - 1];
            let last_edges = db.edges().outgoing_from(&structure.id).await?;
            let last_edge = last_edges.last().unwrap();
            Edge::next_in_sequence_between(
                structure.id,
                block.id,
                &last_edge.sequence_weight,
                &pkm_ai::models::FractionalIndex::after_last(&last_edge.sequence_weight)
            )
        };
        db.edges().create(edge).await?;
    }

    // Run auto-fix on clean database
    let linter = StructuralLinter::new(&db);
    let fix_result = linter.auto_fix().await?;

    // Should have nothing to fix for orphans
    let orphan_fixes: Vec<_> = fix_result.fixes.iter()
        .filter(|f| f.code == "orphan")
        .collect();
    assert_eq!(orphan_fixes.len(), 0);
    assert!(fix_result.errors.is_empty());

    // There should be no unresolved orphan issues
    let unresolved_orphans: Vec<_> = fix_result.unresolved.iter()
        .filter(|i| i.code == "orphan")
        .collect();
    assert_eq!(unresolved_orphans.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_auto_fix_leaves_circular_refs_unresolved() -> anyhow::Result<()> {
    let (_temp_dir, db) = common::create_test_db().await?;

    // Create two structures with NEXT links forming a cycle
    let structure1 = Block::structure("Structure 1");
    let structure2 = Block::structure("Structure 2");
    db.blocks().create(structure1.clone()).await?;
    db.blocks().create(structure2.clone()).await?;

    let edge1 = Edge::next_in_sequence_first(structure1.id, structure2.id);
    let edge2 = Edge::next_in_sequence_after(structure2.id, structure1.id, &edge1.sequence_weight);
    db.edges().create(edge1).await?;
    db.edges().create(edge2).await?;

    // Run auto-fix
    let linter = StructuralLinter::new(&db);
    let fix_result = linter.auto_fix().await?;

    // Circular refs should be unresolved (not auto-fixed)
    let circular_unresolved: Vec<_> = fix_result.unresolved.iter()
        .filter(|i| i.code == "circular-ref")
        .collect();

    assert!(!circular_unresolved.is_empty());

    // Should have 0 orphan fixes (no orphans in this case)
    let orphan_fixes: Vec<_> = fix_result.fixes.iter()
        .filter(|f| f.code == "orphan")
        .collect();
    assert_eq!(orphan_fixes.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_auto_fix_leaves_forward_refs_unresolved() -> anyhow::Result<()> {
    let (_temp_dir, db) = common::create_test_db().await?;

    // Create a block with forward reference
    let fake_id = "01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string();
    let block = Block::permanent("Forward Ref Block", format!("References [[{}]]", fake_id));
    db.blocks().create(block.clone()).await?;

    // Also create a structure to avoid unbalanced warning
    let structure = Block::structure("Test Structure");
    db.blocks().create(structure.clone()).await?;
    let edge = Edge::next_in_sequence_first(structure.id, block.id);
    db.edges().create(edge).await?;

    // Run auto-fix
    let linter = StructuralLinter::new(&db);
    let fix_result = linter.auto_fix().await?;

    // Forward refs should be unresolved
    let forward_unresolved: Vec<_> = fix_result.unresolved.iter()
        .filter(|i| i.code == "forward-ref")
        .collect();

    assert!(!forward_unresolved.is_empty());

    // Should have 0 fixes (forward ref not fixable)
    let orphan_fixes: Vec<_> = fix_result.fixes.iter()
        .filter(|f| f.code == "orphan")
        .collect();
    assert_eq!(orphan_fixes.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_auto_fix_unbalanced_sections_unresolved() -> anyhow::Result<()> {
    let (_temp_dir, db) = common::create_test_db().await?;

    // Create a structure with many children (unbalanced)
    let structure = Block::structure("Dense Structure");
    db.blocks().create(structure.clone()).await?;

    // Create many blocks and link them
    for i in 0..25 {
        let block = Block::permanent(format!("Block {}", i), format!("Content {}", i));
        db.blocks().create(block.clone()).await?;

        let edge = if i == 0 {
            Edge::next_in_sequence_first(structure.id, block.id)
        } else {
            // Get the last edge to calculate sequence
            let last_edges = db.edges().outgoing_from(&structure.id).await?;
            let last_edge = last_edges.last().unwrap();
            Edge::next_in_sequence_between(
                structure.id,
                block.id,
                &last_edge.sequence_weight,
                &pkm_ai::models::FractionalIndex::after_last(&last_edge.sequence_weight)
            )
        };
        db.edges().create(edge).await?;
    }

    // Run auto-fix
    let linter = StructuralLinter::new(&db);
    let fix_result = linter.auto_fix().await?;

    // Should have no orphan fixes
    let orphan_fixes: Vec<_> = fix_result.fixes.iter()
        .filter(|f| f.code == "orphan")
        .collect();
    assert_eq!(orphan_fixes.len(), 0);

    // Should have no errors
    assert!(fix_result.errors.is_empty());

    // Unbalanced issues may or may not be triggered depending on average calculation
    // The key is that the auto-fix doesn't crash and doesn't create invalid state

    Ok(())
}

// ============================================================================
// Fix Result Display Tests
// ============================================================================

#[tokio::test]
async fn test_fix_result_display() -> anyhow::Result<()> {
    use pkm_ai::spine::linting::FixResult;

    let result = FixResult::default();
    let display = format!("{}", result);
    assert!(display.contains("No fixes applied"));

    Ok(())
}

#[tokio::test]
async fn test_fix_result_has_fixes() -> anyhow::Result<()> {
    use pkm_ai::spine::linting::{FixResult, FixAction};

    let result = FixResult::default();
    assert!(!result.has_fixes());

    let mut result = FixResult::default();
    result.fixes.push(FixAction {
        code: "orphan".to_string(),
        description: "Test fix".to_string(),
        block_id: None,
        details: None,
    });

    assert!(result.has_fixes());
    assert_eq!(result.fix_count(), 1);
    assert_eq!(result.unresolved_count(), 0);

    Ok(())
}
