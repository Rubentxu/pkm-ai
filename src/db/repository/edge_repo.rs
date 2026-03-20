//! Edge Repository

use crate::models::{Edge, LinkType};
use crate::{NexusError, NexusResult};
use surrealdb::Surreal;
use surrealdb::engine::local::Db;

pub struct EdgeRepository<'a> {
    db: &'a Surreal<Db>,
}

#[allow(dead_code)]
impl<'a> EdgeRepository<'a> {
    pub fn new(db: &'a Surreal<Db>) -> Self {
        Self { db }
    }

    /// Create a new edge
    pub async fn create(&self, edge: Edge) -> NexusResult<()> {
        let ulid = edge.id.to_string();
        let sql = r#"
            CREATE edge CONTENT {
                ulid: $ulid,
                src: $src,
                dst: $dst,
                link_type: $link_type,
                sequence_weight: $sequence_weight,
                context: $context,
                ai_justification: $ai_justification,
                confidence: $confidence,
                created_at: $created_at,
                verified: $verified
            }
        "#;

        self.db
            .query(sql)
            .bind(("ulid", ulid))
            .bind(("src", edge.from.to_string()))
            .bind(("dst", edge.to.to_string()))
            .bind(("link_type", serde_json::to_string(&edge.link_type).unwrap_or_default().trim_matches('"').to_string()))
            .bind(("sequence_weight", edge.sequence_weight.to_string()))
            .bind(("context", edge.context))
            .bind(("ai_justification", edge.ai_justification))
            .bind(("confidence", edge.confidence))
            .bind(("created_at", edge.created_at.to_rfc3339()))
            .bind(("verified", edge.verified))
            .await
            .map_err(|e| NexusError::Database(e.to_string()))?;

        Ok(())
    }

    /// Get an edge by ID
    pub async fn get(&self, id: &ulid::Ulid) -> NexusResult<Option<Edge>> {
        let id_str = id.to_string();
        let sql = "SELECT * FROM edge WHERE ulid = $ulid LIMIT 1";
        let mut result = self.db
            .query(sql)
            .bind(("ulid", id_str))
            .await
            .map_err(|e| NexusError::Database(e.to_string()))?;

        let edges: Vec<Edge> = result.take(0).map_err(|e| NexusError::Database(e.to_string()))?;
        Ok(edges.into_iter().next())
    }

    /// Delete an edge by ID
    pub async fn delete(&self, id: &ulid::Ulid) -> NexusResult<()> {
        let id_str = id.to_string();
        let sql = "DELETE FROM edge WHERE ulid = $ulid";
        self.db
            .query(sql)
            .bind(("ulid", id_str))
            .await
            .map_err(|e| NexusError::Database(e.to_string()))?;

        Ok(())
    }

    /// Delete a specific edge between source and target blocks
    pub async fn delete_by_source_target(&self, source: &ulid::Ulid, target: &ulid::Ulid) -> NexusResult<()> {
        let src_str = source.to_string();
        let dst_str = target.to_string();
        let sql = "DELETE FROM edge WHERE src = $src AND dst = $dst";
        self.db
            .query(sql)
            .bind(("src", src_str))
            .bind(("dst", dst_str))
            .await
            .map_err(|e| NexusError::Database(e.to_string()))?;

        Ok(())
    }

    /// Delete all edges originating from a block
    pub async fn delete_all_from(&self, source: &ulid::Ulid) -> NexusResult<()> {
        let src_str = source.to_string();
        let sql = "DELETE FROM edge WHERE src = $src";
        self.db
            .query(sql)
            .bind(("src", src_str))
            .await
            .map_err(|e| NexusError::Database(e.to_string()))?;

        Ok(())
    }

    /// Delete all edges targeting a block
    pub async fn delete_all_to(&self, target: &ulid::Ulid) -> NexusResult<()> {
        let dst_str = target.to_string();
        let sql = "DELETE FROM edge WHERE dst = $dst";
        self.db
            .query(sql)
            .bind(("dst", dst_str))
            .await
            .map_err(|e| NexusError::Database(e.to_string()))?;

        Ok(())
    }

    /// Delete all edges associated with a block (both incoming and outgoing)
    pub async fn delete_for_block(&self, block_id: &ulid::Ulid) -> NexusResult<()> {
        let id_str = block_id.to_string();
        let sql = "DELETE FROM edge WHERE src = $id OR dst = $id";
        self.db
            .query(sql)
            .bind(("id", id_str))
            .await
            .map_err(|e| NexusError::Database(e.to_string()))?;

        Ok(())
    }

    /// Get edges by link type
    pub async fn list_by_type(&self, link_type: LinkType) -> NexusResult<Vec<Edge>> {
        let type_str = serde_json::to_string(&link_type)
            .unwrap_or_default()
            .trim_matches('"')
            .to_string();

        let query = "SELECT * FROM edge WHERE link_type = $type ORDER BY sequence_weight ASC";
        let mut result = self.db
            .query(query)
            .bind(("type", type_str))
            .await
            .map_err(|e| NexusError::Database(e.to_string()))?;

        result
            .take(0)
            .map_err(|e| NexusError::Database(e.to_string()))
    }

    /// Get outgoing edges from a block
    pub async fn outgoing_from(&self, block_id: &ulid::Ulid) -> NexusResult<Vec<Edge>> {
        let id_str = block_id.to_string();
        let query = "SELECT * FROM edge WHERE src = $block_id ORDER BY sequence_weight ASC";
        let mut result = self.db
            .query(query)
            .bind(("block_id", id_str))
            .await
            .map_err(|e| NexusError::Database(e.to_string()))?;

        result
            .take(0)
            .map_err(|e| NexusError::Database(e.to_string()))
    }

    /// Get incoming edges to a block
    pub async fn incoming_to(&self, block_id: &ulid::Ulid) -> NexusResult<Vec<Edge>> {
        let id_str = block_id.to_string();
        let query = "SELECT * FROM edge WHERE dst = $block_id ORDER BY created_at DESC";
        let mut result = self.db
            .query(query)
            .bind(("block_id", id_str))
            .await
            .map_err(|e| NexusError::Database(e.to_string()))?;

        result
            .take(0)
            .map_err(|e| NexusError::Database(e.to_string()))
    }

    /// List all edges
    pub async fn list_all(&self) -> NexusResult<Vec<Edge>> {
        let query = "SELECT * FROM edge ORDER BY created_at DESC";
        let mut result = self.db
            .query(query)
            .await
            .map_err(|e| NexusError::Database(e.to_string()))?;

        result
            .take(0)
            .map_err(|e| NexusError::Database(e.to_string()))
    }

    /// Update an edge
    pub async fn update(&self, edge: Edge) -> NexusResult<()> {
        let id_str = edge.id.to_string();
        let sql = r#"
            UPDATE edge SET
                src = $src,
                dst = $dst,
                link_type = $link_type,
                sequence_weight = $sequence_weight,
                context = $context,
                ai_justification = $ai_justification,
                confidence = $confidence,
                verified = $verified
            WHERE ulid = $ulid
        "#;

        self.db
            .query(sql)
            .bind(("ulid", id_str))
            .bind(("src", edge.from.to_string()))
            .bind(("dst", edge.to.to_string()))
            .bind(("link_type", serde_json::to_string(&edge.link_type).unwrap_or_default().trim_matches('"').to_string()))
            .bind(("sequence_weight", edge.sequence_weight.to_string()))
            .bind(("context", edge.context))
            .bind(("ai_justification", edge.ai_justification))
            .bind(("confidence", edge.confidence))
            .bind(("verified", edge.verified))
            .await
            .map_err(|e| NexusError::Database(e.to_string()))?;

        Ok(())
    }
}