//! Structural Spine: The backbone of ordered knowledge

use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// Lint issues detected in the spine
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LintIssue {
    /// Gap between sequence weights
    Gap {
        after: Ulid,
        before: Ulid,
        gap_size: f32,
    },

    /// Orphan block (not in spine)
    Orphan {
        block_id: Ulid,
    },

    /// Circular reference detected
    CircularReference {
        block_a: Ulid,
        block_b: Ulid,
    },

    /// Anachronism (block placed before its dependencies)
    Anachronism {
        block: Ulid,
        depends_on: Ulid,
    },

    /// Unbalanced load (section too dense)
    UnbalancedLoad {
        section: Ulid,
        density: u32,
        expected: u32,
    },
}
