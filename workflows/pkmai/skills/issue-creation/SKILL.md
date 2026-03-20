---
name: issue-creation
description: >
  Create GitHub issues for blocked tasks or improvement suggestions - PKM-AI.
  Trigger: When user wants to create an issue (create-issue, new-issue commands).
license: MIT
metadata:
  author: pkm-ai
  version: "1.0"
---

## Purpose

Create GitHub issues for tasks that are blocked, require further discussion, or represent improvement opportunities. Links issues to SDD context when applicable.

## When to Use

- User asks to "create an issue", "file a bug", or "open issue"
- User invokes `/create-issue` or `/new-issue`
- SDD workflow encounters a blocker that needs external resolution
- Finding technical debt or improvements during development
- Reporting bugs or feature requests

## What You Receive

- `title`: Issue title (required)
- `body`: Issue description/body (optional, auto-generated if not provided)
- `labels`: List of labels to apply (optional)
- `assignees`: List of assignees (optional)
- `issue_type`: Type of issue - `bug`, `feature`, `task`, `question` (default: `task`)
- `sdd_change`: Optional SDD change name to link issue to SDD context
- `blocked_by`: Optional list of blocking issues or dependencies
- `priority`: Priority level - `high`, `medium`, `low` (default: `medium`)

## Execution

### Step 1: Validate Title

```python
import re

if not title:
    raise Exception("Issue title is required")

# Validate title length and format
if len(title) < 5:
    raise Exception("Issue title must be at least 5 characters")

if len(title) > 200:
    raise Exception("Issue title must be less than 200 characters")

# Sanitize title
title = title.strip()
```

### Step 2: Determine Issue Type and Labels

```python
# Map issue_type to labels
type_labels = {
    "bug": ["bug", "needs-triage"],
    "feature": ["enhancement", "needs-triage"],
    "task": ["task", "needs-triage"],
    "question": ["question", "needs-triage"],
    "improvement": ["improvement", "needs-triage"]
}

# Priority labels
priority_labels = {
    "high": ["priority: high"],
    "medium": ["priority: medium"],
    "low": ["priority: low"]
}

# Apply labels
applied_labels = type_labels.get(issue_type, ["task"])
if priority:
    applied_labels.extend(priority_labels.get(priority, []))

# Add SDD-related label if applicable
if sdd_change:
    applied_labels.append(f"sdd:{sdd_change}")
```

### Step 3: Generate Issue Body

```python
import yaml
from datetime import datetime

if not body:
    # Auto-generate body based on issue_type
    body = f"""## {issue_type.capitalize()} Description

<!-- Describe the {issue_type} here -->

## Steps to Reproduce (if bug)

<!-- If this is a bug, please provide:
1. Steps to reproduce
2. Expected behavior
3. Actual behavior
-->

## Context

- **Issue Type**: {issue_type}
- **Priority**: {priority or 'medium'}
- **Created**: {datetime.now().isoformat()}
"""

    # Add SDD context if applicable
    if sdd_change:
        # Try to get SDD context
        results = search_blocks(
            query=f"sdd/{sdd_change}/project",
            tags=["sdd-project", f"sdd-{sdd_change}"],
            limit=5
        )

        body += f"""
## SDD Context

This issue is related to SDD change: **{sdd_change}**

"""

    # Add blocked_by information
    if blocked_by:
        body += f"""
## Blocked By

This issue is blocked by:
"""
        for blocker in blocked_by:
            body += f"- {blocker}\n"

    # Add standard sections
    body += f"""

## Acceptance Criteria

- [ ] Issue is reproducible
- [ ] Clear description provided
- [ ] All relevant context documented

## Notes

<!-- Additional notes, links, or context -->
"""

# Apply priority to body if specified
if priority == "high":
    body = f"**⚠️ HIGH PRIORITY**\n\n{body}"
elif priority == "low":
    body = f"*Low priority issue*\n\n{body}"
```

### Step 4: Check for Duplicate Issues

```python
# Search for existing issues with similar title
result = subprocess.run(
    ["gh", "issue", "list", "--limit", "20", "--state", "open"],
    capture_output=True,
    text=True
)

if result.returncode == 0:
    existing_issues = result.stdout.strip().split('\n')
    for issue_line in existing_issues:
        if issue_line:
            parts = issue_line.split('\t')
            if len(parts) >= 2:
                existing_title = parts[1].strip()
                # Simple duplicate detection - same words
                title_words = set(title.lower().split())
                existing_words = set(existing_title.lower().split())
                overlap = title_words & existing_words
                if len(overlap) >= 3:  # 3+ common words
                    print(f"⚠️ Potential duplicate of issue: #{parts[0]}")
                    print(f"   Existing: {existing_title}")
                    print(f"   New: {title}")
```

### Step 5: Create Issue via GitHub CLI

```python
import subprocess
import json

# Build gh command
cmd = [
    "gh", "issue", "create",
    "--title", title,
    "--body", body,
]

# Add labels
if applied_labels:
    cmd.extend(["--label", ",".join(applied_labels)])

# Add assignees
if assignees:
    for assignee in assignees:
        cmd.extend(["--assignee", assignee])

# Execute
result = subprocess.run(cmd, capture_output=True, text=True)

if result.returncode == 0:
    issue_url = result.stdout.strip()
    issue_number = re.search(r'/issues/(\d+)$', issue_url).group(1)
else:
    raise Exception(f"Failed to create issue: {result.stderr}")
```

### Step 6: Create PKM-AI Block for Issue (Optional)

```python
# Create a tracking block for the issue in PKM-AI
issue_block = create_block(
    block_type="permanent",
    title=f"issue/#{issue_number}",
    content=f"""# Issue #{issue_number}: {title}

## Summary
{body[:200]}...

## Metadata
```json
{{
  "number": {issue_number},
  "url": "{issue_url}",
  "type": "{issue_type}",
  "priority": "{priority or 'medium'}",
  "status": "open",
  "labels": {json.dumps(applied_labels)},
  "sdd_change": {json.dumps(sdd_change)},
  "created": "{datetime.now().isoformat()}"
}}
```

## Related Issues
- {blocked_by.join('\n- ') if blocked_by else 'None'}

## Resolution
<!-- Fill in when issue is resolved -->
""",
    tags=["issue", f"issue-{issue_number}", f"issue-{issue_type}"]
)

# Link issue block to SDD if applicable
if sdd_change:
    # Find SDD project block
    results = search_blocks(
        query=f"sdd/{sdd_change}/project",
        tags=["sdd-project", f"sdd-{sdd_change}"],
        limit=5
    )

    if results.blocks:
        create_link(
            source_id=results.blocks[0].id,
            target_id=issue_block.id,
            link_type="related"
        )
```

### Step 7: Return Issue Summary

```python
return f"""
## Issue Creation Complete

**Status**: success | failed
**Issue Number**: #{issue_number}
**Issue URL**: {issue_url}

### Issue Details

| Field | Value |
|-------|-------|
| Title | {title} |
| Type | {issue_type} |
| Priority | {priority or 'medium'} |
| Labels | {', '.join(applied_labels)} |

### SDD Link
{sdd_change if sdd_change else 'Not linked to SDD'}

### Next Steps
1. Open issue URL and review
2. Add more context if needed
3. Address or close when resolved

### PKM-AI Block Created
- Block: issue/#{issue_number}
- ULID: {issue_block.id if issue_block else 'N/A'}

### Commands
```bash
gh issue view {issue_number}    # View issue details
gh issue close {issue_number}  # Close issue
gh issue edit {issue_number}    # Edit issue
```

### Skill Registry Entry
This skill is registered in PKM-AI skill-registry.
Use `skill-registry` to discover other available skills.
"""
```

## Issue Templates

### Bug Report Template

```markdown
## Bug Description
<!-- Describe the bug clearly -->

## Steps to Reproduce
1.
2.
3.

## Expected Behavior
<!-- What should happen -->

## Actual Behavior
<!-- What actually happens -->

## Environment
- OS:
- Version:
- Branch:

## Possible Fix
<!-- Optional: suggest a fix -->
```

### Feature Request Template

```markdown
## Feature Description
<!-- Describe the feature -->

## Use Case
<!-- Why is this needed? -->

## Proposed Solution
<!-- Optional: suggest implementation -->

## Alternatives Considered
<!-- Optional: alternatives that were considered -->
```

### Task Template

```markdown
## Task Description
<!-- What needs to be done -->

## Context
<!-- Background information -->

## Deliverables
- [ ]
- [ ]

## Dependencies
<!-- What does this depend on? -->
```

## Return Envelope

```markdown
## Issue Creation Complete

**Status**: success | failed
**Issue Number**: #{issue_number}
**Issue URL**: {issue_url}

### Issue Details
| Field | Value |
|-------|-------|
| Title | {title} |
| Type | {issue_type} |
| Priority | {priority or 'medium'} |
| Labels | {labels} |

### SDD Context
{sdd_change or "Not linked to SDD"}

### Artifacts Created
| Artifact | ULID | Action |
|----------|------|--------|
| issue/#{issue_number} | {ulid} | Created |

### Linked Artifacts
| From | To | Type |
|------|----|------|
| sdd/{sdd_change}/project | issue/#{issue_number} | related |

### Next Recommended
- Review issue in GitHub
- Add more context if needed
- Link to related issues

### Status
Issue created and linked to PKM-AI. Ready for assignment.
```

## Error Handling

### Title Required

```python
if not title:
    raise Exception("Issue title is required. Usage: /create-issue <title>")
```

### GitHub CLI Not Available

```python
result = subprocess.run(["which", "gh"], capture_output=True)
if result.returncode != 0:
    raise Exception(
        "GitHub CLI (gh) is not installed. "
        "Install from: https://cli.github.com/"
    )
```

### Not in Git Repository

```python
result = subprocess.run(["git", "rev-parse", "--git-dir"], capture_output=True)
if result.returncode != 0:
    raise Exception(
        "Not in a Git repository. "
        "issue-creation must be run within a Git repository."
    )
```

## PKM-AI Tool Reference

```python
# Create issue block
create_block(
    block_type="permanent",
    title=f"issue/#{issue_number}",
    content="{issue content}",
    tags=["issue", f"issue-{issue_type}", f"issue-{issue_number}"]
)

# Link to SDD
create_link(
    source_id="{sdd_project_ulid}",
    target_id="{issue_ulid}",
    link_type="related"
)

# Search for issue
search_blocks(
    query=f"issue/{issue_number}",
    tags=["issue"]
)
```

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-03-20 | Initial version for PKM-AI |
