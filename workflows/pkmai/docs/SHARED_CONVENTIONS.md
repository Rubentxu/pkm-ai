# PKM-AI Shared Conventions

## Overview

Shared conventions provide common patterns and formats used across all SDD phase skills.

## Convention Files

| File | Purpose |
|------|---------|
| `phase-common.md` | Return envelope format |
| `pkmai-convention.md` | PKM-AI tool conventions |
| `openspec-convention.md` | File-based alternative |
| `persistence-contract.md` | Storage contracts |

## Loading Conventions

Skills load conventions from `${PKM_AI_SHARED}`:

```python
import os
SHARED = os.environ.get("PKM_AI_SHARED", os.path.expanduser("~/.pkm-ai/sdd/_shared"))

# Load required conventions
with open(f"{SHARED}/phase-common.md") as f:
    PHASE_COMMON = f.read()
```

## phase-common.md

Defines the return envelope format for all SDD phases.

### Envelope Structure

```markdown
## Phase Complete

**Status**: success | blocked | failed
**Change**: {change-name}
**Phase**: {phase-name}
**Artifact ULID**: {ulid}

### Executive Summary
{2-3 sentences max}

### Artifacts
| Artifact | ULID | Block Type | Action |
```

## pkmai-convention.md

PKM-AI specific conventions for SDD artifact storage.

### Block Types by Phase

| Phase | Block Type |
|-------|------------|
| project | `structure` |
| tracker | `outline` |
| explore | `permanent` |
| proposal | `permanent` |
| spec | `structure` |
| design | `permanent` |
| tasks | `outline` |
| progress | `permanent` |
| verify | `permanent` |
| archive | `permanent` |

### Tag Conventions

All SDD artifacts MUST have:
- `sdd` - SDD marker
- `sdd-{phase}` - Phase marker
- `sdd-{change}` - Change marker

### Link Types

| From | To | Link Type |
|------|----|-----------|
| Proposal | Spec | `refines` |
| Spec | Design | `refines` |
| Design | Tasks | `contains` |

## persistence-contract.md

Defines the persistence contract for SDD artifacts.

### Mode Selection

| Mode | Behavior |
|------|----------|
| `pkmai` | Default, PKM-AI blocks |
| `openspec` | File-based |
| `hybrid` | Both backends |
| `none` | Return only |

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-03-20 | Initial version |
