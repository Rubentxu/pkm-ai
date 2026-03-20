# SDD Phases Index

This directory contains individual phase skills for the SDD workflow.

## Phase Skills

| Phase | File | Purpose |
|-------|------|---------|
| `sdd-explore` | `sdd-explore.md` | Research and gather information |
| `sdd-propose` | `sdd-propose.md` | Create change proposal |
| `sdd-spec` | `sdd-spec.md` | Create detailed specification |
| `sdd-design` | `sdd-design.md` | Create technical design |
| `sdd-tasks` | `sdd-tasks.md` | Break down into tasks |
| `sdd-apply` | `sdd-apply.md` | Implement tasks |
| `sdd-verify` | `sdd-verify.md` | Verify against spec |
| `sdd-archive` | `sdd-archive.md` | Archive completed change |

## Usage

Each phase skill is designed to be loaded by a sub-agent when executing that specific phase. The orchestrator delegates to sub-agents for each phase, passing:

- Change name
- Relevant artifact ULIDs
- Phase-specific instructions

## Dependency Chain

```
sdd-explore
    ↓
sdd-propose → sdd-spec
                  ↓
              sdd-design
                  ↓
               sdd-tasks
                  ↓
               sdd-apply (iterations)
                  ↓
               sdd-verify
                  ↓
               sdd-archive
```

## Shared Files

| File | Purpose |
|------|---------|
| `_shared/phase-common.md` | Common return envelope format |
| `_shared/pkmai-convention.md` | PKM-AI specific conventions |

## Quick Reference

### Phase Inputs/Outputs

| Phase | Inputs | Output | Block Type |
|-------|--------|--------|------------|
| `sdd-explore` | None | Explore | `permanent` |
| `sdd-propose` | Explore (opt) | Proposal | `permanent` |
| `sdd-spec` | Proposal | Spec | `structure` |
| `sdd-design` | Proposal | Design | `permanent` |
| `sdd-tasks` | Spec + Design | Tasks | `outline` |
| `sdd-apply` | Tasks + Spec + Design | Progress | `permanent` |
| `sdd-verify` | Spec + Tasks | Verify | `permanent` |
| `sdd-archive` | All | Archive | `permanent` |

### Block Tags

| Phase | Tags |
|-------|------|
| `sdd-explore` | `sdd`, `explore`, `sdd-explore`, `sdd-{change}` |
| `sdd-propose` | `sdd`, `proposal`, `sdd-proposal`, `sdd-{change}` |
| `sdd-spec` | `sdd`, `spec`, `sdd-spec`, `sdd-{change}` |
| `sdd-design` | `sdd`, `design`, `sdd-design`, `sdd-{change}` |
| `sdd-tasks` | `sdd`, `tasks`, `sdd-tasks`, `sdd-{change}` |
| `sdd-apply` | `sdd`, `progress`, `sdd-progress`, `sdd-{change}` |
| `sdd-verify` | `sdd`, `verify`, `sdd-verify`, `sdd-{change}` |
| `sdd-archive` | `sdd`, `archive`, `sdd-archive`, `sdd-{change}` |
