# SDD Persistence Contract

This document defines the persistence contract for SDD artifacts. PKM-AI is the **default** artifact store, with openspec as an optional file-based alternative.

## Contract Overview

| Aspect | PKM-AI (Default) | Openspec (Optional) |
|--------|-----------------|---------------------|
| Primary Store | PKM-AI blocks | Filesystem |
| Artifact ID | ULID | File path |
| Search | `search_blocks` | Grep/fs |
| Retrieval | `get_block` | File read |
| Creation | `create_block` | File write |
| Updates | `update_block` | File write |
| Links | `create_link` | Markdown links |
| Mode Flag | `pkmai` | `openspec` |

## Mode Selection

| Mode | Behavior |
|------|----------|
| `pkmai` | Default. Artifacts stored as PKM-AI blocks with tags. Use MCP tools. |
| `openspec` | File-based artifacts. Use only when user explicitly requests. |
| `hybrid` | Both backends. PKM-AI for search/navigation, files for persistence. |
| `none` | Return results inline only. Not recommended. |

## PKM-AI Contract (Default)

### Block Naming Convention

```
title: sdd/{change}/{phase}
```

Examples:
- `sdd/mcp-workflow/project`
- `sdd/mcp-workflow/tracker`
- `sdd/mcp-workflow/proposal`
- `sdd/mcp-workflow/spec`

### Tag Contract

All SDD artifacts MUST have these tags:

```python
tags=["sdd", "sdd-{phase}", "sdd-{change}"]
```

Example for spec artifact:
```python
tags=["sdd", "sdd-spec", "sdd-mcp-workflow"]
```

### Block Type Contract

| Phase | Block Type | Enforced |
|-------|------------|----------|
| project | `structure` | Yes |
| tracker | `outline` | Yes |
| discovery | `permanent` | Recommended |
| explore | `permanent` | Yes |
| proposal | `permanent` | Yes |
| spec | `structure` | Yes |
| design | `permanent` | Yes |
| tasks | `outline` | Yes |
| progress | `permanent` | Yes |
| verify | `permanent` | Yes |
| archive | `permanent` | Yes |

### Link Contract

Artifacts MUST be linked using `create_link`:

| From | To | Link Type | Required |
|------|----|-----------|----------|
| Project | Tracker | `contains` | Yes |
| Explore | Proposal | `refines` | Recommended |
| Proposal | Spec | `refines` | Yes |
| Proposal | Design | `refines` | Recommended |
| Spec | Design | `refines` | Recommended |
| Design | Tasks | `contains` | Yes |
| Tasks | Progress | `related` | Yes |
| Spec | Verify | `supports` | Recommended |
| Project | Archive | `contains` | Yes |

## Openspec Contract (Optional)

When mode is `openspec`:

### File Naming Convention

```
openspec/changes/{change}/{phase}.md
```

Examples:
- `openspec/changes/mcp-workflow/project.md`
- `openspec/changes/mcp-workflow/tracker.md`
- `openspec/changes/mcp-workflow/proposal.md`

### State File Contract

`openspec/changes/{change}/state.yaml` MUST be updated after each phase:

```yaml
change: {change-name}
status: active
current_phase: {current-phase}
mode: openspec

phases:
  {phase}:
    status: completed | in_progress | pending
    completed_at: {ISO date or null}
    artifact: {phase}.md

tdd: {boolean}
created: {ISO date}
modified: {ISO date}
```

### Task Marking Contract

In `tasks.md`, use checkbox format:

```markdown
- [x] Task completed
- [ ] Task pending
```

## Hybrid Mode Contract

When mode is `hybrid`:

### Write Order

1. Write to filesystem (primary, source of truth)
2. Create in PKM-AI (secondary, for navigation)

### Read Order

1. Read from filesystem (primary, full content)
2. Search PKM-AI (secondary, for finding artifacts)

### Sync Requirements

- PKM-AI blocks contain content summary + file reference
- Filesystem contains full content
- Both MUST be kept in sync

## Artifact Lifecycle

### Creation

1. Validate required dependencies exist
2. Create artifact in store (PKM-AI or filesystem)
3. Create links to parent artifacts
4. Update tracker if applicable
5. Return artifact reference

### Retrieval

1. Search for artifact by query/tags or path
2. Validate artifact exists
3. Retrieve full content
4. Return to caller

### Updates

1. Find existing artifact
2. Validate ownership (same change/phase)
3. Update content
4. Preserve metadata (created date, ULID)
5. Return updated reference

### Deletion

Not supported. Artifacts are append-only.

## Error Handling

### Artifact Not Found

```python
# PKM-AI
if not results.blocks:
    raise Exception(f"Artifact not found: sdd/{change}/{phase}")

# Openspec
if not os.path.exists(f"openspec/changes/{change}/{phase}.md"):
    raise Exception(f"Artifact not found: {phase}.md")
```

### Write Failure

```python
# PKM-AI
try:
    create_block(...)
except Exception as e:
    raise Exception(f"Failed to create artifact: {e}")

# Openspec
try:
    with open(f"openspec/changes/{change}/{phase}.md", "w") as f:
        f.write(content)
except Exception as e:
    raise Exception(f"Failed to write artifact: {e}")
```

### Link Failure

```python
try:
    create_link(source_id=from_id, target_id=to_id, link_type=link_type)
except Exception:
    # Non-fatal - log and continue
    logger.warning(f"Failed to create link: {from_id} -> {to_id}")
```

## Recovery Protocol

### When Starting a Phase

1. Load shared conventions
2. Search for required artifacts by tags/query
3. If not found, search by alternative criteria
4. If multiple found, use most recent
5. If none found, report blocker or proceed without

### Finding Artifacts

```json
[
  {
    "tool": "search_blocks",
    "args": {
      "query": "sdd/{change}/{phase}",
      "tags": ["sdd-{phase}"],
      "limit": 5
    }
  },
  {
    "tool": "get_block",
    "args": {
      "block_id": "{artifact_id}",
      "include_content": true
    }
  }
]
```

```python
# Openspec pattern
artifact_path = f"openspec/changes/{change}/{phase}.md"
if os.path.exists(artifact_path):
    with open(artifact_path, "r") as f:
        artifact = f.read()
```

## Contract Enforcement

### Skill Requirements

All SDD phase skills MUST:

1. Declare `mode` parameter in skill metadata
2. Load appropriate conventions based on mode
3. Use correct tool for the mode (PKM-AI MCP or filesystem)
4. Return structured envelope with all fields
5. Update tracker when applicable

### Validation Checklist

- [ ] Artifact title follows `sdd/{change}/{phase}` format
- [ ] Artifact has required tags: `sdd`, `sdd-{phase}`, `sdd-{change}`
- [ ] Artifact uses correct block type for phase
- [ ] Links created to parent artifacts
- [ ] Tracker updated (if applicable)
- [ ] Return envelope includes all required fields

## Quick Reference

### PKM-AI (Default)

```json
[
  {
    "tool": "create_block",
    "args": {
      "block_type": "{type}",
      "title": "sdd/{change}/{phase}",
      "content": "{content}",
      "tags": ["sdd", "sdd-{phase}", "sdd-{change}"]
    }
  },
  {
    "tool": "search_blocks",
    "args": {
      "query": "sdd/{change}/{phase}",
      "tags": ["sdd-{phase}"]
    }
  },
  {
    "tool": "get_block",
    "args": {
      "block_id": "{results.blocks[0].id}",
      "include_content": true
    }
  },
  {
    "tool": "update_block",
    "args": {
      "block_id": "{ulid}",
      "content": "{updated_content}"
    }
  },
  {
    "tool": "create_link",
    "args": {
      "source_id": "{from}",
      "target_id": "{to}",
      "link_type": "{type}"
    }
  }
]
```

### Openspec (Optional)

```python
# Read
with open(f"openspec/changes/{change}/{phase}.md", "r") as f:
    content = f.read()

# Write
with open(f"openspec/changes/{change}/{phase}.md", "w") as f:
    f.write(content)

# Update state
import yaml
with open(f"openspec/changes/{change}/state.yaml", "r") as f:
    state = yaml.safe_load(f)
state["phases"]["{phase}"]["status"] = "completed"
with open(f"openspec/changes/{change}/state.yaml", "w") as f:
    yaml.safe_dump(state, f)
```

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-03-20 | Initial version with PKM-AI as default |
