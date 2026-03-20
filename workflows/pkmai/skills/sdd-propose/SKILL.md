---
name: sdd-propose
description: >
  SDD Proposal Phase - Create a proposal for the change.
  Trigger: When assigned as sdd-propose phase sub-agent.
license: MIT
metadata:
  author: pkm-ai
  version: "1.0"
---

## Purpose

Create a structured proposal that defines:
- The problem being solved
- Proposed solution approach
- Expected outcomes and benefits
- Scope and constraints

## What You Receive

- `change-name`: The name of the change
- `explore-ulid`: ULID of exploration artifact (if exists)

## Execution

### Step 1: Load Skills

Load `${PKM_AI_SHARED:-~/.pkm-ai/sdd/_shared}/phase-common.md` for return format.
Load `${PKM_AI_SHARED:-~/.pkm-ai/sdd/_shared}/pkmai-convention.md` for PKM-AI conventions.

### Step 2: Check for Existing Proposal

Before creating, check if proposal already exists:

```json
{
  "tool": "search_blocks",
  "arguments": {"query": "sdd/{change-name}/proposal", "tags": ["sdd-proposal"]}
}
```

If found with matching `change-name`, update existing. If not found, create new.

### Step 3: Retrieve Exploration (if provided)

If `explore-ulid` provided, retrieve it:
```json
{
  "tool": "get_block",
  "arguments": {"block_id": "{explore-ulid}", "include_content": true}
}
```

### Step 4: Create Proposal Block

```json
{
  "tool": "create_block",
  "arguments": {"block_type": "permanent", "title": "sdd/{change-name}/proposal", "content": "# Proposal: {change-name}\n\n## Problem Statement\n{What problem does this change solve?}\n\n## Proposed Solution\n{How will we solve it?}\n\n## Expected Outcomes\n- {outcome 1}\n- {outcome 2}\n\n## Benefits\n- {benefit 1}\n- {benefit 2}\n\n## Scope\n### In Scope\n- {item in scope}\n\n### Out of Scope\n- {item out of scope}\n\n## Constraints\n- {constraint 1}\n- {constraint 2}\n\n## Risks\n| Risk | Impact | Mitigation |\n|------|--------|------------|\n| {risk} | High/Med/Low | {mitigation} |\n\n## Success Criteria\n- {criterion 1}\n- {criterion 2}\n\n## Metadata\n```json\n{\n  \"change\": \"{change-name}\",\n  \"phase\": \"proposal\",\n  \"based_on\": \"{explore-ulid or null}\",\n  \"created\": \"{ISO date}\"\n}\n```\n", "tags": ["sdd", "proposal", "sdd-proposal", "sdd-{change-name}"]}
}
```

### Step 5: Link to Exploration (if exists)

```json
{
n  "tool": "create_link",
  "arguments": {"source_id": "{explore-ulid}", "target_id": "{proposal-ulid}", "link_type": "refines"}
}
```

### Step 6: Return Envelope

Return structured summary per phase-common.md format.

## Proposal Template

```markdown
# Proposal: {change-name}

## Problem Statement
{Formal problem description. Be specific.}

## Proposed Solution
{Description of the proposed approach.}

## Motivation
{Why is this change needed? What triggers it?}

## Expected Outcomes

### Primary Outcomes
1. **{Outcome}**: {description}

### Secondary Outcomes
1. **{Outcome}**: {description}

## Benefits
| Benefit | Impact | Effort |
|---------|--------|--------|
| {benefit} | High/Med/Low | High/Med/Low |

## Scope

### In Scope
- Feature or change 1
- Feature or change 2

### Out of Scope
- Related item 1 (explain why)
- Related item 2 (explain why)

## Constraints
1. **{Constraint}**: {description}
2. **{Constraint}**: {description}

## Assumptions
- {assumption 1}
- {assumption 2}

## Risks
| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| {risk} | High/Med/Low | High/Med/Low | {mitigation} |

## Success Criteria
- [ ] {Criterion 1}
- [ ] {Criterion 2}

## Alternatives Considered
### Alternative 1
{Why not chosen}

### Alternative 2
{Why not chosen}

## Metadata
```json
{
  "change": "{change-name}",
  "phase": "proposal",
  "based_on": "{explore-ulid or null}",
  "created": "{ISO date}"
}
```
```

## Rules

- Check for existing proposal before creating new
- Base proposal on exploration findings (if available)
- Be specific about scope and constraints
- Identify risks proactively
- Don't go into implementation details (that's for design phase)
- Return structured envelope with all required fields

## Output

Creates: `sdd/{change-name}/proposal` block
Links: -> Exploration (if exists)

## PKM-AI Tool Reference

```json
{
  "tool": "search_blocks",
  "arguments": {"query": "sdd/{change-name}/proposal", "tags": ["sdd-proposal"]}
}
```

```json
{
  "tool": "get_block",
  "arguments": {"block_id": "{explore-ulid}", "include_content": true}
}
```

```json
{
  "tool": "create_block",
  "arguments": {"block_type": "permanent", "title": "sdd/{change-name}/proposal", "content": "{markdown content}", "tags": ["sdd", "proposal", "sdd-proposal", "sdd-{change-name}"]}
}
```

```json
{
  "tool": "create_link",
  "arguments": {"source_id": "{explore-ulid}", "target_id": "{proposal-ulid}", "link_type": "refines"}
}
```
