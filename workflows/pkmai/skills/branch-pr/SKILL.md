---
name: branch-pr
description: >
  Create a Pull Request from current changes - adapted for PKM-AI.
  Trigger: When user wants to create a PR from changes (create-pr, branch-pr commands).
license: MIT
metadata:
  author: pkm-ai
  version: "1.0"
---

## Purpose

Create a Pull Request from the current set of changes in the working directory. This skill handles branch creation, commit staging, and PR creation while linking to the SDD archive for context.

## When to Use

- User asks to "create a PR", "create pull request", or "branch PR"
- User invokes `/create-pr` or `/branch-pr`
- SDD workflow requires creating a PR after `sdd-apply` phase
- Changes are ready to be reviewed and merged

## Prerequisites

1. Working tree must be clean (all changes committed)
2. Must have a remote configured
3. Branch must not already exist on remote (unless force push requested)
4. SDD change should be in `archive` phase for full context

## What You Receive

- `branch_name`: Name for the new branch (optional, auto-generated if not provided)
- `pr_title`: Title for the PR (optional, auto-generated from commits if not provided)
- `pr_body`: Body/description for the PR (optional, linked to SDD archive if available)
- `base_branch`: Target branch for PR (default: main)
- `force_push`: Whether to force push if branch exists (default: false)
- `sdd_change`: Optional SDD change name to link to sdd-archive

## Execution

### Step 1: Check Working Tree Status

```python
# Check git status
import subprocess

result = subprocess.run(
    ["git", "status", "--porcelain"],
    capture_output=True,
    text=True
)

if result.stdout.strip():
    raise Exception(
        "Working tree is not clean. Please commit or stash changes before creating PR.\n"
        f"Pending changes:\n{result.stdout}"
    )

# Get current branch
result = subprocess.run(
    ["git", "branch", "--show-current"],
    capture_output=True,
    text=True
)
current_branch = result.stdout.strip()
```

### Step 2: Get Remote and Base Branch

```python
# Get remote URL
result = subprocess.run(
    ["git", "remote", "get-url", "origin"],
    capture_output=True,
    text=True
)
remote_url = result.stdout.strip()

# Get default branch from remote
result = subprocess.run(
    ["git", "symbolic-ref", "refs/remotes/origin/HEAD"],
    capture_output=True,
    text=True
)
base_branch = result.stdout.strip().replace("refs/remotes/origin/", "")

# Allow override
if base_branch:
    pr_base = base_branch
else:
    pr_base = "main"
```

### Step 3: Generate Branch Name (if not provided)

```python
import re
from datetime import datetime

if not branch_name:
    # Get latest commit message
    result = subprocess.run(
        ["git", "log", "-1", "--format=%s"],
        capture_output=True,
        text=True
    )
    commit_msg = result.stdout.strip()

    # Create branch name from commit or timestamp
    if commit_msg:
        # Convert commit message to branch name (kebab-case)
        branch_name = re.sub(r'[^a-zA-Z0-9]+', '-', commit_msg.lower())
        branch_name = re.sub(r'-+', '-', branch_name).strip('-')[:50]
    else:
        branch_name = f"pr/{datetime.now().strftime('%Y%m%d-%H%M%S')}"

    # Add sdd prefix if SDD change
    if sdd_change:
        branch_name = f"sdd/{sdd_change}/{branch_name}"
```

### Step 4: Check if Branch Exists

```python
# Check local branch
result = subprocess.run(
    ["git", "branch", "--list", branch_name],
    capture_output=True,
    text=True
)
local_exists = bool(result.stdout.strip())

# Check remote branch
result = subprocess.run(
    ["git", "ls-remote", "--heads", "origin", branch_name],
    capture_output=True,
    text=True
)
remote_exists = bool(result.stdout.strip())

if local_exists or remote_exists:
    if force_push:
        action = "will force push"
    else:
        raise Exception(
            f"Branch '{branch_name}' already exists. "
            "Use force_push=true to overwrite, or provide a different branch name."
        )
```

### Step 5: Create Branch and Commit

```python
# Create new branch from current HEAD
subprocess.run(["git", "checkout", "-b", branch_name], check=True)

# Commit any remaining changes (should be none at this point)
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(
    ["git", "commit", "--allow-empty", "-m", f"Ready for PR: {branch_name}"],
    check=True
)
```

### Step 6: Get or Create SDD Archive Link

```python
# Try to find SDD archive for context
sdd_archive_url = None
sdd_archive_content = None

if sdd_change:
    # Search for SDD archive in PKM-AI
    results = search_blocks(
        query=f"sdd/{sdd_change}/archive",
        tags=["sdd-archive", f"sdd-{sdd_change}"],
        limit=5
    )

    if results.blocks:
        archive = get_block(block_id=results.blocks[0].id, include_content=true)
        sdd_archive_content = archive.content
```

### Step 7: Generate PR Body

```python
if not pr_body:
    # Generate from commits
    result = subprocess.run(
        ["git", "log", f"origin/{pr_base}..HEAD", "--format=%h %s"],
        capture_output=True,
        text=True
    )
    commits = result.stdout.strip()

    pr_body = f"""## Summary
{commits if commits else 'No commits yet'}

## Changes
- See commit history for full details

## Testing
- [ ] Tests pass
- [ ] Code reviewed
"""

    # Add SDD archive link if available
    if sdd_archive_content:
        pr_body += f"""

## SDD Context
This PR implements the changes specified in SDD archive for `{sdd_change}`.
Linked via PKM-AI skill-registry.
"""

# Generate title if not provided
if not pr_title:
    result = subprocess.run(
        ["git", "log", "-1", "--format=%s"],
        capture_output=True,
        text=True
    )
    pr_title = result.stdout.strip() or f"Merge {branch_name}"
```

### Step 8: Create PR via GitHub CLI

```python
# Check if gh CLI is available
result = subprocess.run(["which", "gh"], capture_output=True)
has_gh = result.returncode == 0

if has_gh:
    # Create PR using gh
    result = subprocess.run(
        [
            "gh", "pr", "create",
            "--title", pr_title,
            "--body", pr_body,
            "--base", pr_base,
            "--assignee", "@me"
        ],
        capture_output=True,
        text=True
    )

    if result.returncode == 0:
        pr_url = result.stdout.strip()
    else:
        raise Exception(f"Failed to create PR: {result.stderr}")
else:
    # Fallback: output instructions
    pr_url = None
    print(f"""
# Manual PR Creation Required

## Branch Created
`{branch_name}`

## Push to Remote
```bash
git push -u origin {branch_name}
```

## Create PR
```bash
gh pr create --title "{pr_title}" --body "{pr_body}" --base {pr_base}
```

Or open in GitHub:
https://github.com/owner/repo/compare/{pr_base}...{branch_name}
""")
```

### Step 9: Push Branch (if gh not available or as backup)

```python
if not pr_url:
    subprocess.run(
        ["git", "push", "-u", "origin", branch_name],
        check=True
    )
```

### Step 10: Return PR Summary

```python
# Get repository name from remote
result = subprocess.run(
    ["git", "remote", "get-url", "origin"],
    capture_output=True,
    text=True
)
remote_url = result.stdout.strip()
repo_match = re.search(r'[:/](\S+)/(\S+?)(?:\.git)?$', remote_url)
repo_name = f"{repo_match.group(1)}/{repo_match.group(2)}" if repo_match else "owner/repo"

return f"""
## PR Creation Complete

**Status**: {'success' if pr_url else 'manual_required'}
**Branch**: {branch_name}
**Base**: {pr_base}

### PR URL
{pr_url if pr_url else 'Manual creation required - see instructions above'}

### Repository
{repo_name}

### SDD Link
{sdd_change if sdd_change else 'Not linked to SDD'}

### Next Steps
1. Review PR in GitHub UI
2. Address any review comments
3. Merge when approved

### Commands
```bash
git checkout {pr_base}  # Return to base branch
git pull                 # Update base branch
```

### Skill Registry Entry
This skill is registered in PKM-AI skill-registry.
Use `skill-registry` to discover other available skills.
"""
```

## Link to SDD Archive

When `sdd_change` is provided, this skill should be called after `sdd-archive`:

```python
# After sdd-archive completes, user calls branch-pr
results = search_blocks(
    query=f"sdd/{sdd_change}/archive",
    tags=["sdd-archive", f"sdd-{sdd_change}"],
    limit=5
)

if results.blocks:
    archive = get_block(block_id=results.blocks[0].id, include_content=true)
    # PR body includes summary from archive
```

## Return Envelope

```markdown
## Branch/PR Creation Complete

**Status**: success | manual_required | failed
**Branch**: {branch_name}
**Base**: {base_branch}
**PR URL**: {url or "manual_required"}

### Repository
{owner/repo}

### SDD Change
{sdd_change or "Not specified"}

### Artifacts Created
| Artifact | Action |
|----------|--------|
| Branch `{branch_name}` | Created and pushed |
| PR | Created or instructions provided |

### Linked Artifacts
| Artifact | ULID | Relationship |
|----------|------|--------------|
| sdd/{sdd_change}/archive | {ulid} | context |

### Next Recommended
- Review PR in GitHub
- Request review from team
- Monitor CI/CD pipeline

### Status
PR workflow complete. Ready for code review.
```

## Error Handling

### Working Tree Not Clean

```python
if result.stdout.strip():
    raise Exception(
        f"Working tree is not clean. Commit or stash changes first.\n"
        f"Pending:\n{result.stdout}"
    )
```

### Branch Exists

```python
if local_exists or remote_exists:
    if not force_push:
        raise Exception(
            f"Branch '{branch_name}' already exists. "
            "Use force_push=true to overwrite."
        )
```

### Remote Not Configured

```python
if not remote_url:
    raise Exception(
        "No remote configured. "
        "Please add a remote: git remote add origin <url>"
    )
```

## PKM-AI Tool Reference

```python
# Link PR to SDD archive
create_link(
    source_id="{sdd_archive_ulid}",
    target_id="{pr_block_ulid}",
    link_type="related"
)

# Search for SDD archive
search_blocks(
    query="sdd/{change}/archive",
    tags=["sdd-archive", "sdd-{change}"]
)
```

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-03-20 | Initial version for PKM-AI |
