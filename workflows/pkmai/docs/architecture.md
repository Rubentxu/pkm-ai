# PKM-AI Workflow Architecture

## Overview

The PKM-AI workflow system implements **Spec-Driven Development (SDD)** with PKM-AI as the persistent artifact store. It follows an **agent-teams-lite** pattern where a coordinator orchestrates specialized sub-agents through a structured phase pipeline.

```
┌─────────────────────────────────────────────────────────────────────┐
│                         ORCHESTRATOR                                │
│  (Coordinator Agent - thin conversation thread with user)            │
│                                                                     │
│  - Maintains one conversation thread                                │
│  - Delegates ALL execution work to sub-agents                       │
│  - Synthesizes sub-agent results                                    │
│  - Tracks state via PKM-AI MCP tools                               │
└────────────────────────────┬────────────────────────────────────────┘
                             │
          ┌──────────────────┼──────────────────┐
          │                  │                  │
          ▼                  ▼                  ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│  Sub-Agent:     │ │  Sub-Agent:     │ │  Sub-Agent:     │
│  sdd-explore    │ │  sdd-propose    │ │  sdd-spec       │
│                 │ │                 │ │                 │
│  Skill: explore │ │ Skill: propose  │ │ Skill: spec     │
│  + PKM-AI tools │ │ + PKM-AI tools │ │ + PKM-AI tools  │
└────────┬────────┘ └────────┬────────┘ └────────┬────────┘
         │                    │                    │
         └────────────────────┼────────────────────┘
                              │
                              ▼
                   ┌─────────────────────┐
                   │      PKM-AI         │
                   │   (Artifact Store)  │
                   │                     │
                   │  - Blocks (ULID)    │
                   │  - Links (graph)    │
                   │  - Tags (classify)  │
                   └─────────────────────┘
```

## Core Architecture Principles

### 1. Orchestrator as Coordinator

The orchestrator follows a strict **delegation-first** policy:

| Allowed (Orchestrator) | Forbidden (Orchestrator) |
|------------------------|--------------------------|
| Short answers | Reading source code |
| Coordinating phases | Writing/editing code |
| Showing summaries | Writing specs or proposals |
| Asking decisions | Doing analysis inline |
| Tracking state | "Quick" inline work |

**Hard Stop Rule**: Before using Read, Edit, Write, or Grep on source files, the orchestrator must delegate to a sub-agent.

### 2. Sub-Agents as Executors

Sub-agents receive:
- Skill path to load
- Artifact store mode
- Change name and project
- PKM-AI context (recent relevant blocks)

Sub-agents return:
- Structured envelope with status
- Artifact ULIDs created
- Links established
- Next recommendations

### 3. PKM-AI as Artifact Store

PKM-AI serves as the persistent memory layer:

| PKM-AI Concept | Description |
|----------------|-------------|
| Blocks | Primary storage unit (replaces Observations) |
| Tags + Links | Relationships (replaces Topic keys) |
| ULID | Unique identifier system |
| Graph | Structured relationships between artifacts |

### PKM-AI Tools

| Tool | Purpose |
|------|---------|
| `search_blocks` | Find artifacts by query and tags |
| `get_block` | Get full artifact content |
| `create_block` | Create new artifact |
| `create_link` | Link artifacts together |
| `update_block` | Update artifact content |

## How Orchestration Works

### Phase Delegation Pattern

```
User Request
      │
      ▼
┌─────────────────────────────────┐
│      ORCHESTRATOR               │
│                                 │
│  1. Validate request            │
│  2. Check PKM-AI for context   │
│  3. Launch sub-agent (async)   │
│  4. Return "Working on it..."   │
└─────────────────────────────────┘
      │
      ▼ (async)
┌─────────────────────────────────┐
│      SUB-AGENT                  │
│                                 │
│  1. Load skill                 │
│  2. Search PKM-AI for inputs   │
│  3. Execute phase work        │
│  4. Create artifacts in PKM-AI │
│  5. Return structured envelope │
└─────────────────────────────────┘
      │
      ▼
┌─────────────────────────────────┐
│      ORCHESTRATOR               │
│                                 │
│  1. Receive envelope           │
│  2. Synthesize results         │
│  3. Present to user           │
│  4. Await next instruction     │
└─────────────────────────────────┘
```

### Parallel Phase Execution

Independent phases can run in parallel:

```python
# Parallel launch of independent phases
sub_agent_explore = Agent(
    skill="workflows/pkmai/sdd/phases/sdd-explore.md",
    async=True
)

sub_agent_design = Agent(
    skill="workflows/pkmai/sdd/phases/sdd-design.md",
    async=True
)

# Both run concurrently
await sub_agent_explore
await sub_agent_design
```

### Phase Dependency Graph

```
sdd-explore ──► sdd-propose ──┬──► sdd-spec ──► sdd-tasks ──► sdd-apply ──► sdd-verify ──► sdd-archive
                              │
                              └──► sdd-design ──┘
```

## Skills as Modular Instructions

### Skill Structure

Each skill is a self-contained module:

```
workflows/pkmai/
├── skills/
│   ├── sdd-init/
│   │   └── SKILL.md           # Initialization skill
│   └── ...
└── sdd/
    ├── phases/
    │   ├── sdd-explore.md     # Exploration phase
    │   ├── sdd-propose.md     # Proposal phase
    │   ├── sdd-spec.md        # Specification phase
    │   ├── sdd-design.md      # Design phase
    │   ├── sdd-tasks.md       # Task breakdown phase
    │   ├── sdd-apply.md       # Implementation phase
    │   ├── sdd-verify.md      # Verification phase
    │   └── sdd-archive.md     # Archive phase
    └── SKILL.md               # Base SDD skill
```

### Skill Loading

Sub-agents load skills at startup:

```markdown
## Skill Loading Sequence

1. Load base SDD skill:
   `workflows/pkmai/sdd/SKILL.md`

2. Load shared conventions:
   `workflows/pkmai/sdd/_shared/pkmai-convention.md`
   `workflows/pkmai/sdd/_shared/phase-common.md`

3. Load phase-specific skill:
   `workflows/pkmai/sdd/phases/sdd-{phase}.md`
```

### Skill Composition

Skills compose through inheritance:

```
Base Skill (sdd/SKILL.md)
    │
    ├── Common conventions (_shared/pkmai-convention.md)
    │
    └── Phase Skill (phases/sdd-{phase}.md)
```

## PKM-AI as Artifact Store

### Key Characteristics

| Aspect | PKM-AI |
|--------|--------|
| Storage | Blocks |
| ID System | ULID |
| Relationships | Explicit links |
| Classification | Block type + Tags |
| Structure | Graph |
| Query | Full-text + Tags + Type |

### PKM-AI Advantages for SDD

1. **Persistent Memory**: Blocks survive session boundaries
2. **Semantic Links**: Graph relationships between artifacts
3. **Structured Types**: Block types match artifact categories
4. **Tag Classification**: Tags enable faceted search
5. **Graph Traversal**: Navigate via links, not just search
6. **Zettelkasten**: Proven methodology for atomic knowledge

### Artifact Storage Convention

Artifacts are stored as PKM-AI blocks:

```json
[
  {
    "tool": "create_block",
    "args": {
      "block_type": "structure",
      "title": "sdd/mcp-workflow/spec",
      "content": "# Specification\n\n...",
      "tags": ["sdd", "spec", "sdd-mcp-workflow"]
    }
  }
]
```

## Skill Discovery and Registry

### Skill Registry Location

Skills are registered in:

```
workflows/pkmai/sdd/skill-registry/
├── README.md           # Registry documentation
└── skills/             # Individual skill manifests
```

### Skill Discovery Process

1. **Command triggers skill**: `/sdd-init` triggers `sdd-init` skill
2. **Phase triggers skill**: `sdd-explore` phase loads `sdd-explore.md`
3. **Registry lookup**: Orchestrator looks up skill path in registry

### Adding New Skills

To add a new skill:

1. Create skill file: `workflows/pkmai/skills/{skill-name}/SKILL.md`
2. Register in skill manifest
3. Update orchestrator commands if needed

### Skill Manifest Format

```yaml
# workflows/pkmai/sdd/skill-registry/skills/{skill-name}/manifest.yaml
name: {skill-name}
description: >
  What this skill does.
trigger: {command or phase name}
skill_path: workflows/pkmai/skills/{skill-name}/SKILL.md
version: "1.0"
```

## State Management

### State Location

All SDD state lives in PKM-AI:

```
┌─────────────────────────────────────────────────────┐
│                    PKM-AI                           │
│                                                     │
│  ┌─────────────────┐  ┌─────────────────┐         │
│  │  Project Block  │  │  Tracker Block  │         │
│  │  sdd/{change}   │──│  Phase Status   │         │
│  │  /project       │  │  Last Update    │         │
│  └─────────────────┘  └─────────────────┘         │
│                                                     │
│  ┌─────────────────────────────────────────┐       │
│  │           Artifact Blocks                │       │
│  │                                          │       │
│  │  sdd/{change}/explore ──► proposal      │       │
│  │  sdd/{change}/proposal ──► spec        │       │
│  │  sdd/{change}/spec ──► design         │       │
│  │  ...                                    │       │
│  └─────────────────────────────────────────┘       │
└─────────────────────────────────────────────────────┘
```

### Recovery Protocol

When resuming a change:

```python
# Step 1: Find project context
projects = search_blocks(
    query="sdd/{change}/project",
    tags=["sdd", "project"]
)

# Step 2: Find all artifacts
artifacts = search_blocks(
    query="sdd-{change}",
    tags=["sdd"]
)

# Step 3: Find tracker
trackers = search_blocks(
    query="sdd/{change}/tracker",
    tags=["sdd", "tracker"]
)

# Step 4: Determine next phase from tracker
tracker = get_block(block_id=trackers[0].id)
# Read phase status table
```

## Communication Patterns

### Orchestrator ↔ Sub-Agent

**Launch**:
```python
Agent(
    skill="workflows/pkmai/sdd/phases/sdd-explore.md",
    args={
        "change": "mcp-workflow",
        "project": "hodei-pkm",
        "mode": "pkmai"
    },
    run_in_background=True
)
```

**Result**:
```markdown
## Phase Complete

**Status**: success
**Change**: mcp-workflow
**Phase**: sdd-explore
**Artifact ULID**: 01ARZ3NDEKTSV4RRFFQ69G5FAV

### Executive Summary
Completed exploration of MCP protocol integration...

### Next Recommended
sdd-propose
```

### Sub-Agent → PKM-AI

**Create Artifact**:
```python
create_block(
    block_type="permanent",
    title="sdd/mcp-workflow/explore",
    content="# Exploration\n\n...",
    tags=["sdd", "explore", "sdd-mcp-workflow"]
)
```

**Create Link**:
```python
create_link(
    source_id="01ARZ3NDEKTSV4RRFFQ69G5FAV",  # explore
    target_id="01ARZ3NDEKTSV4RRFFQ69G5FAW",  # proposal
    link_type="refines"
)
```

## Anti-Patterns to Avoid

### Orchestrator Anti-Patterns

| Anti-Pattern | Problem | Solution |
|--------------|---------|----------|
| Inline code reading | Bloats context | Delegate to sub-agent |
| "Quick" analysis | Leads to more inline work | Delegate to sub-agent |
| Skipping delegation for small tasks | Two tasks become five | Delegate always |
| Reading source to "understand" | Not orchestration | Delegate to sub-agent |

### Sub-Agent Anti-Patterns

| Anti-Pattern | Problem | Solution |
|--------------|---------|----------|
| Using search preview instead of get_block | Truncated content | Always use get_block |
| Creating artifact without linking | Orphan artifacts | Always link to parent |
| Skipping skill load | Missing conventions | Always load skills first |

## Summary

The PKM-AI workflow architecture:

1. **Orchestrator** = thin coordinator, never executes
2. **Sub-agents** = skill-based executors
3. **PKM-AI** = persistent artifact store
4. **Skills** = modular instruction packages
5. **Links** = artifact relationships
6. **Tags** = artifact classification

This separation ensures:
- No context bloat from inline work
- Persistent memory across sessions
- Structured artifact retrieval
- Graph-based navigation
- Modular skill composition