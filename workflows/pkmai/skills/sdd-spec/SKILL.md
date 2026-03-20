---
name: sdd-spec
description: >
  SDD Specification Phase - Create detailed specification with Given/When/Then scenarios.
  Uses PKM-AI block storage with block_type="structure".
  Trigger: When assigned as sdd-spec phase sub-agent.
license: MIT
metadata:
  author: pkm-ai
  version: "1.0"
---

## Purpose

Create a detailed specification with concrete scenarios that define:
- Functional requirements with Given/When/Then scenarios
- Acceptance criteria for each feature
- User interactions and flows
- Data models and structures

## PKM-AI Storage

| Field | Value |
|-------|-------|
| Block Type | `structure` |
| Title Format | `sdd/{change}/spec` |
| Tags | `["sdd", "spec", "sdd-spec", "sdd-{change}"]` |
| Link Type | `refines` (to proposal) |

## What You Receive

- `change`: The name of the change (e.g., `mcp-workflow`)
- `proposal-ulid`: ULID of the proposal artifact (required)

## Execution

### Step 1: Load Shared Conventions

Load `workflows/pkmai/sdd/_shared/phase-common.md` for the return envelope format.

### Step 2: Retrieve Proposal

```json
{
  "tool": "get_block",
  "arguments": {"block_id": "{proposal-ulid}", "include_content": true}
}
```

### Step 3: Create Spec Block

```json
{
  "tool": "create_block",
  "arguments": {"block_type": "structure", "title": "sdd/{change}/spec", "content": "# Specification: {change}\n\n## Overview\n{One paragraph overview of what this spec covers}\n\n## Functional Requirements\n\n### FR-1: {Feature Name}\n**Description**: {What this feature does}\n\n**Scenario 1.1: {Scenario Name}**\n- Given: {precondition}\n- When: {action}\n- Then: {expected result}\n\n**Scenario 1.2: {Scenario Name}**\n- Given: {precondition}\n- When: {action}\n- Then: {expected result}\n\n### FR-2: {Feature Name}\n...\n\n## User Interactions\n\n### UI Flow 1: {Flow Name}\n1. {Step 1}\n2. {Step 2}\n3. {Step 3}\n\n## Data Model\n\n### {Entity Name}\n| Field | Type | Description |\n|-------|------|-------------|\n| field | type | desc |\n\n## Edge Cases\n- {Case 1}: {how handled}\n- {Case 2}: {how handled}\n\n## Acceptance Criteria\n- [ ] {Criterion 1}\n- [ ] {Criterion 2}\n\n## Metadata\n```json\n{\n  \"change\": \"{change}\",\n  \"phase\": \"spec\",\n  \"based_on\": \"{proposal-ulid}\",\n  \"created\": \"{ISO date}\"\n}\n```\n", "tags": ["sdd", "spec", "sdd-spec", "sdd-{change}"]}
}
```

### Step 4: Link to Proposal

```json
{
  "tool": "create_link",
  "arguments": {"source_id": "{proposal-ulid}", "target_id": "{spec-ulid}", "link_type": "refines"}
}
```

### Step 5: Return Envelope

Return structured summary per `phase-common.md` format.

## Spec Template

```markdown
# Specification: {change}

## Overview
{2-3 sentences describing what this system/component does}

## Functional Requirements

### FR-1: {Feature Name}

**Description**: {What this feature does and why it exists}

**Scenario 1.1: {Happy Path Scenario}**
- **Given** {precondition}
- **When** {action performed}
- **Then** {expected outcome}

**Scenario 1.2: {Error Scenario}**
- **Given** {error precondition}
- **When** {error action}
- **Then** {expected error handling}

**Scenario 1.3: {Edge Case}**
- **Given** {edge case precondition}
- **When** {action}
- **Then** {expected edge case behavior}

### FR-2: {Next Feature}
...

## User Interactions and Flows

### Main Flow: {Flow Name}
1. User does X
2. System responds with Y
3. User does Z
4. System shows result

## Data Model

### Entity: {Name}
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | ULID | Yes | Unique identifier |

## API Contracts

### Endpoint: {Name}
- **Method**: GET/POST/PUT/DELETE
- **Path**: /api/v1/...
- **Request**: {schema}
- **Response**: {schema}

## Edge Cases

| Case | Handling |
|------|----------|
| {case} | {how handled} |

## Acceptance Criteria

### Must Have (MVP)
- [ ] {criterion}

### Should Have
- [ ] {criterion}

## Out of Scope
- {item 1}

## Dependencies
- {dependency 1}

## Metadata
```json
{
  "change": "{change}",
  "phase": "spec",
  "based_on": "{proposal-ulid}",
  "created": "{ISO date}"
}
```
```

## Rules

- Use `block_type="structure"` for spec blocks
- Follow Given/When/Then format for scenarios
- Specify acceptance criteria that can be verified
- Include edge cases and error handling
- Link to proposal as source of truth
- Return structured envelope with all required fields

## Output

Creates: `sdd/{change}/spec` block (type: structure)
Links: Proposal → Spec (refines)

## Block Type Note

Specs use `structure` block type because they have internal structure with scenarios organized by functional requirements. This allows structured access to individual scenarios while maintaining the complete spec as a single artifact.
