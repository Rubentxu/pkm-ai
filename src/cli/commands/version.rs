//! Version commands: status, log, diff, add, commit
//!
//! Implements Git-like versioning commands for the PKM system.

use std::path::PathBuf;
use ulid::Ulid;

use pkm_ai::versioning::{
    AgentId, BlockDelta, Commit, CommitId, MergeResult, MergeStrategy, VersionRepo, VersionError,
    ViewName,
};

// ============================================================================
// Status Command
// ============================================================================

pub async fn status(repo_path: Option<&str>) -> anyhow::Result<()> {
    let root = resolve_repo_path(repo_path);
    let repo = VersionRepo::new(&root);

    // Try to initialize if not exists
    let _ = repo.init();

    // Show branch info
    let branch = repo.get_head_branch()?.map(|n| n.to_string()).unwrap_or_else(|| "(no branch)".to_string());
    println!("\n  📍 On branch: {}", branch);

    let ws = repo.get_working_set()?;
    let staged_blocks = ws.staged_blocks().len();
    let staged_edges = ws.staged_edges().len();
    let removed_blocks = ws.removed_blocks_count();
    let removed_edges = ws.removed_edges_count();

    // Show staged changes
    println!("\n  ✅ Staged changes:");
    if staged_blocks == 0 && staged_edges == 0 && removed_blocks == 0 && removed_edges == 0 {
        println!("     (no staged changes)");
    } else {
        if staged_blocks > 0 {
            println!("     {} block(s)", staged_blocks);
        }
        if staged_edges > 0 {
            println!("     {} link(s)", staged_edges);
        }
        if removed_blocks > 0 {
            println!("     {} block(s) marked for removal", removed_blocks);
        }
        if removed_edges > 0 {
            println!("     {} link(s) marked for removal", removed_edges);
        }
    }

    // Show unstaged changes (if any tracked locally)
    println!("\n  📝 Unstaged changes:");
    println!("     (none)");

    // Show last commit info
    if let Ok(Some(last_commit)) = repo.get_head_commit() {
        let short_id = &last_commit.id.to_string()[..8];
        let first_line = last_commit.message.lines().next().unwrap_or("");
        println!("\n  🎯 Last commit: {} \"{}\"", short_id, first_line);
    } else {
        println!("\n  🎯 No commits yet");
    }

    println!();

    Ok(())
}

// ============================================================================
// Log Command
// ============================================================================

pub async fn log(repo_path: Option<&str>, oneline: bool, _graph: bool, limit: usize) -> anyhow::Result<()> {
    let root = resolve_repo_path(repo_path);
    let repo = VersionRepo::new(&root);
    let _ = repo.init();

    let commits = repo.get_commits(limit)?;

    if commits.is_empty() {
        println!("No commits yet.");
        return Ok(());
    }

    for commit in commits {
        if oneline {
            let short_id = &commit.id.to_string()[..8];
            let message = commit.message.lines().next().unwrap_or("");
            println!("{} {}", short_id, message);
        } else {
            println!("╔══════════════════════════════════════════════════════════╗");
            println!("║ commit {}                                      ║", commit.id);
            println!("╠══════════════════════════════════════════════════════════╣");
            println!("║ Author: {:<46} ║", commit.author);
            println!("║ Date:   {:<46} ║", commit.created_at.format("%Y-%m-%d %H:%M:%S"));
            println!("╠══════════════════════════════════════════════════════════╣");
            println!("║ Message:                                                      ║");
            for line in commit.message.lines() {
                println!("║   {}                                                       ║", line);
            }
            if !commit.blocks_added.is_empty() {
                println!("╠══════════════════════════════════════════════════════════╣");
                println!("║ Blocks added: {}                                              ║", commit.blocks_added.len());
            }
            if !commit.blocks_removed.is_empty() {
                println!("║ Blocks removed: {}                                            ║", commit.blocks_removed.len());
            }
            println!("╚══════════════════════════════════════════════════════════╝");
            println!();
        }
    }

    Ok(())
}

// ============================================================================
// Diff Command
// ============================================================================

pub async fn diff(repo_path: Option<&str>, block_id: Option<&str>) -> anyhow::Result<()> {
    let root = resolve_repo_path(repo_path);
    let repo = VersionRepo::new(&root);
    let _ = repo.init();

    match block_id {
        Some(id) => {
            let ulid = Ulid::from_string(id)
                .map_err(|_| anyhow::anyhow!("Invalid block ID: {}", id))?;
            let diff_result = repo.get_block_diff(ulid, None, None)?;
            match diff_result {
                Some(diff) => println!("{}", diff),
                None => println!("No diff available for block {}", id),
            }
        }
        None => {
            println!("Usage: nexus diff <block-id>");
        }
    }

    Ok(())
}

// ============================================================================
// Add Command (Stage)
// ============================================================================

pub async fn add(repo_path: Option<&str>, block_id: &str) -> anyhow::Result<()> {
    let root = resolve_repo_path(repo_path);
    let repo = VersionRepo::new(&root);
    let _ = repo.init();

    let block_ulid = Ulid::from_string(block_id)
        .map_err(|_| anyhow::anyhow!("Invalid block ID: {}", block_id))?;

    // Create a BlockDelta for the block (simplified - assume created)
    let delta = BlockDelta::Created {
        block_id: block_ulid,
        title: format!("Block {}", block_id),
        content: String::new(),
        block_type: "note".to_string(),
    };

    repo.add_block(block_ulid, delta)?;

    println!("Staged block: {}", block_id);
    Ok(())
}

// ============================================================================
// Commit Command
// ============================================================================

pub async fn commit(repo_path: Option<&str>, message: &str, author_name: &str, amend: bool, no_edit: bool) -> anyhow::Result<()> {
    let root = resolve_repo_path(repo_path);
    let repo = VersionRepo::new(&root);
    let _ = repo.init();

    let author = AgentId::new(author_name);

    if amend {
        return commit_amend(&repo, message, no_edit).await;
    }

    match repo.commit(message, author) {
        Ok(commit_id) => {
            println!("Created commit: {}", commit_id);
        }
        Err(VersionError::NothingToCommit) => {
            println!("Nothing to commit (working tree clean)");
        }
        Err(e) => {
            return Err(anyhow::anyhow!("Commit failed: {}", e));
        }
    }

    Ok(())
}

/// Amend the last commit
async fn commit_amend(repo: &VersionRepo, message: &str, no_edit: bool) -> anyhow::Result<()> {
    // Get the last commit
    let last_commit = repo.get_head_commit()?
        .ok_or_else(|| anyhow::anyhow!("No commits to amend"))?;

    // Get current working set
    let ws = repo.get_working_set()?;

    // If no changes and no message change, just return
    if ws.is_empty() && (no_edit || message.is_empty()) {
        println!("Nothing to amend.");
        return Ok(());
    }

    // Get the new message
    let new_message = if no_edit {
        last_commit.message.clone()
    } else {
        message.to_string()
    };

    // Get working set changes if any
    let blocks_added: Vec<Ulid> = ws.staged_blocks().keys().cloned().collect();
    let blocks_removed = ws.removed_blocks().to_vec();

    // Create new commit with same parents but new changes
    let new_commit_id = CommitId::new(Ulid::new());
    let structure = pkm_ai::versioning::StructureSnapshot {
        id: Ulid::new(),
        block_order: if blocks_added.is_empty() {
            last_commit.structure_snapshot.block_order.clone()
        } else {
            blocks_added.clone()
        },
        edges: Vec::new(),
    };

    let new_commit = Commit {
        id: new_commit_id,
        structure_snapshot: structure,
        parents: last_commit.parents.clone(),
        author: last_commit.author.clone(),
        message: new_message,
        created_at: chrono::Utc::now(),
        blocks_added,
        blocks_removed,
        blocks_modified: Vec::new(),
    };

    // Save new commit
    repo.object_store.put_commit(&new_commit).map_err(VersionError::ObjectStore)?;

    // Update HEAD branch to point to new commit
    if let Some(head_name) = repo.get_head_branch()? {
        let mut view = repo.ref_store.get_branch(&head_name).map_err(VersionError::RefStore)?;
        view.set_target(new_commit_id.as_ulid());
        repo.ref_store.put_branch(&view).map_err(VersionError::RefStore)?;
    }

    // Clear working set
    repo.working_set_store.clear().map_err(VersionError::WorkingSetStore)?;

    println!("Amended commit: {}", new_commit_id);
    Ok(())
}

// ============================================================================
// Branch Command
// ============================================================================

pub async fn branch_list(repo_path: Option<&str>) -> anyhow::Result<()> {
    let root = resolve_repo_path(repo_path);
    let repo = VersionRepo::new(&root);
    let _ = repo.init();

    let branches = repo.list_branches_detailed()?;

    if branches.is_empty() {
        println!("No branches yet.");
        return Ok(());
    }

    let _current_head = repo.get_head_branch()?;

    for branch in branches {
        let mark = if branch.is_head { "*" } else { " " };
        let name = branch.name;
        let short_target = &branch.target.to_string()[..8];
        println!("{} {} -> {}", mark, name, short_target);
    }

    Ok(())
}

pub async fn branch_create(repo_path: Option<&str>, name: &str) -> anyhow::Result<()> {
    let root = resolve_repo_path(repo_path);
    let repo = VersionRepo::new(&root);
    let _ = repo.init();

    match repo.create_branch(name) {
        Ok(view_name) => {
            println!("Created branch: {}", view_name);
        }
        Err(VersionError::BranchAlreadyExists(_)) => {
            println!("Branch '{}' already exists.", name);
        }
        Err(e) => {
            return Err(anyhow::anyhow!("Failed to create branch: {}", e));
        }
    }

    Ok(())
}

pub async fn branch_delete(repo_path: Option<&str>, name: &str, force: bool) -> anyhow::Result<()> {
    let root = resolve_repo_path(repo_path);
    let repo = VersionRepo::new(&root);
    let _ = repo.init();

    match repo.delete_branch(name, force) {
        Ok(()) => {
            println!("Deleted branch: {}", name);
        }
        Err(VersionError::BranchNotFound(_)) => {
            println!("Branch '{}' not found.", name);
        }
        Err(VersionError::CannotDeleteHead) => {
            println!("Cannot delete the current HEAD branch.");
        }
        Err(e) => {
            return Err(anyhow::anyhow!("Failed to delete branch: {}", e));
        }
    }

    Ok(())
}

// ============================================================================
// Checkout Command
// ============================================================================

pub async fn checkout(repo_path: Option<&str>, branch_name: &str, create_new: bool) -> anyhow::Result<()> {
    let root = resolve_repo_path(repo_path);
    let repo = VersionRepo::new(&root);
    let _ = repo.init();

    let result = if create_new {
        repo.checkout_new_branch(branch_name)
    } else {
        repo.checkout(branch_name)
    };

    match result {
        Ok(view_name) => {
            println!("Switched to branch: {}", view_name);
        }
        Err(VersionError::BranchNotFound(_)) => {
            println!("Branch '{}' not found.", branch_name);
            println!("Hint: use 'checkout -b {}' to create and switch to a new branch.", branch_name);
        }
        Err(VersionError::BranchAlreadyExists(_)) => {
            println!("Branch '{}' already exists.", branch_name);
        }
        Err(e) => {
            return Err(anyhow::anyhow!("Checkout failed: {}", e));
        }
    }

    Ok(())
}

// ============================================================================
// Merge Command
// ============================================================================

pub async fn merge(repo_path: Option<&str>, branch_name: &str, strategy: Option<&str>) -> anyhow::Result<()> {
    let root = resolve_repo_path(repo_path);
    let repo = VersionRepo::new(&root);
    let _ = repo.init();

    let merge_strategy = match strategy {
        Some("ours") => MergeStrategy::Ours,
        Some("theirs") => MergeStrategy::Theirs,
        _ => MergeStrategy::Merge,
    };

    match repo.merge(branch_name, merge_strategy) {
        Ok(MergeResult::AlreadyMerged) => {
            println!("Branch '{}' is already merged.", branch_name);
        }
        Ok(MergeResult::FastForward) => {
            println!("Fast-forward merge to branch '{}'.", branch_name);
        }
        Ok(MergeResult::Clean { .. }) => {
            println!("Merge made with strategy: {:?}.", merge_strategy);
        }
        Ok(MergeResult::Conflicts { .. }) => {
            println!("Merge with conflicts. Please resolve manually.");
        }
        Err(VersionError::BranchNotFound(_)) => {
            println!("Branch '{}' not found.", branch_name);
        }
        Err(VersionError::NoHeadCommit) => {
            println!("No commits yet. Cannot merge.");
        }
        Err(e) => {
            return Err(anyhow::anyhow!("Merge failed: {}", e));
        }
    }

    Ok(())
}

// ============================================================================
// Tag Command
// ============================================================================

pub async fn tag_list(repo_path: Option<&str>) -> anyhow::Result<()> {
    let root = resolve_repo_path(repo_path);
    let repo = VersionRepo::new(&root);
    let _ = repo.init();

    let tags = repo.list_tags_detailed()?;

    if tags.is_empty() {
        println!("No tags yet.");
        return Ok(());
    }

    for tag in tags {
        let short_target = &tag.target.to_string()[..8];
        if let Some(msg) = tag.message {
            println!("{} {} - {}", tag.name, short_target, msg);
        } else {
            println!("{} {}", tag.name, short_target);
        }
    }

    Ok(())
}

pub async fn tag_create(repo_path: Option<&str>, name: &str, commit_id: Option<&str>, message: Option<&str>) -> anyhow::Result<()> {
    let root = resolve_repo_path(repo_path);
    let repo = VersionRepo::new(&root);
    let _ = repo.init();

    let commit = commit_id.and_then(|id| {
        Ulid::from_string(id).ok().map(CommitId::new)
    });

    match repo.create_tag(name, commit, message) {
        Ok(view_name) => {
            println!("Created tag: {}", view_name);
        }
        Err(VersionError::TagAlreadyExists(_)) => {
            println!("Tag '{}' already exists.", name);
        }
        Err(VersionError::NoHeadCommit) => {
            println!("No commits yet. Cannot create tag.");
        }
        Err(e) => {
            return Err(anyhow::anyhow!("Failed to create tag: {}", e));
        }
    }

    Ok(())
}

pub async fn tag_delete(repo_path: Option<&str>, name: &str) -> anyhow::Result<()> {
    let root = resolve_repo_path(repo_path);
    let repo = VersionRepo::new(&root);
    let _ = repo.init();

    match repo.delete_tag(name) {
        Ok(()) => {
            println!("Deleted tag: {}", name);
        }
        Err(VersionError::TagNotFound(_)) => {
            println!("Tag '{}' not found.", name);
        }
        Err(e) => {
            return Err(anyhow::anyhow!("Failed to delete tag: {}", e));
        }
    }

    Ok(())
}

// ============================================================================
// Log --grep Command
// ============================================================================

pub async fn log_grep(repo_path: Option<&str>, pattern: &str, limit: usize) -> anyhow::Result<()> {
    let root = resolve_repo_path(repo_path);
    let repo = VersionRepo::new(&root);
    let _ = repo.init();

    let commits = repo.get_commits(limit)?;
    let pattern_lower = pattern.to_lowercase();
    let mut found = false;

    for commit in commits {
        if commit.message.to_lowercase().contains(&pattern_lower) {
            found = true;
            println!("commit {}", commit.id);
            println!("Author: {}", commit.author);
            println!("Date: {}", commit.created_at.format("%Y-%m-%d %H:%M:%S"));
            println!();
            println!("    {}", commit.message.lines().next().unwrap_or(""));
            println!();
        }
    }

    if !found {
        println!("No commits found matching '{}'", pattern);
    }

    Ok(())
}

// ============================================================================
// Reset Command
// ============================================================================

pub async fn reset(repo_path: Option<&str>, soft: bool, hard: bool, commit_ref: Option<&str>) -> anyhow::Result<()> {
    let root = resolve_repo_path(repo_path);
    let repo = VersionRepo::new(&root);
    let _ = repo.init();

    // Get current HEAD commit
    let current_head = repo.get_head_commit()?
        .ok_or_else(|| anyhow::anyhow!("No commits to reset"))?;

    // Determine target commit
    let target_id = if let Some(refspec) = commit_ref {
        // Parse commit reference (ULID or relative like HEAD~1)
        if let Some(stripped) = refspec.strip_prefix("HEAD~") {
            let steps: usize = stripped.parse()
                .map_err(|_| anyhow::anyhow!("Invalid HEAD reference: {}", refspec))?;
            let mut commit = current_head;
            for _ in 0..steps {
                commit = repo.object_store.get_commit(CommitId::new(commit.parents.first()
                    .ok_or_else(|| anyhow::anyhow!("Cannot go back further"))?
                    .as_ulid()))
                    .map_err(VersionError::ObjectStore)?;
            }
            commit.id
        } else {
            // Treat as ULID
            let ulid = Ulid::from_string(refspec)
                .map_err(|_| anyhow::anyhow!("Invalid commit ID: {}", refspec))?;
            CommitId::new(ulid)
        }
    } else {
        // Default to HEAD~1 (parent of current)
        *current_head.parents.first()
            .ok_or_else(|| anyhow::anyhow!("Cannot reset - no parent commit"))?
    };

    // Verify target commit exists
    let _target_commit = repo.object_store.get_commit(target_id)
        .map_err(|e| anyhow::anyhow!("Target commit not found: {}", e))?;

    // Handle hard reset first (most destructive)
    if hard {
        // Clear working set completely
        repo.working_set_store.clear().map_err(VersionError::WorkingSetStore)?;
        println!("Discarded all working directory changes.");
    } else if soft {
        // Soft reset: just leave staged changes as they are (working set already has them)
        println!("Soft reset - changes kept in staging area.");
    }

    // Update HEAD to point to target commit
    if let Some(head_name) = repo.get_head_branch()? {
        let mut view = repo.ref_store.get_branch(&head_name).map_err(VersionError::RefStore)?;
        view.set_target(target_id.as_ulid());
        repo.ref_store.put_branch(&view).map_err(VersionError::RefStore)?;
    }

    let short_id = &target_id.to_string()[..8];
    println!("Reset to commit: {}", short_id);

    Ok(())
}

// ============================================================================
// Rebase Command
// ============================================================================

pub async fn rebase(repo_path: Option<&str>, branch_name: &str) -> anyhow::Result<()> {
    let root = resolve_repo_path(repo_path);
    let repo = VersionRepo::new(&root);
    let _ = repo.init();

    let view_name = ViewName::new(branch_name);

    // Check if branch exists
    if !repo.ref_store.has_branch(&view_name) {
        return Err(anyhow::anyhow!("Branch '{}' not found", branch_name));
    }

    // Get current HEAD info
    let current_head = repo.get_head_commit()
        .map_err(|e| anyhow::anyhow!("Failed to get current HEAD: {}", e))?
        .ok_or_else(|| anyhow::anyhow!("No current HEAD commit"))?;

    // Get target branch's HEAD commit
    let branch_view = repo.ref_store.get_branch(&view_name).map_err(VersionError::RefStore)?;
    let branch_commit = repo.object_store.get_commit(CommitId::new(branch_view.target()))
        .map_err(|e| anyhow::anyhow!("Failed to get branch commit: {}", e))?;

    // Get the current branch name
    let current_branch = repo.get_head_branch()?
        .ok_or_else(|| anyhow::anyhow!("Not on any branch"))?;

    println!("Rebasing {} onto {}...", current_branch, branch_name);

    // Collect commits to replay (from current HEAD back to but not including the merge base)
    // For simplicity, we replay all commits from current HEAD back to the branch point
    let mut commits_to_replay = Vec::new();
    let mut current: Option<Commit> = Some(current_head);

    while let Some(commit) = current {
        // Stop if we hit the branch commit (rebase base)
        if commit.id == branch_commit.id {
            break;
        }
        // Stop if we run out of parents (shouldn't happen in normal case)
        if commit.parents.is_empty() {
            break;
        }
        // Get parent ID before moving commit
        let parent_id = commit.parents.first().unwrap().as_ulid();
        commits_to_replay.push(commit);
        current = repo.object_store.get_commit(CommitId::new(parent_id)).ok();
    }

    if commits_to_replay.is_empty() {
        println!("Nothing to rebase - current branch is already up to date.");
        return Ok(());
    }

    println!("Found {} commits to replay.", commits_to_replay.len());

    // Replay commits in reverse order (oldest first)
    commits_to_replay.reverse();

    let mut new_base_id = branch_commit.id;

    for old_commit in commits_to_replay {
        // Create new commit with same message and author, but new parent
        let new_commit_id = CommitId::new(Ulid::new());
        let structure = pkm_ai::versioning::StructureSnapshot {
            id: Ulid::new(),
            block_order: old_commit.structure_snapshot.block_order.clone(),
            edges: old_commit.structure_snapshot.edges.clone(),
        };

        let new_commit = Commit {
            id: new_commit_id,
            structure_snapshot: structure,
            parents: vec![new_base_id],
            author: old_commit.author.clone(),
            message: old_commit.message.clone(),
            created_at: chrono::Utc::now(),
            blocks_added: old_commit.blocks_added.clone(),
            blocks_removed: old_commit.blocks_removed.clone(),
            blocks_modified: old_commit.blocks_modified.clone(),
        };

        repo.object_store.put_commit(&new_commit).map_err(VersionError::ObjectStore)?;

        let short_old = &old_commit.id.to_string()[..8];
        let short_new = &new_commit_id.to_string()[..8];
        println!("  {} -> {}", short_old, short_new);

        new_base_id = new_commit_id;
    }

    // Update current branch to point to the last new commit
    let mut view = repo.ref_store.get_branch(&current_branch).map_err(VersionError::RefStore)?;
    view.set_target(new_base_id.as_ulid());
    repo.ref_store.put_branch(&view).map_err(VersionError::RefStore)?;

    println!("Successfully rebased {} onto {}.", current_branch, branch_name);

    Ok(())
}

// ============================================================================
// Orphan Command (blocks without incoming edges)
// ============================================================================

pub async fn orphan_list(repo_path: Option<&str>) -> anyhow::Result<()> {
    let root = resolve_repo_path(repo_path);
    let repo = VersionRepo::new(&root);
    let _ = repo.init();

    println!("Orphan blocks (blocks with no incoming edges):");
    println!("(This feature requires database integration to fully detect orphans)");

    let commits = repo.get_commits(100)?;
    let mut referenced_blocks = std::collections::HashSet::new();

    for commit in &commits {
        for block_id in &commit.structure_snapshot.block_order {
            referenced_blocks.insert(block_id);
        }
    }

    if referenced_blocks.is_empty() {
        println!("  (no blocks in repository)");
    } else {
        println!("  Total blocks referenced by commits: {}", referenced_blocks.len());
        println!("  Use 'nexus list --help' to see all blocks in the database");
    }

    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

fn resolve_repo_path(repo_path: Option<&str>) -> PathBuf {
    match repo_path {
        Some(path) => PathBuf::from(path),
        None => std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".pkm"),
    }
}
