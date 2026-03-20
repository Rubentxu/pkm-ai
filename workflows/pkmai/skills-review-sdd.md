# SDD Skills Review: PKM-AI MCP Tool Integration

**Date**: 2026-03-20
**Scope**: All 9 SDD phase skills in `workflows/pkmai/skills/sdd-*`
**Context**: New PKM-AI MCP tools available -- `capture_block`, `list_session_blocks`, `delete_block`, `delete_link`, `unstage_block`, `discard_working_set` -- and session-level tracking of `created_blocks` and `modified_blocks`.

---

## Executive Summary

All SDD skills currently use a **create-at-the-end** artifact pattern. They retrieve input artifacts, do their work, then create exactly one output artifact. This works but misses opportunities to:

1. **Capture intermediate working artifacts** during phase execution (e.g., research notes, draft sections, rejected alternatives)
2. **Track session provenance** -- what was created during this session vs. what existed before
3. **Support staged creation** -- working set that can be committed or discarded
4. **Clean up failed attempts** without leaving orphan blocks

The new MCP tools enable a **working set + staged commit** model that gives phases more flexibility in how they build and refine artifacts.

---

## Current Artifact Storage Pattern (All Phases)

### What's Common Across All 9 Skills

| Aspect | Current Behavior |
|--------|-----------------|
| **Artifact creation** | Single `create_block` call at phase end |
| **Intermediate work** | Discarded or embedded in final block |
| **Session tracking** | None -- no `created_blocks` or `modified_blocks` awareness |
| **Staging** | None -- all blocks are immediately committed |
| **Cleanup** | No mechanism -- failed attempts leave orphan blocks |
| **Link cleanup** | No mechanism -- broken links persist |

### Current Tool Subset (All Skills Use Only)

- `search_blocks` -- find input artifacts
- `get_block` -- retrieve artifact content
- `create_block` -- create output artifact
- `create_link` -- link artifacts
- `update_block` -- update tasks (sdd-apply only)

---

## Phase-by-Phase Analysis

---

### 1. sdd-init

**Current artifact storage**: Creates 3 blocks (project, tracker, discovery) in sequence via separate `create_block` calls. No staging.

**Current flow**:
```
validate name → create_block(project) → create_block(tracker) → create_block(discovery) → create_link(project→tracker) → return
```

**Issues**:
- All 3 blocks are immediately committed -- no ability to discard if name validation fails mid-way
- No session tracking -- the orchestrator doesn't know these 3 blocks were created together
- No draft phase -- if discovery content is incomplete, it stays as a permanent block

**Recommended improvements**:
1. Use `capture_block` for each artifact during creation, staging them in the working set
2. Use `list_session_blocks` after creation to verify all 3 were captured before committing
3. If validation fails, use `discard_working_set` to clean up all 3 instead of leaving orphan blocks
4. Add `created_blocks` to session metadata so orchestrator knows the ULIDs immediately

**Specific changes to SKILL.md**:
- Step 2-4 (Create blocks): Add `capture_block` calls instead of direct `create_block`
- Add new Step: Verify session blocks with `list_session_blocks`
- Add Step: Commit working set (implicit via capture success)
- If any validation fails before commit: call `discard_working_set`
- Update Return Envelope to include `session_created_blocks` array

---

### 2. sdd-explore

**Current artifact storage**: Creates single `permanent` exploration block at end. Intermediate research notes are not captured.

**Current flow**:
```
load conventions → web search research → create_block(explore) → return
```

**Issues**:
- All intermediate research (search queries, findings, rejected directions) is discarded
- Only the final polished exploration block is saved
- No ability to iterate on exploration -- if user wants to add more research, entire block must be replaced
- Single shot -- `update_block` can modify but there's no version history

**Recommended improvements**:
1. Use `capture_block` for each research finding as it is gathered (staged)
2. Use `capture_block` for rejected alternatives (documenting why they were rejected)
3. Use `list_session_blocks` before final commit to review what was captured
4. Use `delete_block` to remove low-quality captures before final commit
5. Only the final consolidated exploration block gets `create_block` (committed)

**Specific changes to SKILL.md**:
- Add new Step: "Capture Working Findings" -- for each research result, call `capture_block` with tags `["sdd", "explore", "sdd-{change}", "working"]`
- Add new Step: "Review Session Blocks" -- call `list_session_blocks` to see all captured findings
- Add new Step: "Prune Low-Quality Captures" -- use `delete_block` on captures that don't meet bar
- Step 5 (Create Exploration Block): Consolidate captured findings into final block
- Add `session_created_blocks` to metadata in final block content

---

### 3. sdd-propose

**Current artifact storage**: Creates single `permanent` proposal block. Links to exploration if provided.

**Current flow**:
```
load conventions → check existing proposal → get_block(explore, if provided) → create_block(proposal) → create_link(explore→proposal) → return
```

**Issues**:
- No intermediate capture of problem statement drafts, solution alternatives considered, etc.
- If explore-ulid is provided but content is inadequate, no way to flag that the proposal is built on weak foundations
- Proposal is created in one shot -- no iterative refinement session

**Recommended improvements**:
1. Capture draft problem statements during analysis with `capture_block` (staged)
2. Capture considered alternatives (even rejected ones) as staged blocks
3. Use `list_session_blocks` before final commit
4. If proposal is an update to existing, use `delete_block` on old version first (or mark as superseded via link)

**Specific changes to SKILL.md**:
- Add new Step: "Capture Problem Statement Draft" -- staged capture of initial problem statement
- Add new Step: "Capture Considered Alternatives" -- staged captures for each alternative evaluated (with disposition)
- Add new Step: "Review Session Blocks" -- verify captures before committing final proposal
- If updating existing: use `delete_block` on old proposal (after creating new one) to avoid orphans
- Update link creation to also link to superseded proposal if replacing

---

### 4. sdd-spec

**Current artifact storage**: Creates single `structure` block with Given/When/Then scenarios. Requires proposal-ulid.

**Current flow**:
```
load conventions → get_block(proposal) → create_block(spec) → create_link(proposal→spec) → return
```

**Issues**:
- All scenarios are created in one shot -- no intermediate capture of individual scenario drafts
- No mechanism to validate scenarios against each other for consistency
- If spec phase is re-run (due to spec changes), no easy way to mark old spec as superseded

**Recommended improvements**:
1. Capture each functional requirement's scenarios as staged blocks during drafting
2. Use `capture_block` for each FR's scenarios -- enables review before final assembly
3. Use `list_session_blocks` to verify all FRs have scenarios before committing
4. If re-spec'ing: capture old spec's ULID in new spec's metadata for traceability

**Specific changes to SKILL.md**:
- Add new Step: "Capture FR Scenarios" -- for each functional requirement, capture its scenarios as staged blocks
- Add new Step: "Verify Scenario Completeness" -- use `list_session_blocks` to ensure all scenarios are captured
- Add new Step: "Prune Incomplete Scenarios" -- delete blocks for FRs that couldn't be fully specified
- Final Step 3 (Create Spec Block): Consolidate staged scenarios into final spec
- Add `predecessor_spec` to metadata if this replaces an existing spec

---

### 5. sdd-design

**Current artifact storage**: Creates single `permanent` block with AD-1, AD-2 format. Requires proposal-ulid.

**Current flow**:
```
load conventions → get_block(proposal) → create_block(design) → create_link(proposal→design) → return
```

**Issues**:
- Design decisions are captured in final block only
- Rejected alternatives mentioned in rationale but not captured as separate blocks
- No intermediate working space for design iteration

**Recommended improvements**:
1. Capture each architecture decision as a staged block (AD-1, AD-2, etc.) during design
2. Capture rejected alternatives as separate staged blocks with disposition
3. Use `list_session_blocks` to review all decisions before final commit
4. If design replaces existing, mark old as superseded in metadata

**Specific changes to SKILL.md**:
- Add new Step: "Capture Architecture Decisions" -- for each AD-N, use `capture_block` with `["sdd", "design", "sdd-{change}", "working", "ad-{n}"]`
- Add new Step: "Capture Rejected Alternatives" -- staged blocks for alternatives rejected during design
- Add new Step: "Review Session Blocks" -- verify all ADs captured
- Consolidate into final design block

---

### 6. sdd-tasks

**Current artifact storage**: Creates single `outline` block with checklist. Requires spec-ulid and design-ulid.

**Current flow**:
```
load conventions → get_block(spec) → get_block(design) → analyze and break down → create_block(tasks) → create_link(design→tasks) → return
```

**Issues**:
- Tasks are derived from spec and design but there's no traceability captured as working blocks
- No intermediate capture of task drafts or groupings
- Phase 1, 2, 3 organization is decided in one shot

**Recommended improvements**:
1. Capture each task as a staged block during breakdown
2. Capture task dependencies as separate staged blocks
3. Use `list_session_blocks` to verify all tasks are captured
4. Use `delete_block` to remove tasks that don't meet definition of "atomic, testable unit"

**Specific changes to SKILL.md**:
- Add new Step: "Capture Individual Tasks" -- for each task ID (1.1, 2.1, etc.), use `capture_block` with `["sdd", "tasks", "sdd-{change}", "working", "task-{id}"]`
- Add new Step: "Capture Task Dependencies" -- staged blocks for dependencies between tasks
- Add new Step: "Review Session Blocks" -- verify all tasks captured before final assembly
- Consolidate into final tasks block

---

### 7. sdd-apply

**Current artifact storage**: Updates tasks block with `[x]` completion marks. Creates progress blocks. Requires tasks-ulid, spec-ulid, design-ulid.

**Current flow**:
```
load conventions → get_block(tasks) → get_block(spec) → get_block(design) → execute tasks → update_block(tasks with [x]) → create_block(progress) → create_link(tasks→progress) → return
```

**Issues**:
- Task updates are in-place via `update_block` -- no version history
- Progress is timestamped but tasks block is mutated -- no record of when each task was completed
- No intermediate capture of implementation notes or decisions made during apply

**Recommended improvements**:
1. When a task is completed, capture an implementation note block (staged) before marking task done
2. Instead of mutating tasks block via `update_block`, create a task-completion block for each completed task
3. Use `list_session_blocks` to track all task completions in this session
4. Progress block becomes a summary of session-created task-completion blocks
5. Use `delete_block` to remove staged captures that aren't worth keeping

**Specific changes to SKILL.md**:
- Add new Step: "Capture Task Implementation Notes" -- for each completed task, `capture_block` with implementation notes (staged)
- Add new Step: "Track Session Completions" -- use `list_session_blocks` to see all task completions this session
- Add new Step: "Prune Unnecessary Captures" -- use `delete_block` on low-value implementation notes
- Modify Step 5 (Update Tasks Block): Keep tasks block as source of truth for checkboxes, but also create task-completion blocks for history
- Progress block references `session_created_blocks` for this apply session

---

### 8. sdd-verify

**Current artifact storage**: Creates single `permanent` verification report. Requires spec-ulid and optionally tasks-ulid, proposal-ulid.

**Current flow**:
```
load conventions → get_block(spec) → get_block(tasks) → get_block(proposal) → verify each criterion → create_block(verify) → create_link(spec→verify) → create_link(tasks→verify) → return
```

**Issues**:
- Verification of each acceptance criterion is not captured as separate blocks
- Test results, evidence for each criterion is embedded in final block only
- No intermediate working space to explore failures before finalizing verification report

**Recommended improvements**:
1. Capture each acceptance criterion verification as a staged block during verification
2. For each criterion: capture test output, evidence, and determination as separate staged blocks
3. Use `list_session_blocks` to review all criterion verifications
4. Use `delete_block` to remove verifications that aren't convincing before final commit

**Specific changes to SKILL.md**:
- Add new Step: "Capture Criterion Verification" -- for each AC, use `capture_block` with `["sdd", "verify", "sdd-{change}", "working", "ac-{n}"]`
- Add new Step: "Review Session Blocks" -- verify all ACs have verifications before final report
- Add new Step: "Prune Weak Verifications" -- delete blocks for ACs that couldn't be verified convincingly
- Consolidate into final verify block

---

### 9. sdd-archive

**Current artifact storage**: Creates single `permanent` archive block linking to all phase artifacts. Requires all phase ULIDs.

**Current flow**:
```
load conventions → get_block(proposal) → ... → get_block(verify) → analyze → create_block(archive) → create_link(archive→each phase) → return
```

**Issues**:
- Lessons learned, reusable patterns are captured in final block only
- No intermediate capture of individual lessons or patterns
- Archive is meant to be immutable but there's no mechanism to prevent updates

**Recommended improvements**:
1. Capture each lesson learned as a staged block during analysis
2. Capture each reusable pattern as a staged block
3. Use `list_session_blocks` to review all captures before final archive
4. Final archive block becomes the committed snapshot -- any corrections create new archive blocks (not updates)
5. Use `delete_block` on staged captures that aren't worth including

**Specific changes to SKILL.md**:
- Add new Step: "Capture Lessons Learned" -- for each lesson, use `capture_block` with `["sdd", "archive", "sdd-{change}", "working", "lesson"]`
- Add new Step: "Capture Reusable Patterns" -- staged blocks for each pattern
- Add new Step: "Review Session Blocks" -- verify all captures before final archive
- Add new Step: "Prune Low-Value Captures" -- delete blocks not worth immortalizing
- Consolidate into final archive block
- Add `archive_type: immutable` to metadata -- future corrections create new archive blocks

---

## Cross-Cutting Recommendations

### 1. Add New PKM-AI Tools to All Skill Reference Sections

Each SKILL.md needs its PKM-AI Tool Reference section expanded to include the new tools:

```json
// NEW: Session working set tools
{ "tool": "capture_block", "args": { ... } }
{ "tool": "list_session_blocks", "args": { "session_id": "{session_id}" } }
{ "tool": "delete_block", "args": { "block_id": "{ulid}" } }
{ "tool": "delete_link", "args": { "source_id": "{ulid}", "target_id": "{ulid}" } }
{ "tool": "unstage_block", "args": { "block_id": "{ulid}" } }
{ "tool": "discard_working_set", "args": { "session_id": "{session_id}" } }
```

### 2. Introduce Session Metadata in All Artifact Content

Each final artifact block should include session tracking:

```json
{
  "change": "{change}",
  "phase": "{phase}",
  "session": {
    "created_blocks": ["{ulid1}", "{ulid2}"],
    "modified_blocks": ["{ulid3}"]
  },
  "created": "{ISO date}"
}
```

### 3. Working Block Tag Convention

Introduce a `working` tag for staged captures that are not yet part of the final artifact:

```json
"tags": ["sdd", "sdd-{phase}", "sdd-{change}", "working"]
```

Final committed blocks drop the `working` tag.

### 4. Block Type for Working Captures

Consider using `ghost` block type for staged/working captures, graduating to appropriate type (`permanent`, `structure`, `outline`) on commit.

### 5. Update PKM-AI Conventions Document

The `_shared/pkmai-convention.md` should be updated to document:

- New toolset availability
- Session-based artifact creation pattern
- Working set lifecycle (capture → review → commit OR discard)
- Tag conventions for working vs. committed blocks
- Block type graduation pattern

### 6. Update Persistence Contract

The `persistence-contract.md` should be updated to define:

- When to use `capture_block` vs. `create_block`
- Session boundary semantics
- Working set commit vs. discard protocol
- Cleanup requirements for failed phases

---

## Summary of Specific SKILL.md Changes

### sdd-init/SKILL.md
- Steps 2-4: Replace `create_block` with `capture_block` (staged)
- Add new Step: Verify session blocks with `list_session_blocks`
- Add Step: On failure, `discard_working_set` instead of leaving orphans
- Return envelope: include `session_created_blocks` array

### sdd-explore/SKILL.md
- Add new Step after "Research Topic": Capture each finding with `capture_block`
- Add new Step: "Review Session Blocks" via `list_session_blocks`
- Add new Step: "Prune Low-Quality Captures" via `delete_block`
- Final artifact is consolidated from session captures

### sdd-propose/SKILL.md
- Add new Step: Capture draft problem statement with `capture_block`
- Add new Step: Capture considered alternatives (rejected) with `capture_block`
- Add new Step: Review session blocks before final commit
- If updating existing: `delete_block` on old after creating new

### sdd-spec/SKILL.md
- Add new Step: Capture each FR's scenarios as staged blocks
- Add new Step: Verify scenario completeness via `list_session_blocks`
- Add new Step: Prune incomplete scenarios via `delete_block`
- Add `predecessor_spec` to metadata

### sdd-design/SKILL.md
- Add new Step: Capture each AD as staged block
- Add new Step: Capture rejected alternatives as staged blocks
- Add new Step: Review session blocks before final
- Consolidate into design block

### sdd-tasks/SKILL.md
- Add new Step: Capture individual tasks as staged blocks
- Add new Step: Capture task dependencies as staged blocks
- Add new Step: Review via `list_session_blocks`
- Consolidate into tasks block

### sdd-apply/SKILL.md
- Add new Step: Capture implementation notes per task completion
- Add new Step: Track session completions via `list_session_blocks`
- Add new Step: Prune unnecessary captures via `delete_block`
- Progress block references `session_created_blocks`

### sdd-verify/SKILL.md
- Add new Step: Capture each AC verification as staged block
- Add new Step: Review all verifications via `list_session_blocks`
- Add new Step: Prune weak verifications via `delete_block`
- Consolidate into verify block

### sdd-archive/SKILL.md
- Add new Step: Capture lessons learned as staged blocks
- Add new Step: Capture reusable patterns as staged blocks
- Add new Step: Review via `list_session_blocks`
- Add new Step: Prune low-value captures
- Add `archive_type: immutable` to metadata

---

## Tool Coverage Matrix

| Tool | sdd-init | sdd-explore | sdd-propose | sdd-spec | sdd-design | sdd-tasks | sdd-apply | sdd-verify | sdd-archive |
|------|----------|-------------|-------------|----------|------------|-----------|-----------|------------|-------------|
| `capture_block` | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| `list_session_blocks` | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| `delete_block` | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| `delete_link` | Yes | No | Yes | No | No | No | No | No | No |
| `unstage_block` | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| `discard_working_set` | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes | Yes |

---

## Next Steps

1. Update `_shared/pkmai-convention.md` with new tool documentation and session-based patterns
2. Update `_shared/persistence-contract.md` with working set lifecycle
3. Update each phase SKILL.md following the specific changes outlined above
4. Add `session_created_blocks` and `session_modified_blocks` to all return envelope metadata
5. Update orchestrator (CLAUDE.md/AGENTS.md) to handle new session-based return fields
