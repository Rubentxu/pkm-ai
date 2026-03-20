# PKM-AI Workflow System

Quick start guide for the Spec-Driven Development (SDD) workflow using PKM-AI.

## Quick Start

### 1. Initialize a New Change

```bash
/sdd-init <change-name>
```

Example:
```
/sdd-init mcp-workflow
```

This creates:
- Project context block (`sdd/{change}/project`)
- Phase tracker block (`sdd/{change}/tracker`)
- Discovery block (`sdd/{change}/discovery`)

### 2. Run SDD Phases

| Phase | Command | Purpose |
|-------|---------|---------|
| Explore | `/sdd-explore <topic>` | Research and gather information |
| Propose | `/sdd-propose` | Define problem and solution |
| Spec | `/sdd-spec` | Create detailed specification |
| Design | `/sdd-design` | Make architectural decisions |
| Tasks | `/sdd-tasks` | Break down implementation |
| Apply | `/sdd-apply` | Execute implementation |
| Verify | `/sdd-verify` | Validate implementation |
| Archive | `/sdd-archive` | Summarize completed change |

### 3. Create PR

```bash
/branch-pr
```

---

## Key Concepts

### SDD Workflow

Spec-Driven Development (SDD) is a structured planning methodology that decomposes substantial changes into discrete phases:

```
proposal -> spec --> tasks -> apply -> verify -> archive
             ^
             |
           design
```

### PKM-AI Storage

All artifacts are stored as **blocks** in PKM-AI:

| Block Type | SDD Usage |
|------------|-----------|
| `permanent` | Explore, Proposal, Design, Verify, Archive |
| `structure` | Spec (has internal structure with scenarios) |
| `outline` | Tasks (checklist format) |

### Artifact Titles

Artifacts follow the format: `sdd/{change-name}/{phase}`

Examples:
- `sdd/mcp-workflow/explore`
- `sdd/mcp-workflow/proposal`
- `sdd/mcp-workflow/spec`

---

## SDD Phases Overview

### Phase 1: Explore (`sdd-explore`)
Research and gather information about a topic. Creates exploration artifact with:
- Key concepts and relationships
- Current state analysis
- Challenges and opportunities
- Research sources

### Phase 2: Propose (`sdd-propose`)
Define the problem and proposed solution. Creates proposal artifact with:
- Problem statement
- Proposed solution
- Expected outcomes and benefits
- Scope and constraints
- Risks and mitigations

### Phase 3: Spec (`sdd-spec`)
Create detailed specification with Given/When/Then scenarios. Creates spec artifact with:
- Functional requirements
- Acceptance criteria
- User interactions
- Data models
- Edge cases

### Phase 4: Design (`sdd-design`)
Make architectural decisions. Creates design artifact with:
- Module design
- Data structures
- API contracts
- Decision log

### Phase 5: Tasks (`sdd-tasks`)
Break down implementation into actionable tasks. Creates tasks artifact with:
- Ordered task list
- Dependencies
- Time estimates
- Testing strategy

### Phase 6: Apply (`sdd-apply`)
Execute implementation tasks. Creates progress artifact tracking:
- Completed tasks
- Remaining tasks
- Implementation notes

### Phase 7: Verify (`sdd-verify`)
Validate implementation against spec. Creates verify artifact with:
- Pass/fail for each acceptance criteria
- Test results
- Issues found

### Phase 8: Archive (`sdd-archive`)
Summarize completed change. Creates archive artifact with:
- Final summary
- Lessons learned
- Links to all artifacts

---

## Commands Reference

| Command | Action |
|---------|--------|
| `/sdd-init <change>` | Initialize new SDD project |
| `/sdd-explore <topic>` | Research topic |
| `/sdd-propose` | Create proposal |
| `/sdd-spec` | Create specification |
| `/sdd-design` | Create design |
| `/sdd-tasks` | Create task breakdown |
| `/sdd-apply` | Apply/implement tasks |
| `/sdd-verify` | Verify implementation |
| `/sdd-archive` | Archive completed change |
| `/sdd-continue [change]` | Continue missing phase |
| `/sdd-ff [change]` | Fast forward to next pending phase |
| `/branch-pr` | Create pull request |

---

## Example Usage

### Full SDD Workflow

```
User: /sdd-init feature-flags

Orchestrator: Initializing SDD project for 'feature-flags'...

User: /sdd-explore feature flag systems

Orchestrator: Running explore phase...
→ Creates: sdd/feature-flags/explore

User: /sdd-propose feature-flags

Orchestrator: Running propose phase...
→ Creates: sdd/feature-flags/proposal

User: /sdd-spec feature-flags

Orchestrator: Running spec phase...
→ Creates: sdd/feature-flags/spec

User: /sdd-design feature-flags

Orchestrator: Running design phase...
→ Creates: sdd/feature-flags/design

User: /sdd-tasks feature-flags

Orchestrator: Running tasks phase...
→ Creates: sdd/feature-flags/tasks

User: /sdd-apply feature-flags

Orchestrator: Running apply phase...
→ Creates: sdd/feature-flags/progress

User: /sdd-verify feature-flags

Orchestrator: Running verify phase...
→ Creates: sdd/feature-flags/verify

User: /sdd-archive feature-flags

Orchestrator: Running archive phase...
→ Creates: sdd/feature-flags/archive

User: /branch-pr

Orchestrator: Creating pull request...
→ Creates branch and PR
```

---

## Links to Full Documentation

- [CLAUDE.md](CLAUDE.md) - Orchestrator role and coordination rules
- [docs/concepts.md](docs/concepts.md) - Core SDD concepts and PKM-AI conventions
- [docs/architecture.md](docs/architecture.md) - System architecture
- [docs/installation.md](docs/installation.md) - Installation and setup
- [docs/persistence.md](docs/persistence.md) - PKM-AI storage modes
- [docs/sub-agents.md](docs/sub-agents.md) - Sub-agent phase details
- [docs/token-economics.md](docs/token-economics.md) - Token efficiency guide
- [docs/best-practices.md](docs/best-practices.md) - PKM-AI usage best practices
- [AGENTS.md](AGENTS.md) - Project-level workspace rules

---

## Skills Directory

All SDD phases are implemented as skills in `workflows/pkmai/skills/`:

```
workflows/pkmai/skills/
├── sdd-init/          # Initialize SDD project
├── sdd-explore/       # Research phase
├── sdd-propose/       # Proposal phase
├── sdd-spec/          # Specification phase
├── sdd-design/        # Design phase
├── sdd-tasks/         # Task breakdown phase
├── sdd-apply/         # Implementation phase
├── sdd-verify/        # Verification phase
├── sdd-archive/       # Archive phase
├── branch-pr/         # Pull request creation
├── issue-creation/    # Issue creation
└── skill-registry/    # Skill discovery
```

See [SKILL-MANIFEST.md](SKILL-MANIFEST.md) for complete skill index.
