---
name: sdd-verify
description: >
  SDD Verify Phase - Verify implementation against specification.
  Trigger: When assigned as sdd-verify phase sub-agent.
license: MIT
metadata:
  author: pkm-ai
  version: "1.0"
---

## Purpose

Verify that the implementation meets the specification:
- Check each acceptance criterion
- Run tests to confirm behavior
- Identify gaps or deviations
- Report verification results

## What You Receive

- `change-name`: The name of the change
- `spec-ulid`: ULID of spec block
- `tasks-ulid`: ULID of tasks block
- `progress-ulids`: ULIDs of progress reports

## Execution

### Step 1: Load Skills

Load `workflows/pkmai/sdd/_shared/phase-common.md` for return format.

### Step 2: Retrieve Artifacts

```json
[
  {
    "tool": "get_block",
    "args": {
      "block_id": "{spec-ulid}",
      "include_content": true
    }
  },
  {
    "tool": "get_block",
    "args": {
      "block_id": "{tasks-ulid}",
      "include_content": true
    }
  }
]
```

### Step 3: Run Verification

For each acceptance criterion:
1. Check if criterion is met
2. Run relevant tests
3. Document result

### Step 4: Create Verify Block

```json
[
  {
    "tool": "create_block",
    "args": {
      "block_type": "permanent",
      "title": "sdd/{change-name}/verify",
      "content": "# Verification Report: {change-name}\n\n## Summary\n{Overview of verification results}\n\n## Verification Results\n\n### Acceptance Criteria\n| Criterion | Status | Evidence |\n|-----------|--------|----------|\n| {criterion 1} | ✅ Pass | {evidence} |\n| {criterion 2} | ❌ Fail | {evidence} |\n| {criterion 3} | ⚠️ Partial | {evidence} |\n\n### Test Results\n| Test Suite | Result | Coverage |\n|------------|--------|----------|\n| Unit Tests | ✅ Pass | {X%} |\n| Integration Tests | ✅ Pass | {X%} |\n| E2E Tests | ❌ Fail | {X%} |\n\n## Gaps Identified\n- {gap 1}\n- {gap 2}\n\n## Deviations from Spec\n- {deviation 1}\n- {deviation 2}\n\n## Recommendations\n- {recommendation 1}\n- {recommendation 2}\n\n## Metadata\n```json\n{\n  \"change\": \"{change-name}\",\n  \"phase\": \"verify\",\n  \"spec\": \"{spec-ulid}\",\n  \"passed\": {N},\n  \"failed\": {M},\n  \"created\": \"{ISO date}\"\n}\n```",
      "tags": ["sdd", "verify", "sdd-verify", "sdd-{change-name}"]
    }
  }
]
```

### Step 5: Link to Spec and Tasks

```json
[
  {
    "tool": "create_link",
    "args": {
      "source_id": "{spec-ulid}",
      "target_id": "{verify-ulid}",
      "link_type": "supports"
    }
  },
  {
    "tool": "create_link",
    "args": {
      "source_id": "{tasks-ulid}",
      "target_id": "{verify-ulid}",
      "link_type": "related"
    }
  }
]
```

### Step 6: Return Envelope

Return structured summary per phase-common.md format.

## Verify Template

```markdown
# Verification Report: {change-name}

**Date**: {ISO date}
**Verifier**: {agent or user}

## Executive Summary
{2-3 sentences on overall verification status}

## Verification Criteria Results

### Must Have (MVP)

| # | Criterion | Source | Status | Evidence |
|---|-----------|--------|--------|----------|
| 1 | {criterion} | FR-1 | ✅ Pass | {test/inspection} |
| 2 | {criterion} | FR-1 | ❌ Fail | {test/inspection} |
| 3 | {criterion} | FR-2 | ⚠️ Partial | {test/inspection} |

### Should Have

| # | Criterion | Source | Status | Evidence |
|---|-----------|--------|--------|----------|
| 1 | {criterion} | FR-3 | ✅ Pass | {test/inspection} |
| 2 | {criterion} | FR-3 | ⏳ Pending | {reason} |

## Test Coverage

### Unit Tests
- **Status**: ✅ Pass / ❌ Fail
- **Coverage**: {X%}
- **Tests**: {N} passed, {M} failed

### Integration Tests
- **Status**: ✅ Pass / ❌ Fail
- **Coverage**: {X%}
- **Tests**: {N} passed, {M} failed

### E2E Tests
- **Status**: ✅ Pass / ❌ Fail
- **Coverage**: {X%}
- **Tests**: {N} passed, {M} failed

## Scenario Verification

### Scenario 1.1: {Scenario Name}
- **Given**: {precondition}
- **When**: {action}
- **Then**: {expected result}
- **Status**: ✅ Verified / ❌ Failed / ⚠️ Partial
- **Evidence**: {how verified}

### Scenario 1.2: {Scenario Name}
...

## Gaps Identified

| Gap | Severity | Impact | Recommendation |
|-----|----------|--------|----------------|
| {gap} | High/Med/Low | {impact} | {recommendation} |

## Deviations from Specification

| Deviation | Spec Section | Impact | Resolution |
|-----------|--------------|--------|------------|
| {deviation} | FR-X | {impact} | {resolution or "Requires spec update"} |

## Security Verification
- [ ] {Security check 1}
- [ ] {Security check 2}

## Performance Verification
- [ ] {Performance check 1}
- [ ] {Performance check 2}

## Overall Status

| Category | Status |
|----------|--------|
| Functional Requirements | ✅ Complete / ⚠️ Partial / ❌ Incomplete |
| Non-Functional Requirements | ✅ Complete / ⚠️ Partial / ❌ Incomplete |
| Tests | ✅ All Pass / ⚠️ Some Fail / ❌ Many Fail |
| Security | ✅ Verified / ⚠️ Issues Found / ❌ Not Verified |

## Sign-off

| Role | Name | Date | Status |
|------|------|------|--------|
| Developer | {name} | {date} | ✅ Approved |
| Reviewer | {name} | {date} | ⏳ Pending |

## Metadata
```json
{
  "change": "{change-name}",
  "phase": "verify",
  "spec": "{spec-ulid}",
  "tasks": "{tasks-ulid}",
  "criteria_passed": {N},
  "criteria_failed": {M},
  "test_coverage": "{X%}",
  "created": "{ISO date}"
}
```
```

## Rules

- Verify against spec, not assumptions
- Be objective — report what is vs. what should be
- Include evidence for each check
- Distinguish between gaps and deviations
- Make clear recommendations
- Link to spec as source of truth
- Return structured envelope with all required fields

## Output

Creates: `sdd/{change-name}/verify` block
Links: → Spec (supports), → Tasks (related)
