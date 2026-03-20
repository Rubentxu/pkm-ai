# Openspec SDD Conventions

This document describes the openspec (file-based) SDD artifact storage and retrieval. Use this when the user explicitly requests file-based storage or when `mode` is `openspec`.

## Overview

Openspec stores SDD artifacts as Markdown files on the filesystem under `openspec/changes/{change}/`. This provides:
- Human-readable artifacts
- Version control friendly
- Easy to review and edit
- No dependency on external services

## Directory Structure

```
openspec/
├── config.yaml                    # Project configuration
└── changes/
    └── {change-name}/
        ├── state.yaml             # Current state and phase tracking
        ├── project.md             # Project context
        ├── tracker.md             # Phase tracker
        ├── discovery.md           # Initial discovery
        ├── explore.md             # Exploration
        ├── proposal.md            # Proposal
        ├── spec.md                # Specification
        ├── design.md              # Design
        ├── tasks.md               # Task list
        ├── progress.md            # Progress reports
        ├── verify.md              # Verification
        └── archive.md             # Archive
```

## File Formats

### config.yaml

```yaml
project: {project-name}
version: "1.0"
sdd:
  mode: openspec
  rules:
    apply:
      tdd: false
      test_command: "cargo test"
    phases:
      require_explore: true
      require_proposal: true

metadata:
  created: {ISO date}
  modified: {ISO date}
```

### state.yaml

```yaml
change: {change-name}
status: active
current_phase: spec
mode: openspec

phases:
  explore:
    status: completed
    completed_at: {ISO date}
    artifact: explore.md
  proposal:
    status: completed
    completed_at: {ISO date}
    artifact: proposal.md
  spec:
    status: in_progress
    started_at: {ISO date}
    artifact: spec.md
  design:
    status: pending
    artifact: design.md
  tasks:
    status: pending
    artifact: tasks.md
  apply:
    status: pending
    artifact: progress.md
  verify:
    status: pending
    artifact: verify.md
  archive:
    status: pending
    artifact: archive.md

artifacts:
  - path: explore.md
    type: permanent
    phase: explore
  - path: proposal.md
    type: permanent
    phase: proposal
  # ... etc

tdd: false
created: {ISO date}
modified: {ISO date}
```

## Artifact File Formats

### project.md

```markdown
# SDD Project: {change-name}

## Overview
{One sentence description}

## Metadata
```json
{
  "change": "{change-name}",
  "project": "{project-name}",
  "status": "active",
  "mode": "openspec",
  "tdd": false
}
```

## Dependencies
- {list of dependent changes or "None"}

## Team Context
- Owner: {owner or "unassigned"}
- Phases: {team or "agent pipeline"}
```

### tracker.md

```markdown
# SDD Phase Tracker: {change-name}

## Phase Status

| Phase | Status | File | Completed |
|-------|--------|------|----------|
| explore | completed | explore.md | {date} |
| proposal | completed | proposal.md | {date} |
| spec | in_progress | spec.md | - |
| design | pending | design.md | - |
| tasks | pending | tasks.md | - |
| apply | pending | progress.md | - |
| verify | pending | verify.md | - |
| archive | pending | archive.md | - |

## Next Action
{spec|design|tasks|apply|verify|archive}

## Blocked Phases
{list of blocked phases or "None"}
```

### {phase}.md Files

Each phase artifact follows the standard SDD phase format:

```markdown
# {Phase Title}

## Summary
{2-3 sentence summary}

## Content
{Phase-specific content}

## Metadata
```json
{
  "change": "{change-name}",
  "phase": "{phase-name}",
  "status": "completed",
  "completed_at": "{ISO date}",
  "tdd": false
}
```

## Links
| To | Type | File |
|----|------|------|
| {artifact} | {link-type} | {file} |

## Next Recommended
{next phase or "Complete"}
```

## Openspec Mode Execution

### Reading Artifacts

```python
# Read from filesystem
with open(f"openspec/changes/{change}/{phase}.md", "r") as f:
    content = f.read()

# Read state
import yaml
with open(f"openspec/changes/{change}/state.yaml", "r") as f:
    state = yaml.safe_load(f)
```

### Writing Artifacts

```python
# Write artifact
with open(f"openspec/changes/{change}/{phase}.md", "w") as f:
    f.write(content)

# Update state
with open(f"openspec/changes/{change}/state.yaml", "w") as f:
    yaml.safe_dump(state, f)
```

### Marking Tasks Complete

In `tasks.md`, change `- [ ]` to `- [x]`:

```markdown
## Phase 1: Foundation

- [x] 1.1 Create `internal/auth/middleware.go` with JWT validation
- [x] 1.2 Add `AuthConfig` struct to `internal/config/config.go`
- [ ] 1.3 Add auth routes to `internal/server/server.go`  <- still pending
```

## PKM-AI Integration

When mode is `hybrid`, use PKM-AI for search and navigation while persisting to filesystem:

### Search Pattern

```json
[
  {
    "tool": "search_blocks",
    "args": {
      "query": "sdd/{change}/{phase}",
      "tags": ["sdd-{phase}"]
    }
  }
]
```

But read actual content from filesystem:
```python
with open(f"openspec/changes/{change}/{phase}.md", "r") as f:
    content = f.read()
```

### Dual Write Pattern

```json
[
  {
    "tool": "create_block",
    "args": {
      "block_type": "structure",
      "title": "sdd/{change}/{phase}",
      "content": "{content summary + reference to file}",
      "tags": ["sdd", "sdd-{phase}", "sdd-{change}"]
    }
  }
]
```

Write to filesystem for persistence:
```python
with open(f"openspec/changes/{change}/{phase}.md", "w") as f:
    f.write(full_content)
```

### Reading Pattern (Hybrid)

```python
# Always read full content from filesystem (source of truth)
with open(f"openspec/changes/{change}/{phase}.md", "r") as f:
    content = f.read()
```

Use PKM-AI for finding artifacts (navigation):
```json
[
  {
    "tool": "search_blocks",
    "args": {
      "query": "sdd/{change}/{phase}",
      "tags": ["sdd-{phase}"]
    }
  }
]
```

## Openspec-Only Mode

When mode is `openspec`:
1. Store all artifacts as files
2. Update `state.yaml` with phase status
3. Mark tasks in `tasks.md` as completed
4. Do NOT use PKM-AI tools

## TDD Mode with Openspec

When TDD is detected:
1. Read `openspec/config.yaml` → `rules.apply.tdd`
2. If true, follow RED → GREEN → REFACTOR cycle
3. Mark tasks complete as tests pass

## Phase Return Envelope (Openspec)

```markdown
## {Phase} Complete

**Status**: success | blocked | failed
**Change**: {change-name}
**Phase**: {phase-name}
**Artifact**: openspec/changes/{change}/{phase}.md

### Executive Summary
{2-3 sentence summary}

### Artifacts
| Artifact | Path | Action |
|----------|------|--------|
| {phase}.md | openspec/changes/{change}/{phase}.md | Created/Updated |

### State Updated
| Phase | Status | File |
|-------|--------|------|
| {phase} | {status} | {phase}.md |

### Next Recommended
{next phase or "Complete"}

### Risks
| Risk | Severity | Mitigation |
|------|----------|------------|
| {description} | High/Med/Low | {mitigation or "None"}
```

## Directory Creation

When initializing a new change:

```bash
mkdir -p openspec/changes/{change-name}
touch openspec/changes/{change-name}/state.yaml
```

### Initial state.yaml

```yaml
change: {change-name}
status: active
current_phase: null
mode: openspec

phases:
  explore:
    status: pending
    artifact: explore.md
  proposal:
    status: pending
    artifact: proposal.md
  spec:
    status: pending
    artifact: spec.md
  design:
    status: pending
    artifact: design.md
  tasks:
    status: pending
    artifact: tasks.md
  apply:
    status: pending
    artifact: progress.md
  verify:
    status: pending
    artifact: verify.md
  archive:
    status: pending
    artifact: archive.md

artifacts: []
tdd: false
created: {ISO date}
modified: {ISO date}
```

## Config File Reference

### rules.apply

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `tdd` | boolean | false | Enable TDD workflow |
| `test_command` | string | "cargo test" | Command to run tests |

### rules.phases

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `require_explore` | boolean | true | Require explore before proposal |
| `require_proposal` | boolean | true | Require proposal before spec |

## Quick Reference

| Mode | PKM-AI | Filesystem | Use Case |
|------|--------|-----------|----------|
| `pkmai` | Primary | Backup | Default, full PKM-AI |
| `openspec` | None | Primary | Explicit file-based |
| `hybrid` | Navigation | Primary | Best of both |
| `none` | None | None | Return only |

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-03-20 | Initial version with PKM-AI integration |
