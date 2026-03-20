//! Mock Edge Repository for testing
//!
//! Provides an in-memory implementation for testing without database dependencies.

use std::collections::HashMap;
use std::sync::RwLock;
use ulid::Ulid;
use crate::models::{Edge, LinkType};

/// Mock Edge Repository implementation for testing
pub struct MockEdgeRepository {
    edges: RwLock<HashMap<Ulid, Edge>>,
    create_calls: RwLock<Vec<Edge>>,
    delete_calls: RwLock<Vec<Ulid>>,
}

impl MockEdgeRepository {
    /// Create a new empty MockEdgeRepository
    pub fn new() -> Self {
        Self {
            edges: RwLock::new(HashMap::new()),
            create_calls: RwLock::new(Vec::new()),
            delete_calls: RwLock::new(Vec::new()),
        }
    }

    /// Create a MockEdgeRepository pre-populated with edges
    pub fn with_edges(edges: Vec<Edge>) -> Self {
        let mut map = HashMap::new();
        for edge in edges {
            map.insert(edge.id, edge);
        }
        Self {
            edges: RwLock::new(map),
            create_calls: RwLock::new(Vec::new()),
            delete_calls: RwLock::new(Vec::new()),
        }
    }

    /// Get all edges
    pub fn get_all_edges(&self) -> Vec<Edge> {
        self.edges.read().unwrap().values().cloned().collect()
    }

    /// Get calls to create method
    pub fn take_create_calls(&self) -> Vec<Edge> {
        let calls = self.create_calls.read().unwrap().clone();
        self.create_calls.write().unwrap().clear();
        calls
    }

    /// Get calls to delete method
    pub fn take_delete_calls(&self) -> Vec<Ulid> {
        let calls = self.delete_calls.read().unwrap().clone();
        self.delete_calls.write().unwrap().clear();
        calls
    }

    /// Create a new edge
    pub async fn create(&self, edge: Edge) -> Result<(), String> {
        self.create_calls.write().unwrap().push(edge.clone());
        self.edges.write().unwrap().insert(edge.id, edge);
        Ok(())
    }

    /// Get an edge by ID
    pub async fn get(&self, id: &Ulid) -> Result<Option<Edge>, String> {
        Ok(self.edges.read().unwrap().get(id).cloned())
    }

    /// Delete an edge
    pub async fn delete(&self, id: &Ulid) -> Result<(), String> {
        self.delete_calls.write().unwrap().push(*id);
        self.edges.write().unwrap().remove(id);
        Ok(())
    }

    /// List edges by link type
    pub async fn list_by_type(&self, link_type: LinkType) -> Result<Vec<Edge>, String> {
        let edges: Vec<Edge> = self.edges
            .read()
            .unwrap()
            .values()
            .filter(|e| e.link_type == link_type)
            .cloned()
            .collect();
        Ok(edges)
    }

    /// Get outgoing edges from a block
    pub async fn outgoing_from(&self, block_id: &Ulid) -> Result<Vec<Edge>, String> {
        let edges: Vec<Edge> = self.edges
            .read()
            .unwrap()
            .values()
            .filter(|e| e.from == *block_id)
            .cloned()
            .collect();
        Ok(edges)
    }

    /// Get incoming edges to a block
    pub async fn incoming_to(&self, block_id: &Ulid) -> Result<Vec<Edge>, String> {
        let edges: Vec<Edge> = self.edges
            .read()
            .unwrap()
            .values()
            .filter(|e| e.to == *block_id)
            .cloned()
            .collect();
        Ok(edges)
    }

    /// List all edges
    pub async fn list_all(&self) -> Result<Vec<Edge>, String> {
        Ok(self.edges.read().unwrap().values().cloned().collect())
    }

    /// Update an edge
    pub async fn update(&self, _edge: Edge) -> Result<(), String> {
        // For mock, we just return Ok for now
        Ok(())
    }
}

impl Default for MockEdgeRepository {
    fn default() -> Self {
        Self::new()
    }
}