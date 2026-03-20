# Core SDD Concepts for PKM-AI

## SDD Workflow Phases

Spec-Driven Development (SDD) is a structured planning methodology that decomposes substantial changes into discrete phases. Each phase produces artifacts that flow into subsequent phases.

### Phase Overview

| Phase | Purpose | Input | Output |
|-------|---------|-------|--------|
| `sdd-explore` | Research and discover | None | Explore block |
| `sdd-propose` | Define problem/solution | Explore (optional) | Proposal block |
| `sdd-spec` | Create detailed specification | Proposal | Spec block (structure) |
| `sdd-design` | Make architectural decisions | Proposal | Design block |
| `sdd-tasks` | Break down implementation | Spec + Design | Tasks block (outline) |
| `sdd-apply` | Execute implementation | Tasks + Spec + Design | Progress block |
| `sdd-verify` | Validate implementation | Spec + Tasks | Verify block |
| `sdd-archive` | Summarize completed change | All | Archive block |

### Phase Flow

```
                    ┌──────────────┐
                    │ sdd-explore  │
                    └──────┬───────┘
                           │
                           ▼
                    ┌──────────────┐
                    │ sdd-propose  │
                    └──────┬───────┘
                           │
               ┌───────────┴───────────┐
               │                       │
               ▼                       ▼
        ┌──────────────┐        ┌──────────────┐
        │  sdd-spec   │        │  sdd-design  │
        └──────┬───────┘        └──────┬───────┘
               │                       │
               └───────────┬───────────┘
                           │
                           ▼
                    ┌──────────────┐
                    │  sdd-tasks   │
                    └──────┬───────┘
                           │
                           ▼
                    ┌──────────────┐
                    │  sdd-apply   │
                    └──────┬───────┘
                           │
                           ▼
                    ┌──────────────┐
                    │  sdd-verify  │
                    └──────┬───────┘
                           │
                           ▼
                    ┌──────────────┐
                    │ sdd-archive  │
                    └──────────────┘
```

## Artifact Lifecycle

Artifacts are the primary output of each SDD phase. They persist in PKM-AI and are linked to form a traceable history.

### Lifecycle Stages

```
┌─────────┐    Create     ┌─────────┐    Link     ┌─────────┐
│  New    │──────────────▶│ Staged  │────────────▶│Linked   │
│Artifact │               │Artifact │             │Artifact │
└─────────┘               └─────────┘             └─────────┘
                                                          │
                                                          ▼
                                                   ┌───────────┐
                                                   │ Retrieved │
                                                   │ by Future │
                                                   │   Phase   │
                                                   └───────────┘
```

### Lifecycle Operations

| Operation | Tool | Description |
|-----------|------|-------------|
| Create | `create_block` | Create new artifact block |
| Read | `get_block` | Retrieve full artifact content |
| Update | `update_block` | Modify artifact content |
| Search | `search_blocks` | Find artifacts by query/tags |
| Link | `create_link` | Establish relationship between artifacts |
| Delete | Not used | Soft-delete not needed - artifacts are permanent |

### Artifact Dependencies

Each phase may depend on artifacts from prior phases:

| Phase | Requires | May Also Use |
|-------|----------|--------------|
| `sdd-explore` | Nothing | User-provided context |
| `sdd-propose` | Explore (optional) | User-provided topic |
| `sdd-spec` | Proposal | - |
| `sdd-design` | Proposal | - |
| `sdd-tasks` | Spec + Design | - |
| `sdd-apply` | Tasks + Spec + Design | Progress tracking |
| `sdd-verify` | Spec + Tasks | Implementation reality |
| `sdd-archive` | All | - |

## PKM-AI Block Types

PKM-AI blocks are the storage unit for all SDD artifacts. Different block types serve different purposes.

### Block Type Reference

| Type | Purpose | SDD Usage | Aliases |
|------|---------|-----------|---------|
| `fleeting` | Quick captures, temporary | Not used in SDD | `f` |
| `literature` | Reference from external sources | Not used in SDD | `l` |
| `permanent` | Atomic Zettelkasten notes | Explore, Proposal, Design, Verify, Archive | `p` |
| `structure` | Structural containers | Spec (has internal structure) | `s`, `index`, `moc` |
| `hub` | Central topic connectors | Not used in SDD | `h` |
| `task` | Action items and todos | Not used in SDD (use outline) | `t` |
| `reference` | External references | Not used in SDD | `r` |
| `outline` | Hierarchical outlines | Tasks (checklist format) | `o` |
| `ghost` | Placeholder predictions | Not used in SDD | `g` |

### Block Type Selection Guide

```json
[
  {
    "tool": "create_block",
    "args": {
      "block_type": "permanent",
      "title": "sdd/mcp-workflow/explore",
      "content": "# Exploration\n\n..."
    }
  },
  {
    "tool": "create_block",
    "args": {
      "block_type": "structure",
      "title": "sdd/mcp-workflow/spec",
      "content": "# Specification\n\n## Scenario 1..."
    }
  },
  {
    "tool": "create_block",
    "args": {
      "block_type": "outline",
      "title": "sdd/mcp-workflow/tasks",
      "content": "# Tasks\n\n- [ ] Task 1\n- [ ] Task 2..."
    }
  }
]
```

### Why Block Types Matter

1. **Retrieval**: `search_blocks(block_type="structure")` finds specs
2. **Validation**: Specs must be `structure` type
3. **Navigation**: Block type hints at artifact purpose
4. **Rendering**: Different types render differently in PKM-AI clients

## Tags for Classification

Tags provide faceted classification for SDD artifacts. They enable efficient search and discovery.

### Tag Naming Convention

All SDD artifacts receive a standard tag set:

| Tag | Description | Example |
|-----|-------------|---------|
| `sdd` | Universal SDD marker | All artifacts |
| `sdd-{phase}` | Phase-specific marker | `sdd-spec`, `sdd-design` |
| `sdd-{change}` | Change-specific marker | `sdd-mcp-workflow` |

### Tag Set per Artifact Type

```json
[
  {
    "comment": "Explore artifact",
    "tags": ["sdd", "explore", "sdd-explore", "sdd-{change}"]
  },
  {
    "comment": "Proposal artifact",
    "tags": ["sdd", "proposal", "sdd-proposal", "sdd-{change}"]
  },
  {
    "comment": "Spec artifact",
    "tags": ["sdd", "spec", "sdd-spec", "sdd-{change}"]
  },
  {
    "comment": "Design artifact",
    "tags": ["sdd", "design", "sdd-design", "sdd-{change}"]
  },
  {
    "comment": "Tasks artifact",
    "tags": ["sdd", "tasks", "sdd-tasks", "sdd-{change}"]
  },
  {
    "comment": "Progress artifact",
    "tags": ["sdd", "progress", "sdd-progress", "sdd-{change}"]
  },
  {
    "comment": "Verify artifact",
    "tags": ["sdd", "verify", "sdd-verify", "sdd-{change}"]
  },
  {
    "comment": "Archive artifact",
    "tags": ["sdd", "archive", "sdd-archive", "sdd-{change}"]
  }
]
```

### Search by Tags

```json
[
  {
    "tool": "search_blocks",
    "args": {
      "query": "mcp-workflow",
      "tags": ["sdd"],
      "limit": 50
    }
  },
  {
    "tool": "search_blocks",
    "args": {
      "query": "mcp-workflow",
      "tags": ["sdd-spec"],
      "limit": 10
    }
  },
  {
    "tool": "search_blocks",
    "args": {
      "block_type": "structure",
      "tags": ["sdd-spec"],
      "limit": 100
    }
  }
]
```

## Links for Relationships

Links establish directed relationships between artifacts, forming a navigable graph.

### Link Type Reference

| Link Type | Meaning | SDD Usage |
|-----------|---------|-----------|
| `extends` | Builds upon another block | Not typically used |
| `refines` | Elaborates or specializes | Proposal→Spec, Spec→Design |
| `contradicts` | Opposes another block | Not typically used |
| `questions` | Questions another block | Not typically used |
| `supports` | Provides evidence for | Verify→Spec |
| `references` | Cites or refers to | Not typically used |
| `related` | Associated but not strictly | Tasks→Progress |
| `similar_to` | Similar to another | Not typically used |
| `section_of` | Part of a larger block | Not typically used |
| `subsection_of` | Sub-part of block | Not typically used |
| `ordered_child` | Ordered child of | Not typically used |
| `next` | Sequential relationship | Not typically used |
| `next_sibling` | Next at same level | Not typically used |
| `first_child` | First child of | Not typically used |
| `contains` | Parent contains child | Design→Tasks |
| `parent` | Parent of another | Not typically used |
| `ai_suggested` | AI-suggested link | Link suggestions |

### SDD Link Graph

```
                    ┌─────────────┐
                    │   Explore   │
                    └──────┬──────┘
                           │ refines
                           ▼
                    ┌─────────────┐
                    │  Proposal   │
                    └──────┬──────┘
                           │ refines
              ┌────────────┴────────────┐
              │                         │
              ▼                         ▼
       ┌─────────────┐          ┌─────────────┐
       │    Spec     │          │   Design    │
       └──────┬──────┘          └──────┬──────┘
              │                         │
              │         ┌────────────────┘
              │         │ contains
              ▼         ▼
       ┌─────────────┐
       │    Tasks    │
       └──────┬──────┘
              │ related
              ▼
       ┌─────────────┐
       │  Progress   │
       └──────┬──────┘
              │ related
              ▼
       ┌─────────────┐
       │   Verify    │
       └──────┬──────┘
              │ supports
              ▼
       ┌─────────────┐
       │   Archive   │
       └─────────────┘
```

### Creating Links

```json
[
  {
    "tool": "create_link",
    "args": {
      "source_id": "{explore-ulid}",
      "target_id": "{proposal-ulid}",
      "link_type": "refines"
    }
  },
  {
    "tool": "create_link",
    "args": {
      "source_id": "{proposal-ulid}",
      "target_id": "{spec-ulid}",
      "link_type": "refines"
    }
  },
  {
    "tool": "create_link",
    "args": {
      "source_id": "{proposal-ulid}",
      "target_id": "{design-ulid}",
      "link_type": "refines"
    }
  },
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
      "source_id": "{verify-ulid}",
      "target_id": "{spec-ulid}",
      "link_type": "supports"
    }
  }
]
```

## Artifact Title Format

Artifacts follow a strict naming convention:

```
sdd/{change-name}/{phase}
```

### Examples

| Artifact | Title |
|----------|-------|
| Project context | `sdd/mcp-workflow/project` |
| Phase tracker | `sdd/mcp-workflow/tracker` |
| Exploration | `sdd/mcp-workflow/explore` |
| Proposal | `sdd/mcp-workflow/proposal` |
| Specification | `sdd/mcp-workflow/spec` |
| Design | `sdd/mcp-workflow/design` |
| Tasks | `sdd/mcp-workflow/tasks` |
| Progress | `sdd/mcp-workflow/progress` |
| Verification | `sdd/mcp-workflow/verify` |
| Archive | `sdd/mcp-workflow/archive` |

## Content Format

### Standard Artifact Structure

```markdown
# {Title}

## Summary
{2-3 sentence overview of the artifact}

## Key Points
- {Point 1}
- {Point 2}
- {Point 3}

---

## Metadata
```json
{{
  "change": "{change-name}",
  "phase": "{phase-name}",
  "created": "{ISO 8601 timestamp}",
  "artifact_ulid": "{ulid}",
  "dependencies": ["{parent-ulid}", ...]
}}
```

### Explore Content Format

```markdown
# Exploration: {topic}

## Research Questions
- {Question 1}
- {Question 2}

## Findings
### {Finding Title 1}
{Details}

### {Finding Title 2}
{Details}

## Resources Explored
- {Resource 1}
- {Resource 2}

## Open Questions
- {Question that needs more research}
```

### Proposal Content Format

```markdown
# Proposal: {change-name}

## Problem Statement
{What problem does this solve?}

## Proposed Solution
{How do we solve it?}

## Expected Outcomes
- {Outcome 1}
- {Outcome 2}

## Risks and Mitigations
| Risk | Mitigation |
|------|------------|
| {Risk 1} | {Mitigation 1} |

## Dependencies
- {Dependency 1}
```

### Spec Content Format

```markdown
# Specification: {change-name}

## Overview
{One paragraph overview}

## Functional Requirements

### Scenario 1: {Title}
**Given** {precondition}
**When** {action}
**Then** {expected result}

### Scenario 2: {Title}
...

## Non-Functional Requirements
- {Requirement 1}
- {Requirement 2}

## Acceptance Criteria
- [ ] {Criterion 1}
- [ ] {Criterion 2}
```

### Design Content Format

```markdown
# Design: {change-name}

## Architectural Decisions

### Decision 1: {Title}
**Status**: Proposed | Accepted
**Context**: {Situation}
**Decision**: {What we decided}
**Consequences**: {Positive, Negative}

## Module Design

### {Module Name}
**Responsibility**: {What it does}
**Public API**:
- `{method signature}`
- `{method signature}`

## Data Structures
```{language}
{code or structure}
```
```

### Tasks Content Format

```markdown
# Tasks: {change-name}

## Task Summary
| Task | Status | Complexity |
|------|--------|------------|
| {Task 1} | pending | Low |
| {Task 2} | pending | Med |

## Implementation Tasks

- [ ] **{Task 1}** (Low)
  - {Sub-step 1}
  - {Sub-step 2}

- [ ] **{Task 2}** (Med)
  - {Sub-step 1}
  - {Sub-step 2}
```

## Search Patterns

### Find Project Context

```json
[
  {
    "tool": "search_blocks",
    "args": {
      "query": "sdd/{change}/project",
      "tags": ["sdd", "project"]
    }
  }
]
```

### Find Phase Artifacts

```json
[
  {
    "tool": "search_blocks",
    "args": {
      "query": "sdd/{change}/{phase}",
      "tags": ["sdd-{phase}"]
    }
  }
]
```

### Find All Change Artifacts

```json
[
  {
    "tool": "search_blocks",
    "args": {
      "query": "sdd-{change}",
      "tags": ["sdd"]
    }
  }
]
```

## Key Concepts Summary

| Concept | PKM-AI Representation | Purpose |
|---------|---------------------|---------|
| SDD Phase | Block with phase tag | Defines work type |
| Artifact | Block with `sdd/{change}/{phase}` title | Phase output |
| Relationship | Link between blocks | Artifact dependency |
| Classification | Tags on block | Faceted search |
| Structure | Block type | Semantic meaning |
| Content | Markdown in block | Human-readable artifact |

## Next Steps

- See [installation.md](installation.md) for setup
- See [persistence.md](persistence.md) for storage modes
- See [sub-agents.md](sub-agents.md) for phase details
- See [token-economics.md](token-economics.md) for efficiency analysis