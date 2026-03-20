//! Promote command: Promote blocks to a higher order type
//!
//! Promotes a block to a more permanent state in the Zettelkasten hierarchy.
//!
//! Valid transitions:
//! - fleeting → literature, permanent, ghost
//! - literature → permanent, ghost
//! - ghost → permanent (fill)
//! - permanent → (no promotion needed)

use crate::cli::commands::create::parse_block_type;
use crate::db::Database;
use crate::models::BlockType;
use crate::NexusError;

/// Execute promote command
pub async fn execute(
    db: &Database,
    id: &str,
    block_type: &str,
    stage: bool,
) -> anyhow::Result<()> {
    let block_id = ulid::Ulid::from_string(id)
        .map_err(|_| NexusError::BlockNotFound(id.to_string()))?;

    let target_type = parse_block_type(block_type)?;

    // Get the block
    let repo = db.blocks();
    let block = repo.get(&block_id).await?
        .ok_or_else(|| NexusError::BlockNotFound(id.to_string()))?;

    // Validate transition
    validate_transition(&block.block_type, &target_type)?;

    let old_type = block.block_type.clone();

    // Promote the block
    let mut promoted_block = block.clone();
    promoted_block.block_type = target_type.clone();
    promoted_block.updated_at = chrono::Utc::now();

    // If promoting ghost to permanent, mark it as filled
    if old_type == BlockType::Ghost && target_type == BlockType::Permanent {
        promoted_block.metadata.insert("filled".to_string(), serde_json::json!(true));
        promoted_block.metadata.insert("ghost_filled_at".to_string(), serde_json::json!(chrono::Utc::now().to_rfc3339()));
    }

    repo.update(promoted_block.clone()).await?;

    // Print result
    println!("✅ Block promoted!");
    println!();
    println!("   ID: {}", block.id_str());
    println!("   Title: {}", block.title);
    println!("   {} → {:?}", format!("{:?}", old_type).to_lowercase(), target_type);

    if stage {
        println!();
        println!("   📦 Block staged for commit");
    }

    println!();
    println!("💡 Run `nexus show {}` to view the promoted block", id);

    Ok(())
}

/// Validate that a transition is allowed
pub fn validate_transition(from: &BlockType, to: &BlockType) -> anyhow::Result<()> {
    // Same type - no promotion needed
    if from == to {
        anyhow::bail!("Block is already {:?}", from);
    }

    match (from, to) {
        // Fleeting can go to literature, permanent, or ghost
        (BlockType::Fleeting, BlockType::Literature) => Ok(()),
        (BlockType::Fleeting, BlockType::Permanent) => Ok(()),
        (BlockType::Fleeting, BlockType::Ghost) => Ok(()),

        // Literature can go to permanent or ghost
        (BlockType::Literature, BlockType::Permanent) => Ok(()),
        (BlockType::Literature, BlockType::Ghost) => Ok(()),

        // Ghost can go to permanent (fill operation)
        (BlockType::Ghost, BlockType::Permanent) => Ok(()),

        // Structure, Hub, Task, Reference, Outline - can go to Permanent
        // (these are already high-order types)
        (BlockType::Structure, BlockType::Permanent) => Ok(()),
        (BlockType::Hub, BlockType::Permanent) => Ok(()),
        (BlockType::Task, BlockType::Permanent) => Ok(()),
        (BlockType::Reference, BlockType::Permanent) => Ok(()),
        (BlockType::Outline, BlockType::Permanent) => Ok(()),

        // Permanent cannot be promoted further
        (BlockType::Permanent, _) => {
            anyhow::bail!("Block is already permanent - no promotion needed")
        }

        // Any other transition is invalid
        _ => {
            anyhow::bail!(
                "Invalid promotion: {:?} → {:?} is not allowed",
                from,
                to
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transition_fleeting_to_literature() {
        assert!(validate_transition(&BlockType::Fleeting, &BlockType::Literature).is_ok());
    }

    #[test]
    fn test_valid_transition_fleeting_to_permanent() {
        assert!(validate_transition(&BlockType::Fleeting, &BlockType::Permanent).is_ok());
    }

    #[test]
    fn test_valid_transition_fleeting_to_ghost() {
        assert!(validate_transition(&BlockType::Fleeting, &BlockType::Ghost).is_ok());
    }

    #[test]
    fn test_valid_transition_literature_to_permanent() {
        assert!(validate_transition(&BlockType::Literature, &BlockType::Permanent).is_ok());
    }

    #[test]
    fn test_valid_transition_literature_to_ghost() {
        assert!(validate_transition(&BlockType::Literature, &BlockType::Ghost).is_ok());
    }

    #[test]
    fn test_valid_transition_ghost_to_permanent() {
        assert!(validate_transition(&BlockType::Ghost, &BlockType::Permanent).is_ok());
    }

    #[test]
    fn test_valid_transition_structure_to_permanent() {
        assert!(validate_transition(&BlockType::Structure, &BlockType::Permanent).is_ok());
    }

    #[test]
    fn test_valid_transition_hub_to_permanent() {
        assert!(validate_transition(&BlockType::Hub, &BlockType::Permanent).is_ok());
    }

    #[test]
    fn test_valid_transition_task_to_permanent() {
        assert!(validate_transition(&BlockType::Task, &BlockType::Permanent).is_ok());
    }

    #[test]
    fn test_valid_transition_reference_to_permanent() {
        assert!(validate_transition(&BlockType::Reference, &BlockType::Permanent).is_ok());
    }

    #[test]
    fn test_valid_transition_outline_to_permanent() {
        assert!(validate_transition(&BlockType::Outline, &BlockType::Permanent).is_ok());
    }

    #[test]
    fn test_invalid_transition_same_type() {
        let result = validate_transition(&BlockType::Fleeting, &BlockType::Fleeting);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already"));
    }

    #[test]
    fn test_invalid_transition_permanent_to_anything() {
        let result = validate_transition(&BlockType::Permanent, &BlockType::Literature);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already permanent"));
    }

    #[test]
    fn test_invalid_transition_literature_to_fleeting() {
        let result = validate_transition(&BlockType::Literature, &BlockType::Fleeting);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_transition_permanent_to_literature() {
        let result = validate_transition(&BlockType::Permanent, &BlockType::Literature);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_transition_ghost_to_literature() {
        let result = validate_transition(&BlockType::Ghost, &BlockType::Literature);
        assert!(result.is_err());
    }
}