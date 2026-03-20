---
name: sdd-apply
description: >
  SDD Apply Phase - Execute implementation with TDD support.
  Uses PKM-AI block storage with block_type="outline" for tasks and "permanent" for progress.
  Supports RED → GREEN → REFACTOR TDD cycle and standard mode.
  Trigger: When assigned as sdd-apply phase sub-agent.
license: MIT
metadata:
  author: pkm-ai
  version: "1.0"
---

## Purpose

Execute the task breakdown from the design and spec:
- Implement features following the specified phases
- Support TDD mode (RED → GREEN → REFACTOR)
- Support Standard mode (Read spec → Write code → Mark done)
- Track progress with updateable task checklist
- Create progress report blocks
- Follow design decisions exactly (no freelancing)

## PKM-AI Storage

| Field | Value |
|-------|-------|
| Task Block Type | `outline` |
| Progress Block Type | `permanent` |
| Tasks Title Format | `sdd/{change}/tasks` |
| Progress Title Format | `sdd/{change}/progress/{timestamp}` |
| Tasks Tags | `["sdd", "tasks", "sdd-tasks", "sdd-{change}"]` |
| Progress Tags | `["sdd", "progress", "sdd-progress", "sdd-{change}"]` |
| Link Type | `contains` (from design), `related` (tasks to progress) |

## What You Receive

- `change`: The name of the change (e.g., `mcp-workflow`)
- `tasks-ulid`: ULID of the tasks artifact (required)
- `spec-ulid`: ULID of the spec artifact (required)
- `design-ulid`: ULID of the design artifact (required)
- `tdd-mode`: Boolean indicating TDD preference (optional, default: false)

## Execution

### Step 1: Load Shared Conventions

Load `${PKM_AI_SHARED:-~/.pkm-ai/sdd/_shared}/phase-common.md` for the return envelope format.

### Step 2: Retrieve Required Artifacts

```json
[
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

### Step 3: Detect TDD Mode

Check for TDD indicators (in priority order):

1. Look for `tdd_mode` in tasks block metadata
2. Check for test patterns in the codebase
3. Default to standard mode

```python
# Check tasks block for tdd_mode flag
tdd_mode = tasks.content.metadata.get("tdd_mode", False)
```

### Step 4: Execute Tasks by Phase

#### Standard Mode

For each incomplete task:
1. Read the task criteria from tasks block
2. Read relevant spec scenario and design module
3. Implement the code
4. Verify against task criteria
5. Update tasks block with `[x]` for completed

#### TDD Mode (RED → GREEN → REFACTOR)

For each incomplete task:
1. **RED**: Write a failing test first
   - Create test file
   - Write test that describes expected behavior
   - Verify test fails
2. **GREEN**: Write minimal code to pass
   - Implement the simplest code to make test pass
   - Do not optimize, just make it work
3. **REFACTOR**: Improve code quality
   - Clean up code while tests pass
   - Apply design patterns from design decisions

### Step 5: Update Tasks Block

After completing each task, update the tasks block:

```json
[
  {
    "tool": "search_blocks",
    "args": {
      "query": "sdd/{change}/tasks",
      "tags": ["sdd-tasks"]
    }
  }
]
```

Then update with completed task marks:

```json
[
  {
    "tool": "update_block",
    "args": {
      "block_id": "{tasks-ulid}",
      "content": "{updated_content}"
    }
  }
]
```

### Step 6: Create Progress Report

After completing each phase or batch of tasks:

```json
[
  {
    "tool": "create_block",
    "args": {
      "block_type": "permanent",
      "title": "sdd/{change}/progress/{timestamp}",
      "content": "# Progress Report: {change}\n\n## Timestamp\n{ISO date}\n\n## Phase: {phase-name}\n\n## Tasks Completed\n- [x] 1.1 Task description\n- [x] 1.2 Task description\n\n## Tasks Remaining\n- [ ] 2.1 Task description\n- [ ] 2.2 Task description\n\n## TDD Cycle Log (if TDD mode)\n| Task | RED | GREEN | REFACTOR |\n|------|-----|-------|----------|\n| 1.1 | {timestamp} | {timestamp} | {timestamp} |\n\n## Design Decisions Applied\n- {AD-N}: {decision title}\n\n## Notes\n{Any observations or blockers}\n\n## Metadata\n```json\n{\n  \"change\": \"{change}\",\n  \"phase\": \"progress\",\n  \"tasks_completed\": {count},\n  \"tasks_remaining\": {count},\n  \"tdd_mode\": {true|false}\n}\n```",
      "tags": ["sdd", "progress", "sdd-progress", "sdd-{change}"]
    }
  }
]
```

### Step 7: Link Progress to Tasks

```json
[
  {
    "tool": "create_link",
    "args": {
      "source_id": "{tasks-ulid}",
      "target_id": "{progress-ulid}",
      "link_type": "related"
    }
  }
]
```

### Step 8: Return Envelope

Return structured summary per `phase-common.md` format.

## TDD Cycle Detail

### RED Phase
```
1. Write test that describes desired behavior
2. Test must compile but fail (not build errors)
3. Test name should describe what it verifies
4. Include assertions for expected behavior

Example:
#[test]
fn test_feature_returns_correct_value() {
    let result = calculate_value();
    assert_eq!(result, expected_value);
}
```

### GREEN Phase
```
1. Write minimal code to make test pass
2. Hardcode values if necessary
3. Do not optimize or add features
4. Focus on making the test green

Example:
fn calculate_value() -> i32 {
    42 // minimal implementation
}
```

### REFACTOR Phase
```
1. Clean up code while tests pass
2. Remove duplication
3. Apply design patterns
4. Improve readability
5. Ensure tests still pass after each change
```

## Apply Template

```markdown
# Apply: {change}

## Mode: {Standard|TDD}

## Current Phase: {phase}

## Progress

### Completed
- [x] 1.1 {Task}
- [x] 1.2 {Task}

### In Progress
- [ ] 2.1 {Task} (currently working)

### Pending
- [ ] 2.2 {Task}
- [ ] 3.1 {Task}

## TDD Log (if TDD mode)
| Task | RED | GREEN | REFACTOR | Status |
|------|-----|-------|----------|--------|
| 1.1 | 10:00 | 10:15 | 10:30 | Done |
| 1.2 | 10:35 | 10:45 | 11:00 | Done |
| 2.1 | 11:05 | - | - | In Progress |

## Design Decisions Applied
- [x] AD-1: {title} - {how applied}
- [x] AD-2: {title} - {how applied}

## Notes
{observations, blockers, decisions made}

## Metadata
```json
{
  "change": "{change}",
  "phase": "apply",
  "mode": "{standard|tdd}",
  "started": "{ISO date}",
  "tasks_completed": {n},
  "tasks_total": {n}
}
```
```

## Rules

### Core Principles
- **Follow design decisions exactly** - Do not freelance or improve upon the design
- **One task at a time** - Complete current task before moving to next
- **Verify against criteria** - Each task has completion criteria in the tasks block
- **Update progress frequently** - Create progress blocks after each phase

### TDD Rules
- Always write the failing test first (RED)
- Write minimal code to pass (GREEN)
- Refactor only after tests pass (REFACTOR)
- All tests must pass before calling a task complete
- Never skip the REFACTOR phase for "simplicity"

### Standard Mode Rules
- Read spec scenario before implementing
- Read design module before implementing
- Write code that satisfies the task criteria
- Mark task complete only when criteria verified

### Progress Tracking
- Update tasks block after each task completion
- Create progress report after each phase
- Log TDD cycle times if in TDD mode
- Note any design decisions applied

## Output

Creates:
- `sdd/{change}/progress/{timestamp}` block (type: permanent)
- Updates: `sdd/{change}/tasks` block (marking tasks as [x])

Links:
- Tasks → Progress (related)
- Design → Progress (via tasks link chain)

## Block Type Note

Apply uses `outline` for tasks tracking (updating [ ] to [x]) and `permanent` for progress reports (atomic status snapshots). The progress block is `permanent` because each progress report is an immutable snapshot at a point in time, useful for tracking implementation history.
