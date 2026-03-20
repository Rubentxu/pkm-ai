# SDD Phase Sub-Agents and Skill Registry

## Overview

The PKM-AI workflow uses **skill-based sub-agents** to execute SDD phases. Each phase has a dedicated skill that encapsulates the work to be done, the inputs required, and the outputs produced.

```
┌─────────────────────────────────────────────────────────────┐
│                      ORCHESTRATOR                           │
│                                                              │
│  /sdd-new my-feature    →    Launch sdd-init skill         │
│  /sdd-explore my-feature →    Launch sdd-explore skill      │
│  /sdd-propose my-feature →    Launch sdd-propose skill      │
│  ...                                                        │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      SKILL EXECUTOR                         │
│                                                              │
│  Load Skill ──► Read Inputs ──► Execute ──► Store ──► Return│
│                                                              │
│  Skill: workflows/pkmai/sdd/phases/sdd-{phase}.md          │
│  Tools: PKM-AI MCP (search_blocks, create_block, etc.)     │
└─────────────────────────────────────────────────────────────┘
```

## SDD Phase Skills

### Skill Inventory

| Phase | Skill File | Purpose | Block Type |
|-------|------------|---------|------------|
| `sdd-init` | `workflows/pkmai/skills/sdd-init/SKILL.md` | Initialize change | structure |
| `sdd-explore` | `workflows/pkmai/sdd/phases/sdd-explore.md` | Research topic | permanent |
| `sdd-propose` | `workflows/pkmai/sdd/phases/sdd-propose.md` | Define problem/solution | permanent |
| `sdd-spec` | `workflows/pkmai/sdd/phases/sdd-spec.md` | Create detailed spec | structure |
| `sdd-design` | `workflows/pkmai/sdd/phases/sdd-design.md` | Make decisions | permanent |
| `sdd-tasks` | `workflows/pkmai/sdd/phases/sdd-tasks.md` | Break into tasks | outline |
| `sdd-apply` | `workflows/pkmai/sdd/phases/sdd-apply.md` | Implement tasks | permanent |
| `sdd-verify` | `workflows/pkmai/sdd/phases/sdd-verify.md` | Validate implementation | permanent |
| `sdd-archive` | `workflows/pkmai/sdd/phases/sdd-archive.md` | Summarize change | permanent |

### Phase Skill Details

#### sdd-init

**Purpose**: Initialize a new SDD change project

**Trigger**: `/sdd-init {change-name}`

**Inputs**:
- Change name
- Project name
- Optional description

**Outputs**:
- Project context block (`structure`)
- Phase tracker block (`outline`)
- Discovery block (`permanent`)

**Required Artifacts**: None

---

#### sdd-explore

**Purpose**: Research and discover information about a topic

**Trigger**: `/sdd-explore {change-name}`

**Inputs**:
- Change name
- Optional initial topic

**Outputs**:
- Exploration block with findings

**Required Artifacts**: None

**Optional Artifacts**:
- Discovery block (from init)

**Skill Loading**:
```markdown
SKILL: Load `workflows/pkmai/sdd/phases/sdd-explore.md`
MODE: pkmai
CHANGE: {change-name}
PROJECT: {project-name}
```

---

#### sdd-propose

**Purpose**: Define the problem statement and proposed solution

**Trigger**: `/sdd-propose {change-name}`

**Inputs**:
- Change name
- Exploration findings (if exists)

**Outputs**:
- Proposal block with problem/solution/outcomes

**Required Artifacts**: None (exploration optional)

**Link Created**:
- `explore` → `proposal` via `refines` (if explore exists)

---

#### sdd-spec

**Purpose**: Create detailed functional specification with scenarios

**Trigger**: `/sdd-spec {change-name}`

**Inputs**:
- Change name
- Proposal block content

**Outputs**:
- Specification block (`structure`)

**Required Artifacts**:
- Proposal block

**Link Created**:
- `proposal` → `spec` via `refines`

---

#### sdd-design

**Purpose**: Make architectural and design decisions

**Trigger**: `/sdd-design {change-name}`

**Inputs**:
- Change name
- Proposal block content

**Outputs**:
- Design block with decisions

**Required Artifacts**:
- Proposal block

**Link Created**:
- `proposal` → `design` via `refines`

---

#### sdd-tasks

**Purpose**: Break specification and design into implementable tasks

**Trigger**: `/sdd-tasks {change-name}`

**Inputs**:
- Change name
- Spec block content
- Design block content

**Outputs**:
- Tasks block (`outline`)

**Required Artifacts**:
- Spec block
- Design block

**Link Created**:
- `design` → `tasks` via `contains`

---

#### sdd-apply

**Purpose**: Execute implementation tasks

**Trigger**: `/sdd-apply {change-name}`

**Inputs**:
- Change name
- Tasks block content
- Spec block content
- Design block content

**Outputs**:
- Progress block

**Required Artifacts**:
- Tasks block
- Spec block
- Design block

**Link Created**:
- `tasks` → `progress` via `related`

---

#### sdd-verify

**Purpose**: Verify implementation against specification

**Trigger**: `/sdd-verify {change-name}`

**Inputs**:
- Change name
- Spec block content
- Tasks block content
- Implementation reality

**Outputs**:
- Verification block

**Required Artifacts**:
- Spec block
- Tasks block

**Link Created**:
- `verify` → `spec` via `supports`

---

#### sdd-archive

**Purpose**: Summarize the completed change

**Trigger**: `/sdd-archive {change-name}`

**Inputs**:
- Change name
- All prior artifact blocks

**Outputs**:
- Archive block

**Required Artifacts**:
- All prior artifacts

**Link Created**: None (final phase)

---

## Skill Registry

### Registry Structure

```
workflows/pkmai/sdd/
├── skill-registry/
│   ├── README.md           # This file
│   └── skills/
│       ├── sdd-init/
│       │   └── manifest.yaml
│       ├── sdd-explore/
│       │   └── manifest.yaml
│       └── ...
```

### Manifest Format

```yaml
# workflows/pkmai/sdd/skill-registry/skills/sdd-explore/manifest.yaml
name: sdd-explore
description: >
  Exploration phase for SDD workflow.
  Executes research and discovery for a change topic.
trigger: sdd-explore
skill_path: workflows/pkmai/sdd/phases/sdd-explore.md
version: "1.0"
phase: explore

inputs:
  required:
    - change_name
  optional:
    - initial_topic

outputs:
  artifacts:
    - type: explore
      block_type: permanent
      tags:
        - sdd
        - explore
        - sdd-{change}

dependencies:
  phases:
    - none  # First phase

tools:
  pkmai:
    - search_blocks
    - get_block
    - create_block
    - create_link
```

### Registry Index

```yaml
# workflows/pkmai/sdd/skill-registry/index.yaml
skills:
  - name: sdd-init
    path: workflows/pkmai/skills/sdd-init/SKILL.md
    phase: init

  - name: sdd-explore
    path: workflows/pkmai/sdd/phases/sdd-explore.md
    phase: explore

  - name: sdd-propose
    path: workflows/pkmai/sdd/phases/sdd-propose.md
    phase: propose

  - name: sdd-spec
    path: workflows/pkmai/sdd/phases/sdd-spec.md
    phase: spec

  - name: sdd-design
    path: workflows/pkmai/sdd/phases/sdd-design.md
    phase: design

  - name: sdd-tasks
    path: workflow/pkmai/sdd/phases/sdd-tasks.md
    phase: tasks

  - name: sdd-apply
    path: workflows/pkmai/sdd/phases/sdd-apply.md
    phase: apply

  - name: sdd-verify
    path: workflows/pkmai/sdd/phases/sdd-verify.md
    phase: verify

  - name: sdd-archive
    path: workflows/pkmai/sdd/phases/sdd-archive.md
    phase: archive
```

## Skill Loading Process

### Sub-Agent Launch Pattern

```python
# Orchestrator launches sub-agent with skill reference
agent = Agent(
    skill="workflows/pkmai/sdd/phases/sdd-explore.md",
    args={
        "change": "mcp-workflow",
        "project": "hodei-pkm",
        "mode": "pkmai"
    },
    run_in_background=True
)
```

### Skill Loading Sequence

When a sub-agent executes, it loads skills in this order:

```
1. Base SDD Skill
   └─> workflows/pkmai/sdd/SKILL.md
       └── Contains: PKM-AI tool mapping, conventions

2. Shared Conventions
   ├─> workflows/pkmai/sdd/_shared/pkmai-convention.md
   │   └── Contains: PKM-AI artifact storage rules
   └─> workflows/pkmai/sdd/_shared/phase-common.md
       └── Contains: Return envelope format

3. Phase-Specific Skill
   └─> workflows/pkmai/sdd/phases/sdd-{phase}.md
       └── Contains: Phase-specific instructions
```

### Skill Execution Flow

```
1. Load skill file
   │
   ▼
2. Parse inputs from orchestrator
   │
   ▼
3. Search PKM-AI for required artifacts
   │
   ▼
4. Retrieve full artifact content
   │
   ▼
5. Execute phase work
   │
   ▼
6. Create output artifact in PKM-AI
   │
   ▼
7. Create links to parent artifacts
   │
   ▼
8. Return structured envelope
```

## Adding New Skills

### Step 1: Create Skill File

```markdown
# workflows/pkmai/sdd/phases/sdd-newphase.md
---
name: sdd-newphase
description: >
  Description of what this phase does.
trigger: sdd-newphase
---

## Purpose

Describe the purpose of this phase.

## Inputs

| Input | Source | Required |
|-------|--------|----------|
| change_name | orchestrator | Yes |
| something_else | prior artifact | No |

## Execution

### Step 1: Search for Required Artifacts

```json
[
  {
    "tool": "search_blocks",
    "args": {
      "query": "sdd/{change}/required-phase",
      "tags": ["sdd", "sdd-required-phase"]
    }
  }
]
```

### Step 2: Execute Phase Work

{Phase-specific work}

### Step 3: Create Artifact

```json
[
  {
    "tool": "create_block",
    "args": {
      "block_type": "permanent",
      "title": "sdd/{change}/newphase",
      "content": "{artifact content}",
      "tags": ["sdd", "sdd-newphase", "sdd-{change}"]
    }
  }
]
```

### Step 4: Create Links

```json
[
  {
    "tool": "create_link",
    "args": {
      "source_id": "{parent-ulid}",
      "target_id": "{new-ulid}",
      "link_type": "refines"
    }
  }
]
```

## Return Envelope

```markdown
## Phase Complete

**Status**: success | blocked | failed
**Change**: {change}
**Phase**: sdd-newphase
**Artifact ULID**: {ulid}

### Executive Summary
{2-3 sentence summary}

### Artifacts
| Title | ULID | Block Type | Action |
|-------|------|------------|--------|
| sdd/{change}/newphase | {ulid} | permanent | Created |

### Links
| From | To | Type | Status |
|------|----|------|--------|
| {parent} | {new} | refines | Created |

### Next Recommended
sdd-nextphase

### Risks
None identified
```

## Skill Templates

### Phase Skill Template

```markdown
---
name: sdd-{phase}
description: >
  {phase_description}
trigger: sdd-{phase}
---

## Purpose

{What this phase does}

## PKM-AI Tools

| Tool | Usage |
|------|-------|
| `search_blocks` | Find artifacts |
| `get_block` | Get content |
| `create_block` | Create artifact |
| `update_block` | Update artifact |
| `create_link` | Link artifacts |

## Inputs

```python
# From orchestrator:
change_name: str      # e.g., "mcp-workflow"
project_name: str     # e.g., "hodei-pkm"
mode: str            # "pkmai" | "openspec" | "hybrid" | "none"
```

## Required Artifacts

| Artifact | Tags | Purpose |
|----------|------|---------|
| {required-phase} | sdd-{required-phase} | {purpose} |

## Execution

### Step 1: Search for Required Artifacts

```json
[
  {
    "tool": "search_blocks",
    "args": {
      "query": "sdd/{change}/{required-phase}",
      "tags": ["sdd", "sdd-{required-phase}"],
      "limit": 10
    }
  }
]
```

### Step 2: Retrieve Full Content

```json
[
  {
    "tool": "get_block",
    "args": {
      "block_id": "{results.blocks[0].id}",
      "include_content": true
    }
  }
]
```

### Step 3: Execute Phase Work

{Phase-specific implementation}

### Step 4: Create Artifact

```json
[
  {
    "tool": "create_block",
    "args": {
      "block_type": "{block-type}",
      "title": "sdd/{change}/{phase}",
      "content": "# {Phase Title}\n\n{content}",
      "tags": ["sdd", "sdd-{phase}", "sdd-{change}"]
    }
  }
]
```

### Step 5: Link to Parent

```json
[
  {
    "tool": "create_link",
    "args": {
      "source_id": "{parent_ulid}",
      "target_id": "{new_ulid}",
      "link_type": "{link-type}"
    }
  }
]
```

## Return Envelope

Follow the format in `workflows/pkmai/sdd/_shared/phase-common.md`

## Dependencies

| Phase | Link Type |
|-------|-----------|
| {parent-phase} | {link-type} |
| {child-phase} | {link-type} |
```

## Sub-Agent Context Protocol

### What Sub-Agents Receive

| Input | Description | Example |
|-------|-------------|---------|
| `skill` | Path to skill file | `workflows/pkmai/sdd/phases/sdd-explore.md` |
| `change` | Change name | `mcp-workflow` |
| `project` | Project name | `hodei-pkm` |
| `mode` | Persistence mode | `pkmai` |
| `context` | Prior artifacts | `{ulids: [...], summaries: [...]}` |

### What Sub-Agents Must Do

1. **Load skill file** before starting work
2. **Search PKM-AI** for required input artifacts
3. **Retrieve full content** via `get_block` (not search preview)
4. **Create artifact** in PKM-AI before returning
5. **Link to parent** artifact when applicable
6. **Return structured envelope** with all required fields

### What Sub-Agents Must Not Do

- Read source code files directly (delegate to further sub-agents)
- Skip skill loading
- Use search preview instead of `get_block`
- Create orphan artifacts without links

## Phase Execution Matrix

| Phase | Async? | Parallel With | Blocked By |
|-------|--------|---------------|------------|
| sdd-init | No | None | None |
| sdd-explore | Yes | None | None |
| sdd-propose | Yes | None | sdd-explore (optional) |
| sdd-spec | Yes | sdd-design | sdd-propose |
| sdd-design | Yes | sdd-spec | sdd-propose |
| sdd-tasks | No | None | sdd-spec + sdd-design |
| sdd-apply | No | None | sdd-tasks |
| sdd-verify | No | None | sdd-apply |
| sdd-archive | No | None | sdd-verify |

## Common Patterns

### Search Pattern for Required Artifact

```python
def find_artifact(change: str, phase: str) -> Optional[dict]:
    """Find a required artifact by phase."""

    results = search_blocks(
        query=f"sdd/{change}/{phase}",
        tags=["sdd", f"sdd-{phase}"],
        limit=5
    )

    if not results.blocks:
        return None

    return get_block(
        block_id=results.blocks[0].id,
        include_content=True
    )
```

### Create Artifact Pattern

```python
def create_artifact(
    change: str,
    phase: str,
    block_type: str,
    content: str,
    tags: list[str] = None
) -> dict:
    """Create an SDD artifact block."""

    if tags is None:
        tags = ["sdd", f"sdd-{phase}", f"sdd-{change}"]

    return create_block(
        block_type=block_type,
        title=f"sdd/{change}/{phase}",
        content=content,
        tags=tags
    )
```

### Link Creation Pattern

```python
def link_artifacts(
    source_id: str,
    target_id: str,
    link_type: str
) -> dict:
    """Create a link between artifacts."""

    return create_link(
        source_id=source_id,
        target_id=target_id,
        link_type=link_type
    )
```

## Summary

| Component | Description |
|-----------|-------------|
| Skill | Self-contained instruction package for a phase |
| Registry | Index of skills with metadata |
| Manifest | Per-skill configuration |
| Sub-agent | Executor of a skill |
| Orchestrator | Launcher of sub-agents |

## Next Steps

- See [architecture.md](architecture.md) for orchestration details
- See [concepts.md](concepts.md) for phase descriptions
- See [token-economics.md](token-economics.md) for efficiency analysis