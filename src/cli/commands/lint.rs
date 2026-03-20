//! Lint command: Structural integrity validation
//!
//! Runs structural lint rules to detect issues in the knowledge graph:
//! - Orphan blocks (no incoming links)
//! - Sequence gaps (missing weights)
//! - Circular references
//! - Unbalanced section loads
//! - Stale ghost nodes

use crate::db::Database;
use crate::spine::linting::{LintIssue, LintSeverity, StructuralLinter};

/// Execute the lint command
pub async fn execute(
    db: &Database,
    fix: bool,
) -> anyhow::Result<()> {
    println!("🔍 Running structural lint...");
    println!();

    // Create linter
    let linter = StructuralLinter::new(db);

    if fix {
        println!("🔧 Auto-fix enabled - attempting to resolve issues...");
        println!();

        // Run auto-fix
        let fix_result = linter.auto_fix().await?;

        // Print fix results
        if fix_result.has_fixes() {
            println!("✅ Fixes applied:");
            for fix_action in &fix_result.fixes {
                println!("   • [{}] {}", fix_action.code.to_uppercase(), fix_action.description);
                if let Some(id) = &fix_action.block_id {
                    println!("     Block: {}", id.to_string().chars().take(8).collect::<String>());
                }
                if let Some(details) = &fix_action.details {
                    println!("     Details: {}", details);
                }
            }
            println!();
        }

        if fix_result.unresolved_count() > 0 {
            println!("⚠️  {} issue(s) could not be auto-fixed:", fix_result.unresolved_count());
            for issue in &fix_result.unresolved {
                println!("   • [{}] {}", issue.code.to_uppercase(), issue.message);
            }
            println!();
        }

        if !fix_result.errors.is_empty() {
            println!("❌ {} error(s) during fix:", fix_result.errors.len());
            for err in &fix_result.errors {
                println!("   • {}", err);
            }
            println!();
        }

        // If we had fixes, run lint again to show remaining issues
        if fix_result.has_fixes() {
            let remaining_issues = linter.lint().await?;
            let remaining_stats = linter.get_stats(&remaining_issues).await;

            if remaining_issues.is_empty() {
                println!("✅ All structural issues have been resolved!");
                println!();
                println!("📊 Your knowledge graph is now well-structured.");
            } else {
                println!("📊 Remaining issues after fix:");
                println!();
                print_issues_summary(&remaining_issues, &remaining_stats).await;
            }
        }

        // If no fixes were applied, show the normal output
        if !fix_result.has_fixes() && fix_result.unresolved_count() == 0 && fix_result.errors.is_empty() {
            println!("✅ No issues found - nothing to fix.");
            println!();
        }

        return Ok(());
    }

    // Normal lint mode (no fix)
    let issues = linter.lint().await?;
    let stats = linter.get_stats(&issues).await;

    print_issues_summary(&issues, &stats).await;

    Ok(())
}

/// Print a summary of issues
async fn print_issues_summary(issues: &[LintIssue], stats: &crate::spine::linting::LintStats) {
    // Categorize findings
    let errors: Vec<_> = issues.iter().filter(|i| i.severity == LintSeverity::Error).collect();
    let warnings: Vec<_> = issues.iter().filter(|i| i.severity == LintSeverity::Warning).collect();
    let infos: Vec<_> = issues.iter().filter(|i| i.severity == LintSeverity::Info).collect();

    // Print findings
    if !errors.is_empty() {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("❌ ERRORS ({})", errors.len());
        println!();
        for issue in &errors {
            print_issue(issue);
        }
        println!();
    }

    if !warnings.is_empty() {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("⚠️  WARNINGS ({})", warnings.len());
        println!();
        for issue in &warnings {
            print_issue(issue);
        }
        println!();
    }

    if !infos.is_empty() {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("ℹ️  INFO ({})", infos.len());
        println!();
        for issue in &infos {
            print_issue(issue);
        }
        println!();
    }

    if issues.is_empty() {
        println!("✅ No structural issues found!");
        println!();
        println!("📊 Your knowledge graph is well-structured.");
        return;
    }

    // Calculate score
    let score = calculate_score(issues);
    let score_emoji = if score >= 90.0 {
        "✅"
    } else if score >= 70.0 {
        "⚠️"
    } else {
        "❌"
    };

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("📊 Score: {} {:.0}/100", score_emoji, score);
    println!();
    println!("   Errors: {} (15 pts each)", stats.errors);
    println!("   Warnings: {} (5 pts each)", stats.warnings);
    println!("   Info: {} (1 pt each)", stats.info);

    // Suggestions
    println!();
    println!("💡 Recommendations:");

    if !errors.is_empty() {
        let orphan_errors = errors.iter().filter(|i| i.code == "orphan").count();
        if orphan_errors > 0 {
            println!("   - {} orphan block(s) need linking", orphan_errors);
            println!("     Run `nexus lint --fix` to auto-fix");
        }

        let circular_errors = errors.iter().filter(|i| i.code == "circular-ref").count();
        if circular_errors > 0 {
            println!("   - {} circular reference(s) detected", circular_errors);
            println!("     Remove one link to break the cycle");
        }

        let forward_refs = errors.iter().filter(|i| i.code == "forward-ref").count();
        if forward_refs > 0 {
            println!("   - {} forward reference(s) detected", forward_refs);
            println!("     Move referenced blocks earlier or remove links");
        }
    }

    if !warnings.is_empty() {
        let gap_warnings = warnings.iter().filter(|i| i.code == "gap").count();
        if gap_warnings > 0 {
            println!("   - {} sequence gap(s) detected", gap_warnings);
            println!("     Consider renumbering to fill gaps");
        }

        let unbalanced_warnings = warnings.iter().filter(|i| i.code == "unbalanced").count();
        if unbalanced_warnings > 0 {
            println!("   - {} section(s) with unbalanced load", unbalanced_warnings);
            println!("     Consider refactoring with `nexus refactor split`");
        }

        let anachronisms = warnings.iter().filter(|i| i.code == "anachronism").count();
        if anachronisms > 0 {
            println!("   - {} anachronism(s) detected", anachronisms);
            println!("     Some sections reference content before it's introduced");
        }
    }

    if issues.is_empty() || (errors.is_empty() && warnings.is_empty()) {
        println!("   - No critical issues. Keep up the good work!");
    }

    println!();
    println!("💡 Tip: Run `nexus lint --fix` to auto-fix fixable issues");
}

/// Print a single issue
fn print_issue(issue: &LintIssue) {
    let severity_icon = match issue.severity {
        LintSeverity::Error => "❌",
        LintSeverity::Warning => "⚠️",
        LintSeverity::Info => "ℹ️",
    };

    println!("  {} [{}] {}", severity_icon, issue.code.to_uppercase(), issue.message);
    if let Some(id) = &issue.block_id {
        println!("     Block: {}", id.to_string().chars().take(8).collect::<String>());
    }
    if let Some(ctx) = &issue.context {
        println!("     💡 {}", ctx);
    }
    println!();
}

/// Calculate lint score (0-100)
fn calculate_score(issues: &[LintIssue]) -> f32 {
    if issues.is_empty() {
        return 100.0;
    }

    let error_count = issues.iter().filter(|i| i.severity == LintSeverity::Error).count();
    let warning_count = issues.iter().filter(|i| i.severity == LintSeverity::Warning).count();
    let info_count = issues.iter().filter(|i| i.severity == LintSeverity::Info).count();

    // Weighted penalty
    let penalty = (error_count * 15) as f32 + (warning_count * 5) as f32 + info_count as f32;
    (100.0 - penalty).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_calculation() {
        let empty: Vec<LintIssue> = vec![];
        assert_eq!(calculate_score(&empty), 100.0);

        let issue = LintIssue {
            code: "orphan".to_string(),
            severity: LintSeverity::Error,
            message: "Test".to_string(),
            block_id: None,
            context: None,
        };
        assert_eq!(calculate_score(&[issue]), 85.0);
    }

    #[test]
    fn test_lint_severity_display() {
        let issue = LintIssue {
            code: "test".to_string(),
            severity: LintSeverity::Warning,
            message: "Test message".to_string(),
            block_id: None,
            context: Some("Context".to_string()),
        };
        assert!(issue.message.contains("Test message"));
    }
}
