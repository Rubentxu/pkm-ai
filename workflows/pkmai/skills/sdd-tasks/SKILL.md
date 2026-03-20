---
name: sdd-tasks
description: >
  SDD Tasks Phase - Create detailed task breakdown from spec and design.
  Uses PKM-AI block storage with block_type="outline" for checklist format.
  Trigger: When assigned as sdd-tasks phase sub-agent.
license: MIT
metadata:
  author: pkm-ai
  version: "1.0"
---

## Purpose

Create a detailed task breakdown that transforms the spec and design into actionable tasks:
- Break down features into implementable tasks
- Order tasks by dependency and priority
- Identify task owners and time estimates
- Define task completion criteria

## PKM-AI Storage

| Field | Value |
|-------|-------|
| Block Type | `outline` |
| Title Format | `sdd/{change}/tasks` |
| Tags | `["sdd", "tasks", "sdd-tasks", "sdd-{change}"]` |
| Link Type | `contains` (from design) |

## What You Receive

- `change`: The name of the change (e.g., `mcp-workflow`)
- `spec-ulid`: ULID of the spec artifact (required)
- `design-ulid`: ULID of the design artifact (required)

## Execution

### Step 1: Load Shared Conventions

Load `${PKM_AI_SHARED:-~/.pkm-ai/sdd/_shared}/phase-common.md` for the return envelope format.

### Step 2: Retrieve Spec and Design

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

### Step 3: Analyze and Break Down

For each functional requirement in the spec:
1. Identify the implementation tasks needed
2. Map to design modules and decisions
3. Break down into atomic, testable units
4. Order by dependency

### Step 4: Create Tasks Block

```json
{
  "tool": "create_block",
  "arguments": {"block_type": "outline", "title": "sdd/{change}/tasks", "content": "# Tasks: {change}\n\n## Overview\n{One paragraph describing the task breakdown approach}\n\n## Phase 1: Foundation\n- [ ] 1.1 {Task description}\n  - **Module**: {module}\n  - **Criteria**: {how to verify completion}\n  - **Estimate**: {time estimate}\n\n## Phase 2: Core Implementation\n- [ ] 2.1 {Task description}\n  - **Module**: {module}\n  - **Criteria**: {how to verify completion}\n  - **Estimate**: {time estimate}\n\n## Phase 3: Integration\n- [ ] 3.1 {Task description}\n  - **Module**: {module}\n  - **Criteria**: {how to verify completion}\n  - **Estimate**: {time estimate}\n\n## Phase 4: Polish\n- [ ] 4.1 {Task description}\n  - **Module**: {module}\n  - **Criteria**: {how to verify completion}\n  - **Estimate**: {time estimate}\n\n## Task Index\n| ID | Task | Module | Phase | Estimate |\n|----|------|--------|-------|----------|\n| 1.1 | {task} | {module} | 1 | {time} |\n\n## Metadata\n```json\n{\n  \"change\": \"{change}\",\n  \"phase\": \"tasks\",\n  \"based_on\": [\"{spec-ulid}\", \"{design-ulid}\"],\n  \"created\": \"{ISO date}\",\n  \"total_tasks\": {count},\n  \"tdd_mode\": {true|false}\n}\n```\n", "tags": ["sdd", "tasks", "sdd-tasks", "sdd-{change}"]}
}
```

### Step 5: Link to Design

```json
{
  "tool": "create_link",
  "arguments": {"source_id": "{design-ulid}", "target_id": "{tasks-ulid}", "link_type": "contains"}
}
```

### Step 6: Return Envelope

Return structured summary per `phase-common.md` format.

## Task Template

```markdown
# Tasks: {change}

## Overview
{Description of how tasks are organized and what phases they follow}

## Phase 1: {Phase Name}
- [ ] 1.1 {Task description}
  - **Module**: {module from design}
  - **Criteria**: {concrete completion criteria}
  - **Estimate**: {e.g., 2h, 1d}
  - **Tests**: {test files to create/update}

- [ ] 1.2 {Task description}
  ...

## Phase 2: {Phase Name}
- [ ] 2.1 {Task description}
  ...

## Task Dependencies
| ID | Depends On | Blocked By |
|----|------------|------------|
| 2.1 | 1.1, 1.2 | - |

## Testing Strategy
| Phase | Test Approach |
|-------|---------------|
| Phase 1 | {unit tests for foundation} |
| Phase 2 | {integration tests} |

## Metadata
```json
{
  "change": "{change}",
  "phase": "tasks",
  "based_on": ["{spec-ulid}", "{design-ulid}"],
  "created": "{ISO date}"
}
```
```

## Rules

- Use `block_type="outline"` for tasks blocks (checklist format)
- Each task must have clear completion criteria
- Group tasks into logical phases
- Tasks must be traceable to spec scenarios and design modules
- Include time estimates for planning
- Link tasks to design decisions
- Return structured envelope with all required fields

## Output

Creates: `sdd/{change}/tasks` block (type: outline)
Links: Design → Tasks (contains)

## Block Type Note

Tasks use `outline` block type because they are a checklist format with ordered items. Each task is a discrete unit of work that can be checked off when complete. The outline format supports the TDD workflow where individual tasks may be completed in RED → GREEN → REFACTOR cycles.
