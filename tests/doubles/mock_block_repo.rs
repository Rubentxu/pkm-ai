//! Mock Block Repository for testing
//!
//! Provides an in-memory implementation for testing without database dependencies.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use ulid::Ulid;
use crate::models::{Block, BlockType};

/// Mock Block Repository implementation for testing
pub struct MockBlockRepository {
    blocks: RwLock<HashMap<Ulid, Block>>,
    create_calls: RwLock<Vec<Block>>,
    update_calls: RwLock<Vec<Block>>,
    delete_calls: RwLock<Vec<Ulid>>,
}

impl MockBlockRepository {
    /// Create a new empty MockBlockRepository
    pub fn new() -> Self {
        Self {
            blocks: RwLock::new(HashMap::new()),
            create_calls: RwLock::new(Vec::new()),
            update_calls: RwLock::new(Vec::new()),
            delete_calls: RwLock::new(Vec::new()),
        }
    }

    /// Create a MockBlockRepository pre-populated with blocks
    pub fn with_blocks(blocks: Vec<Block>) -> Self {
        let mut map = HashMap::new();
        for block in blocks {
            map.insert(block.id, block);
        }
        Self {
            blocks: RwLock::new(map),
            create_calls: RwLock::new(Vec::new()),
            update_calls: RwLock::new(Vec::new()),
            delete_calls: RwLock::new(Vec::new()),
        }
    }

    /// Get all blocks
    pub fn get_all_blocks(&self) -> Vec<Block> {
        self.blocks.read().unwrap().values().cloned().collect()
    }

    /// Get calls to create method
    pub fn take_create_calls(&self) -> Vec<Block> {
        let calls = self.create_calls.read().unwrap().clone();
        self.create_calls.write().unwrap().clear();
        calls
    }

    /// Get calls to update method
    pub fn take_update_calls(&self) -> Vec<Block> {
        let calls = self.update_calls.read().unwrap().clone();
        self.update_calls.write().unwrap().clear();
        calls
    }

    /// Get calls to delete method
    pub fn take_delete_calls(&self) -> Vec<Ulid> {
        let calls = self.delete_calls.read().unwrap().clone();
        self.delete_calls.write().unwrap().clear();
        calls
    }

    /// Create a new block
    pub async fn create(&self, block: Block) -> Result<(), String> {
        self.create_calls.write().unwrap().push(block.clone());
        self.blocks.write().unwrap().insert(block.id, block);
        Ok(())
    }

    /// Get a block by ID
    pub async fn get(&self, id: &Ulid) -> Result<Option<Block>, String> {
        Ok(self.blocks.read().unwrap().get(id).cloned())
    }

    /// Update a block
    pub async fn update(&self, block: Block) -> Result<(), String> {
        self.update_calls.write().unwrap().push(block.clone());
        if self.blocks.read().unwrap().contains_key(&block.id) {
            self.blocks.write().unwrap().insert(block.id, block);
            Ok(())
        } else {
            Err(format!("Block not found: {}", id))
        }
    }

    /// Delete a block
    pub async fn delete(&self, id: &Ulid) -> Result<(), String> {
        self.delete_calls.write().unwrap().push(*id);
        self.blocks.write().unwrap().remove(id);
        Ok(())
    }

    /// List blocks by type
    pub async fn list_by_type(&self, block_type: BlockType) -> Result<Vec<Block>, String> {
        let blocks: Vec<Block> = self.blocks
            .read()
            .unwrap()
            .values()
            .filter(|b| b.block_type == block_type)
            .cloned()
            .collect();
        Ok(blocks)
    }

    /// Search blocks by tags
    pub async fn search_by_tags(&self, tags: &[String]) -> Result<Vec<Block>, String> {
        let blocks: Vec<Block> = self.blocks
            .read()
            .unwrap()
            .values()
            .filter(|b| tags.iter().any(|tag| b.tags.contains(tag)))
            .cloned()
            .collect();
        Ok(blocks)
    }

    /// Full-text search in content
    pub async fn search_content(&self, _query: &str) -> Result<Vec<Block>, String> {
        // Simple content search - in real implementation this would use full-text search
        Ok(self.blocks.read().unwrap().values().cloned().collect())
    }

    /// List all blocks
    pub async fn list_all(&self) -> Result<Vec<Block>, String> {
        Ok(self.blocks.read().unwrap().values().cloned().collect())
    }
}

impl Default for MockBlockRepository {
    fn default() -> Self {
        Self::new()
    }
}