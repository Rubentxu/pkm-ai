//! Block Repository

use crate::models::{Block, BlockType};
use crate::{NexusError, NexusResult};
use surrealdb::Surreal;
use surrealdb::engine::local::Db;

pub struct BlockRepository<'a> {
    db: &'a Surreal<Db>,
}

#[allow(dead_code)]
impl<'a> BlockRepository<'a> {
    pub fn new(db: &'a Surreal<Db>) -> Self {
        Self { db }
    }

    /// Create a new block
    pub async fn create(&self, block: Block) -> NexusResult<()> {
        let ulid = block.id.to_string();
        let sql = r#"
            CREATE block CONTENT {
                ulid: $ulid,
                block_type: $block_type,
                title: $title,
                content: $content,
                tags: $tags,
                metadata: $metadata,
                created_at: $created_at,
                updated_at: $updated_at,
                version: $version,
                ai_confidence: $ai_confidence,
                semantic_centroid: $semantic_centroid
            }
        "#;

        self.db
            .query(sql)
            .bind(("ulid", ulid))
            .bind(("block_type", serde_json::to_string(&block.block_type).unwrap_or_default().trim_matches('"').to_string()))
            .bind(("title", block.title))
            .bind(("content", block.content))
            .bind(("tags", block.tags))
            .bind(("metadata", block.metadata))
            .bind(("created_at", block.created_at.to_rfc3339()))
            .bind(("updated_at", block.updated_at.to_rfc3339()))
            .bind(("version", block.version))
            .bind(("ai_confidence", block.ai_confidence))
            .bind(("semantic_centroid", block.semantic_centroid))
            .await
            .map_err(|e| NexusError::Database(e.to_string()))?;

        Ok(())
    }

    /// Get a block by ID
    pub async fn get(&self, id: &ulid::Ulid) -> NexusResult<Option<Block>> {
        let id_str = id.to_string();
        let sql = "SELECT * FROM block WHERE ulid = $ulid LIMIT 1";
        let mut result = self.db
            .query(sql)
            .bind(("ulid", id_str))
            .await
            .map_err(|e| NexusError::Database(e.to_string()))?;

        let blocks: Vec<Block> = result.take(0).map_err(|e| NexusError::Database(e.to_string()))?;
        Ok(blocks.into_iter().next())
    }

    /// Update a block
    pub async fn update(&self, block: Block) -> NexusResult<()> {
        let id_str = block.id.to_string();
        let sql = r#"
            UPDATE block SET
                block_type = $block_type,
                title = $title,
                content = $content,
                tags = $tags,
                metadata = $metadata,
                updated_at = $updated_at,
                version = $version,
                ai_confidence = $ai_confidence,
                semantic_centroid = $semantic_centroid
            WHERE ulid = $ulid
        "#;

        self.db
            .query(sql)
            .bind(("ulid", id_str))
            .bind(("block_type", serde_json::to_string(&block.block_type).unwrap_or_default().trim_matches('"').to_string()))
            .bind(("title", block.title))
            .bind(("content", block.content))
            .bind(("tags", block.tags))
            .bind(("metadata", block.metadata))
            .bind(("updated_at", chrono::Utc::now().to_rfc3339()))
            .bind(("version", block.version))
            .bind(("ai_confidence", block.ai_confidence))
            .bind(("semantic_centroid", block.semantic_centroid))
            .await
            .map_err(|e| NexusError::Database(e.to_string()))?;

        Ok(())
    }

    /// Delete a block
    pub async fn delete(&self, id: &ulid::Ulid) -> NexusResult<()> {
        let id_str = id.to_string();
        let sql = "DELETE FROM block WHERE ulid = $ulid";
        self.db
            .query(sql)
            .bind(("ulid", id_str))
            .await
            .map_err(|e| NexusError::Database(e.to_string()))?;

        Ok(())
    }

    /// List blocks by type
    pub async fn list_by_type(&self, block_type: BlockType) -> NexusResult<Vec<Block>> {
        let type_str = serde_json::to_string(&block_type)
            .unwrap_or_default()
            .trim_matches('"')
            .to_string();

        let query = "SELECT * FROM block WHERE block_type = $type ORDER BY created_at DESC";
        let mut result = self.db
            .query(query)
            .bind(("type", type_str))
            .await
            .map_err(|e| NexusError::Database(e.to_string()))?;

        result
            .take(0)
            .map_err(|e| NexusError::Database(e.to_string()))
    }

    /// Search blocks by tags
    pub async fn search_by_tags(&self, tags: &[String]) -> NexusResult<Vec<Block>> {
        let query = "SELECT * FROM block WHERE array::intersect(tags, $tags) != [] ORDER BY created_at DESC";
        let mut result = self.db
            .query(query)
            .bind(("tags", tags.to_vec()))
            .await
            .map_err(|e| NexusError::Database(e.to_string()))?;

        result
            .take(0)
            .map_err(|e| NexusError::Database(e.to_string()))
    }

    /// Full-text search in content
    pub async fn search_content(&self, query: &str) -> NexusResult<Vec<Block>> {
        let sql = "SELECT * FROM block WHERE content @@ $query ORDER BY created_at DESC";
        let mut result = self.db
            .query(sql)
            .bind(("query", query.to_string()))
            .await
            .map_err(|e| NexusError::Database(e.to_string()))?;

        result
            .take(0)
            .map_err(|e| NexusError::Database(e.to_string()))
    }

    /// List all blocks
    pub async fn list_all(&self) -> NexusResult<Vec<Block>> {
        let query = "SELECT * FROM block ORDER BY created_at DESC";
        let mut result = self.db
            .query(query)
            .await
            .map_err(|e| NexusError::Database(e.to_string()))?;

        result
            .take(0)
            .map_err(|e| NexusError::Database(e.to_string()))
    }
}