---
name: sdd-pkmai
description: >
  Spec-Driven Development workflow adapted for PKM-AI.
  Use this skill when running SDD phases with PKM-AI as the artifact store.
  Triggers: sdd-explore, sdd-propose, sdd-spec, sdd-design, sdd-tasks, sdd-apply, sdd-verify, sdd-archive.
license: MIT
metadata:
  author: pkm-ai
  version: "1.0"
  project: hodei-pkm
---

## Purpose

You are a sub-agent executing an SDD (Spec-Driven Development) phase. You receive phase instructions and artifact references, then produce the phase artifact using PKM-AI MCP tools for storage.

This skill is adapted for PKM-AI:
- **Artifact Store**: PKM-AI blocks with tags
- **Tool Access**: MCP tools via `Tool` calls
- **Artifact ID**: Block ULIDs
- **Relationships**: PKM-AI links

## What You Receive

From the orchestrator:
- Phase name (e.g., `sdd-propose`, `sdd-spec`)
- Change name (e.g., `mcp-workflow`, `auth-system`)
- Artifact store mode (`pkmai | openspec | hybrid | none`)
- Previous artifacts (ULIDs or search queries)

## PKM-AI Tools

| Tool | Usage |
|------|-------|
| `search_blocks` | Find artifacts by query and tags |
| `get_block` | Get full artifact content by ULID |
| `create_block` | Create new artifact block |
| `update_block` | Update existing artifact |
| `create_link` | Link artifacts together |

### Tool Usage Examples

```json
[
  {
    "tool": "search_blocks",
    "args": {
      "query": "sdd/{change-name}/{phase}",
      "tags": ["sdd-{phase}"],
      "limit": 10
    }
  },
  {
    "tool": "get_block",
    "args": {
      "block_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
      "include_content": true
    }
  },
  {
    "tool": "create_block",
    "args": {
      "block_type": "permanent",
      "title": "sdd/{change-name}/{phase}",
      "content": "# {Phase Title}\n\n{content}...",
      "tags": ["sdd", "sdd-{phase}", "sdd-{change-name}"]
    }
  },
  {
    "tool": "update_block",
    "args": {
      "block_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
      "content": "{updated content}"
    }
  },
  {
    "tool": "create_link",
    "args": {
      "source_id": "{ulid-source}",
      "target_id": "{ulid-target}",
      "link_type": "refines"
    }
  }
]
```

## Phase Execution

### Step 1: Load Shared Conventions

Read `workflows/pkmai/sdd/_shared/phase-common.md` for the return envelope format and common instructions.

### Step 2: Search for Required Artifacts

Based on the phase, search PKM-AI for required input artifacts:

| Phase | Searches For |
|-------|-------------|
| `sdd-explore` | Nothing (creates from scratch) |
| `sdd-propose` | Explore block (optional) |
| `sdd-spec` | Proposal block (required) |
| `sdd-design` | Proposal block (required) |
| `sdd-tasks` | Spec + Design blocks (both required) |
| `sdd-apply` | Tasks + Spec + Design blocks |
| `sdd-verify` | Spec + Tasks blocks |
| `sdd-archive` | All artifact blocks |

### Step 3: Retrieve Full Content

For each required artifact, use `get_block` to retrieve full content. **Do NOT use search previews** — they are truncated.

### Step 4: Execute Phase Work

Produce the phase artifact according to its template:

#### sdd-explore
- Research topic using web search if needed
- Create exploration block with findings
- Tags: `["sdd", "explore", "sdd-{change-name}"]`

#### sdd-propose
- Analyze exploration (if exists)
- Create proposal with problem/solution/outcomes
- Tags: `["sdd", "proposal", "sdd-{change-name}"]`

#### sdd-spec
- Create structured spec with scenarios
- Block type: `structure`
- Tags: `["sdd", "spec", "sdd-{change-name}"]`

#### sdd-design
- Create design decisions
- Tags: `["sdd", "design", "sdd-{change-name}"]`

#### sdd-tasks
- Create task breakdown (checklist format)
- Block type: `outline`
- Tags: `["sdd", "tasks", "sdd-{change-name}"]`

#### sdd-apply
- Execute assigned tasks
- Create progress report block
- Tags: `["sdd", "progress", "sdd-{change-name}"]`

#### sdd-verify
- Verify implementation against spec
- Create verification report
- Tags: `["sdd", "verify", "sdd-{change-name}"]`

#### sdd-archive
- Summarize entire change
- Tags: `["sdd", "archive", "sdd-{change-name}"]`

### Step 5: Create Artifact in PKM-AI

```json
[
  {
    "tool": "create_block",
    "args": {
      "block_type": "{appropriate_type}",
      "title": "sdd/{change-name}/{phase}",
      "content": "{artifact_content}",
      "tags": ["sdd", "sdd-{phase}", "sdd-{change-name}"]
    }
  }
]
```

### Step 6: Link to Parent Artifact

If the phase has a parent artifact, create a link:

```json
[
  {
    "tool": "create_link",
    "args": {
      "source_id": "{parent_ulid}",
      "target_id": "{new_artifact_ulid}",
      "link_type": "refines"
    }
  }
]
```

### Step 7: Save Progress (if required)

If mode is `hybrid`, also update local files as needed.

### Step 8: Return Summary

Return to orchestrator with structured envelope:

```markdown
## {Phase} Progress

**Change**: {change-name}
**Phase**: {phase-name}
**Mode**: {mode}

### Artifacts
| Artifact | ULID | Action |
|----------|------|--------|
| sdd/{change}/{phase} | `{ulid}` | Created |

### Links Created
| From | To | Type |
|------|----|------|
| `{parent_ulid}` | `{new_ulid}` | `refines` |

### Content Summary
{brief summary of what was created}

### Next Recommended
{what the next phase should be}

### Risks
{any risks or blockers identified, or "None"}
```

## Rules

- ALWAYS retrieve full artifact content via `get_block`, never use search previews
- ALWAYS create the artifact block in PKM-AI before returning
- ALWAYS link to parent artifact when applicable
- Use appropriate block types: `permanent` for most, `structure` for specs, `outline` for tasks
- Tag consistently: always include `sdd` plus phase-specific tag
- If a required artifact is missing, report it as a risk
- Return structured envelope with all required fields

## Block Type Guidelines

| Artifact Type | Block Type | Notes |
|---------------|------------|-------|
| Exploration | `permanent` | Research findings |
| Proposal | `permanent` | Problem/solution |
| Spec | `structure` | Structured with scenarios |
| Design | `permanent` | Decisions and rationale |
| Tasks | `outline` | Checklist format |
| Progress | `permanent` | Status reports |
| Verify | `permanent` | Verification results |
| Archive | `permanent` | Final summary |

## Link Type Guidelines

| Relationship | Link Type | Usage |
|--------------|-----------|-------|
| Refines | `refines` | Proposal→Spec, Spec→Design |
| Contains | `contains` | Design→Tasks |
| Progress of | `related` | Progress→Tasks |
| Verified by | `supports` | Verify→Spec |

---

**Skill Version**: 1.0
**Adapted for**: PKM-AI MCP
**Project**: hodei-pkm
