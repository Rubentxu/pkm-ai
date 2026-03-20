# SKILL-MANIFEST.md

Complete index of all skills available in the PKM-AI workflow system.

## Overview

| Total Skills | 12 |
|--------------|----|
| SDD Phase Skills | 9 |
| Utility Skills | 3 |

---

## SDD Phase Skills

SDD (Spec-Driven Development) phases are implemented as skills that execute in sequence.

### 1. sdd-init

| Field | Value |
|-------|-------|
| **Path** | `workflows/pkmai/skills/sdd-init` |
| **Purpose** | Initialize a new SDD change project in PKM-AI |
| **Trigger Phrases** | `/sdd-init`, `new change`, `initialize project` |
| **Block Type** | `structure` (project), `outline` (tracker), `permanent` (discovery) |
| **Output** | Creates project context, phase tracker, and discovery blocks |

**When to Use:**
- User requests `/sdd-init <change-name>`
- Starting a new feature, refactor, or substantial change
- First step before running SDD phases

**Triggers:**
```
/sdd-init feature-flags
/sdd-init mcp-workflow
```

---

### 2. sdd-explore

| Field | Value |
|-------|-------|
| **Path** | `workflows/pkmai/skills/sdd-explore` |
| **Purpose** | Research and gather information about a topic |
| **Trigger Phrases** | `/sdd-explore`, `research`, `explore topic`, `investigate` |
| **Block Type** | `permanent` |
| **Output** | Creates exploration artifact with research findings |

**When to Use:**
- User requests `/sdd-explore <topic>`
- Need to understand problem space before proposing solutions
- First research phase of SDD

**Triggers:**
```
/sdd-explore feature flag systems
/sdd-explore MCP server implementation
```

---

### 3. sdd-propose

| Field | Value |
|-------|-------|
| **Path** | `workflows/pkmai/skills/sdd-propose` |
| **Purpose** | Create a proposal defining problem and solution |
| **Trigger Phrases** | `/sdd-propose`, `create proposal`, `propose solution` |
| **Block Type** | `permanent` |
| **Output** | Creates proposal artifact with problem statement, solution, scope |

**When to Use:**
- After exploration is complete
- User requests `/sdd-propose`
- Need to define what to build before specifying how

**Triggers:**
```
/sdd-propose
/propose mcp-workflow
```

---

### 4. sdd-spec

| Field | Value |
|-------|-------|
| **Path** | `workflows/pkmai/skills/sdd-spec` |
| **Purpose** | Create detailed specification with Given/When/Then scenarios |
| **Trigger Phrases** | `/sdd-spec`, `create spec`, `specification`, `detailed requirements` |
| **Block Type** | `structure` |
| **Output** | Creates spec artifact with functional requirements, acceptance criteria |

**When to Use:**
- After proposal is approved
- User requests `/sdd-spec`
- Need detailed scenarios before design

**Triggers:**
```
/sdd-spec
/create specification
```

---

### 5. sdd-design

| Field | Value |
|-------|-------|
| **Path** | `workflows/pkmai/skills/sdd-design` |
| **Purpose** | Make architectural decisions |
| **Trigger Phrases** | `/sdd-design`, `architect`, `design phase`, `create design` |
| **Block Type** | `permanent` |
| **Output** | Creates design artifact with module design, data structures, API contracts |

**When to Use:**
- After spec is complete
- User requests `/sdd-design`
- Need to decide how to build before breaking into tasks

**Triggers:**
```
/sdd-design
/architect the solution
```

---

### 6. sdd-tasks

| Field | Value |
|-------|-------|
| **Path** | `workflows/pkmai/skills/sdd-tasks` |
| **Purpose** | Break down implementation into actionable tasks |
| **Trigger Phrases** | `/sdd-tasks`, `task breakdown`, `create tasks`, `implementation plan` |
| **Block Type** | `outline` |
| **Output** | Creates tasks artifact with ordered checklist, dependencies, estimates |

**When to Use:**
- After spec and design are complete
- User requests `/sdd-tasks`
- Need to break work into implementable units

**Triggers:**
```
/sdd-tasks
/create tasks
/implementation plan
```

---

### 7. sdd-apply

| Field | Value |
|-------|-------|
| **Path** | `workflows/pkmai/skills/sdd-apply` |
| **Purpose** | Execute implementation tasks |
| **Trigger Phrases** | `/sdd-apply`, `implement`, `apply changes`, `start coding` |
| **Block Type** | `permanent` (progress) |
| **Output** | Creates progress artifact tracking completed and remaining tasks |

**When to Use:**
- After tasks are defined
- User requests `/sdd-apply`
- Ready to begin implementation

**Triggers:**
```
/sdd-apply
/start implementation
/implement
```

---

### 8. sdd-verify

| Field | Value |
|-------|-------|
| **Path** | `workflows/pkmai/skills/sdd-verify` |
| **Purpose** | Validate implementation against spec |
| **Trigger Phrases** | `/sdd-verify`, `verify`, `validate`, `test implementation` |
| **Block Type** | `permanent` |
| **Output** | Creates verify artifact with pass/fail for each acceptance criteria |

**When to Use:**
- After implementation is complete
- User requests `/sdd-verify`
- Need to confirm spec requirements are met

**Triggers:**
```
/sdd-verify
/validate implementation
/check against spec
```

---

### 9. sdd-archive

| Field | Value |
|-------|-------|
| **Path** | `workflows/pkmai/skills/sdd-archive` |
| **Purpose** | Summarize completed change |
| **Trigger Phrases** | `/sdd-archive`, `archive`, `complete`, `finalize` |
| **Block Type** | `permanent` |
| **Output** | Creates archive artifact with final summary, lessons learned |

**When to Use:**
- After verification is complete
- User requests `/sdd-archive`
- SDD workflow is finished

**Triggers:**
```
/sdd-archive
/finalize
/complete the change
```

---

## Utility Skills

### 10. branch-pr

| Field | Value |
|-------|-------|
| **Path** | `workflows/pkmai/skills/branch-pr` |
| **Purpose** | Create a pull request from current changes |
| **Trigger Phrases** | `/branch-pr`, `/create-pr`, `create pull request`, `branch PR` |
| **Block Type** | N/A (git operations) |
| **Output** | Creates branch and PR in GitHub |

**When to Use:**
- Changes are ready to be reviewed
- User requests `/branch-pr`
- After `sdd-archive` to submit changes

**Triggers:**
```
/branch-pr
/create PR
/create pull request
```

---

### 11. issue-creation

| Field | Value |
|-------|-------|
| **Path** | `workflows/pkmai/skills/issue-creation` |
| **Purpose** | Create GitHub issues from SDD artifacts |
| **Trigger Phrases** | `/create-issue`, `create issue`, `open issue` |
| **Block Type** | N/A (git operations) |
| **Output** | Creates GitHub issues linked to SDD artifacts |

**When to Use:**
- User wants to create issues from proposals, specs, or tasks
- Need to track work in GitHub alongside PKM-AI

**Triggers:**
```
/create-issue
/create issue from spec
```

---

### 12. skill-registry

| Field | Value |
|-------|-------|
| **Path** | `workflows/pkmai/skills/skill-registry` |
| **Purpose** | Discover, index, and register skills |
| **Trigger Phrases** | `/skill-registry`, `list skills`, `show skills`, `what skills` |
| **Block Type** | `permanent` (registry block) |
| **Output** | Lists all skills or updates skill registry |

**When to Use:**
- User asks to list available skills
- Need to find a skill for a specific task
- Starting a new session and loading available skills

**Triggers:**
```
/skill-registry
/list skills
/what skills exist
/show available skills
```

---

## Skill Index Table

| Skill | Path | Phase | Block Type | Triggers |
|-------|------|-------|------------|----------|
| sdd-init | skills/sdd-init | 0 (Setup) | structure/outline | `/sdd-init`, new change |
| sdd-explore | skills/sdd-explore | 1 | permanent | `/sdd-explore`, research |
| sdd-propose | skills/sdd-propose | 2 | permanent | `/sdd-propose`, proposal |
| sdd-spec | skills/sdd-spec | 3 | structure | `/sdd-spec`, spec |
| sdd-design | skills/sdd-design | 4 | permanent | `/sdd-design`, design |
| sdd-tasks | skills/sdd-tasks | 5 | outline | `/sdd-tasks`, tasks |
| sdd-apply | skills/sdd-apply | 6 | permanent | `/sdd-apply`, implement |
| sdd-verify | skills/sdd-verify | 7 | permanent | `/sdd-verify`, verify |
| sdd-archive | skills/sdd-archive | 8 | permanent | `/sdd-archive`, archive |
| branch-pr | skills/branch-pr | Utility | N/A | `/branch-pr`, create-pr |
| issue-creation | skills/issue-creation | Utility | N/A | `/create-issue` |
| skill-registry | skills/skill-registry | Utility | permanent | `/skill-registry`, list skills |

---

## Phase Flow Diagram

```
┌─────────────┐
│  sdd-init  │ ← Initialize project
└──────┬──────┘
       │
       ▼
┌─────────────┐
│sdd-explore  │ ← Research topic
└──────┬──────┘
       │
       ▼
┌─────────────┐
│sdd-propose  │ ← Define problem/solution
└──────┬──────┘
       │
       ├──────────────────┐
       │                  │
       ▼                  ▼
┌─────────────┐    ┌─────────────┐
│  sdd-spec   │    │ sdd-design  │ ← Parallel possible
└──────┬──────┘    └──────┬──────┘
       │                  │
       └────────┬─────────┘
                │
                ▼
         ┌─────────────┐
         │  sdd-tasks  │ ← Break into tasks
         └──────┬──────┘
                │
                ▼
         ┌─────────────┐
         │  sdd-apply  │ ← Implement
         └──────┬──────┘
                │
                ▼
         ┌─────────────┐
         │ sdd-verify  │ ← Validate
         └──────┬──────┘
                │
                ▼
         ┌─────────────┐
         │sdd-archive  │ ← Complete
         └──────┬──────┘
                │
                ▼
         ┌─────────────┐
         │  branch-pr  │ ← Create PR (utility)
         └─────────────┘
```

---

## Finding Skills

### Search by Trigger

```python
# Find skill for exploration
search_blocks(query="sdd-explore", tags=["skill"])

# Find skill for any SDD phase
search_blocks(query="sdd", tags=["skill", "sdd"])

# Find all utility skills
search_blocks(query="utility", tags=["skill"])
```

### Load Skill File

```bash
# Direct path to skill
workflows/pkmai/skills/{skill-name}/SKILL.md

# Examples:
workflows/pkmai/skills/sdd-init/SKILL.md
workflows/pkmai/skills/sdd-explore/SKILL.md
workflows/pkmai/skills/branch-pr/SKILL.md
```

---

## Adding New Skills

To add a new skill:

1. Create directory: `workflows/pkmai/skills/{skill-name}/`
2. Create `SKILL.md` with frontmatter:

```markdown
---
name: {skill-name}
description: >
  Brief description of what this skill does.
  Trigger: When user asks to do X or Y.
license: MIT
metadata:
  author: pkm-ai
  version: "1.0"
---

## Purpose
{What this skill does}

## When to Use
{Trigger phrases that invoke this skill}

## Execution
{Step-by-step instructions}
```

3. Update this manifest with the new skill entry

---

## Related Documentation

- [README.md](README.md) - Quick start guide
- [AGENTS.md](AGENTS.md) - Project-level workspace rules
- [docs/concepts.md](docs/concepts.md) - Core SDD concepts
- [docs/architecture.md](docs/architecture.md) - System architecture
- [docs/best-practices.md](docs/best-practices.md) - PKM-AI usage best practices
