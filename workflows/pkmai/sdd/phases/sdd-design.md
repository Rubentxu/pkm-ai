---
name: sdd-design
description: >
  SDD Design Phase - Create architectural and technical design.
  Trigger: When assigned as sdd-design phase sub-agent.
license: MIT
metadata:
  author: pkm-ai
  version: "1.0"
---

## Purpose

Create an architectural and technical design that defines:
- High-level architecture
- Module structure and responsibilities
- API design and data contracts
- Technology choices and rationale
- Key design decisions

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

### Step 3: Create Design Block

```json
[
  {
    "tool": "create_block",
    "args": {
      "block_type": "permanent",
      "title": "sdd/{change-name}/design",
      "content": "# Design: {change-name}\n\n## Architecture Overview\n{High-level architecture description}\n\n## Module Design\n\n### Module: {ModuleName}\n**Responsibility**: {What this module does}\n**Public API**:\n- `function()` - {description}\n- `class` - {description}\n\n### Module: {ModuleName}\n...\n\n## Design Decisions\n\n### AD-1: {Decision Title}\n**Context**: {Problem or situation}\n**Decision**: {What was decided}\n**Rationale**: {Why this approach was chosen}\n**Alternatives Considered**:\n- {Alternative 1}: {why not chosen}\n- {Alternative 2}: {why not chosen}\n\n### AD-2: {Decision Title}\n...\n\n## API Design\n\n### {API Name}\n**Endpoint**: {path}\n**Method**: {method}\n**Request**:\n```json\n{schema}\n```\n**Response**:\n```json\n{schema}\n```\n\n## Data Flow\n{How data moves through the system}\n\n## Error Handling\n{How errors are handled}\n\n## Technology Stack\n| Component | Technology | Rationale |\n|-----------|------------|----------|\n| Language | Rust | {reason} |\n| Framework | {name} | {reason} |\n\n## Metadata\n```json\n{\n  \"change\": \"{change-name}\",\n  \"phase\": \"design\",\n  \"based_on\": \"{proposal-ulid}\",\n  \"created\": \"{ISO date}\"\n}\n```",
      "tags": ["sdd", "design", "sdd-design", "sdd-{change-name}"]
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
      "target_id": "{design-ulid}",
      "link_type": "refines"
    }
  }
]
```

### Step 5: Return Envelope

Return structured summary per phase-common.md format.

## Design Template

```markdown
# Design: {change-name}

## Architecture Overview

### High-Level Diagram
{Architecture description or diagram reference}

### Key Components
1. **{Component}**: {responsibility}
2. **{Component}**: {responsibility}
3. **{Component}**: {responsibility}

### Data Flow
{Flow description}

## Module Design

### Module: {ModuleName}
**Responsibility**: {What this module is responsible for}
**Public API**:
- `pub fn function_name()` - {description}
- `pub struct StructName` - {description}
- `impl TraitName for StructName` - {description}

**Dependencies**: {list of dependencies}
**Boundaries**: {what this module does NOT do}

### Module: {ModuleName}
...

## Design Decisions

### AD-1: {Decision Title}
**Context**: {The problem or situation that required a decision}
**Decision**: {What was decided}
**Rationale**: {Why this approach was chosen}
**Consequences**:
- **Positive**: {benefit}
- **Negative**: {tradeoff}

**Alternatives Considered**:
1. **{Alternative}**: {description} — rejected because {reason}
2. **{Alternative}**: {description} — rejected because {reason}

### AD-2: {Decision Title}
...

## API Design

### REST Endpoints

#### GET /api/v1/{resource}
**Description**: {what this endpoint does}
**Request Headers**: {headers if any}
**Response**:
```json
{
  "data": [],
  "pagination": {}
}
```

#### POST /api/v1/{resource}
**Description**: {what this endpoint does}
**Request Body**:
```json
{
  "field": "value"
}
```
**Response**: 201 Created

### gRPC Services (if applicable)
```protobuf
service ServiceName {
  rpc MethodName(Request) returns (Response);
}
```

## Data Model

### Entity: {Name}
{Description}
```rust
pub struct EntityName {
    pub id: Ulid,
    pub field: Type,
}
```

### Entity: {Name}
...

## Error Handling Strategy

| Error Type | HTTP Status | Handling |
|------------|-------------|----------|
| Validation Error | 400 | Return field errors |
| Not Found | 404 | Return resource ID |
| Unauthorized | 401 | Return auth error |
| Internal Error | 500 | Log and return generic |

## Technology Choices

| Component | Choice | Rationale |
|-----------|--------|----------|
| Language | Rust | {reason} |
| Database | {db} | {reason} |
| Framework | {framework} | {reason} |
| Runtime | {runtime} | {reason} |

## Security Considerations
- {consideration 1}
- {consideration 2}

## Performance Considerations
- {consideration 1}
- {consideration 2}

## Testing Strategy
- Unit tests: {coverage}
- Integration tests: {scope}
- E2E tests: {scope}

## Migration Strategy (if applicable)
{How to migrate from current state}

## Metadata
```json
{
  "change": "{change-name}",
  "phase": "design",
  "based_on": "{proposal-ulid}",
  "created": "{ISO date}"
}
```
```

## Rules

- Document decisions with context and rationale
- Include alternatives considered
- Be concrete about API contracts
- Reference spec scenarios
- Link to proposal as source
- Return structured envelope with all required fields

## Output

Creates: `sdd/{change-name}/design` block
Links: → Proposal
