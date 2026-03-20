---
name: sdd-verify
description: >
  SDD Verification Phase - Verify implementation against specification.
  Uses PKM-AI block storage with block_type="permanent".
  Trigger: When assigned as sdd-verify phase sub-agent.
license: MIT
metadata:
  author: pkm-ai
  version: "1.0"
---

## Purpose

Verify that the implementation satisfies all acceptance criteria defined in the specification:
- Check each acceptance criterion against actual implementation
- Run tests and validate results
- Identify gaps, deviations, or missing functionality
- Document findings and any necessary follow-up work

## PKM-AI Storage

| Field | Value |
|-------|-------|
| Block Type | `permanent` |
| Title Format | `sdd/{change}/verify` |
| Tags | `["sdd", "verify", "sdd-verify", "sdd-{change}"]` |
| Link Type | `supports` (to spec), `related` (to tasks) |

## What You Receive

- `change`: The name of the change (e.g., `mcp-workflow`)
- `spec-ulid`: ULID of the spec artifact (required)
- `tasks-ulid`: ULID of the tasks artifact (optional)
- `proposal-ulid`: ULID of the proposal artifact (optional)

## Execution

### Step 1: Load Shared Conventions

Load `${PKM_AI_SHARED:-~/.pkm-ai/sdd/_shared}/sdd-phase-common.md` for the return envelope format.
Load `${PKM_AI_SHARED:-~/.pkm-ai/sdd/_shared}/pkmai-convention.md` for PKM-AI conventions.

### Step 2: Retrieve Required Artifacts

```json
{
  "tool": "get_block",
  "arguments": {"block_id": "{spec-ulid}", "include_content": true}
}
```

```json
{
  "tool": "get_block",
  "arguments": {"block_id": "{tasks-ulid}", "include_content": true}
}
```

```json
{
  "tool": "get_block",
  "arguments": {"block_id": "{proposal-ulid}", "include_content": true}
}
```

### Step 3: Analyze Spec Acceptance Criteria

Extract all acceptance criteria from the spec block and create a verification checklist.

### Step 4: Verify Each Criterion

For each acceptance criterion:
1. Identify the related task(s) from the tasks block
2. Check if the implementation satisfies the criterion
3. Run any relevant tests
4. Document the verification result (pass/fail/partial)

### Step 5: Create Verify Block

```json
{
  "tool": "create_block",
  "arguments": {"block_type": "permanent", "title": "sdd/{change}/verify", "content": "# Verification Report: {change}\n\n## Summary\n\n{2-3 sentence executive summary of verification results}\n\n## Verification Results\n\n### Overall Status: {PASS | FAIL | PARTIAL}\n\n| Criterion | Status | Evidence | Notes |\n|-----------|--------|----------|-------|\n| {AC-1} | PASS/FAIL/PARTIAL | {test output, file path} | {notes} |\n| {AC-2} | PASS/FAIL/PARTIAL | {evidence} | {notes} |\n\n## Criteria Verification Details\n\n### AC-1: {Acceptance Criterion Name}\n**Status**: {PASS | FAIL | PARTIAL}\n\n**Requirement**: {Full text of the acceptance criterion}\n\n**Verification Method**: {How this was verified - test, inspection, etc.}\n\n**Evidence**:\n```\n{evidence or test output}\n```\n\n**Findings**: {Detailed findings}\n\n### AC-2: {Next Criterion}\n...\n\n## Test Results\n\n### Unit Tests\n| Test Suite | Passed | Failed | Skipped |\n|------------|--------|--------|---------|\n| {suite} | {n} | {n} | {n} |\n\n### Integration Tests\n| Test Suite | Passed | Failed | Skipped |\n|------------|--------|--------|---------|\n| {suite} | {n} | {n} | {n} |\n\n### E2E Tests\n| Test Suite | Passed | Failed | Skipped |\n|------------|--------|--------|---------|\n| {suite} | {n} | {n} | {n} |\n\n## Gaps and Deviations\n\n### Critical Gaps\n| Gap | Severity | Description | Recommended Action |\n|-----|----------|-------------|-------------------|\n| {gap} | High/Med/Low | {description} | {action} |\n\n### Minor Deviations\n| Deviation | Description | Impact | Resolution |\n|-----------|-------------|--------|------------|\n| {deviation} | {description} | {impact} | {resolution} |\n\n## Recommendations\n\n### Must Fix (Blockers)\n- {item 1}\n- {item 2}\n\n### Should Fix\n- {item 1}\n- {item 2}\n\n### Could Improve\n- {item 1}\n- {item 2}\n\n## Metadata\n```json\n{\n  \"change\": \"{change}\",\n  \"phase\": \"verify\",\n  \"spec_ulid\": \"{spec-ulid}\",\n  \"tasks_ulid\": \"{tasks-ulid}\",\n  \"overall_status\": \"{PASS|FAIL|PARTIAL}\",\n  \"criteria_passed\": {n},\n  \"criteria_failed\": {n},\n  \"criteria_partial\": {n},\n  \"tests_passed\": {n},\n  \"tests_failed\": {n},\n  \"created\": \"{ISO date}\"\n}\n```\n", "tags": ["sdd", "verify", "sdd-verify", "sdd-{change}"]}
}
```

### Step 6: Create Links

```json
{
  "tool": "create_link",
  "arguments": {"source_id": "{spec-ulid}", "target_id": "{verify-ulid}", "link_type": "supports"}
}
```

```json
{
  "tool": "create_link",
  "arguments": {"source_id": "{tasks-ulid}", "target_id": "{verify-ulid}", "link_type": "related"}
}
```

### Step 7: Return Envelope

Return structured summary per `phase-common.md` format.

## Verification Template

```markdown
# Verification Report: {change}

## Executive Summary

{2-3 sentences describing overall verification results}

## Verification Matrix

### Overall Status
- **Result**: PASS | FAIL | PARTIAL
- **Criteria Passed**: {n}/{total}
- **Tests Passed**: {n}/{total}

## Acceptance Criteria Verification

| ID | Criterion | Status | Evidence |
|----|-----------|--------|----------|
| AC-1 | {criterion text} | PASS/FAIL/PARTIAL | {evidence} |
| AC-2 | {criterion text} | PASS/FAIL/PARTIAL | {evidence} |

## Detailed Findings

### AC-1: {Criterion Name}
**Status**: PASS | FAIL | PARTIAL

**Verification Method**: {test, inspection, manual check}

**Evidence**:
```
{test output or file reference}
```

**Analysis**: {why it passes/fails}

## Test Execution

### Command
```bash
{test command}
```

### Results
```
{test output}
```

## Gap Analysis

### Missing Functionality
| Item | Severity | Description |
|------|----------|-------------|
| {item} | Critical/High/Med/Low | {desc} |

### Deviation from Spec
| Item | Impact | Description |
|------|--------|-------------|
| {item} | High/Med/Low | {desc} |

## Recommendations

### Immediate Actions Required
1. {action 1}
2. {action 2}

### Technical Debt
- {debt 1}
- {debt 2}

## Metadata
```json
{
  "change": "{change}",
  "phase": "verify",
  "spec_ulid": "{spec-ulid}",
  "tasks_ulid": "{tasks-ulid}",
  "overall_status": "{PASS|FAIL|PARTIAL}",
  "criteria": {
    "total": {n},
    "passed": {n},
    "failed": {n},
    "partial": {n}
  },
  "tests": {
    "passed": {n},
    "failed": {n},
    "skipped": {n}
  },
  "created": "{ISO date}"
}
```
```

## Rules

- Use `block_type="permanent"` for verify blocks
- Be objective and factual in verification results
- Provide concrete evidence for each criterion status
- Distinguish between "not implemented" and "incorrectly implemented"
- Link to spec as the source of truth
- Run actual tests when possible, not just static analysis
- Return structured envelope with all required fields

## Output

Creates: `sdd/{change}/verify` block (type: permanent)
Links: Spec → Verify (supports), Tasks → Verify (related)

## Block Type Note

Verify uses `permanent` block type because verification reports are atomic assessment artifacts. The report represents a point-in-time evaluation that should not be modified after creation. Any re-verification should create a new verify block rather than updating an existing one.
