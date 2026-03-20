//! Structural linting implementation
//!
//! Validates structural integrity of the spine and detects issues.

use crate::db::Database;
use crate::models::{BlockType, Edge, LinkType};
use crate::NexusResult;
use ulid::Ulid;

/// Lint severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LintSeverity {
    Info,
    Warning,
    Error,
}

/// Represents an action taken during auto-fix
#[derive(Debug, Clone)]
pub struct FixAction {
    /// Issue code that was fixed
    pub code: String,
    /// Description of what was done
    pub description: String,
    /// Block ID affected
    pub block_id: Option<Ulid>,
    /// Additional details
    pub details: Option<String>,
}

/// Result of an auto-fix operation
#[derive(Debug, Clone, Default)]
pub struct FixResult {
    /// List of fixes applied
    pub fixes: Vec<FixAction>,
    /// Issues that could not be fixed automatically
    pub unresolved: Vec<LintIssue>,
    /// Errors that occurred during fixing
    pub errors: Vec<String>,
}

impl FixResult {
    /// Returns true if any fixes were applied
    pub fn has_fixes(&self) -> bool {
        !self.fixes.is_empty()
    }

    /// Returns the count of fixes applied
    pub fn fix_count(&self) -> usize {
        self.fixes.len()
    }

    /// Returns the count of unresolved issues
    pub fn unresolved_count(&self) -> usize {
        self.unresolved.len()
    }
}

impl std::fmt::Display for FixResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.fixes.is_empty() && self.errors.is_empty() && self.unresolved.is_empty() {
            writeln!(f, "No fixes applied.")?;
            return Ok(());
        }

        if !self.fixes.is_empty() {
            writeln!(f, "Applied {} fix(es):", self.fixes.len())?;
            for fix in &self.fixes {
                writeln!(f, "  - [{}] {}", fix.code, fix.description)?;
                if let Some(id) = &fix.block_id {
                    writeln!(f, "    Block: {}", id.to_string().chars().take(8).collect::<String>())?;
                }
                if let Some(details) = &fix.details {
                    writeln!(f, "    Details: {}", details)?;
                }
            }
        }

        if !self.unresolved.is_empty() {
            writeln!(f)?;
            writeln!(f, "{} issue(s) could not be auto-fixed:", self.unresolved.len())?;
            for issue in &self.unresolved {
                writeln!(f, "  - [{}] {}", issue.code, issue.message)?;
            }
        }

        if !self.errors.is_empty() {
            writeln!(f)?;
            writeln!(f, "{} error(s) during fix:", self.errors.len())?;
            for err in &self.errors {
                writeln!(f, "  - {}", err)?;
            }
        }

        Ok(())
    }
}

/// A lint issue detected in the spine
#[derive(Debug, Clone)]
pub struct LintIssue {
    /// Issue code
    pub code: String,
    /// Severity level
    pub severity: LintSeverity,
    /// Description
    pub message: String,
    /// Block ID if applicable
    pub block_id: Option<Ulid>,
    /// Additional context
    pub context: Option<String>,
}

impl std::fmt::Display for LintIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)?;
        if let Some(id) = &self.block_id {
            write!(f, " (block: {})", id)?;
        }
        Ok(())
    }
}

/// Structural linter
pub struct StructuralLinter<'a> {
    db: &'a Database,
}

impl<'a> StructuralLinter<'a> {
    /// Create a new StructuralLinter
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// Run all lint rules
    pub async fn lint(&self) -> NexusResult<Vec<LintIssue>> {
        let mut issues = Vec::new();

        // Run all lint rules
        issues.extend(self.check_orphans().await?);
        issues.extend(self.check_gaps().await?);
        issues.extend(self.check_density_balance().await?);
        issues.extend(self.check_forward_references().await?);
        issues.extend(self.check_circular_references().await?);
        issues.extend(self.check_anachronisms().await?);

        // Sort by severity (errors first)
        issues.sort();

        Ok(issues)
    }

    /// Auto-fix issues detected by lint
    ///
    /// Returns a FixResult with:
    /// - List of fixes applied
    /// - Unresolved issues (forward-refs, circular-refs - cannot be auto-fixed)
    /// - Errors encountered during fixing
    pub async fn auto_fix(&self) -> NexusResult<FixResult> {
        let issues = self.lint().await?;
        self.fix_issues(issues).await
    }

    /// Fix a specific set of issues
    async fn fix_issues(&self, issues: Vec<LintIssue>) -> NexusResult<FixResult> {
        let mut result = FixResult::default();

        for issue in issues {
            match issue.code.as_str() {
                "orphan" => {
                    // Try to fix orphan blocks by linking them to a structure
                    if let Some(block_id) = issue.block_id {
                        match self.fix_orphan_block(block_id).await {
                            Ok(Some(description)) => {
                                result.fixes.push(FixAction {
                                    code: "orphan".to_string(),
                                    description,
                                    block_id: Some(block_id),
                                    details: None,
                                });
                            }
                            Ok(None) => {
                                // Could not find a suitable parent
                                result.unresolved.push(issue);
                            }
                            Err(e) => {
                                result.errors.push(format!(
                                    "Failed to fix orphan {}: {}",
                                    block_id, e
                                ));
                            }
                        }
                    } else {
                        result.unresolved.push(issue);
                    }
                }
                "unbalanced" => {
                    // Unbalanced sections cannot be auto-fixed (requires content decision)
                    // Just warn about them
                    result.unresolved.push(issue);
                }
                "forward-ref" => {
                    // Forward references cannot be safely auto-fixed
                    // Requires human decision about which block to create or remove
                    result.unresolved.push(issue);
                }
                "circular-ref" => {
                    // Circular references require human decision about which link to remove
                    result.unresolved.push(issue);
                }
                "anachronism" => {
                    // Anachronisms require content restructuring
                    result.unresolved.push(issue);
                }
                "gap" => {
                    // Gaps in FractionalIndex don't affect functionality
                    // No fix needed
                }
                _ => {
                    // Unknown issue type - cannot fix
                    result.unresolved.push(issue);
                }
            }
        }

        Ok(result)
    }

    /// Fix an orphan block by creating a NEXT link to an appropriate structure
    async fn fix_orphan_block(&self, block_id: Ulid) -> NexusResult<Option<String>> {
        // Get the orphan block
        let Some(block) = self.db.blocks().get(&block_id).await? else {
            return Ok(None);
        };

        // Get all structures
        let structures = self.db.blocks().list_by_type(BlockType::Structure).await?;

        if structures.is_empty() {
            // No structures to link to
            return Ok(None);
        }

        // Find the best structure to link to
        // Strategy: Find a structure that has no children yet, or the first one
        // This is a simple heuristic - could be improved with semantic analysis
        let target_structure = &structures[0];

        // Check if structure already has outgoing NEXT edges
        let existing_edges = self.db.edges().outgoing_from(&target_structure.id).await?;
        let next_edges: Vec<_> = existing_edges.iter()
            .filter(|e| e.link_type == LinkType::Next)
            .collect();

        let new_edge = if next_edges.is_empty() {
            // First child - create edge with first sequence weight
            Edge::next_in_sequence_first(target_structure.id, block_id)
        } else {
            // Find the last child and insert after it
            let last_edge = next_edges.last().unwrap();
            Edge::next_in_sequence_between(
                target_structure.id,
                block_id,
                &last_edge.sequence_weight,
                &crate::models::FractionalIndex::after_last(&last_edge.sequence_weight),
            )
        };

        // Create the edge
        self.db.edges().create(new_edge.clone()).await?;

        Ok(Some(format!(
            "Linked '{}' to structure '{}' as NEXT child",
            block.title, target_structure.title
        )))
    }

    /// Check for orphan blocks (blocks without incoming NEXT edge)
    pub async fn check_orphans(&self) -> NexusResult<Vec<LintIssue>> {
        let mut issues = Vec::new();

        // Get all non-structure blocks
        let all_blocks = self.db.blocks().list_all().await?;

        for block in all_blocks {
            // Skip structure blocks - they're valid roots
            if matches!(block.block_type, BlockType::Structure | BlockType::Outline) {
                continue;
            }

            // Check if block has incoming NEXT edge
            let incoming = self.db.edges().incoming_to(&block.id).await?;
            let has_next_parent = incoming.iter().any(|e| e.link_type == LinkType::Next);

            if !has_next_parent {
                issues.push(LintIssue {
                    code: "orphan".to_string(),
                    severity: LintSeverity::Error,
                    message: format!(
                        "Block '{}' has no NEXT link to any section",
                        block.title
                    ),
                    block_id: Some(block.id),
                    context: Some(
                        "Orphan blocks are not part of the spine hierarchy".to_string()
                    ),
                });
            }
        }

        Ok(issues)
    }

    /// Check for gaps in sequence weights
    ///
    /// NOTE: With FractionalIndex, numerical gaps don't cause ordering problems
    /// since lexicographic strings can always be inserted between any two values.
    /// This check is kept for structural analysis but doesn't produce warnings.
    pub async fn check_gaps(&self) -> NexusResult<Vec<LintIssue>> {
        // FractionalIndex eliminates the need for gap checking
        // since any two indices can have a new one inserted between them
        Ok(Vec::new())
    }

    /// Check for unbalanced section loads
    pub async fn check_density_balance(&self) -> NexusResult<Vec<LintIssue>> {
        let mut issues = Vec::new();

        // Get all structure blocks
        let structures = self.db.blocks().list_by_type(BlockType::Structure).await?;

        if structures.is_empty() {
            return Ok(issues);
        }

        // Count children for each structure
        let mut densities: Vec<(Ulid, &str, usize)> = Vec::new();
        for structure in &structures {
            let edges = self.db.edges().outgoing_from(&structure.id).await?;
            let next_count = edges.iter().filter(|e| e.link_type == LinkType::Next).count();
            densities.push((structure.id, structure.title.as_str(), next_count));
        }

        if densities.is_empty() {
            return Ok(issues);
        }

        // Calculate statistics
        let total: usize = densities.iter().map(|(_, _, d)| d).sum();
        let avg = total as f32 / densities.len() as f32;

        // Check each structure's density
        for (id, title, density) in &densities {
            if *density > 20 && *density as f32 > avg * 3.0 {
                // Very dense section
                issues.push(LintIssue {
                    code: "unbalanced".to_string(),
                    severity: LintSeverity::Warning,
                    message: format!(
                        "Section '{}' has {} blocks (avg: {:.0}, ratio: {:.1}x)",
                        title, density, avg, *density as f32 / avg
                    ),
                    block_id: Some(*id),
                    context: Some(
                        "Consider splitting this section into subsections".to_string()
                    ),
                });
            } else if *density <= 1 {
                // Very sparse section
                issues.push(LintIssue {
                    code: "unbalanced".to_string(),
                    severity: LintSeverity::Warning,
                    message: format!(
                        "Section '{}' has only {} block(s)",
                        title, density
                    ),
                    block_id: Some(*id),
                    context: Some(
                        "Consider merging with another section or adding content".to_string()
                    ),
                });
            }
        }

        Ok(issues)
    }

    /// Check for forward references (references to blocks that don't exist yet)
    pub async fn check_forward_references(&self) -> NexusResult<Vec<LintIssue>> {
        let mut issues = Vec::new();

        // Get all blocks
        let all_blocks = self.db.blocks().list_all().await?;
        let block_ids: std::collections::HashSet<_> = all_blocks.iter().map(|b| b.id).collect();

        // Check each block's content for references to non-existent blocks
        // This is a simple check looking for patterns like [block_id] or [[block_id]]
        // Simple regex to find block references like [ULID] or [[ULID]]
        let re = regex::Regex::new(r"\[\[([A-Z0-9]{26})\]\]").unwrap();

        for block in &all_blocks {
            // Check content for forward reference patterns
            let content = &block.content;

            for cap in re.captures_iter(content) {
                if let Some(id_str) = cap.get(1) {
                    let id: Result<Ulid, _> = id_str.as_str().parse();
                    if let Ok(id) = id
                        && !block_ids.contains(&id) {
                            issues.push(LintIssue {
                                code: "forward-ref".to_string(),
                                severity: LintSeverity::Error,
                                message: format!(
                                    "Block '{}' references non-existent block: {}",
                                    block.title, id
                                ),
                                block_id: Some(block.id),
                                context: Some(
                                    "Forward references break document synthesis".to_string()
                                ),
                            });
                        }
                }
            }
        }

        Ok(issues)
    }

    /// Check for circular references in the spine
    pub async fn check_circular_references(&self) -> NexusResult<Vec<LintIssue>> {
        let mut issues = Vec::new();

        // Get all structure blocks
        let structures = self.db.blocks().list_by_type(BlockType::Structure).await?;

        for structure in structures {
            // Check for cycles using DFS
            let mut visited = std::collections::HashSet::new();
            let mut stack = Vec::new();
            stack.push(structure.id);

            while let Some(current) = stack.pop() {
                if visited.contains(&current) {
                    issues.push(LintIssue {
                        code: "circular-ref".to_string(),
                        severity: LintSeverity::Error,
                        message: format!(
                            "Circular reference detected in spine involving '{}'",
                            structure.title
                        ),
                        block_id: Some(structure.id),
                        context: Some(
                            "Circular references cause infinite loops during traversal".to_string()
                        ),
                    });
                    break;
                }

                visited.insert(current);

                // Get NEXT edges
                let edges = self.db.edges().outgoing_from(&current).await?;
                for edge in edges.iter().filter(|e| e.link_type == LinkType::Next) {
                    stack.push(edge.to);
                }
            }
        }

        Ok(issues)
    }

    /// Check for anachronisms (blocks that reference concepts before they're introduced)
    pub async fn check_anachronisms(&self) -> NexusResult<Vec<LintIssue>> {
        let mut issues = Vec::new();

        // This is a simplified check that looks for blocks that mention
        // later sections before they're introduced
        //
        // A full implementation would require:
        // 1. Parsing content for section references
        // 2. Knowing the order of sections in the document
        // 3. Detecting when a block references a section that comes later

        // For now, we do a basic check: look for references to section titles
        // that appear in the content but whose blocks are placed later

        let structures = self.db.blocks().list_by_type(BlockType::Structure).await?;

        // Build a map of section titles to their order
        let mut section_titles: Vec<(Ulid, &str)> = structures
            .iter()
            .map(|s| (s.id, s.title.as_str()))
            .collect();

        // Sort by ULID (which is time-based, so earlier = created earlier)
        section_titles.sort_by(|a, b| a.0.cmp(&b.0));

        for (i, (section_id, section_title)) in section_titles.iter().enumerate() {
            let section = structures.iter().find(|s| s.id == *section_id).unwrap();

            // Check if content mentions any later section titles
            for later in section_titles.iter().skip(i + 1) {
                let later_title = later.1;
                if section.content.contains(later_title) {
                    issues.push(LintIssue {
                        code: "anachronism".to_string(),
                        severity: LintSeverity::Warning,
                        message: format!(
                            "Section '{}' mentions '{}' before it's introduced",
                            section_title, later_title
                        ),
                        block_id: Some(*section_id),
                        context: Some(
                            "Consider moving this reference after the referenced section".to_string()
                        ),
                    });
                }
            }
        }

        Ok(issues)
    }

    /// Get lint statistics
    pub async fn get_stats(&self, issues: &[LintIssue]) -> LintStats {
        let errors = issues.iter().filter(|i| i.severity == LintSeverity::Error).count();
        let warnings = issues.iter().filter(|i| i.severity == LintSeverity::Warning).count();
        let info = issues.iter().filter(|i| i.severity == LintSeverity::Info).count();

        LintStats {
            total: issues.len(),
            errors,
            warnings,
            info,
        }
    }
}

/// Lint statistics
#[derive(Debug, Clone)]
pub struct LintStats {
    pub total: usize,
    pub errors: usize,
    pub warnings: usize,
    pub info: usize,
}

impl std::fmt::Display for LintStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Lint Statistics:")?;
        writeln!(f, "   Total issues: {}", self.total)?;
        writeln!(f, "   Errors: {}", self.errors)?;
        writeln!(f, "   Warnings: {}", self.warnings)?;
        writeln!(f, "   Info: {}", self.info)?;
        Ok(())
    }
}

impl PartialEq for LintIssue {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code && self.severity == other.severity
    }
}

impl Eq for LintIssue {}

impl PartialOrd for LintIssue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LintIssue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // First compare by severity (reversed - errors first)
        let severity_cmp = other.severity.cmp(&self.severity);
        if severity_cmp != std::cmp::Ordering::Equal {
            return severity_cmp;
        }
        // Then by code
        self.code.cmp(&other.code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lint_issue_display() {
        let issue = LintIssue {
            code: "orphan".to_string(),
            severity: LintSeverity::Error,
            message: "Block has no NEXT link".to_string(),
            block_id: Some(Ulid::new()),
            context: None,
        };

        let display = format!("{}", issue);
        assert!(display.contains("orphan"));
        assert!(display.contains("Block has no NEXT link"));
    }

    #[test]
    fn test_lint_stats_display() {
        let stats = LintStats {
            total: 5,
            errors: 2,
            warnings: 2,
            info: 1,
        };

        let display = format!("{}", stats);
        assert!(display.contains("5"));
        assert!(display.contains("2"));
        assert!(display.contains("1"));
    }

    #[test]
    fn test_lint_severity_ordering() {
        assert!(LintSeverity::Error > LintSeverity::Warning);
        assert!(LintSeverity::Warning > LintSeverity::Info);
    }

    #[test]
    fn test_lint_issue_sorting() {
        let mut issues = vec![
            LintIssue {
                code: "warning1".to_string(),
                severity: LintSeverity::Warning,
                message: "".to_string(),
                block_id: None,
                context: None,
            },
            LintIssue {
                code: "error1".to_string(),
                severity: LintSeverity::Error,
                message: "".to_string(),
                block_id: None,
                context: None,
            },
            LintIssue {
                code: "info1".to_string(),
                severity: LintSeverity::Info,
                message: "".to_string(),
                block_id: None,
                context: None,
            },
        ];

        issues.sort();

        // Error should be first
        assert_eq!(issues[0].severity, LintSeverity::Error);
        // Warning should be second
        assert_eq!(issues[1].severity, LintSeverity::Warning);
        // Info should be last
        assert_eq!(issues[2].severity, LintSeverity::Info);
    }

    #[test]
    fn test_fix_result_display_empty() {
        let result = FixResult::default();
        let display = format!("{}", result);
        assert!(display.contains("No fixes applied"));
    }

    #[test]
    fn test_fix_result_display_with_fixes() {
        let mut result = FixResult::default();
        result.fixes.push(FixAction {
            code: "orphan".to_string(),
            description: "Linked block to structure".to_string(),
            block_id: Some(Ulid::new()),
            details: None,
        });

        let display = format!("{}", result);
        assert!(display.contains("Applied 1 fix(es)"));
        assert!(display.contains("orphan"));
        assert!(display.contains("Linked block to structure"));
    }

    #[test]
    fn test_fix_result_display_with_unresolved() {
        let mut result = FixResult::default();
        result.unresolved.push(LintIssue {
            code: "circular-ref".to_string(),
            severity: LintSeverity::Error,
            message: "Circular reference detected".to_string(),
            block_id: Some(Ulid::new()),
            context: None,
        });

        let display = format!("{}", result);
        assert!(display.contains("could not be auto-fixed"));
        assert!(display.contains("circular-ref"));
    }

    #[test]
    fn test_fix_result_display_with_errors() {
        let mut result = FixResult::default();
        result.errors.push("Database error".to_string());

        let display = format!("{}", result);
        assert!(display.contains("1 error(s) during fix"));
        assert!(display.contains("Database error"));
    }

    #[test]
    fn test_fix_result_has_fixes() {
        let mut result = FixResult::default();
        assert!(!result.has_fixes());

        result.fixes.push(FixAction {
            code: "orphan".to_string(),
            description: "Test".to_string(),
            block_id: None,
            details: None,
        });

        assert!(result.has_fixes());
        assert_eq!(result.fix_count(), 1);
    }

    #[test]
    fn test_fix_result_unresolved_count() {
        let mut result = FixResult::default();
        result.unresolved.push(LintIssue {
            code: "test".to_string(),
            severity: LintSeverity::Error,
            message: "Test".to_string(),
            block_id: None,
            context: None,
        });

        assert_eq!(result.unresolved_count(), 1);
    }
}
