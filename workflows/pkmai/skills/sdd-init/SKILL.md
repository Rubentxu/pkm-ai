---
name: sdd-init
description: >
  SDD Project Initialization - Set up a new SDD change project in PKM-AI.
  Trigger: When starting a new SDD change (sdd-init command or new change requested).
license: MIT
metadata:
  author: pkm-ai
  version: "1.0"
---

## Purpose

Initialize a new SDD (Spec-Driven Development) change project in PKM-AI. This creates the foundational project context block that all subsequent phases will reference.

## When to Use

- User requests a new change with `/sdd-init <change-name>`
- Starting a new feature, refactor, or substantial change
- First step before running SDD phases

## What You Receive

- `change-name`: Name of the change (e.g., `mcp-workflow`, `auth-system`)
- `project-name`: Name of the project (e.g., `hodei-pkm`)
- `description`: Brief description of the change (optional)

## Execution

### Step 1: Validate Change Name

Change names must:
- Be lowercase with hyphens (kebab-case)
- Be 3-50 characters
- Not already exist as an SDD project

```json
[
  {
    "tool": "search_blocks",
    "args": {
      "query": "sdd/{change-name}",
      "tags": ["sdd"],
      "limit": 10
    }
  }
]
```

If found, report existing and ask for confirmation before proceeding.

### Step 2: Create Project Context Block

Create the foundational project context block:

```json
[
  {
    "tool": "create_block",
    "args": {
      "block_type": "structure",
      "title": "sdd/{change-name}/project",
      "content": "# SDD Project: {change-name}\n\n## Overview\n{One sentence description of the change}\n\n## Project Metadata\n```json\n{\n  \"change\": \"{change-name}\",\n  \"project\": \"{project-name}\",\n  \"status\": \"active\",\n  \"created\": \"{ISO date}\",\n  \"phases\": {\n    \"explore\": null,\n    \"proposal\": null,\n    \"spec\": null,\n    \"design\": null,\n    \"tasks\": null,\n    \"apply\": null,\n    \"verify\": null,\n    \"archive\": null\n  },\n  \"mode\": \"pkmai\",\n  \"tdd\": false\n}\n```\n\n## Dependencies\n- {list of dependent changes, if any}\n- None if this is a greenfield change\n\n## Team Context\n- Owner: {owner or \"unassigned\"}\n- Phases handled by: {team or \"agent pipeline\"}\n\n## Notes\n{Any initial notes about this change}",
      "tags": ["sdd", "project", "sdd-{change-name}"]
    }
  }
]
```

### Step 3: Create Phase Tracker Block

Create a tracking block for phase status:

```json
[
  {
    "tool": "create_block",
    "args": {
      "block_type": "outline",
      "title": "sdd/{change-name}/tracker",
      "content": "# SDD Phase Tracker: {change-name}\n\n## Phase Status\n\n| Phase | Status | ULID | Completed |\n|-------|--------|------|----------|\n| explore | pending | - | - |\n| proposal | pending | - | - |\n| spec | pending | - | - |\n| design | pending | - | - |\n| tasks | pending | - | - |\n| apply | pending | - | - |\n| verify | pending | - | - |\n| archive | pending | - | - |\n\n## Current Phase\nNone (not started)\n\n## Next Action\nRun `sdd-explore` to begin research\n\n## Blocked Phases\nNone",
      "tags": ["sdd", "tracker", "sdd-{change-name}"]
    }
  }
]
```

### Step 4: Create Discovery Block (Optional)

If user provided a topic, create an initial discovery block:

```json
[
  {
    "tool": "create_block",
    "args": {
      "block_type": "permanent",
      "title": "sdd/{change-name}/discovery",
      "content": "# Initial Discovery: {change-name}\n\n## Topic\n{user-provided topic or \"Not specified\"}\n\n## Initial Questions\n- {question 1}\n- {question 2}\n\n## Resources to Explore\n- {resource 1}\n- {resource 2}\n\n## Notes\n{Any initial notes from the user}",
      "tags": ["sdd", "discovery", "sdd-{change-name}"]
    }
  }
]
```

### Step 5: Link Blocks

Create the initial link structure:

```json
[
  {
    "tool": "create_link",
    "args": {
      "source_id": "{project-context-ulid}",
      "target_id": "{tracker-ulid}",
      "link_type": "contains"
    }
  }
]
```

### Step 6: Return Initialization Summary

Return to orchestrator:

```markdown
## SDD Initialization Complete

**Change**: {change-name}
**Project**: {project-name}
**Mode**: pkmai

### Artifacts Created

| Artifact | ULID | Type |
|----------|------|------|
| sdd/{change-name}/project | {ulid} | structure |
| sdd/{change-name}/tracker | {ulid} | outline |
| sdd/{change-name}/discovery | {ulid} | permanent |

### Links Created

| From | To | Type |
|------|----|------|
| {project-ulid} | {tracker-ulid} | contains |

### Next Steps

1. Run `sdd-explore` to begin research phase
2. Or use `/sdd-new {change-name}` to run full pipeline

### Commands

```
/sdd-explore {change-name}  # Start exploration
/sdd-propose {change-name}  # Propose after explore
/sdd-spec {change-name}     # Create spec after proposal
/sdd-design {change-name}   # Design after spec
/sdd-tasks {change-name}   # Break down into tasks
/sdd-apply {change-name}   # Implement tasks
/sdd-verify {change-name}   # Verify implementation
/sdd-archive {change-name}  # Archive completed change
```

### Status
SDD project initialized. Ready for exploration phase.
```

## Project Context Block Template

```markdown
# SDD Project: {change-name}

## Overview
{Description}

## Metadata
```json
{
  "change": "{change-name}",
  "project": "{project-name}",
  "status": "active",
  "created": "{ISO date}",
  "phases": {
    "explore": null,
    "proposal": null,
    "spec": null,
    "design": null,
    "tasks": null,
    "apply": null,
    "verify": null,
    "archive": null
  },
  "mode": "pkmai",
  "tdd": false
}
```

## Dependencies
{List or "None"}

## Team Context
- Owner: {owner or "unassigned"}
- Phases: {team or "agent pipeline"}
```

## Phase Tracker Block Template

```markdown
# SDD Phase Tracker: {change-name}

## Phase Status

| Phase | Status | ULID | Completed |
|-------|--------|------|----------|
| explore | pending | - | - |
| proposal | pending | - | - |
| spec | pending | - | - |
| design | pending | - | - |
| tasks | pending | - | - |
| apply | pending | - | - |
| verify | pending | - | - |
| archive | pending | - | - |

## Current Phase
None

## Next Action
Run `sdd-explore`
```

## Rules

- Always validate change name doesn't already exist
- Create all three foundational blocks (project, tracker, discovery)
- Use `structure` block type for project context
- Use `outline` block type for tracker
- Use `permanent` block type for discovery
- Tag all blocks with `sdd` and `sdd-{change-name}`
- Link tracker to project context with `contains` relationship
- Return structured envelope with all created artifacts

## PKM-AI Tool Reference

```json
[
  {
    "tool": "search_blocks",
    "args": {
      "query": "sdd/{change-name}",
      "tags": ["sdd"]
    }
  },
  {
    "tool": "create_block",
    "args": {
      "block_type": "structure",
      "title": "sdd/{change-name}/project",
      "content": "{project context content}",
      "tags": ["sdd", "project", "sdd-{change-name}"]
    }
  },
  {
    "tool": "create_block",
    "args": {
      "block_type": "outline",
      "title": "sdd/{change-name}/tracker",
      "content": "{tracker content}",
      "tags": ["sdd", "tracker", "sdd-{change-name}"]
    }
  },
  {
    "tool": "create_block",
    "args": {
      "block_type": "permanent",
      "title": "sdd/{change-name}/discovery",
      "content": "{discovery content}",
      "tags": ["sdd", "discovery", "sdd-{change-name}"]
    }
  },
  {
    "tool": "create_link",
    "args": {
      "source_id": "{project-ulid}",
      "target_id": "{tracker-ulid}",
      "link_type": "contains"
    }
  }
]
```

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-03-20 | Initial version for PKM-AI |
