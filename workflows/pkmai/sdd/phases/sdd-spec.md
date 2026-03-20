---
name: sdd-spec
description: >
  SDD Specification Phase - Create detailed specification with scenarios.
  Trigger: When assigned as sdd-spec phase sub-agent.
license: MIT
metadata:
  author: pkm-ai
  version: "1.0"
---

## Purpose

Create a detailed specification that defines:
- Functional requirements with concrete scenarios
- Acceptance criteria for each feature
- User interactions and flows
- Data models and structures

## What You Receive

- `change-name`: The name of the change
- `proposal-ulid`: ULID of proposal artifact (required)

## Execution

### Step 1: Load Skills

Load `workflows/pkmai/sdd/_shared/phase-common.md` for return format.

### Step 2: Retrieve Proposal

```json
[
  {
    "tool": "get_block",
    "args": {
      "block_id": "{proposal-ulid}",
      "include_content": true
    }
  }
]
```

### Step 3: Create Spec Block

```json
[
  {
    "tool": "create_block",
    "args": {
      "block_type": "structure",
      "title": "sdd/{change-name}/spec",
      "content": "# Specification: {change-name}\n\n## Overview\n{One paragraph overview of what this spec covers}\n\n## Functional Requirements\n\n### FR-1: {Feature Name}\n**Description**: {What this feature does}\n\n**Scenario 1: {Scenario Name}**\n- Given: {precondition}\n- When: {action}\n- Then: {expected result}\n\n**Scenario 2: {Scenario Name}**\n- Given: {precondition}\n- When: {action}\n- Then: {expected result}\n\n### FR-2: {Feature Name}\n...\n\n## User Interactions\n\n### UI Flow 1: {Flow Name}\n1. {Step 1}\n2. {Step 2}\n3. {Step 3}\n\n## Data Model\n\n### {Entity Name}\n| Field | Type | Description |\n|-------|------|-------------|\n| field | type | desc |\n\n## Edge Cases\n- {Case 1}: {how handled}\n- {Case 2}: {how handled}\n\n## Acceptance Criteria\n- [ ] {Criterion 1}\n- [ ] {Criterion 2}\n\n## Metadata\n```json\n{\n  \"change\": \"{change-name}\",\n  \"phase\": \"spec\",\n  \"based_on\": \"{proposal-ulid}\",\n  \"created\": \"{ISO date}\"\n}\n```",
      "tags": ["sdd", "spec", "sdd-spec", "sdd-{change-name}"]
    }
  }
]
```

### Step 4: Link to Proposal

```json
[
  {
    "tool": "create_link",
    "args": {
      "source_id": "{proposal-ulid}",
      "target_id": "{spec-ulid}",
      "link_type": "refines"
    }
  }
]
```

### Step 5: Return Envelope

Return structured summary per phase-common.md format.

## Spec Template

```markdown
# Specification: {change-name}

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
{Sequence of steps}
1. User does X
2. System responds with Y
3. User does Z
4. System shows result

### Alternative Flow: {Flow Name}
{When alternative path is taken}
1. ...
2. ...

## Data Model

### Entity: {Name}
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | ULID | Yes | Unique identifier |
| ... | ... | ... | ... |

### Relationship: {Name}
| From | To | Type |
|------|----|------|
| Entity A | Entity B | 1:N |

## API Contracts

### Endpoint: {Name}
- **Method**: GET/POST/PUT/DELETE
- **Path**: /api/v1/...
- **Request**: {schema}
- **Response**: {schema}
- **Errors**: 400, 401, 404, 500

## Edge Cases

| Case | Handling |
|------|----------|
| {case} | {how handled} |
| {case} | {how handled} |

## Non-Functional Requirements
- **Performance**: {requirement}
- **Security**: {requirement}
- **Scalability**: {requirement}

## Acceptance Criteria

### Must Have (MVP)
- [ ] {criterion}
- [ ] {criterion}

### Should Have
- [ ] {criterion}
- [ ] {criterion}

### Could Have
- [ ] {criterion}

## Out of Scope
- {item 1}
- {item 2}

## Dependencies
- {dependency 1}
- {dependency 2}

## Metadata
```json
{
  "change": "{change-name}",
  "phase": "spec",
  "based_on": "{proposal-ulid}",
  "created": "{ISO date}"
}
```
```

## Rules

- Use concrete scenarios, not abstract requirements
- Follow Given/When/Then format for scenarios
- Specify acceptance criteria that can be verified
- Include edge cases and error handling
- Link to proposal as source of truth
- Return structured envelope with all required fields

## Output

Creates: `sdd/{change-name}/spec` block (type: structure)
Links: → Proposal
