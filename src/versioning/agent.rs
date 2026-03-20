//! Agent identifier for versioning operations
//!
//! Represents the author of commits and operations in the knowledge graph.

use serde::{Deserialize, Serialize};
use std::fmt;

/// AgentId: Value object representing the author of versioning operations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(String);

impl AgentId {
    /// Create a new AgentId from a string identifier
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Create an anonymous agent identifier
    #[must_use]
    pub fn anonymous() -> Self {
        Self("anonymous".to_string())
    }

    /// Get the agent identifier as a string slice
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the agent identifier as a owned string
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl Default for AgentId {
    fn default() -> Self {
        Self::anonymous()
    }
}

impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for AgentId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for AgentId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_id_creation() {
        let agent = AgentId::new("test-agent");
        assert_eq!(agent.as_str(), "test-agent");
    }

    #[test]
    fn test_agent_id_anonymous() {
        let agent = AgentId::anonymous();
        assert_eq!(agent.as_str(), "anonymous");
    }

    #[test]
    fn test_agent_id_display() {
        let agent = AgentId::new("display-test");
        assert_eq!(format!("{agent}"), "display-test");
    }

    #[test]
    fn test_agent_id_default() {
        let default = AgentId::default();
        assert_eq!(default.as_str(), "anonymous");
    }

    #[test]
    fn test_agent_id_from_string() {
        let agent: AgentId = String::from("from-string").into();
        assert_eq!(agent.as_str(), "from-string");
    }
}
