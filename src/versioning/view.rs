//! View: branches and tags pointing to commits
//!
//! Views are named references to commits, similar to Git branches and tags.

use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// ViewName: name of a view (branch or tag)
///
/// Stored separately from View to allow efficient lookup and comparison.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ViewName(String);

impl ViewName {
    /// Create a new view name
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Get the name as a string slice
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Check if this is a valid branch name
    ///
    /// Branch names must not be empty, start with `.`, or be "HEAD"
    #[must_use]
    pub fn is_valid_branch_name(&self) -> bool {
        !self.0.is_empty()
            && !self.0.starts_with('.')
            && self.0 != "HEAD"
    }

    /// Check if this is a valid tag name
    ///
    /// Tag names must not be empty or start with `-`
    #[must_use]
    pub fn is_valid_tag_name(&self) -> bool {
        !self.0.is_empty() && !self.0.starts_with('-')
    }
}

impl Default for ViewName {
    fn default() -> Self {
        Self::new("main")
    }
}

impl std::fmt::Display for ViewName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ViewName {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for ViewName {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

/// View: a named reference to a commit
///
/// Views can be either:
/// - Branch: a movable reference that typically points to the latest commit
/// - Tag: an immutable reference to a specific commit (often for releases)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum View {
    /// A branch reference
    Branch {
        /// Branch name
        name: ViewName,
        /// Target commit ID
        target: Ulid,
        /// Whether this branch is the current HEAD
        #[serde(default)]
        is_head: bool,
    },
    /// A tag reference (immutable)
    Tag {
        /// Tag name
        name: ViewName,
        /// Target commit ID
        target: Ulid,
        /// Optional tag message (for annotated tags)
        #[serde(default)]
        message: String,
    },
}

impl View {
    /// Create a new branch pointing to a commit
    #[must_use]
    pub fn branch(name: impl Into<ViewName>, target: Ulid) -> Self {
        Self::Branch {
            name: name.into(),
            target,
            is_head: false,
        }
    }

    /// Create a new branch that is the current HEAD
    #[must_use]
    pub fn branch_head(name: impl Into<ViewName>, target: Ulid) -> Self {
        Self::Branch {
            name: name.into(),
            target,
            is_head: true,
        }
    }

    /// Create a new tag pointing to a commit
    #[must_use]
    pub fn tag(name: impl Into<ViewName>, target: Ulid) -> Self {
        Self::Tag {
            name: name.into(),
            target,
            message: String::new(),
        }
    }

    /// Create a new annotated tag pointing to a commit
    #[must_use]
    pub fn tag_with_message(name: impl Into<ViewName>, target: Ulid, message: String) -> Self {
        Self::Tag {
            name: name.into(),
            target,
            message,
        }
    }

    /// Get the view name
    #[must_use]
    pub fn name(&self) -> &ViewName {
        match self {
            Self::Branch { name, .. } => name,
            Self::Tag { name, .. } => name,
        }
    }

    /// Get the target commit ID
    #[must_use]
    pub fn target(&self) -> Ulid {
        match self {
            Self::Branch { target, .. } => *target,
            Self::Tag { target, .. } => *target,
        }
    }

    /// Check if this is a branch
    #[must_use]
    pub fn is_branch(&self) -> bool {
        matches!(self, Self::Branch { .. })
    }

    /// Check if this is a tag
    #[must_use]
    pub fn is_tag(&self) -> bool {
        matches!(self, Self::Tag { .. })
    }

    /// Update the target commit (only for branches)
    ///
    /// Returns true if the update was successful
    pub fn set_target(&mut self, target: Ulid) -> bool {
        match self {
            Self::Branch { target: t, .. } => {
                *t = target;
                true
            }
            Self::Tag { .. } => false,
        }
    }

    /// Get the is_head flag (only for branches)
    #[must_use]
    pub fn is_head(&self) -> bool {
        match self {
            Self::Branch { is_head, .. } => *is_head,
            Self::Tag { .. } => false,
        }
    }

    /// Set the is_head flag (only for branches)
    pub fn set_is_head(&mut self, is_head: bool) -> bool {
        match self {
            Self::Branch { is_head: h, .. } => {
                *h = is_head;
                true
            }
            Self::Tag { .. } => false,
        }
    }

    /// Get the tag message (only for tags)
    #[must_use]
    pub fn message(&self) -> Option<&str> {
        match self {
            Self::Tag { message, .. } if !message.is_empty() => Some(message),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_name_creation() {
        let name = ViewName::new("main");
        assert_eq!(name.as_str(), "main");
    }

    #[test]
    fn test_view_name_validity() {
        assert!(ViewName::new("main").is_valid_branch_name());
        assert!(ViewName::new("feature/test").is_valid_branch_name());
        assert!(!ViewName::new("").is_valid_branch_name());
        assert!(!ViewName::new(".hidden").is_valid_branch_name());
        assert!(!ViewName::new("HEAD").is_valid_branch_name());
    }

    #[test]
    fn test_branch_creation() {
        let target = Ulid::new();
        let view = View::branch("main", target);

        assert_eq!(view.name().as_str(), "main");
        assert_eq!(view.target(), target);
        assert!(view.is_branch());
        assert!(!view.is_tag());
        assert!(!view.is_head());
    }

    #[test]
    fn test_branch_head_creation() {
        let target = Ulid::new();
        let view = View::branch_head("main", target);

        assert!(view.is_head());
        assert!(view.is_branch());
    }

    #[test]
    fn test_tag_creation() {
        let target = Ulid::new();
        let view = View::tag("v1.0.0", target);

        assert_eq!(view.name().as_str(), "v1.0.0");
        assert_eq!(view.target(), target);
        assert!(view.is_tag());
        assert!(!view.is_branch());
        assert!(view.message().is_none());
    }

    #[test]
    fn test_tag_with_message_creation() {
        let target = Ulid::new();
        let view = View::tag_with_message("v1.0.0", target, "Release 1.0.0".to_string());

        assert!(view.is_tag());
        assert_eq!(view.message(), Some("Release 1.0.0"));
    }

    #[test]
    fn test_branch_set_target() {
        let mut view = View::branch("main", Ulid::new());
        let new_target = Ulid::new();
        let result = view.set_target(new_target);

        assert!(result);
        assert_eq!(view.target(), new_target);
    }

    #[test]
    fn test_tag_cannot_change_target() {
        let mut view = View::tag("v1", Ulid::new());
        let original_target = view.target();
        let new_target = Ulid::new();
        let result = view.set_target(new_target);

        assert!(!result);
        assert_eq!(view.target(), original_target);
    }

    #[test]
    fn test_branch_set_is_head() {
        let mut view = View::branch("main", Ulid::new());
        assert!(!view.is_head());

        view.set_is_head(true);
        assert!(view.is_head());

        view.set_is_head(false);
        assert!(!view.is_head());
    }

    #[test]
    fn test_tag_cannot_set_is_head() {
        let mut view = View::tag("v1", Ulid::new());
        let result = view.set_is_head(true);

        assert!(!result);
        assert!(!view.is_head());
    }

    #[test]
    fn test_view_display() {
        let view = View::branch("display-test", Ulid::new());
        assert_eq!(format!("{}", view.name()), "display-test");
    }
}
