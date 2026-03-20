---
name: sdd-apply
description: >
  SDD Apply Phase - Implement tasks from the change.
  Trigger: When assigned as sdd-apply phase sub-agent.
license: MIT
metadata:
  author: pkm-ai
  version: "1.0"
---

## Purpose

Implement assigned tasks following the spec and design:
- Write actual code
- Follow TDD or standard workflow
- Update task completion status
- Track progress

## What You Receive

- `change-name`: The name of the change
- `tasks-to-implement`: Specific task IDs (e.g., "1.1-1.3")
- `tasks-ulid`: ULID of tasks block
- `spec-ulid`: ULID of spec block
- `design-ulid`: ULID of design block
- `mode`: Implementation mode (`tdd` or `standard`)

## Execution

### Step 1: Load Skills

Load `workflows/pkmai/sdd/_shared/phase-common.md` for return format.

### Step 2: Retrieve Artifacts

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

### Step 3: Implement Each Task

For TDD mode:
```
FOR EACH TASK:
├── 1. RED - Write failing test
├── 2. GREEN - Write minimal code to pass
├── 3. REFACTOR - Clean up
└── 4. Mark complete
```

For Standard mode:
```
FOR EACH TASK:
├── 1. Read task description
├── 2. Read relevant spec/design
├── 3. Write code
└── 4. Mark complete
```

### Step 4: Update Tasks Block

When a task is completed, update the block:
```json
[
  {
    "tool": "update_block",
    "args": {
      "block_id": "{tasks-ulid}",
      "content": "{updated content with [x] marks}"
    }
  }
]
```

### Step 5: Create Progress Block

```json
[
  {
    "tool": "create_block",
    "args": {
      "block_type": "permanent",
      "title": "sdd/{change-name}/progress/{timestamp}",
      "content": "# Progress Report: {change-name}\n\n## Summary\n{Overview of what was accomplished}\n\n## Tasks Completed\n- [x] 1.1 {Task description}\n- [x] 1.2 {Task description}\n- [ ] 1.3 {Task description}\n\n## Tasks Remaining\n- [ ] 1.3 {Task description}\n- [ ] 2.1 {Task description}\n\n## Files Changed\n| File | Action | Summary |\n|------|--------|---------|\n| `src/file.rs` | Modified | {what changed} |\n\n## Tests\n| Task | Test File | Result |\n|------|-----------|--------|\n| 1.1 | `tests/file.rs` | Pass |\n| 1.2 | `tests/file.rs` | Pass |\n\n## Deviations from Design\n{Any deviations or issues found}\n\n## Next Tasks\n- 1.3 (depends on 1.2)\n- 2.1\n\n## Metadata\n```json\n{\n  \"change\": \"{change-name}\",\n  \"phase\": \"progress\",\n  \"tasks_completed\": 2,\n  \"tasks_total\": 7,\n  \"created\": \"{ISO date}\"\n}\n```",
      "tags": ["sdd", "progress", "sdd-progress", "sdd-{change-name}"]
    }
  }
]
```

### Step 6: Link Progress to Tasks

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

### Step 7: Return Envelope

Return structured summary per phase-common.md format.

## Apply Workflow (TDD Mode)

```python
FOR EACH TASK:
├── UNDERSTAND
│   ├── Read task description
│   ├── Read relevant spec scenarios
│   ├── Read design decisions
│   └── Read existing code patterns
│
├── RED — Write failing test FIRST
│   ├── Write test that describes expected behavior
│   ├── Run test — confirm it FAILS
│   └── If passes immediately → behavior exists or test wrong
│
├── GREEN — Write minimum code to pass
│   ├── Implement ONLY what's needed
│   ├── Run tests — confirm they PASS
│   └── Do NOT add extra functionality
│
├── REFACTOR — Clean up
│   ├── Improve structure, naming, duplication
│   ├── Run tests — confirm STILL PASS
│   └── Match project conventions
│
└── Mark task complete in tasks block
```

## Apply Workflow (Standard Mode)

```python
FOR EACH TASK:
├── Read task description
├── Read relevant spec scenarios
├── Read design decisions
├── Read existing code patterns
├── Write code following project conventions
├── Mark task complete in tasks block
└── Note any issues or deviations
```

## Task Implementation Template

```markdown
### {Task ID}: {Task Name}

**Status**: Pending / In Progress / Done

**Description**:
{What to implement}

**Acceptance Criteria**:
- [ ] {criterion from spec}

**Implementation**:
```rust
// Code implementation
```

**Test**:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_feature() {
        // Test implementation
    }
}
```

**Notes**:
- {any issues or decisions made during implementation}
```

## Progress Report Template

```markdown
# Progress Report: {change-name}

**Date**: {ISO date}
**Phase**: {phase number}

## Summary
{2-3 sentences on what was accomplished}

## Tasks Status

| Task | Status | Notes |
|------|--------|-------|
| 1.1 | ✅ Done | {notes} |
| 1.2 | ✅ Done | {notes} |
| 1.3 | 🔄 In Progress | {notes} |
| 2.1 | ⏳ Pending | {notes} |

## Completed This Session

### {Task ID}: {Task Name}
- Files: created/modified
- Tests: X new, Y passing
- Deviation: None / {description}

### {Task ID}: {Task Name}
...

## Files Changed

### Created
- `src/new_file.rs` - {description}

### Modified
- `src/existing_file.rs` - {changes made}

## Issues Found
| Issue | Severity | Resolution |
|-------|----------|------------|
| {issue} | High/Med/Low | {resolution} |

## Next Steps
- Complete task 1.3
- Start task 2.1
- Update spec if needed

## Blockers
- None / {blocker description}

## Metadata
```json
{
  "change": "{change-name}",
  "phase": "progress",
  "tasks_completed": {N},
  "tasks_remaining": {M},
  "created": "{ISO date}"
}
```
```

## Rules

- ALWAYS read spec before implementing — specs are acceptance criteria
- ALWAYS follow design decisions — don't freelance different approach
- ALWAYS match existing code patterns
- If TDD mode, ALWAYS follow RED → GREEN → REFACTOR cycle
- Update tasks block as you complete tasks
- Create progress block to track overall status
- Return structured envelope with all required fields

## Output

Creates: `sdd/{change-name}/progress/{timestamp}` block
Links: → Tasks block
