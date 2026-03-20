# SDD Phase Common Protocol

All SDD phase sub-agents MUST follow this protocol. This document defines the return envelope format, required steps, and common conventions.

## Envelope Structure

All phases MUST return this exact structure:

```markdown
## {Phase} Complete

**Status**: success | blocked | failed
**Change**: {change-name}
**Phase**: {phase-name}
**Artifact ULID**: {ulid-or-n/a}
**Mode**: {pkmai|openspec|hybrid|none}

### Executive Summary
{2-3 sentences max. What was done and why it matters.}

### Artifacts
| Artifact | ULID | Block Type | Action |
|----------|------|------------|--------|
| `sdd/{change}/{phase}` | `{ulid}` | `{block_type}` | Created/Updated |

### Links
| From | To | Type | Status |
|------|----|------|--------|
| `{parent_ulid}` | `{new_ulid}` | `{link_type}` | Created/Skipped |

### Content Summary
{Brief summary of artifact content (100-300 chars)}

### Next Recommended
- `sdd-{next_phase}` — if more phases remain
- `Complete` — if all phases done
- `Blocked` — if blocked by missing artifacts

### Risks
| Risk | Severity | Mitigation |
|------|----------|------------|
| {description} | High/Med/Low | {mitigation or "None"} |

### Metadata
- **Timestamp**: {ISO 8601}
- **Skill Version**: 1.0
```

## Status Values

| Status | Meaning | When to Use |
|--------|---------|-------------|
| `success` | Phase completed successfully | All required work done |
| `blocked` | Cannot proceed | Missing required artifacts |
| `failed` | Phase failed | Error occurred during execution |

## Phase Dependencies

| Phase | Required Inputs | Produces |
|-------|-----------------|----------|
| `sdd-init` | None | Project, Tracker, Discovery |
| `sdd-explore` | None (creates fresh) | Explore block |
| `sdd-propose` | Explore (optional) | Proposal block |
| `sdd-spec` | Proposal | Spec block |
| `sdd-design` | Proposal | Design block |
| `sdd-tasks` | Spec + Design | Tasks block |
| `sdd-apply` | Tasks + Spec + Design | Progress block |
| `sdd-verify` | Spec + Tasks | Verify block |
| `sdd-archive` | All phases | Archive block |

## Required Steps

### Phase Start

1. **Load Skills**
   - Load `workflows/pkmai/skills/_shared/sdd-phase-common.md` (this file)
   - Load `workflows/pkmai/skills/_shared/pkmai-convention.md` (for PKM-AI)
   - Load `workflows/pkmai/skills/_shared/openspec-convention.md` (for openspec)

2. **Detect Mode**
   - `pkmai` - Use PKM-AI MCP tools
   - `openspec` - Use filesystem
   - `hybrid` - Use both
   - `none` - Return only, don't persist

3. **Search for Required Artifacts**
   - Based on phase, find required input artifacts
   - Use `search_blocks` (PKM-AI) or read files (openspec)

4. **Retrieve Full Content**
   - For each required artifact, get full content
   - **CRITICAL**: Never use search previews as source material
   - Use `get_block` (PKM-AI) or file read (openspec)

### Phase Execution

5. **Execute Phase Work**
   - Produce the phase artifact according to its template
   - Follow the phase-specific guidelines

6. **Create Artifact**
   - Create in appropriate store based on mode
   - Use correct block type for phase
   - Use correct title format: `sdd/{change}/{phase}`

7. **Create Links**
   - Link to parent artifact using `create_link`
   - Use correct link type (see pkmai-convention.md)

8. **Update Tracker (if applicable)**
   - Update phase status in tracker block

### Phase End

9. **Return Structured Envelope**
   - Follow envelope format exactly
   - Include all required fields

## When Status is Blocked

If a required artifact is missing:

```markdown
**Status**: blocked

### Blocked By
- Required artifact: `sdd/{change}/{required-phase}`
- Searched tags: `["sdd-{required-phase}", "sdd"]`
- Found: No

### Recommendation
Run `sdd-{required_phase}` before this phase.
```

## When Status is Failed

If an error occurred:

```markdown
**Status**: failed

### Error
```
{error message}
```

### Recommendation
Fix error and re-run phase.
```

## Risk Severity Guidelines

| Severity | When to Use |
|----------|-------------|
| `High` | Blocks next phase, requires human intervention |
| `Med` | May affect quality, workaround available |
| `Low` | Minor issue, can be addressed later |

## Skill Loading Reference

At the start of execution, load these files:

```
workflows/pkmai/skills/_shared/sdd-phase-common.md     # This file
workflows/pkmai/skills/_shared/pkmai-convention.md     # PKM-AI conventions
workflows/pkmai/skills/_shared/openspec-convention.md  # Openspec conventions
workflows/pkmai/skills/_shared/persistence-contract.md # Persistence contract
```

## Quick Reference Tables

### Block Types by Phase

| Phase | Block Type |
|-------|------------|
| project | `structure` |
| tracker | `outline` |
| discovery | `permanent` |
| explore | `permanent` |
| proposal | `permanent` |
| spec | `structure` |
| design | `permanent` |
| tasks | `outline` |
| progress | `permanent` |
| verify | `permanent` |
| archive | `permanent` |

### Tag Requirements

All artifacts MUST have:
- `sdd` (primary tag)
- `sdd-{phase}` (phase tag)
- `sdd-{change}` (change tag)

### Link Types

| From | To | Link Type |
|------|----|-----------|
| Project | Tracker | `contains` |
| Explore | Proposal | `refines` |
| Proposal | Spec | `refines` |
| Proposal | Design | `refines` |
| Spec | Design | `refines` |
| Design | Tasks | `contains` |
| Tasks | Progress | `related` |
| Spec | Verify | `supports` |
| Project | Archive | `contains` |

### Phase Titles

```
sdd/{change}/project
sdd/{change}/tracker
sdd/{change}/discovery
sdd/{change}/explore
sdd/{change}/proposal
sdd/{change}/spec
sdd/{change}/design
sdd/{change}/tasks
sdd/{change}/progress
sdd/{change}/verify
sdd/{change}/archive
```

## TDD Mode Detection

Detect TDD from (in priority order):
1. `openspec/config.yaml` → `rules.apply.tdd`
2. Project skills (e.g., `tdd/SKILL.md` exists)
3. Existing test patterns in codebase
4. Default: standard mode

If TDD detected, apply RED → GREEN → REFACTOR cycle per task.

## Hybrid Mode Notes

When mode is `hybrid`:
1. PKM-AI is used for search and navigation
2. Filesystem is the source of truth for content
3. Write to both stores
4. Read from filesystem for full content

## Common Pitfalls

1. **Using search previews as source material** - ALWAYS retrieve full content
2. **Skipping link creation** - Links are required for graph navigation
3. **Wrong block type** - Use correct type for phase
4. **Missing tags** - All artifacts need `sdd`, `sdd-{phase}`, `sdd-{change}`
5. **Not updating tracker** - Tracker must reflect current state

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-03-20 | Initial version for PKM-AI |
