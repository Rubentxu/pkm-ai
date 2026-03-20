---
name: sdd-tasks
description: >
  SDD Tasks Phase - Break down into actionable tasks.
  Trigger: When assigned as sdd-tasks phase sub-agent.
license: MIT
metadata:
  author: pkm-ai
  version: "1.0"
---

## Purpose

Break down the design into actionable tasks:
- Phase-based task groupings
- Concrete task descriptions
- Dependencies between tasks
- Estimated complexity/effort

## What You Receive

- `change-name`: The name of the change
- `spec-ulid`: ULID of spec artifact
- `design-ulid`: ULID of design artifact

## Execution

### Step 1: Load Skills

Load `workflows/pkmai/sdd/_shared/phase-common.md` for return format.

### Step 2: Retrieve Spec and Design

```json
[
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
  }
]
```

### Step 3: Create Tasks Block

```json
[
  {
    "tool": "create_block",
    "args": {
      "block_type": "outline",
      "title": "sdd/{change-name}/tasks",
      "content": "# Tasks: {change-name}\n\n## Overview\n{Number of phases/tasks overview}\n\n## Phase 1: {PhaseName}\n- [ ] 1.1 {Task description}\n- [ ] 1.2 {Task description}\n  - Blocked by: 1.1\n- [ ] 1.3 {Task description}\n\n## Phase 2: {PhaseName}\n- [ ] 2.1 {Task description}\n  - Depends on: 1.3\n- [ ] 2.2 {Task description}\n\n## Task Details\n\n### 1.1: {Task Name}\n**Description**: {What to do}\n**Acceptance Criteria**:\n- [ ] {criterion}\n**Files Affected**:\n- `{file path}`\n**Complexity**: Low/Med/High\n\n### 1.2: {Task Name}\n...\n\n## Metadata\n```json\n{\n  \"change\": \"{change-name}\",\n  \"phase\": \"tasks\",\n  \"spec\": \"{spec-ulid}\",\n  \"design\": \"{design-ulid}\",\n  \"created\": \"{ISO date}\"\n}\n```",
      "tags": ["sdd", "tasks", "sdd-tasks", "sdd-{change-name}"]
    }
  }
]
```

### Step 4: Link to Spec and Design

```json
[
  {
    "tool": "create_link",
    "args": {
      "source_id": "{design-ulid}",
      "target_id": "{tasks-ulid}",
      "link_type": "contains"
    }
  },
  {
    "tool": "create_link",
    "args": {
      "source_id": "{spec-ulid}",
      "target_id": "{tasks-ulid}",
      "link_type": "supports"
    }
  }
]
```

### Step 5: Return Envelope

Return structured summary per phase-common.md format.

## Tasks Template

```markdown
# Tasks: {change-name}

## Overview
This document breaks down the implementation into {N} phases with {total} tasks.

## Phase 1: {Phase Name}

### Goals
{What this phase accomplishes}

### Tasks
- [ ] 1.1 {Task description}
- [ ] 1.2 {Task description}
- [ ] 1.3 {Task description}

## Phase 2: {Phase Name}

### Goals
{What this phase accomplishes}

### Tasks
- [ ] 2.1 {Task description}
- [ ] 2.2 {Task description}

## Phase 3: {Phase Name}
...

## Task Details

### 1.1: {Task Name}
**Phase**: 1
**Description**: {Detailed description of what to implement}
**Acceptance Criteria**:
- [ ] {Criterion 1 - from spec scenario}
- [ ] {Criterion 2 - from spec scenario}
**Files to Create**:
- `src/path/to/new_file.rs`

**Files to Modify**:
- `src/path/to/existing_file.rs`

**Dependencies**: None / Task 1.2 / External
**Complexity**: Low / Medium / High
**Estimated Time**: Xh

---

### 1.2: {Task Name}
...

## Dependencies Graph
```
1.1 → 1.2 → 1.3
           ↓
           2.1 → 2.2
                  ↓
                  3.1
```

## Task Checklist Summary

| Phase | Tasks | Completed |
|-------|-------|-----------|
| Phase 1 | 3 | 0 |
| Phase 2 | 2 | 0 |
| Phase 3 | 2 | 0 |
| **Total** | **7** | **0** |

## Metadata
```json
{
  "change": "{change-name}",
  "phase": "tasks",
  "spec": "{spec-ulid}",
  "design": "{design-ulid}",
  "total_tasks": 7,
  "phases": 3,
  "created": "{ISO date}"
}
```
```

## Rules

- Break tasks to ~2-4 hour complexity
- Include acceptance criteria from spec
- Track dependencies between tasks
- Group by logical phases
- Link to spec and design
- Return structured envelope with all required fields

## Output

Creates: `sdd/{change-name}/tasks` block (type: outline)
Links: → Design (contains), → Spec (supports)
