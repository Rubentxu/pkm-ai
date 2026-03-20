//! Packfile implementation for differential sync
//!
//! Packfile is a binary format for efficiently transferring objects between repositories.
//! Only new objects are included in the packfile, reducing network bandwidth.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use ulid::Ulid;

#[derive(Error, Debug)]
pub enum PackfileError {
    #[error("Failed to read packfile: {0}")]
    ReadError(String),

    #[error("Failed to write packfile: {0}")]
    WriteError(String),

    #[error("Invalid packfile format: {0}")]
    InvalidFormat(String),

    #[error("Object not found: {0}")]
    ObjectNotFound(Ulid),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Single entry in a packfile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackfileEntry {
    pub object_id: Ulid,
    pub object_type: PackfileObjectType,
    pub compressed_data: Vec<u8>,
}

/// Object types that can be stored in a packfile
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackfileObjectType {
    Commit,
    Structure,
    Block,
}

/// Packfile containing compressed objects for transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Packfile {
    pub version: u32,
    pub entries: Vec<PackfileEntry>,
    pub total_objects: usize,
}

impl Packfile {
    /// Create a new empty packfile
    pub fn new(object_ids: Vec<u32>) -> Self {
        let total_objects = object_ids.len();
        Self {
            version: 1,
            entries: Vec::new(),
            total_objects,
        }
    }

    /// Create a packfile with entries
    pub fn with_entries(entries: Vec<PackfileEntry>) -> Self {
        let total_objects = entries.len();
        Self {
            version: 1,
            entries,
            total_objects,
        }
    }

    /// Add an entry to the packfile
    pub fn add_entry(&mut self, entry: PackfileEntry) {
        self.entries.push(entry);
        self.total_objects = self.entries.len();
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if packfile is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Serialize packfile to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, PackfileError> {
        let mut bytes = Vec::new();

        // Write header
        bytes.extend_from_slice(b"PACK");
        bytes.extend_from_slice(&1u32.to_be_bytes()); // version
        bytes.extend_from_slice(&(self.entries.len() as u32).to_be_bytes());

        // Write entries
        for entry in &self.entries {
            let entry_bytes = serde_json::to_vec(entry)?;
            bytes.extend_from_slice(&(entry_bytes.len() as u32).to_be_bytes());
            bytes.extend_from_slice(&entry_bytes);
        }

        Ok(bytes)
    }

    /// Deserialize packfile from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, PackfileError> {
        if bytes.len() < 12 {
            return Err(PackfileError::InvalidFormat("Too short".into()));
        }

        // Verify header
        if &bytes[0..4] != b"PACK" {
            return Err(PackfileError::InvalidFormat("Invalid magic".into()));
        }

        let version = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        if version != 1 {
            return Err(PackfileError::InvalidFormat(format!("Unknown version: {}", version)));
        }

        let entry_count = u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;

        let mut entries = Vec::new();
        let mut offset = 12;

        for _ in 0..entry_count {
            if offset + 4 > bytes.len() {
                return Err(PackfileError::InvalidFormat("Unexpected end".into()));
            }

            let entry_len = u32::from_be_bytes([bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]]) as usize;
            offset += 4;

            if offset + entry_len > bytes.len() {
                return Err(PackfileError::InvalidFormat("Entry truncated".into()));
            }

            let entry_bytes = &bytes[offset..offset + entry_len];
            let entry: PackfileEntry = serde_json::from_slice(entry_bytes)?;
            entries.push(entry);
            offset += entry_len;
        }

        let total = entries.len();
        Ok(Self {
            version: 1,
            entries,
            total_objects: total,
        })
    }

    /// Write packfile to a file
    pub fn write_to(&self, path: &std::path::Path) -> Result<(), PackfileError> {
        let bytes = self.to_bytes()?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    /// Read packfile from a file
    pub fn read_from(path: &std::path::Path) -> Result<Self, PackfileError> {
        let bytes = std::fs::read(path)?;
        Self::from_bytes(&bytes)
    }
}

/// Builder for creating packfiles
#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct PackfileBuilder {
    entries: Vec<PackfileEntry>,
}

#[allow(dead_code)]
impl PackfileBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_commit(mut self, commit_id: Ulid, data: Vec<u8>) -> Self {
        self.entries.push(PackfileEntry {
            object_id: commit_id,
            object_type: PackfileObjectType::Commit,
            compressed_data: data,
        });
        self
    }

    pub fn add_structure(mut self, structure_id: Ulid, data: Vec<u8>) -> Self {
        self.entries.push(PackfileEntry {
            object_id: structure_id,
            object_type: PackfileObjectType::Structure,
            compressed_data: data,
        });
        self
    }

    pub fn add_block(mut self, block_id: Ulid, data: Vec<u8>) -> Self {
        self.entries.push(PackfileEntry {
            object_id: block_id,
            object_type: PackfileObjectType::Block,
            compressed_data: data,
        });
        self
    }

    pub fn build(self) -> Packfile {
        Packfile::with_entries(self.entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packfile_creation() {
        let packfile = Packfile::new(vec![1, 2, 3]);
        assert_eq!(packfile.total_objects, 3);
        assert!(packfile.is_empty());
    }

    #[test]
    fn test_packfile_serialization() {
        let mut packfile = Packfile::new(vec![]);
        packfile.add_entry(PackfileEntry {
            object_id: Ulid::new(),
            object_type: PackfileObjectType::Commit,
            compressed_data: vec![1, 2, 3],
        });

        let bytes = packfile.to_bytes().unwrap();
        assert!(bytes.starts_with(b"PACK"));

        let deserialized = Packfile::from_bytes(&bytes).unwrap();
        assert_eq!(deserialized.entries.len(), 1);
        assert_eq!(deserialized.version, 1);
    }

    #[test]
    fn test_packfile_builder() {
        let packfile = PackfileBuilder::new()
            .add_commit(Ulid::new(), vec![1, 2, 3])
            .add_structure(Ulid::new(), vec![4, 5, 6])
            .build();

        assert_eq!(packfile.entries.len(), 2);
        assert_eq!(packfile.entries[0].object_type, PackfileObjectType::Commit);
        assert_eq!(packfile.entries[1].object_type, PackfileObjectType::Structure);
    }
}