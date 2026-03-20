//! Show command

use crate::db::Database;
use crate::NexusError;

pub async fn execute(
    db: &Database,
    id: &str,
    related: bool,
) -> anyhow::Result<()> {
    let block_id = ulid::Ulid::from_string(id)
        .map_err(|_| NexusError::BlockNotFound(id.to_string()))?;

    let repo = db.blocks();
    let block = repo.get(&block_id).await?
        .ok_or_else(|| NexusError::BlockNotFound(id.to_string()))?;

    println!("╔════════════════════════════════════════════════════════╗");
    println!("║ Block: {:<46} ║", block.title);
    println!("╠════════════════════════════════════════════════════════╣");
    println!("║ Type:    {:<44} ║", format!("{:?}", block.block_type).to_lowercase());
    println!("║ ULID:    {:<44} ║", block.id_str());
    println!("║ Created: {:<44} ║", block.created_at.format("%Y-%m-%d %H:%M"));
    println!("║ Updated: {:<44} ║", block.updated_at.format("%Y-%m-%d %H:%M"));
    println!("║ Version: {:<44} ║", block.version);
    println!("╠════════════════════════════════════════════════════════╣");

    if !block.tags.is_empty() {
        println!("║ Tags:    {:<44} ║", block.tags.join(", "));
    }

    if let Some(confidence) = block.ai_confidence {
        println!("║ AI Confidence: {:<37} ║", format!("{:.2}%", confidence * 100.0));
    }

    println!("╠════════════════════════════════════════════════════════╣");
    println!("║ Content:                                               ║");
    println!("╚════════════════════════════════════════════════════════╝");
    println!();
    println!("{}", block.content);

    if related {
        println!("\n📤 Outgoing Links:");
        let edges = db.edges();
        let outgoing = edges.outgoing_from(&block_id).await?;
        for edge in outgoing {
            println!("  {} → {:?} (weight: {})", edge.id_str(), edge.link_type, edge.sequence_weight);
        }

        println!("\n📥 Incoming Links:");
        let incoming = edges.incoming_to(&block_id).await?;
        for edge in incoming {
            println!("  {} ← {:?} (weight: {})", edge.id_str(), edge.link_type, edge.sequence_weight);
        }
    }

    Ok(())
}
