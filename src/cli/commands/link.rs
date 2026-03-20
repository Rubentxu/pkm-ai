//! Link command

use std::path::PathBuf;

use pkm_ai::versioning::VersionRepo;

use crate::db::Database;
use crate::models::{Edge, FractionalIndex, LinkType};
use crate::{NexusError, NexusResult};

pub async fn execute(
    db: &Database,
    from: &str,
    to: &str,
    link_type: &str,
    weight: f32,
    context: &Option<String>,
    auto_stage: bool,
) -> anyhow::Result<()> {
    let from_id = ulid::Ulid::from_string(from)
        .map_err(|_| NexusError::BlockNotFound(from.to_string()))?;

    let to_id = ulid::Ulid::from_string(to)
        .map_err(|_| NexusError::BlockNotFound(to.to_string()))?;

    let link_type = parse_link_type(link_type)?;

    // Convert f32 weight to FractionalIndex for backward compatibility
    // weight of 0.0 -> first(), otherwise compute position
    let sequence_weight = if weight <= 0.0 {
        FractionalIndex::first()
    } else {
        // Create a sequence of indices based on weight
        // This is a simple mapping for backward compatibility
        let mut idx = FractionalIndex::first();
        for _ in 0..(weight as usize) {
            idx = FractionalIndex::after_last(&idx);
        }
        idx
    };

    let mut edge = Edge::new(from_id, to_id, link_type);
    edge.sequence_weight = sequence_weight;

    if let Some(ctx) = context {
        edge.context = Some(ctx.clone());
    }

    let repo = db.edges();
    repo.create(edge.clone()).await?;

    println!("Created edge: {} -> {} ({:?})", from, to, edge.link_type);
    println!("   ULID: {}", edge.id_str());
    println!("   Weight: {}", edge.sequence_weight);
    if let Some(ctx) = &edge.context {
        println!("   Context: {}", ctx);
    }

    // Auto-stage the edge if requested
    if auto_stage {
        let link_type_str = format!("{:?}", edge.link_type).to_lowercase();
        if let Err(e) = stage_edge(from_id, to_id, &link_type_str).await {
            eprintln!("Warning: Failed to stage edge: {}", e);
        } else {
            println!("   Staged for commit");
        }
    }

    Ok(())
}

/// Stage an edge in the version repository
async fn stage_edge(source: ulid::Ulid, target: ulid::Ulid, relation: &str) -> anyhow::Result<()> {
    let repo_path = resolve_repo_path();
    let version_repo = VersionRepo::new(&repo_path);
    version_repo.init()?;
    version_repo.add_edge(source, target, relation)?;
    Ok(())
}

/// Resolve the repository path (same logic as in version.rs)
fn resolve_repo_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".pkm")
}

pub fn parse_link_type(s: &str) -> NexusResult<LinkType> {
    match s.to_lowercase().replace("-", "_").as_str() {
        "extends" | "elaborates" => Ok(LinkType::Extends),
        "refines" | "improves" => Ok(LinkType::Refines),
        "contradicts" | "opposes" => Ok(LinkType::Contradicts),
        "questions" | "asks" => Ok(LinkType::Questions),
        "supports" | "evidence" => Ok(LinkType::Supports),
        "references" | "cites" => Ok(LinkType::References),
        "related" | "see_also" => Ok(LinkType::Related),
        "similar_to" | "similar" => Ok(LinkType::SimilarTo),
        "section_of" | "belongs_to" => Ok(LinkType::SectionOf),
        "subsection_of" | "child_of" => Ok(LinkType::SubsectionOf),
        "ordered_child" | "seq_child" => Ok(LinkType::OrderedChild),
        "next" | "seq_next" => Ok(LinkType::Next),
        "next_sibling" | "sibling_next" => Ok(LinkType::NextSibling),
        "first_child" | "first" => Ok(LinkType::FirstChild),
        "contains" | "parent_of" => Ok(LinkType::Contains),
        "parent" | "has_parent" => Ok(LinkType::Parent),
        "ai_suggested" | "ai_link" => Ok(LinkType::AiSuggested),
        _ => Err(NexusError::InvalidEdgeType(s.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_link_type_extends() {
        assert_eq!(parse_link_type("extends").unwrap(), LinkType::Extends);
        assert_eq!(parse_link_type("elaborates").unwrap(), LinkType::Extends);
        assert_eq!(parse_link_type("EXTENDS").unwrap(), LinkType::Extends);
    }

    #[test]
    fn test_parse_link_type_refines() {
        assert_eq!(parse_link_type("refines").unwrap(), LinkType::Refines);
        assert_eq!(parse_link_type("improves").unwrap(), LinkType::Refines);
    }

    #[test]
    fn test_parse_link_type_contradicts() {
        assert_eq!(parse_link_type("contradicts").unwrap(), LinkType::Contradicts);
        assert_eq!(parse_link_type("opposes").unwrap(), LinkType::Contradicts);
    }

    #[test]
    fn test_parse_link_type_questions() {
        assert_eq!(parse_link_type("questions").unwrap(), LinkType::Questions);
        assert_eq!(parse_link_type("asks").unwrap(), LinkType::Questions);
    }

    #[test]
    fn test_parse_link_type_supports() {
        assert_eq!(parse_link_type("supports").unwrap(), LinkType::Supports);
        assert_eq!(parse_link_type("evidence").unwrap(), LinkType::Supports);
    }

    #[test]
    fn test_parse_link_type_references() {
        assert_eq!(parse_link_type("references").unwrap(), LinkType::References);
        assert_eq!(parse_link_type("cites").unwrap(), LinkType::References);
    }

    #[test]
    fn test_parse_link_type_related() {
        assert_eq!(parse_link_type("related").unwrap(), LinkType::Related);
        assert_eq!(parse_link_type("see_also").unwrap(), LinkType::Related);
    }

    #[test]
    fn test_parse_link_type_similar_to() {
        assert_eq!(parse_link_type("similar_to").unwrap(), LinkType::SimilarTo);
        assert_eq!(parse_link_type("similar").unwrap(), LinkType::SimilarTo);
    }

    #[test]
    fn test_parse_link_type_section_of() {
        assert_eq!(parse_link_type("section_of").unwrap(), LinkType::SectionOf);
        assert_eq!(parse_link_type("belongs_to").unwrap(), LinkType::SectionOf);
    }

    #[test]
    fn test_parse_link_type_subsection_of() {
        assert_eq!(parse_link_type("subsection_of").unwrap(), LinkType::SubsectionOf);
        assert_eq!(parse_link_type("child_of").unwrap(), LinkType::SubsectionOf);
    }

    #[test]
    fn test_parse_link_type_ordered_child() {
        assert_eq!(parse_link_type("ordered_child").unwrap(), LinkType::OrderedChild);
        assert_eq!(parse_link_type("seq_child").unwrap(), LinkType::OrderedChild);
    }

    #[test]
    fn test_parse_link_type_next() {
        assert_eq!(parse_link_type("next").unwrap(), LinkType::Next);
        assert_eq!(parse_link_type("seq_next").unwrap(), LinkType::Next);
    }

    #[test]
    fn test_parse_link_type_next_sibling() {
        assert_eq!(parse_link_type("next_sibling").unwrap(), LinkType::NextSibling);
        assert_eq!(parse_link_type("sibling_next").unwrap(), LinkType::NextSibling);
    }

    #[test]
    fn test_parse_link_type_first_child() {
        assert_eq!(parse_link_type("first_child").unwrap(), LinkType::FirstChild);
        assert_eq!(parse_link_type("first").unwrap(), LinkType::FirstChild);
    }

    #[test]
    fn test_parse_link_type_contains() {
        assert_eq!(parse_link_type("contains").unwrap(), LinkType::Contains);
        assert_eq!(parse_link_type("parent_of").unwrap(), LinkType::Contains);
    }

    #[test]
    fn test_parse_link_type_parent() {
        assert_eq!(parse_link_type("parent").unwrap(), LinkType::Parent);
        assert_eq!(parse_link_type("has_parent").unwrap(), LinkType::Parent);
    }

    #[test]
    fn test_parse_link_type_ai_suggested() {
        assert_eq!(parse_link_type("ai_suggested").unwrap(), LinkType::AiSuggested);
        assert_eq!(parse_link_type("ai_link").unwrap(), LinkType::AiSuggested);
    }

    #[test]
    fn test_parse_link_type_with_hyphen() {
        // Hyphens should be converted to underscores
        assert_eq!(parse_link_type("similar-to").unwrap(), LinkType::SimilarTo);
        assert_eq!(parse_link_type("section-of").unwrap(), LinkType::SectionOf);
    }

    #[test]
    fn test_parse_link_type_invalid() {
        let result = parse_link_type("invalid_type");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NexusError::InvalidEdgeType(_)));
    }

    #[test]
    fn test_parse_link_type_case_insensitive() {
        assert_eq!(parse_link_type("RELATED").unwrap(), LinkType::Related);
        assert_eq!(parse_link_type("Related").unwrap(), LinkType::Related);
        assert_eq!(parse_link_type("ReLaTeD").unwrap(), LinkType::Related);
    }

    #[test]
    fn test_resolve_repo_path_ends_with_pkm() {
        let path = resolve_repo_path();
        assert!(path.to_string_lossy().ends_with(".pkm"));
    }
}