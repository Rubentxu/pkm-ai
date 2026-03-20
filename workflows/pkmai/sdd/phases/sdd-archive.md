---
name: sdd-archive
description: >
  SDD Archive Phase - Archive completed change with summary.
  Trigger: When assigned as sdd-archive phase sub-agent.
license: MIT
metadata:
  author: pkm-ai
  version: "1.0"
---

## Purpose

Archive a completed change:
- Summarize the entire change
- Document lessons learned
- Archive all artifacts
- Mark change as complete

## What You Receive

- `change-name`: The name of the change
- All artifact ULIDs from previous phases

## Execution

### Step 1: Load Skills

Load `workflows/pkmai/sdd/_shared/phase-common.md` for return format.

### Step 2: Retrieve All Artifacts

```json
[
  {
    "tool": "get_block",
    "args": {
      "block_id": "{proposal-ulid}",
      "include_content": true
    }
  },
  {
    "tool": "get_block",
    "args": {
      "block_id": "{spec-ulid}",
      "include_content": true
    }
  },
  {
    "tool": "get_block",
    "args": {
      "block_id": "{design-ulid}",
      "include_content": true
    }
  },
  {
    "tool": "get_block",
    "args": {
      "block_id": "{tasks-ulid}",
      "include_content": true
    }
  },
  {
    "tool": "get_block",
    "args": {
      "block_id": "{verify-ulid}",
      "include_content": true
    }
  }
]
```

### Step 3: Create Archive Block

```json
[
  {
    "tool": "create_block",
    "args": {
      "block_type": "permanent",
      "title": "sdd/{change-name}/archive",
      "content": "# Archive: {change-name}\n\n## Summary\n{Executive summary of the completed change}\n\n## Change Overview\n- **Status**: Completed\n- **Duration**: {start date} - {end date}\n- **Phases Completed**: All 7\n\n## What Was Built\n{2-3 paragraphs on what was delivered}\n\n## Key Artifacts\n| Artifact | ULID | Description |\n|----------|------|-------------|\n| Proposal | {ulid} | Problem and solution |\n| Spec | {ulid} | Detailed specification |\n| Design | {ulid} | Technical design |\n| Tasks | {ulid} | Implementation tasks |\n| Verify | {ulid} | Verification report |\n\n## Statistics\n- **Tasks Completed**: {N}/{M}\n- **Tests**: {X} passed\n- **Code Changed**: {files} files, {lines} lines\n\n## Lessons Learned\n### What Went Well\n- {lesson 1}\n- {lesson 2}\n\n### What Could Be Improved\n- {lesson 1}\n- {lesson 2}\n\n### Technical Debt\n- {item 1}\n- {item 2}\n\n## Reusable Patterns\n- {pattern 1}\n- {pattern 2}\n\n## References\n- {reference 1}\n- {reference 2}\n\n## Metadata\n```json\n{\n  \"change\": \"{change-name}\",\n  \"phase\": \"archive\",\n  \"status\": \"completed\",\n  \"completed\": \"{ISO date}\",\n  \"proposal\": \"{ulid}\",\n  \"spec\": \"{ulid}\",\n  \"design\": \"{ulid}\",\n  \"tasks\": \"{ulid}\",\n  \"verify\": \"{ulid}\"\n}\n```",
      "tags": ["sdd", "archive", "sdd-archive", "sdd-{change-name}"]
    }
  }
]
```

### Step 4: Link All Artifacts to Archive

```json
[
  {
    "tool": "create_link",
    "args": {
      "source_id": "{archive-ulid}",
      "target_id": "{proposal-ulid}",
      "link_type": "related"
    }
  },
  {
    "tool": "create_link",
    "args": {
      "source_id": "{archive-ulid}",
      "target_id": "{spec-ulid}",
      "link_type": "related"
    }
  },
  {
    "tool": "create_link",
    "args": {
      "source_id": "{archive-ulid}",
      "target_id": "{design-ulid}",
      "link_type": "related"
    }
  },
  {
    "tool": "create_link",
    "args": {
      "source_id": "{archive-ulid}",
      "target_id": "{tasks-ulid}",
      "link_type": "related"
    }
  },
  {
    "tool": "create_link",
    "args": {
      "source_id": "{archive-ulid}",
      "target_id": "{verify-ulid}",
      "link_type": "related"
    }
  }
]
```

### Step 5: Return Envelope

Return structured summary per phase-common.md format.

## Archive Template

```markdown
# Archive: {change-name}

**Status**: ✅ Completed
**Archived**: {ISO date}

## Executive Summary

{2-3 paragraphs summarizing the change, what was built, and the outcome.}

## Change Metadata

| Field | Value |
|-------|-------|
| Change Name | {change-name} |
| Status | Completed |
| Start Date | {date} |
| End Date | {date} |
| Duration | {X weeks/days} |
| Proposer | {name} |
| Implementor | {name} |

## Problem Solved

{Description of the problem this change addressed.}

## Solution Delivered

{Description of the solution implemented.}

## Key Deliverables

### Artifacts Produced
| Artifact | Block ULID | Description |
|----------|------------|-------------|
| Proposal | `{ulid}` | Problem statement and proposed solution |
| Specification | `{ulid}` | Detailed functional requirements |
| Design | `{ulid}` | Technical architecture and decisions |
| Task List | `{ulid}` | Implementation breakdown |
| Verification | `{ulid}` | Test results and verification |

### Code Deliverables
| Component | Files | Lines Added | Lines Changed |
|-----------|-------|-------------|---------------|
| Core | {N} | +{X} | ~{Y} |
| Tests | {N} | +{X} | ~{Y} |
| Config | {N} | +{X} | ~{Y} |
| **Total** | **{N}** | **+{X}** | **~{Y}** |

## Verification Summary

| Category | Target | Actual | Status |
|----------|--------|--------|--------|
| Unit Test Coverage | {X%} | {Y%} | ✅ |
| Integration Tests | {N} | {M} | ✅ |
| Acceptance Criteria | {N} | {M} | ✅ |

## Lessons Learned

### What Went Well
1. **{Positive Pattern}**: {description of what worked}
2. **{Positive Pattern}**: {description}

### Challenges Faced
1. **{Challenge}**: {description and how it was resolved}
2. **{Challenge}**: {description and how it was resolved}

### Insights for Future
- **{Insight}**: {what this teaches us}
- **{Insight}**: {what we would do differently}

## Technical Debt

| Item | Severity | Description | Recommendation |
|------|----------|-------------|----------------|
| {item} | High/Med/Low | {description} | {action} |

## Reusable Components

The following components can be reused in future changes:

### {Component Name}
**Purpose**: {what it does}
**Location**: `src/{path}`
**Usage**: {how to use it}

## Related Changes

- **{Change}**: {relationship}
- **{Change}**: {relationship}

## Sign-off

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Developer | {name} | {date} | {signature} |
| Reviewer | {name} | {date} | {signature} |
| Product Owner | {name} | {date} | {signature} |

## Metadata
```json
{
  "change": "{change-name}",
  "phase": "archive",
  "status": "completed",
  "completed": "{ISO date}",
  "duration_days": {N},
  "tasks_completed": {N},
  "tasks_total": {M},
  "test_coverage": "{X%}",
  "artifacts": {
    "proposal": "{ulid}",
    "spec": "{ulid}",
    "design": "{ulid}",
    "tasks": "{ulid}",
    "verify": "{ulid}"
  }
}
```

---

**This change is archived and considered complete.**
```

## Rules

- Summarize the entire change lifecycle
- Capture lessons learned while they're fresh
- Document reusable patterns for future reference
- Link to all artifacts for traceability
- Be objective about challenges and improvements
- Return structured envelope with all required fields

## Output

Creates: `sdd/{change-name}/archive` block
Links: → All other artifacts (related)
