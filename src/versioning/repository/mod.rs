//! Repository layer for versioning storage
//!
//! This module provides filesystem-based storage for the versioning layer:
//! - ObjectStore: stores commits, blocks, and structures as JSON files
//! - RefStore: stores branches and tags as JSON files
//! - WorkingSetStore: stores the staging area as a JSON file

pub mod object_store;
pub mod ref_store;
pub mod working_set_store;

pub use object_store::{ObjectStore, ObjectStoreError};
pub use ref_store::{RefStore, RefStoreError};
pub use working_set_store::{WorkingSetStore, WorkingSetStoreError};