# PKM-AI SDD Conventions

This document describes the PKM-AI specific conventions for SDD artifact storage and retrieval.

## Overview

PKM-AI uses blocks with ULIDs as artifact identifiers:
- **Block titles** in format `sdd/{change}/{phase}`
- **Tags** for classification: `["sdd", "sdd-{phase}", "sdd-{change}"]`
- **Block types** to indicate purpose: `permanent`, `structure`, `outline`, `ghost`
- **Links** to establish relationships between blocks

## PKM-AI Tools

| Tool | Purpose |
|------|---------|
| `search_blocks` | Find artifacts by query and tags |
| `get_block` | Get full artifact content by ULID |
| `create_block` | Create new artifact block |
| `update_block` | Update existing artifact |
| `create_link` | Link artifacts together |

## Block Structure

Each SDD artifact is stored as a PKM-AI block with:

| Field | Value | Description |
|-------|-------|-------------|
| `block_type` | See block type table | Determines artifact category |
| `title` | `sdd/{change}/{phase}` | Full topic key equivalent |
| `content` | Markdown + JSON | Artifact content |
| `tags` | `["sdd", "sdd-{phase}", "sdd-{change}"]` | Classification |

### Block Types by Phase

| Phase | Block Type | Rationale |
|-------|------------|-----------|
| `project` | `structure` | Project metadata has structure |
| `tracker` | `outline` | Task tracking is checklist |
| `discovery` | `permanent` | Initial discovery is atomic |
| `explore` | `permanent` | Research is atomic knowledge |
| `proposal` | `permanent` | Decision is atomic |
| `spec` | `structure` | Spec has internal structure (scenarios) |
| `design` | `permanent` | Design decisions are atomic |
| `tasks` | `outline` | Tasks are a checklist |
| `progress` | `permanent` | Progress is status |
| `verify` | `permanent` | Verification is atomic |
| `archive` | `permanent` | Archive summary is atomic |

## Tag Conventions

### Primary Tags

| Tag | Usage |
|-----|-------|
| `sdd` | All SDD artifacts |
| `sdd-{phase}` | Phase-specific artifacts (e.g., `sdd-spec`) |
| `sdd-{change}` | Change-specific tag (e.g., `sdd-mcp-workflow`) |

### Secondary Tags (Optional)

| Tag | Usage |
|-----|-------|
| `sdd-active` | Currently active change |
| `sdd-blocked` | Change blocked on something |
| `sdd-completed` | Completed change |
| `discovery` | Initial discovery blocks |
| `tracker` | Phase tracker blocks |
| `project` | Project context blocks |

## Search Patterns

### Find All SDD Artifacts for a Change

```json
{
n  "tool": "search_blocks",
  "arguments": {"query": "sdd-{change-name}", "tags": ["sdd"], "limit": 50}
}
```

### Find Specific Phase Artifact

```json
{
  "tool": "search_blocks",
  "arguments": {"query": "sdd/{change-name}/{phase}", "tags": ["sdd-{phase}"], "limit": 5}
}
```

### Find by Block Type

```json
{
  "tool": "search_blocks",
  "arguments": {"block_type": "structure", "tags": ["sdd", "spec"], "limit": 20}
}
```

### Find Project Context

```json
{
  "tool": "search_blocks",
  "arguments": {"query": "sdd/{change-name}/project", "tags": ["sdd", "project"], "limit": 5}
}
```

### Find Phase Tracker

```json
{
  "tool": "search_blocks",
  "arguments": {"query": "sdd/{change-name}/tracker", "tags": ["sdd", "tracker"], "limit": 5}
}
```

## Link Types for SDD

| From | To | Link Type | Meaning |
|------|----|-----------|---------|
| Project | Tracker | `contains` | Tracker tracks project phases |
| Explore | Proposal | `refines` | Proposal based on exploration |
| Proposal | Spec | `refines` | Spec elaborates proposal |
| Proposal | Design | `refines` | Design elaborates proposal |
| Spec | Design | `refines` | Design satisfies spec |
| Design | Tasks | `contains` | Tasks implement design |
| Tasks | Progress | `related` | Progress tracks tasks |
| Spec | Verify | `supports` | Verification confirms spec |
| Tasks | Verify | `related` | Verification checks tasks |
| Project | Archive | `contains` | Archive summarizes project |

### Link Type Definitions

| Link Type | Usage |
|-----------|-------|
| `refines` | Elaboration, specification, or enhancement |
| `contains` | Container relationship (project contains phases) |
| `supports` | Verification or evidence relationship |
| `related` | General relationship without containment |

## Content Format

Artifact content is Markdown with optional JSON metadata:

```markdown
# Artifact Title

## Summary
Brief description of the artifact.

## Key Points
- Point 1
- Point 2

---

## Metadata
```json
{
  "change": "change-name",
  "phase": "phase-name",
  "created": "2026-03-20T10:00:00Z",
  "dependencies": ["01ARZ3NDEKTSV4RRFFQ69G5FAV"]
}
```
```

## Recovery Protocol

### When Starting a Phase

1. Search for required artifacts by tags
2. If not found, search by query string
3. If multiple found, use most recent (by `created_at`)
4. If none found, proceed without or report as blocker

### Finding Artifact ULIDs

```json
{
n  "tool": "search_blocks",
  "arguments": {"query": "sdd/mcp-workflow/proposal", "tags": ["sdd-proposal"]}
}
```

### Full Retrieval Pattern

```json
{
  "tool": "search_blocks",
  "arguments": {"query": "sdd/mcp-workflow/proposal", "tags": ["sdd-proposal"]}
}
```

**Check if found**: If no results, raise exception

**Get full content**:
```json
{
  "tool": "get_block",
  "arguments": {"block_id": "$proposal_ulid", "include_content": true}
}
```

## Title Format

```
sdd/{change-name}/{phase}
```

Examples:
- `sdd/mcp-workflow/project`
- `sdd/mcp-workflow/tracker`
- `sdd/mcp-workflow/proposal`
- `sdd/mcp-workflow/spec`
- `sdd/mcp-workflow/design`
- `sdd/mcp-workflow/tasks`

## Upsert Semantics

For upserts, PKM-AI uses title + tags combination:

```json
{
  "tool": "search_blocks",
  "arguments": {"query": "sdd/{change}/{phase}", "tags": ["sdd-{phase}"]}
}
```

**If block exists**: Update it
```json
{
  "tool": "update_block",
  "arguments": {"block_id": "$block_id", "content": "{new content}"}
}
```

**If block does not exist**: Create it
```json
{
  "tool": "create_block",
  "arguments": {"block_type": "permanent", "title": "sdd/{change}/{phase}", "content": "{content}", "tags": ["sdd", "sdd-{phase}", "sdd-{change}"]}
}
```

## Phase Return Envelope

All phases must return this structure:

```markdown
## {Phase} Complete

**Status**: success | blocked | failed
**Change**: {change-name}
**Phase**: {phase-name}
**Artifact ULID**: {ulid}

### Executive Summary
{2-3 sentence summary of what was accomplished}

### Artifacts
| Title | ULID | Type | Action |
|-------|------|------|--------|
| `sdd/{change}/{phase}` | `{ulid}` | `{block_type}` | Created/Updated |

### Links
| From | To | Type | Status |
|------|----|------|--------|
| `{parent_ulid}` | `{new_ulid}` | `{link_type}` | Created/Skipped |

### Content Summary
{Brief summary of artifact content}

### Next Recommended
- `sdd-{next_phase}` — if more phases remain
- `Complete` — if all phases done
- `Blocked` — if blocked by missing artifacts

### Risks
| Risk | Severity | Mitigation |
|------|----------|------------|
| {description} | High/Med/Low | {mitigation or "None"} |

### Metadata
- **Mode**: pkmai
- **Timestamp**: {ISO 8601}
- **Skill Version**: 1.0
```

## Hybrid Mode with Openspec

When mode is `hybrid`:
1. Store primary artifact in PKM-AI
2. Also write to filesystem at `openspec/changes/{change}/{phase}.md`
3. Update `openspec/changes/{change}/state.yaml`

### Filesystem Structure (hybrid)

```
openspec/changes/{change}/
├── state.yaml           # Current state
├── project.md           # Project context
├── tracker.md           # Phase tracker
├── discovery.md         # Initial discovery
├── explore.md           # Exploration
├── proposal.md          # Proposal
├── spec.md              # Specification
├── design.md            # Design
├── tasks.md             # Task list
├── progress.md          # Progress reports
├── verify.md            # Verification
└── archive.md           # Archive
```

## TDD Mode Detection

Detect TDD from (in priority order):
1. `openspec/config.yaml` → `rules.apply.tdd`
2. Project skills (e.g., `tdd/SKILL.md` exists)
3. Existing test patterns in codebase
4. Default: standard mode

If TDD detected, apply RED → GREEN → REFACTOR cycle per task.

## Quick Reference

### Tool Usage Summary

```json
{
  "tool": "search_blocks",
  "arguments": {"query": "{query}", "tags": ["{tags}"]}
}
```

**Get**:
```json
{
  "tool": "get_block",
  "arguments": {"block_id": "{ulid}", "include_content": true}
}
```

**Create**:
```json
{
  "tool": "create_block",
  "arguments": {"block_type": "{type}", "title": "sdd/{change}/{phase}", "content": "{content}", "tags": ["sdd", "sdd-{phase}", "sdd-{change}"]}
}
```

**Update**:
```json
{
  "tool": "update_block",
  "arguments": {"block_id": "{ulid}", "content": "{updated_content}"}
}
```

**Link**:
```json
{
  "tool": "create_link",
  "arguments": {"source_id": "{from}", "target_id": "{to}", "link_type": "{type}"}
}
```

### Block Types

| Type | Use Case |
|------|----------|
| `permanent` | Atomic content (explore, proposal, design, verify) |
| `structure` | Structured content with sections (spec, project) |
| `outline` | Checklist/task content (tracker, tasks) |
| `ghost` | Temporary or placeholder content |

### Tag Summary

| Tag | Meaning |
|-----|---------|
| `sdd` | SDD artifact marker |
| `sdd-{phase}` | Phase-specific (e.g., `sdd-spec`) |
| `sdd-{change}` | Change-specific (e.g., `sdd-mcp-workflow`) |

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-03-20 | Initial version |
