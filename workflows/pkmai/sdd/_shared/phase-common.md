# SDD Phase Common — Return Envelope Format

All SDD phase sub-agents must return a structured envelope.

## Envelope Structure

```markdown
## Phase Complete

**Status**: success | blocked | failed
**Change**: {change-name}
**Phase**: {phase-name}
**Artifact ULID**: {ulid-or-n/a}

### Executive Summary
{2-3 sentences max. What was done and why it matters.}

### Detailed Report
{Optional. Extended explanation if needed.}

### Artifacts
| Artifact | ULID | Block Type | Action |
|----------|------|------------|--------|
| `sdd/{change}/{phase}` | `{ulid}` | `{block_type}` | Created/Updated |

### Links
| From | To | Type | Status |
|------|----|------|--------|
| `{parent_ulid}` | `{new_ulid}` | `{link_type}` | Created/Skipped |

### Content Summary
{Brief summary of artifact content}

### Next Recommended
- `sdd-{next_phase}` — if more phases remain
- `Complete` — if all phases done
- `Blocked` — if blocked by missing artifacts

### Risks
| Risk | Severity | Mitigation |
|------|----------|------------|
| {description} | High/Med/Low | {mitigation or "None"} |

### Metadata
- **Mode**: {pkmai|openspec|hybrid|none}
- **Timestamp**: {ISO 8601}
- **Skill Version**: 1.0
```

## Status Values

| Status | Meaning |
|--------|---------|
| `success` | Phase completed successfully |
| `blocked` | Cannot proceed - missing required artifacts |
| `failed` | Phase failed - error occurred |

## When Status is Blocked

If a required artifact is missing:

```markdown
**Status**: blocked

### Blocked By
- Required artifact: `sdd/{change}/{required-phase}`
- Searched tags: `["sdd-{required-phase}", "sdd"]`
- Found: No

### Recommendation
Run `sdd-{required-phase}` before this phase.
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
| `High` | Blocks next phase, requires intervention |
| `Med` | May affect quality, workaround available |
| `Low` | Minor issue, can be addressed later |

## Skill Loading

At the start of execution, load:
1. This skill (`workflows/pkmai/sdd/SKILL.md`)
2. Shared conventions (`workflows/pkmai/sdd/_shared/pkmai-convention.md`)

## Required Tool Calls

### Phase Start
1. Search for required artifacts via `search_blocks`
2. Retrieve full content via `get_block`

### Phase End
1. Create artifact via `create_block`
2. Create links via `create_link` (if applicable)
3. Return structured envelope

## Quick Reference

| Phase | Required Inputs | Output |
|-------|----------------|--------|
| `sdd-explore` | None | Explore block |
| `sdd-propose` | Explore (optional) | Proposal block |
| `sdd-spec` | Proposal | Spec block |
| `sdd-design` | Proposal | Design block |
| `sdd-tasks` | Spec + Design | Tasks block |
| `sdd-apply` | Tasks + Spec + Design | Progress block |
| `sdd-verify` | Spec + Tasks | Verify block |
| `sdd-archive` | All | Archive block |
