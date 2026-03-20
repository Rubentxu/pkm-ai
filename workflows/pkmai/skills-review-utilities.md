# PKM-AI Utility Skills Review Report

**Date:** 2026-03-20
**Reviewer:** Claude Code Agent
**Scope:** Utility skills (`branch-pr`, `issue-creation`, `skill-registry`) plus integration with SDD phases

---

## 1. Overview

The PKM-AI system implements a **Git-like workflow for knowledge management** with:
- **WorkingSet**: Staging area tracking `created_blocks` and `modified_blocks`
- **Session tracking**: `list_session_blocks`, `capture_block`
- **Staging operations**: `unstage_block`, `discard_working_set`
- **Deletion operations**: `delete_block`, `delete_link`

The current utility skills were designed before these tools were fully integrated. This review identifies improvements to align them with the Zettelkasten/Git-like workflow.

---

## 2. Skill: `branch-pr`

### Current Implementation Approach

1. Validates working tree is clean (Git)
2. Generates branch name from commit message or timestamp
3. Creates branch and pushes to remote
4. Optionally links to SDD archive block in PKM-AI
5. Creates PR via GitHub CLI (`gh pr create`)

### Recommended Improvements

| Current Behavior | Recommended Change | Rationale |
|-----------------|-------------------|-----------|
| Links PR to SDD archive after creation | Use `capture_block` to create a PR tracking block in the working set first | Enables preview before committing |
| Uses `create_link` for archive reference only | Add `list_session_blocks` to verify PR artifacts before creation | Session awareness |
| No working set integration | Stage PR block to WorkingSet before commit | Git-like atomic commit pattern |
| No `delete_link` usage | Offer to clean up SDD links after PR merge | Lifecycle cleanup |

### Specific Changes to SKILL.md

**New PKM-AI Tools to Add:**

```json
[
  {
    "tool": "list_session_blocks",
    "args": {
      "session_id": "{current_session}",
      "include_content": false
    }
  },
  {
    "tool": "capture_block",
    "args": {
      "block_type": "permanent",
      "title": "pr/{branch_name}",
      "content": "## PR: {branch_name}\n\n**Status**: draft\n**Base**: {base_branch}\n**URL**: {url_or_pending}\n**Created**: {ISO date}",
      "tags": ["pr", "sdd-{change}"]
    }
  },
  {
    "tool": "unstage_block",
    "args": {
      "block_id": "{pr_block_ulid}"
    }
  }
]
```

**New Workflow Step (before Step 9):**

```python
# NEW Step 8.5: Create PR block in staging
pr_block = capture_block(
    block_type="permanent",
    title=f"pr/{branch_name}",
    content=f"""## PR: {branch_name}

**Status**: draft
**Base**: {base_branch}
**Commits**: {commits_count}

## Summary
{auto_generated_summary}

## Linked SDD
- Archive: {sdd_archive_url or "Not linked"}

## Artifacts
| Type | Count |
|------|-------|
| Commits | {commits_count} |
| Files changed | {files_changed} |
""",
    tags=["pr", f"sdd-{sdd_change}"] if sdd_change else ["pr"]
)

# NEW Step 8.6: Link PR block to SDD archive
if sdd_change and sdd_archive_ulid:
    create_link(
        source_id=sdd_archive_ulid,
        target_id=pr_block.id,
        link_type="related"
    )

# NEW Step 8.7: Offer discard option
# If user wants to undo before commit, they can call discard_working_set
```

**Error Handling Additions:**

```python
# NEW: Handle working set state
def check_pr_readiness():
    """Check if PR block is ready to commit."""
    session_blocks = list_session_blocks(session_id=current_session)

    pending = [b for b in session_blocks.blocks
               if b.title.startswith("pr/") and "draft" in b.content]

    if pending:
        return {
            "ready": False,
            "pending_pr_blocks": pending,
            "message": "PR block(s) in staging. Commit or unstage before proceeding."
        }
    return {"ready": True}
```

---

## 3. Skill: `issue-creation`

### Current Implementation Approach

1. Validates title (5-200 chars)
2. Maps issue type to labels
3. Auto-generates issue body with SDD context
4. Checks for duplicate issues via `gh issue list`
5. Creates issue via GitHub CLI
6. Optionally creates PKM-AI block for tracking

### Recommended Improvements

| Current Behavior | Recommended Change | Rationale |
|-----------------|-------------------|-----------|
| Creates PKM-AI block after GitHub issue | Use `capture_block` to stage block first | Enables review before commit |
| No duplicate detection using PKM-AI | Use `search_blocks` to check for similar issues in PKM | Cross-reference with knowledge graph |
| No link cleanup on close | Use `delete_link` when issue is closed | Lifecycle management |
| Block created with `create_block` directly | Use `unstage_block` for discard workflow | Better atomicity |

### Specific Changes to SKILL.md

**New PKM-AI Tools to Add:**

```json
[
  {
    "tool": "search_blocks",
    "args": {
      "query": "issue/{issue_number}",
      "tags": ["issue"],
      "limit": 5
    }
  },
  {
    "tool": "capture_block",
    "args": {
      "block_type": "permanent",
      "title": "issue/#{issue_number}",
      "content": "## Issue #{issue_number}\n\n**Status**: open\n**Type**: {issue_type}\n**Priority**: {priority}\n**Labels**: {labels}\n**URL**: {issue_url}",
      "tags": ["issue", f"issue-{issue_type}"]
    }
  }
]
```

**New Workflow Step (after duplicate check):**

```python
# NEW Step 4.5: Check PKM-AI for related issues
related = search_blocks(
    query=f"issue/{issue_number}",
    tags=["issue"],
    limit=10
)

if related.blocks:
    print(f"Found {len(related.blocks)} potentially related PKM-AI issues")
    for block in related.blocks[:3]:
        print(f"  - {block.id}: {block.title}")

# NEW Step 4.6: Create issue block in staging (not committed yet)
issue_block = capture_block(
    block_type="permanent",
    title=f"issue/#{issue_number}",
    content=f"""# Issue #{issue_number}: {title}

## Summary
{body[:500]}...

## Metadata
```json
{{
  "number": {issue_number},
  "url": "{issue_url}",
  "type": "{issue_type}",
  "priority": "{priority}",
  "status": "open",
  "labels": {json.dumps(applied_labels)},
  "sdd_change": {json.dumps(sdd_change)}
}}
```

## SDD Context
{sdd_change or "Not linked to SDD"}

## Related Issues
{block_id_list or "None"}
""",
    tags=["issue", f"issue-{issue_type}", f"issue-{issue_number}"]
)

# NEW Step 4.7: Link to SDD project
if sdd_change:
    project_results = search_blocks(
        query=f"sdd/{sdd_change}/project",
        tags=["sdd", "project"]
    )
    if project_results.blocks:
        create_link(
            source_id=project_results.blocks[0].id,
            target_id=issue_block.id,
            link_type="related"
        )
```

**On Issue Close Workflow:**

```python
def close_issue_workflow(issue_number: int):
    """Cleanup workflow when issue is closed."""

    # Find issue block
    results = search_blocks(
        query=f"issue/{issue_number}",
        tags=["issue"]
    )

    for block in results.blocks:
        # Update status to closed
        update_block(
            block_id=block.id,
            content=block.content.replace("**Status**: open", "**Status**: closed")
        )

        # Note: delete_link would be used if we tracked outgoing links
        # that should be cleaned up on close
```

---

## 4. Skill: `skill-registry`

### Current Implementation Approach

1. **Discover**: Lists all skills via `search_blocks` on `skill-registry` tag
2. **Register**: Adds new skill by updating registry block
3. **Lookup**: Searches for skills matching a task
4. **Sync**: Scans filesystem for SKILL.md files and updates registry

### Recommended Improvements

| Current Behavior | Recommended Change | Rationale |
|-----------------|-------------------|-----------|
| Uses `search_blocks` for discovery | Add `list_session_blocks` to show recently loaded skills | Session awareness |
| Registry stored as single block | Use WorkingSet to stage registry changes before commit | Atomic updates |
| Manual sync to filesystem | Add skill blocks to WorkingSet, commit together | Transactional update |
| No `delete_block` usage | Add deprecation workflow for old skills | Skill lifecycle |

### Specific Changes to SKILL.md

**New PKM-AI Tools to Add:**

```json
[
  {
    "tool": "list_session_blocks",
    "args": {
      "session_id": "{current_session}",
      "filter_tags": ["skill"],
      "include_content": false
    }
  },
  {
    "tool": "capture_block",
    "args": {
      "block_type": "permanent",
      "title": "skill/{skill_name}",
      "content": "# Skill: {skill_name}\n\n**Path**: {skill_path}\n**Purpose**: {description}\n**Triggers**: {trigger_phrases}",
      "tags": ["skill", "skill-registry"]
    }
  },
  {
    "tool": "delete_block",
    "args": {
      "block_id": "{old_skill_ulid}"
    }
  },
  {
    "tool": "delete_link",
    "args": {
      "source_id": "{registry_ulid}",
      "target_id": "{skill_ulid}",
      "link_type": "contains"
    }
  }
]
```

**New Registry Block Structure:**

```python
# NEW: Use structured format for registry
registry_content = f"""# Skill Registry

## Overview
Total skills: {len(skills)}
Last synced: {ISO date}

## Skills Index

| Skill | Path | Purpose | ULID |
|-------|------|---------|------|
{skill_rows}

## Session Context
This session has loaded: {current_session_skills}

## Directory Structure
- `.claude/skills/`: User-defined project skills
- `workflows/pkmai/skills/`: PKM-AI native skills
"""
```

**New Sync Workflow:**

```python
# NEW: Atomic sync using working set
def sync_registry_atomic():
    """Sync registry with filesystem atomically."""

    # 1. Capture current registry state
    current = search_blocks(
        query="skill-registry",
        tags=["skill-registry"]
    )

    # 2. Scan filesystem for skills
    filesystem_skills = scan_skill_directories()

    # 3. Compare and stage changes
    to_add = [s for s in filesystem_skills if s.ulid is None]
    to_update = [s for s in filesystem_skills if s.ulid and s.needs_update]
    to_remove = [s for s in current.blocks if s not in filesystem_skills]

    # 4. Stage additions
    for skill in to_add:
        capture_block(
            block_type="permanent",
            title=f"skill/{skill.name}",
            content=skill.to_markdown(),
            tags=["skill", "skill-registry"]
        )

    # 5. Stage updates
    for skill in to_update:
        update_block(
            block_id=skill.ulid,
            content=skill.to_markdown()
        )

    # 6. Stage removals (delete_block)
    for skill_ulid in to_remove:
        delete_block(block_id=skill_ulid)

    # 7. discard_working_set if user wants to cancel
    # OR commit when ready
```

**Deprecation Workflow:**

```python
def deprecate_skill(skill_name: str, replacement: str = None):
    """Mark a skill as deprecated."""

    # 1. Find skill block
    results = search_blocks(
        query=f"skill/{skill_name}",
        tags=["skill"]
    )

    if not results.blocks:
        raise Exception(f"Skill not found: {skill_name}")

    skill_block = results.blocks[0]

    # 2. Update content with deprecation notice
    updated_content = skill_block.content + f"""

## Deprecation Notice

**Deprecated**: {ISO date}
**Replacement**: {replacement or "None specified"}
**Reason**: This skill is deprecated. Please use the replacement or updated workflow.

This block will be removed after {grace_period_days} days.
"""

    update_block(block_id=skill_block.id, content=updated_content)

    # 3. Remove from registry link
    registry_results = search_blocks(
        query="skill-registry",
        tags=["skill-registry"]
    )

    if registry_results.blocks:
        delete_link(
            source_id=registry_results.blocks[0].id,
            target_id=skill_block.id,
            link_type="contains"
        )
```

---

## 5. Cross-Cutting Improvements

### 5.1 Session Integration

All three utility skills should track their artifacts in the session:

```python
# Add to all utility skills
SESSION_TRACKED_ARTIFACTS = {
    "branch-pr": ["pr/*"],
    "issue-creation": ["issue/*"],
    "skill-registry": ["skill/*", "skill-registry"]
}

def get_session_artifacts(skill_name: str) -> list:
    """Get artifacts created by this skill in current session."""
    patterns = SESSION_TRACKED_ARTIFACTS.get(skill_name, [])
    session_blocks = list_session_blocks(
        session_id=current_session,
        include_content=False
    )
    return [
        b for b in session_blocks.blocks
        if any(b.title.match(p) for p in patterns)
    ]
```

### 5.2 WorkingSet Integration Pattern

```python
# Standard pattern for all utility skills

def utility_workflow_with_staging():
    """Git-like workflow for utility skills."""

    # 1. Create artifact in staging (not committed)
    artifact = capture_block(...)

    # 2. Offer preview
    print(f"Created: {artifact.id}")
    print("Use unstage_block to discard before commit")

    # 3. On user confirmation:
    #    - Artifact stays in working set
    #    - Eventually committed via session close

    # 4. On user cancellation:
    unstage_block(block_id=artifact.id)

    # 5. On explicit save:
    #    - Already in working set, will commit with session
    pass
```

### 5.3 Consistent Error Handling

```python
# Add to all skills
class WorkingSetAwareError(Exception):
    """Error that can be recovered via working set operations."""
    pass

def handle_staging_error(e: Exception, block_id: str = None):
    """Standard error handling for staging-aware operations."""
    if block_id:
        # Offer unstaging as recovery
        return {
            "error": str(e),
            "recovery": "unstage_block",
            "block_id": block_id
        }
    raise e
```

---

## 6. Summary of Recommended Tool Usage

| Tool | branch-pr | issue-creation | skill-registry |
|------|-----------|----------------|----------------|
| `search_blocks` | Existing | Existing + new duplicate check | Existing |
| `get_block` | Existing | Existing | Existing |
| `create_block` | Existing | Existing | Existing |
| `update_block` | Existing | Existing | Existing |
| `create_link` | Existing | Enhanced | Existing |
| `capture_block` | **NEW** - PR staging | **NEW** - Issue staging | **NEW** - Skill staging |
| `list_session_blocks` | **NEW** - Session awareness | **NEW** - Session awareness | **NEW** - Session awareness |
| `delete_block` | — | — | **NEW** - Skill deprecation |
| `delete_link` | — | — | **NEW** - Registry cleanup |
| `unstage_block` | **NEW** - Discard PR | **NEW** - Discard issue | **NEW** - Discard skill |
| `discard_working_set` | **NEW** - Cancel all | **NEW** - Cancel all | **NEW** - Cancel all |

---

## 7. Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-03-20 | Initial review with WorkingSet integration recommendations |

---

## 8. Appendix: Full Tool Reference for Utilities

### Tools for branch-pr

```json
{
  "tool": "list_session_blocks",
  "args": {"session_id": "{s}", "include_content": false}
}
{
  "tool": "capture_block",
  "args": {
    "block_type": "permanent",
    "title": "pr/{branch}",
    "content": "...",
    "tags": ["pr", "sdd-{change}"]
  }
}
{
  "tool": "unstage_block",
  "args": {"block_id": "{ulid}"}
}
```

### Tools for issue-creation

```json
{
  "tool": "search_blocks",
  "args": {"query": "issue/{number}", "tags": ["issue"]}
}
{
  "tool": "capture_block",
  "args": {
    "block_type": "permanent",
    "title": "issue/#{number}",
    "content": "...",
    "tags": ["issue", "issue-{type}"]
  }
}
{
  "tool": "unstage_block",
  "args": {"block_id": "{ulid}"}
}
```

### Tools for skill-registry

```json
{
  "tool": "list_session_blocks",
  "args": {"session_id": "{s}", "filter_tags": ["skill"], "include_content": false}
}
{
  "tool": "capture_block",
  "args": {
    "block_type": "permanent",
    "title": "skill/{name}",
    "content": "...",
    "tags": ["skill", "skill-registry"]
  }
}
{
  "tool": "delete_block",
  "args": {"block_id": "{ulid}"}
}
{
  "tool": "delete_link",
  "args": {"source_id": "{reg}", "target_id": "{skill}", "link_type": "contains"}
}
{
  "tool": "unstage_block",
  "args": {"block_id": "{ulid}"}
}
{
  "tool": "discard_working_set",
  "args": {"session_id": "{s}"}
}
```
