---
name: sdd-archive
description: >
  SDD Archive Phase - Create final archive summary of the completed change.
  Uses PKM-AI block storage with block_type="permanent".
  Trigger: When assigned as sdd-archive phase sub-agent.
license: MIT
metadata:
  author: pkm-ai
  version: "1.0"
---

## Purpose

Create a comprehensive archive of the completed change that serves as:
- A final summary of what was built
- A record of lessons learned and decisions made
- A reference for future similar work
- A complete artifact linking all phases of the change

## PKM-AI Storage

| Field | Value |
|-------|-------|
| Block Type | `permanent` |
| Title Format | `sdd/{change}/archive` |
| Tags | `["sdd", "archive", "sdd-archive", "sdd-{change}"]` |
| Link Type | `related` (to all artifacts) |

## What You Receive

- `change`: The name of the change (e.g., `mcp-workflow`)
- All artifact ULIDs from previous phases (proposal, spec, design, tasks, progress, verify)

## Execution

### Step 1: Load Shared Conventions

Load `workflows/pkmai/skills/_shared/sdd-phase-common.md` for the return envelope format.
Load `workflows/pkmai/skills/_shared/pkmai-convention.md` for PKM-AI conventions.

### Step 2: Retrieve All Phase Artifacts

```json
{
  "tool": "get_block",
  "arguments": {"block_id": "{proposal-ulid}", "include_content": true}
}
```

```json
{
  "tool": "get_block",
  "arguments": {"block_id": "{spec-ulid}", "include_content": true}
}
```

```json
{
  "tool": "get_block",
  "arguments": {"block_id": "{design-ulid}", "include_content": true}
}
```

```json
{
  "tool": "get_block",
  "arguments": {"block_id": "{tasks-ulid}", "include_content": true}
}
```

```json
{
  "tool": "get_block",
  "arguments": {"block_id": "{progress-ulid}", "include_content": true}
}
```

```json
{
  "tool": "get_block",
  "arguments": {"block_id": "{verify-ulid}", "include_content": true}
}
```

### Step 3: Analyze Change History

From the retrieved artifacts, extract:
- Original problem/opportunity from proposal
- Key requirements and decisions from spec/design
- Tasks completed and their status
- Verification results
- Any notable deviations or lessons learned

### Step 4: Create Archive Block

```json
{
  "tool": "create_block",
  "arguments": {"block_type": "permanent", "title": "sdd/{change}/archive", "content": "# Archive: {change}\n\n## Summary\n\n{2-3 sentence executive summary of the completed change}\n\n## What Was Built\n\n### Overview\n{Description of what was delivered}\n\n### Key Features Delivered\n1. **{Feature 1}**: {brief description}\n2. **{Feature 2}**: {brief description}\n3. **{Feature 3}**: {brief description}\n\n### Components Changed\n| Component | Type | Description |\n|-----------|------|-------------|\n| {component} | module/file | {description} |\n| {component} | module/file | {description} |\n\n## Requirements Fulfilled\n\n### From Proposal\n| Requirement | Status | Implementation |\n|-------------|--------|----------------|\n| {req} | Fulfilled/Partial/Deferred | {how} |\n\n### From Spec\n| Acceptance Criterion | Verification Result |\n|---------------------|---------------------|\n| {criterion} | {PASS/FAIL} |\n\n## Key Decisions\n\n### AD-1: {Decision Title}\n**Context**: {problem or situation}\n**Decision**: {what was decided}\n**Rationale**: {why this approach}\n\n### AD-2: {Decision Title}\n...\n\n## Lessons Learned\n\n### What Went Well\n1. **{Aspect}**: {why it worked well}\n2. **{Aspect}**: {why it worked well}\n\n### What Could Be Improved\n1. **{Aspect}**: {suggestion for improvement}\n2. **{Aspect}**: {suggestion for improvement}\n\n### Risks Realized\n| Risk | Impact | How It Affected the Project |\n|------|--------|----------------------------|\n| {risk} | High/Med/Low | {effect} |\n\n## Reusable Patterns\n\n### Pattern 1: {Pattern Name}\n**Problem**: {what problem this solves}\n**Solution**: {brief description}\n**Applicability**: {when to use this pattern}\n\n### Pattern 2: {Pattern Name}\n...\n\n## Technical Debt\n\n| Item | Severity | Description | Resolution |\n|------|----------|-------------|------------|\n| {debt} | High/Med/Low | {desc} | {future action} |\n\n## Statistics\n\n| Metric | Value |\n|--------|-------|\n| Duration | {start} to {end} |\n| Tasks Completed | {n}/{total} |\n| Tests Added | {n} |\n| Files Changed | {n} |\n| Lines Added | ~{n} |\n| Lines Removed | ~{n} |\n\n## Artifact Links\n\n| Phase | ULID | Title |\n|-------|------|-------|\n| Proposal | {ulid} | sdd/{change}/proposal |\n| Spec | {ulid} | sdd/{change}/spec |\n| Design | {ulid} | sdd/{change}/design |\n| Tasks | {ulid} | sdd/{change}/tasks |\n| Progress | {ulid} | sdd/{change}/progress |\n| Verify | {ulid} | sdd/{change}/verify |\n\n## Metadata\n```json\n{\n  \"change\": \"{change}\",\n  \"phase\": \"archive\",\n  \"status\": \"completed\",\n  \"duration\": {\n    \"start\": \"{ISO date}\",\n    \"end\": \"{ISO date}\"\n  },\n  \"artifacts\": {\n    \"proposal\": \"{ulid}\",\n    \"spec\": \"{ulid}\",\n    \"design\": \"{ulid}\",\n    \"tasks\": \"{ulid}\",\n    \"progress\": \"{ulid}\",\n    \"verify\": \"{ulid}\"\n  },\n  \"statistics\": {\n    \"tasks_completed\": {n},\n    \"tasks_total\": {n},\n    \"tests_added\": {n},\n    \"files_changed\": {n}\n  },\n  \"created\": \"{ISO date}\"\n}\n```\n", "tags": ["sdd", "archive", "sdd-archive", "sdd-{change}"]}
}
```

### Step 5: Create Links to All Artifacts

```json
{
  "tool": "create_link",
  "arguments": {"source_id": "{archive-ulid}", "target_id": "{proposal-ulid}", "link_type": "related"}
}
```

```json
{
  "tool": "create_link",
  "arguments": {"source_id": "{archive-ulid}", "target_id": "{spec-ulid}", "link_type": "related"}
}
```

```json
{
  "tool": "create_link",
  "arguments": {"source_id": "{archive-ulid}", "target_id": "{design-ulid}", "link_type": "related"}
}
```

```json
{
  "tool": "create_link",
  "arguments": {"source_id": "{archive-ulid}", "target_id": "{tasks-ulid}", "link_type": "related"}
}
```

```json
{
  "tool": "create_link",
  "arguments": {"source_id": "{archive-ulid}", "target_id": "{progress-ulid}", "link_type": "related"}
}
```

```json
{
  "tool": "create_link",
  "arguments": {"source_id": "{archive-ulid}", "target_id": "{verify-ulid}", "link_type": "related"}
}
```

### Step 6: Return Envelope

Return structured summary per `phase-common.md` format.

## Archive Template

```markdown
# Archive: {change}

## Executive Summary

{2-3 paragraph overview of:
- What the change was about
- What was delivered
- Key outcomes and impact}

## Project Overview

### Problem Statement
{From proposal - the original problem or opportunity}

### Solution Delivered
{From spec/design - what was built}

### Target Users
{Who will benefit from this change}

## What Was Built

### Features
1. **{Feature Name}**: {description}
   - **Priority**: {P0/P1/P2}
   - **Status**: {Delivered/Deferred/Removed}

### Technical Changes

#### New Files
| Path | Purpose |
|------|---------|
| {path} | {purpose} |

#### Modified Files
| Path | Change Type | Description |
|------|-------------|-------------|
| {path} | feature/refactor/fix | {desc} |

#### Removed Files
| Path | Reason |
|------|--------|
| {path} | {reason} |

## Requirements Traceability

### Acceptance Criteria

| Criterion | Source | Status | Evidence |
|-----------|--------|--------|----------|
| {criterion} | Spec | PASS/FAIL | {test/file} |

### Deferred Requirements

| Requirement | Original Priority | Reason for Deferral | Future Plan |
|-------------|-------------------|---------------------|-------------|
| {req} | P0/P1/P2 | {reason} | {plan} |

## Key Decisions Made

### AD-1: {Decision Title}
- **Context**: {problem}
- **Decision**: {what was chosen}
- **Rationale**: {why}
- **Alternatives**: {what else was considered}

## Lessons Learned

### Successes
1. **{What worked well}**: {why}
   - **{Detail}**: {specific example}

### Challenges
1. **{Challenge encountered}**: {how it was addressed}
   - **{Detail}**: {specific example}

### Insights
- **{Insight 1}**: {what was learned}
- **{Insight 2}**: {what was learned}

## Reusable Patterns Discovered

### Pattern: {Name}
**Context**: {when to use}
**Implementation**: {how}
**Example**: {specific use case from this project}

## Technical Debt

| Item | Severity | Description | Remediation |
|------|----------|-------------|-------------|
| {debt} | High/Med/Low | {desc} | {future work} |

## Metrics

| Metric | Value |
|--------|-------|
| Total Duration | {n} weeks |
| Development Time | {n} days |
| Tasks Completed | {n}/{total} ({pct}%) |
| Tests Added | {n} |
| Test Coverage | {pct}% |
| Files Changed | {n} |
| Lines Added | ~{n} |
| Lines Removed | ~{n} |
| Breaking Changes | {n} |

## Future Work

### Recommended Follow-ups
1. **{Item}**: {reason and expected benefit}
2. **{Item}**: {reason and expected benefit}

### Related Future Changes
| Change | Relationship | Description |
|--------|-------------|-------------|
| {change} | builds upon | {relationship} |

## Artifact Index

All artifacts from this SDD are linked below:

| Artifact | ULID | Created | Key Content |
|----------|------|---------|-------------|
| Proposal | {ulid} | {date} | {summary} |
| Spec | {ulid} | {date} | {summary} |
| Design | {ulid} | {date} | {summary} |
| Tasks | {ulid} | {date} | {n} tasks |
| Progress | {ulid} | {date} | {n} updates |
| Verify | {ulid} | {date} | {n} criteria checked |

## Metadata
```json
{
  "change": "{change}",
  "phase": "archive",
  "status": "completed",
  "archive_date": "{ISO date}",
  "duration": {
    "initiated": "{ISO date}",
    "completed": "{ISO date}",
    "total_days": {n}
  },
  "statistics": {
    "tasks_completed": {n},
    "tasks_total": {n},
    "files_changed": {n},
    "tests_added": {n},
    "lines_added": {n},
    "lines_removed": {n}
  },
  "quality": {
    "acceptance_criteria_passed": {n},
    "acceptance_criteria_total": {n},
    "test_coverage": "{pct}%"
  },
  "artifacts": {
    "proposal": "{ulid}",
    "spec": "{ulid}",
    "design": "{ulid}",
    "tasks": "{ulid}",
    "progress": "{ulid}",
    "verify": "{ulid}",
    "archive": "{ulid}"
  },
  "created": "{ISO date}"
}
```
```

## Rules

- Use `block_type="permanent"` for archive blocks
- Create comprehensive links to all phase artifacts
- Focus on objective recording of facts and decisions
- Include lessons learned while context is fresh
- Extract reusable patterns for future reference
- Provide statistics where available
- Return structured envelope with all required fields

## Output

Creates: `sdd/{change}/archive` block (type: permanent)
Links: Archive → All phase artifacts (related)

## Block Type Note

Archive uses `permanent` block type because archives are permanent historical records. The archive represents the final state of a completed change and should never be modified. Any updates or corrections should be made as new artifacts rather than editing the archive.
