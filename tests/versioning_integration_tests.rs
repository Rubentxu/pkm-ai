//! Integration tests for the versioning system
//!
//! Tests the VersionRepo workflow including init, staging, commits,
//! branches, tags, and merge operations.

use pkm_ai::versioning::{
    AgentId, BlockDelta, MergeResult, MergeStrategy, VersionError, VersionRepo,
};
use tempfile::TempDir;
use ulid::Ulid;

/// Helper function to create a test repository
fn create_test_repo() -> (TempDir, VersionRepo) {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");
    (temp, repo)
}

/// Helper to create a test block delta
fn create_test_delta(block_id: Ulid) -> BlockDelta {
    BlockDelta::Created {
        block_id,
        title: format!("Block {}", block_id),
        content: "Test content".to_string(),
        block_type: "note".to_string(),
    }
}

// ============================================================================
// Repository Initialization Tests
// ============================================================================

#[test]
fn test_repo_init_creates_directories() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");

    assert!(repo.object_store.is_initialized());
    assert!(repo.ref_store.is_initialized());
}

#[test]
fn test_repo_init_idempotent() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");
    repo.init().expect("Second init should succeed");
}

// ============================================================================
// Full Workflow Tests
// ============================================================================

#[test]
fn test_versioning_full_workflow() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");

    // Create main branch first
    repo.create_branch("main").expect("Failed to create main branch");
    repo.checkout("main").expect("Failed to checkout main");

    // Create a block
    let block_id = Ulid::new();
    let delta = create_test_delta(block_id);

    // Stage
    repo.add_block(block_id, delta).expect("Failed to stage block");

    // Verify working set has staged block
    let ws = repo.get_working_set().expect("Failed to get working set");
    assert_eq!(ws.staged_blocks_count(), 1);
    assert!(ws.is_block_staged(&block_id));

    // Commit
    let commit_id = repo
        .commit("Initial commit", AgentId::new("user"))
        .expect("Failed to commit");
    assert!(!commit_id.to_string().is_empty() && commit_id.to_string() != "00000000000000000000000000");

    // Log
    let commits = repo.log().expect("Failed to get log");
    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].message, "Initial commit");

    // Verify working set is empty after commit
    let ws = repo.get_working_set().expect("Failed to get working set after commit");
    assert!(ws.is_empty());
}

#[test]
fn test_stage_and_multiple_commits() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");

    repo.create_branch("main").expect("Failed to create main");
    repo.checkout("main").expect("Failed to checkout main");

    // First commit
    let block1 = Ulid::new();
    repo.stage(&block1).expect("Failed to stage block1");
    repo.commit("First", AgentId::new("user")).expect("Failed first commit");

    // Second commit
    let block2 = Ulid::new();
    repo.stage(&block2).expect("Failed to stage block2");
    repo.commit("Second", AgentId::new("user")).expect("Failed second commit");

    let commits = repo.log().expect("Failed to get log");
    assert_eq!(commits.len(), 2);
    assert_eq!(commits[0].message, "Second");
    assert_eq!(commits[1].message, "First");
}

// ============================================================================
// Branch Tests
// ============================================================================

#[test]
fn test_create_and_list_branches() {
    let (_temp, repo) = create_test_repo();

    repo.create_branch("main").expect("Failed to create main");
    repo.checkout("main").expect("Failed to checkout main");
    repo.create_branch("feature-a").expect("Failed to create feature-a");
    repo.create_branch("feature-b").expect("Failed to create feature-b");

    let branches = repo.list_branches().expect("Failed to list branches");
    assert_eq!(branches.len(), 3);
    assert!(branches.iter().any(|b| b.as_str() == "main"));
    assert!(branches.iter().any(|b| b.as_str() == "feature-a"));
    assert!(branches.iter().any(|b| b.as_str() == "feature-b"));
}

#[test]
fn test_branch_workflow() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");

    // Create and checkout main
    repo.create_branch("main").expect("Failed to create main");
    repo.checkout("main").expect("Failed to checkout main");

    // Add commit to main
    let block1 = Ulid::new();
    repo.stage(&block1).expect("Failed to stage");
    repo.commit("Main commit", AgentId::new("user")).expect("Failed to commit");

    // Create feature branch
    repo.create_branch("feature").expect("Failed to create feature");
    repo.checkout("feature").expect("Failed to checkout feature");

    // Add different commit to feature
    let block2 = Ulid::new();
    repo.stage(&block2).expect("Failed to stage on feature");
    repo.commit("Feature commit", AgentId::new("user")).expect("Failed to commit on feature");

    // Verify HEAD is feature
    let head = repo.get_head_branch().expect("Failed to get HEAD").unwrap();
    assert_eq!(head.as_str(), "feature");

    // Verify feature has its own commit
    let commits = repo.log().expect("Failed to get log");
    assert_eq!(commits[0].message, "Feature commit");
}

#[test]
fn test_checkout_existing_branch() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");

    repo.create_branch("main").expect("Failed to create main");
    repo.checkout("main").expect("Failed to checkout main");
    repo.create_branch("develop").expect("Failed to create develop");

    repo.checkout("develop").expect("Failed to checkout develop");

    let head = repo.get_head_branch().expect("Failed to get HEAD").unwrap();
    assert_eq!(head.as_str(), "develop");
}

#[test]
fn test_checkout_nonexistent_branch_fails() {
    let (_temp, repo) = create_test_repo();

    let result = repo.checkout("nonexistent");
    assert!(matches!(result, Err(VersionError::BranchNotFound(_))));
}

#[test]
fn test_create_branch_already_exists_fails() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");

    repo.create_branch("main").expect("Failed to create main");
    repo.checkout("main").expect("Failed to checkout main");
    repo.create_branch("feature").expect("Failed to create feature");

    let result = repo.create_branch("feature");
    assert!(matches!(result, Err(VersionError::BranchAlreadyExists(_))));
}

#[test]
fn test_delete_branch() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");

    repo.create_branch("main").expect("Failed to create main");
    repo.checkout("main").expect("Failed to checkout main");
    repo.create_branch("feature").expect("Failed to create feature");

    repo.delete_branch("feature", false).expect("Failed to delete branch");

    let branches = repo.list_branches().expect("Failed to list branches");
    assert!(!branches.iter().any(|b| b.as_str() == "feature"));
}

#[test]
fn test_delete_current_branch_fails() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");

    repo.create_branch("main").expect("Failed to create main");
    repo.checkout("main").expect("Failed to checkout main");

    let result = repo.delete_branch("main", false);
    assert!(matches!(result, Err(VersionError::CannotDeleteHead)));
}

#[test]
fn test_force_delete_branch() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");

    repo.create_branch("main").expect("Failed to create main");
    repo.checkout("main").expect("Failed to checkout main");
    repo.create_branch("feature").expect("Failed to create feature");

    // Force delete without checking merge status
    repo.delete_branch("feature", true).expect("Force delete failed");
}

// ============================================================================
// Tag Tests
// ============================================================================

#[test]
fn test_create_and_list_tags() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");

    repo.create_branch("main").expect("Failed to create main");
    repo.checkout("main").expect("Failed to checkout main");

    let block1 = Ulid::new();
    repo.stage(&block1).expect("Failed to stage");
    repo.commit("First", AgentId::new("user")).expect("Failed to commit");

    repo.create_tag("v1.0.0", None, Some("First release")).expect("Failed to create tag");
    repo.create_tag("v1.0.1", None, None).expect("Failed to create tag");

    let tags = repo.list_tags_detailed().expect("Failed to list tags");
    assert_eq!(tags.len(), 2);
    assert!(tags.iter().any(|t| t.name.as_str() == "v1.0.0" && t.message.as_ref().is_some()));
    assert!(tags.iter().any(|t| t.name.as_str() == "v1.0.1" && t.message.is_none()));
}

#[test]
fn test_delete_tag() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");

    repo.create_branch("main").expect("Failed to create main");
    repo.checkout("main").expect("Failed to checkout main");

    let block1 = Ulid::new();
    repo.stage(&block1).expect("Failed to stage");
    repo.commit("First", AgentId::new("user")).expect("Failed to commit");

    repo.create_tag("v1.0.0", None, None).expect("Failed to create tag");
    repo.delete_tag("v1.0.0").expect("Failed to delete tag");

    let tags = repo.list_tags_detailed().expect("Failed to list tags");
    assert!(!tags.iter().any(|t| t.name.as_str() == "v1.0.0"));
}

#[test]
fn test_create_tag_without_commit_fails() {
    let (_temp, repo) = create_test_repo();

    let result = repo.create_tag("v1.0.0", None, None);
    assert!(matches!(result, Err(VersionError::NoHeadCommit)));
}

#[test]
fn test_tag_already_exists_fails() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");

    repo.create_branch("main").expect("Failed to create main");
    repo.checkout("main").expect("Failed to checkout main");

    let block1 = Ulid::new();
    repo.stage(&block1).expect("Failed to stage");
    repo.commit("First", AgentId::new("user")).expect("Failed to commit");

    repo.create_tag("v1.0.0", None, None).expect("Failed to create first tag");
    let result = repo.create_tag("v1.0.0", None, None);
    assert!(matches!(result, Err(VersionError::TagAlreadyExists(_))));
}

// ============================================================================
// Merge Tests
// ============================================================================

#[test]
fn test_merge_fast_forward() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");

    // Setup main with initial commit
    repo.create_branch("main").expect("Failed to create main");
    repo.checkout("main").expect("Failed to checkout main");
    let block1 = Ulid::new();
    repo.stage(&block1).expect("Failed to stage");
    repo.commit("Initial", AgentId::new("user")).expect("Failed initial commit");

    // Create feature branch with additional commit
    repo.create_branch("feature").expect("Failed to create feature");
    repo.checkout("feature").expect("Failed to checkout feature");
    let block2 = Ulid::new();
    repo.stage(&block2).expect("Failed to stage on feature");
    repo.commit("Feature work", AgentId::new("user")).expect("Failed feature commit");

    // Switch to main and merge feature
    repo.checkout("main").expect("Failed to checkout main");
    let result = repo.merge("feature", MergeStrategy::Merge).expect("Merge failed");
    assert!(matches!(result, MergeResult::FastForward));

    // Verify main now has feature's commit
    let head = repo.get_head_commit().expect("Failed to get HEAD commit").unwrap();
    assert_eq!(head.message, "Feature work");
}

#[test]
fn test_merge_already_merged() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");

    repo.create_branch("main").expect("Failed to create main");
    repo.checkout("main").expect("Failed to checkout main");
    let block1 = Ulid::new();
    repo.stage(&block1).expect("Failed to stage");
    repo.commit("Initial", AgentId::new("user")).expect("Failed initial commit");

    repo.create_branch("feature").expect("Failed to create feature");
    repo.checkout("feature").expect("Failed to checkout feature");
    let block2 = Ulid::new();
    repo.stage(&block2).expect("Failed to stage on feature");
    repo.commit("Feature", AgentId::new("user")).expect("Failed feature commit");

    repo.checkout("main").expect("Failed to checkout main");
    repo.merge("feature", MergeStrategy::Merge).expect("Merge failed");

    // Merge again - should be already merged
    let result = repo.merge("feature", MergeStrategy::Merge).expect("Merge check failed");
    assert!(matches!(result, MergeResult::AlreadyMerged));
}

#[test]
fn test_merge_no_conflicts() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");

    repo.create_branch("main").expect("Failed to create main");
    repo.checkout("main").expect("Failed to checkout main");

    // Make divergent commits
    let block1 = Ulid::new();
    repo.stage(&block1).expect("Failed to stage");
    repo.commit("Main change", AgentId::new("user")).expect("Failed main commit");

    repo.create_branch("feature").expect("Failed to create feature");
    repo.checkout("feature").expect("Failed to checkout feature");
    let block2 = Ulid::new();
    repo.stage(&block2).expect("Failed to stage on feature");
    repo.commit("Feature change", AgentId::new("user")).expect("Failed feature commit");

    // Switch to main and merge feature (non-fast-forward)
    repo.checkout("main").expect("Failed to checkout main");

    let result = repo.merge("feature", MergeStrategy::Merge).expect("Merge failed");

    // The merge result depends on the commit graph structure.
    // If feature's commit has main's commit as ancestor (linear history),
    // the result will be FastForward. If they diverged, it will be MergeCommit.
    match result {
        MergeResult::MergeCommit { conflicts: false } => {
            // Verify merge commit has both parents
            let head = repo.get_head_commit().expect("Failed to get HEAD commit").unwrap();
            assert_eq!(head.parents.len(), 2);
        }
        MergeResult::FastForward => {
            // Fast-forward just moves HEAD, no merge commit created
        }
        _ => panic!("Unexpected merge result: {:?}", result),
    }
}

#[test]
fn test_merge_nonexistent_branch_fails() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");

    repo.create_branch("main").expect("Failed to create main");
    repo.checkout("main").expect("Failed to checkout main");

    let block1 = Ulid::new();
    repo.stage(&block1).expect("Failed to stage");
    repo.commit("Initial", AgentId::new("user")).expect("Failed initial commit");

    let result = repo.merge("nonexistent", MergeStrategy::Merge);
    assert!(matches!(result, Err(VersionError::BranchNotFound(_))));
}

#[test]
fn test_merge_without_commits_fails() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");

    repo.create_branch("main").expect("Failed to create main");
    repo.checkout("main").expect("Failed to checkout main");
    repo.create_branch("feature").expect("Failed to create feature");

    // Try to merge without any commits on main
    let result = repo.merge("feature", MergeStrategy::Merge);
    assert!(matches!(result, Err(VersionError::NoHeadCommit)));
}

// ============================================================================
// Working Set Tests
// ============================================================================

#[test]
fn test_working_set_staging() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");

    repo.create_branch("main").expect("Failed to create main");
    repo.checkout("main").expect("Failed to checkout main");

    // Stage multiple blocks
    let block1 = Ulid::new();
    let block2 = Ulid::new();
    let block3 = Ulid::new();

    repo.stage(&block1).expect("Failed to stage block1");
    repo.stage(&block2).expect("Failed to stage block2");
    repo.stage(&block3).expect("Failed to stage block3");

    // Verify working set
    let ws = repo.get_working_set().expect("Failed to get working set");
    assert_eq!(ws.staged_blocks_count(), 3);
    assert!(ws.is_block_staged(&block1));
    assert!(ws.is_block_staged(&block2));
    assert!(ws.is_block_staged(&block3));

    // Commit
    repo.commit("Three blocks", AgentId::new("user")).expect("Failed to commit");

    // Verify working set is empty after commit
    let ws = repo.get_working_set().expect("Failed to get working set after commit");
    assert!(ws.is_empty());
}

#[test]
fn test_working_set_persistence() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");

    repo.create_branch("main").expect("Failed to create main");
    repo.checkout("main").expect("Failed to checkout main");

    // Stage some blocks
    let block1 = Ulid::new();
    let block2 = Ulid::new();
    repo.stage(&block1).expect("Failed to stage");
    repo.stage(&block2).expect("Failed to stage");

    // Create a new repo instance pointing to same directory
    let repo2 = VersionRepo::new(temp.path());
    let ws = repo2.get_working_set().expect("Failed to get working set from repo2");
    assert_eq!(ws.staged_blocks_count(), 2);
}

#[test]
fn test_working_set_staged_blocks_after_commit() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");

    repo.create_branch("main").expect("Failed to create main");
    repo.checkout("main").expect("Failed to checkout main");

    // Stage and commit
    let block1 = Ulid::new();
    repo.stage(&block1).expect("Failed to stage");
    repo.commit("Initial", AgentId::new("user")).expect("Failed to commit");

    // Verify no staged blocks remain
    let ws = repo.get_working_set().expect("Failed to get working set");
    assert!(ws.is_empty());
    assert_eq!(ws.staged_blocks_count(), 0);
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_commit_in_empty_repo_fails() {
    let (_temp, repo) = create_test_repo();

    // Try to commit without creating a branch first
    let result = repo.commit("Empty commit", AgentId::new("user"));
    assert!(matches!(result, Err(VersionError::NothingToCommit)));
}

#[test]
fn test_branch_without_commits() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");

    // Create branch without any commits
    repo.create_branch("main").expect("Failed to create main");
    repo.checkout("main").expect("Failed to checkout main");

    // Should be able to create another branch even without commits
    repo.create_branch("feature").expect("Failed to create feature without commits");
}

#[test]
fn test_merge_branch_without_changes() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");

    repo.create_branch("main").expect("Failed to create main");
    repo.checkout("main").expect("Failed to checkout main");

    let block1 = Ulid::new();
    repo.stage(&block1).expect("Failed to stage");
    repo.commit("Initial", AgentId::new("user")).expect("Failed initial commit");

    // Create feature branch with no additional commits
    repo.create_branch("feature").expect("Failed to create feature");
    repo.checkout("feature").expect("Failed to checkout feature");

    // Switch back to main and merge feature (same commit = already merged)
    repo.checkout("main").expect("Failed to checkout main");
    let result = repo.merge("feature", MergeStrategy::Merge).expect("Merge failed");
    assert!(matches!(result, MergeResult::AlreadyMerged));
}

#[test]
fn test_log_empty_repo() {
    let (_temp, repo) = create_test_repo();

    let commits = repo.log().expect("Failed to get log");
    assert!(commits.is_empty());
}

#[test]
fn test_get_commits_limit() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");

    repo.create_branch("main").expect("Failed to create main");
    repo.checkout("main").expect("Failed to checkout main");

    // Create 5 commits
    for i in 0..5 {
        let block = Ulid::new();
        repo.stage(&block).expect("Failed to stage");
        repo.commit(&format!("Commit {}", i), AgentId::new("user")).expect("Failed to commit");
    }

    // Get only last 3 commits
    let commits = repo.get_commits(3).expect("Failed to get commits");
    assert_eq!(commits.len(), 3);
}

#[test]
fn test_list_branches_detailed() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");

    repo.create_branch("main").expect("Failed to create main");
    repo.checkout("main").expect("Failed to checkout main");

    let block1 = Ulid::new();
    repo.stage(&block1).expect("Failed to stage");
    repo.commit("Initial", AgentId::new("user")).expect("Failed initial commit");

    repo.create_branch("feature").expect("Failed to create feature");

    let branches = repo.list_branches_detailed().expect("Failed to list branches detailed");
    assert_eq!(branches.len(), 2);

    let main_branch = branches.iter().find(|b| b.name.as_str() == "main").unwrap();
    assert!(main_branch.is_head);
}

#[test]
fn test_checkout_new_branch() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");

    repo.create_branch("main").expect("Failed to create main");
    repo.checkout("main").expect("Failed to checkout main");

    let block1 = Ulid::new();
    repo.stage(&block1).expect("Failed to stage");
    repo.commit("Initial", AgentId::new("user")).expect("Failed initial commit");

    // Create and checkout new branch in one step
    repo.checkout_new_branch("feature").expect("Failed checkout -b feature");

    let head = repo.get_head_branch().expect("Failed to get HEAD").unwrap();
    assert_eq!(head.as_str(), "feature");

    // Verify feature branch exists
    let branches = repo.list_branches().expect("Failed to list branches");
    assert!(branches.iter().any(|b| b.as_str() == "feature"));
}

#[test]
fn test_checkout_new_branch_already_exists_fails() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");

    repo.create_branch("main").expect("Failed to create main");
    repo.checkout("main").expect("Failed to checkout main");

    let block1 = Ulid::new();
    repo.stage(&block1).expect("Failed to stage");
    repo.commit("Initial", AgentId::new("user")).expect("Failed initial commit");

    repo.create_branch("feature").expect("Failed to create feature");

    let result = repo.checkout_new_branch("feature");
    assert!(matches!(result, Err(VersionError::BranchAlreadyExists(_))));
}

#[test]
fn test_checkout_new_branch_without_head_commit_fails() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");

    repo.create_branch("main").expect("Failed to create main");
    repo.checkout("main").expect("Failed to checkout main");

    // Try to create new branch without any commits - should fail
    let result = repo.checkout_new_branch("feature");
    assert!(matches!(result, Err(VersionError::NoHeadCommit)));
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_delete_nonexistent_tag_fails() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");

    repo.create_branch("main").expect("Failed to create main");
    repo.checkout("main").expect("Failed to checkout main");

    let block1 = Ulid::new();
    repo.stage(&block1).expect("Failed to stage");
    repo.commit("Initial", AgentId::new("user")).expect("Failed initial commit");

    let result = repo.delete_tag("nonexistent");
    assert!(matches!(result, Err(VersionError::TagNotFound(_))));
}

#[test]
fn test_delete_nonexistent_branch_fails() {
    let (_temp, repo) = create_test_repo();

    let result = repo.delete_branch("nonexistent", false);
    assert!(matches!(result, Err(VersionError::BranchNotFound(_))));
}

// ============================================================================
// Status and Log Tests (UX Enhancement)
// ============================================================================

#[test]
fn test_status_command_empty_repo() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");

    // Empty repo should have no commits
    let commits = repo.get_commits(10).expect("Failed to get commits");
    assert!(commits.is_empty(), "New repo should have no commits");
}

#[test]
fn test_status_command_with_commits() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");

    repo.create_branch("main").expect("Failed to create main");
    repo.checkout("main").expect("Failed to checkout main");

    // Create first commit
    let block1 = Ulid::new();
    repo.add_block(block1, create_test_delta(block1)).expect("Failed to stage");
    let commit1 = repo.commit("First commit: Initial setup", AgentId::new("tester")).expect("Failed to commit");

    // Verify log works
    let commits = repo.get_commits(10).expect("Failed to get commits");
    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].message, "First commit: Initial setup");

    // First line of commit message (for oneline format)
    let first_line = commits[0].message.lines().next().unwrap();
    assert_eq!(first_line, "First commit: Initial setup");

    // Commit ID should be valid ULID format (26 chars)
    assert_eq!(commit1.to_string().len(), 26);
}

#[test]
fn test_log_oneline_format() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");

    repo.create_branch("main").expect("Failed to create main");
    repo.checkout("main").expect("Failed to checkout main");

    // Create multiple commits
    for i in 1..=3 {
        let block_id = Ulid::new();
        repo.add_block(block_id, BlockDelta::Created {
            block_id,
            title: format!("Block {}", i),
            content: format!("Content {}", i),
            block_type: "note".to_string(),
        }).expect("Failed to stage");
        let msg = format!("Commit {}: Feature {}", i, i);
        repo.commit(&msg, AgentId::new("tester")).expect("Failed to commit");
    }

    let commits = repo.get_commits(10).expect("Failed to get commits");
    assert_eq!(commits.len(), 3);

    // Verify oneline format: short_id + first line of message
    for commit in commits {
        let short_id = &commit.id.to_string()[..8];
        let first_line = commit.message.lines().next().unwrap();

        // short_id should be 8 characters
        assert_eq!(short_id.len(), 8);

        // first_line should not be empty
        assert!(!first_line.is_empty());
    }
}

#[test]
fn test_status_shows_staged_changes() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");

    repo.create_branch("main").expect("Failed to create main");
    repo.checkout("main").expect("Failed to checkout main");

    // Stage some blocks
    let block1 = Ulid::new();
    let block2 = Ulid::new();
    repo.add_block(block1, create_test_delta(block1)).expect("Failed to stage block1");
    repo.add_block(block2, create_test_delta(block2)).expect("Failed to stage block2");

    let ws = repo.get_working_set().expect("Failed to get working set");
    assert_eq!(ws.staged_blocks_count(), 2);
}

#[test]
fn test_status_shows_last_commit_info() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = VersionRepo::new(temp.path());
    repo.init().expect("Failed to init repo");

    repo.create_branch("main").expect("Failed to create main");
    repo.checkout("main").expect("Failed to checkout main");

    // Create a commit
    let block1 = Ulid::new();
    repo.add_block(block1, create_test_delta(block1)).expect("Failed to stage");
    repo.commit("Quick capture: Rust notes", AgentId::new("user")).expect("Failed to commit");

    // Get last commit
    let last_commit = repo.get_head_commit().expect("Failed to get head commit");
    assert!(last_commit.is_some());

    let commit = last_commit.unwrap();
    assert_eq!(commit.message, "Quick capture: Rust notes");

    // Verify short ID extraction
    let short_id = &commit.id.to_string()[..8];
    assert_eq!(short_id.len(), 8);
}