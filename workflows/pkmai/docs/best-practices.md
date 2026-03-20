# PKM-AI Best Practices Guide

## Overview

This guide consolidates best practices for efficient PKM-AI usage, combining the intelligent capture workflow with session management patterns for AI agents.

## Core Philosophy: Git-like Workflow

PKM-AI follows Git's philosophy: **nothing is committed until truly ready**. This allows AI agents to make informed decisions about knowledge graph structure before finalizing changes.

```
┌─────────────────────────────────────────────────────────────┐
│                    Knowledge Lifecycle                       │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│   ┌──────────┐    ┌──────────┐    ┌──────────┐             │
│   │  Block   │───▶│  Stage   │───▶│  Commit  │             │
│   │ Created  │    │ (Working │    │ (Snap-   │             │
│   │          │    │   Set)   │    │  shot)   │             │
│   └──────────┘    └──────────┘    └──────────┘             │
│        │               │               │                      │
│        ▼               ▼               ▼                      │
│   ┌──────────────────────────────────────────┐              │
│   │  AI Analysis: Link suggestions, Gravity,   │              │
│   │  Type detection, Tag recommendations       │              │
│   └──────────────────────────────────────────┘              │
│                                                              │
│  Key Principle: Nothing auto-stages. AI decides when ready.  │
└─────────────────────────────────────────────────────────────┘
```

## Intelligent Capture Workflow

### Step 1: Create Block with Enrichment

When creating a block, request enriched data for intelligent decisions:

```json
{
  "tool": "create_block",
  "arguments": {
    "block_type": "fleeting",
    "title": "Attention is all you need",
    "content": "The Transformer architecture...",
    "enrich": true
  }
}
```

**Response with enrichment:**

```json
{
  "id": "01HX5V8F3K7QV9BZEC4N6P0M",
  "block_type": "fleeting",
  "title": "Attention is all you need",
  "created_at": "2026-03-20T10:30:00Z",
  "enrichment": {
    "link_suggestions": [
      {
        "target_id": "01HX5V8F3K7QV9BZEC4N6P0A",
        "link_type": "extends",
        "confidence": 0.85,
        "reason": "Content extends discussion on attention mechanisms"
      }
    ],
    "tag_suggestions": ["transformers", "nlp", "deep-learning"],
    "gravity_info": {
      "gravity_score": 3.2,
      "outgoing_links": 5,
      "incoming_links": 2
    },
    "type_suggestion": {
      "suggested_type": "literature",
      "confidence": 0.72,
      "reason": "Content has reference-like structure"
    }
  }
}
```

### Step 2: AI Interprets Suggestions

The AI analyzes enrichment data to decide:

| Decision | Data Source | Action |
|----------|-------------|--------|
| **Link creation** | `link_suggestions` | Create links with `create_link` |
| **Tag assignment** | `tag_suggestions` | Update block with suggested tags |
| **Type promotion** | `type_suggestion` | Consider promoting `fleeting` → `literature` |
| **Gravity check** | `gravity_info` | Identify hub topics with many connections |

### Step 3: Stage When Ready

Once the AI has processed the block:

```json
{
  "tool": "stage_block",
  "arguments": {
    "block_id": "01HX5V8F3K7QV9BZEC4N6P0M"
  }
}
```

### Step 4: Commit When Truly Ready

```json
{
  "tool": "commit_changes",
  "arguments": {
    "message": "Add Transformer paper notes with neural network links",
    "author": "ai-agent"
  }
}
```

### Step 5: Check Working Set Status

```json
{
  "tool": "get_working_set_status",
  "arguments": {}
}
```

## Session Lifecycle Management

### Starting a Session

Begin each agent session with proper initialization:

```json
{
  "tool": "create_block",
  "arguments": {
    "block_type": "structure",
    "title": "session/{project}/{date}",
    "content": "# Agent Session\n\n**Started**: {timestamp}\n**Project**: {project}\n**Goals**: \n\n",
    "tags": ["session", "active", "sdd-{change}"]
  }
}
```

### Session Lifecycle Hooks

#### Start Hook

Execute at session start:

```bash
#!/bin/bash
export PKM_SESSION_ID="$(uuidgen)"
pkmai session start \
  --agent "claude-code" \
  --project "$(basename $(pwd))" \
  --session-id "$PKM_SESSION_ID" \
  --cwd "$(pwd)"
```

#### End Hook

Execute at session end:

```bash
#!/bin/bash
pkmai session end \
  --session-id "$PKM_SESSION_ID" \
  --auto-summary true
```

## When to Save to PKM

### Save When: Architectural Decisions

Create `permanent` blocks for design choices:

```json
{
  "tool": "create_block",
  "arguments": {
    "block_type": "permanent",
    "title": "DECISION: Use async/await for I/O",
    "content": "**Decision**: Use async/await for all I/O operations\n\n**Rationale**: \n- Cleaner code than manual futures\n- Better error propagation\n- Native Rust pattern\n\n**Alternatives Considered**:\n- Tokio spawn - more control but complex\n- Callback-based - callback hell",
    "tags": ["decision", "architecture", "rust", "async"]
  }
}
```

### Save When: Bug Fixes

Create `permanent` blocks for bug fixes with root cause analysis:

```json
{
  "tool": "create_block",
  "arguments": {
    "block_type": "permanent",
    "title": "FIX: Race condition in cache layer",
    "content": "**Problem**: Users saw stale data after concurrent updates\n\n**Root Cause**: Cache invalidation happened before DB write completion\n\n**Solution**: Invalidate cache AFTER successful DB write, use transaction\n\n**Files**: \n- `src/cache/mod.rs`\n- `src/db/transactions.rs`\n\n**Test Added**: `tests/cache_race.rs`",
    "tags": ["bugfix", "cache", "concurrency", "fixed"]
  }
}
```

### Save When: Patterns Discovered

Create `permanent` blocks for reusable patterns:

```json
{
  "tool": "create_block",
  "arguments": {
    "block_type": "permanent",
    "title": "PATTERN: Repository with Unit of Work",
    "content": "**Pattern**: Repository + Unit of Work\n\n**Context**: Data access with transactional semantics\n\n**Implementation**:\n```rust\npub struct UnitOfWork<'ctx> {\n    db: &'ctx Database,\n    users: UserRepository<'ctx>,\n    posts: PostRepository<'ctx>,\n}\n\nimpl<'ctx> UnitOfWork<'ctx> {\n    pub async fn commit(self) -> Result<()> {\n        // Commit all changes atomically\n    }\n}\n```\n\n**When to Use**: Complex aggregates requiring atomic updates",
    "tags": ["pattern", "repository", "ddd", "rust"]
  }
}
```

## When to Search PKM

### Search Before: Starting New Feature

```json
{
  "tool": "search_blocks",
  "arguments": {
    "query": "authentication similar",
    "block_type": "permanent",
    "limit": 10
  }
}
```

### Search Before: Debugging

```json
{
  "tool": "search_blocks",
  "arguments": {
    "query": "token expiry",
    "tags": ["bugfix", "auth"],
    "limit": 5
  }
}
```

### Search Before: Refactoring

```json
{
  "tool": "search_blocks",
  "arguments": {
    "query": "repository pattern",
    "tags": ["pattern", "architecture"],
    "limit": 10
  }
}
```

## Block Types for Long-Term Survival

| Type | Purpose | TTL | Use Case |
|------|---------|-----|----------|
| `permanent` | Evergreen knowledge | Infinite | Decisions, patterns, fixes |
| `structure` | Project architecture | Infinite | Project context, specs |
| `literature` | External references | Infinite | Papers, articles, docs |
| `fleeting` | Temporary notes | 7 days | Quick captures |
| `task` | Action items | Until done | TODOs, action items |

## Enrichment Data Reference

### Link Suggestions

```json
{
  "target_id": "01HX...",
  "link_type": "extends|refines|contradicts|questions|supports|references|related|similar_to|section_of|next",
  "confidence": 0.0-1.0,
  "reason": "Human-readable explanation"
}
```

### Gravity Info

```json
{
  "gravity_score": 0.0-10.0,
  "outgoing_links": number,
  "incoming_links": number
}
```

Higher gravity = more connected hub topic.

### Type Suggestion

```json
{
  "suggested_type": "literature|permanent|...",
  "confidence": 0.0-1.0,
  "reason": "Why this type is suggested"
}
```

## Compaction Survival Guide

When context window compacts, memories in PKM survive:

| Context Type | Survives? | Storage |
|--------------|-----------|---------|
| Chat history | No | PKM (if saved) |
| Working memory | No | PKM (if saved) |
| Architectural decisions | **Yes** | PKM permanent |
| Bug fixes & rationale | **Yes** | PKM permanent |
| Code patterns | **Yes** | PKM permanent |
| Project conventions | **Yes** | PKM structure |

### Recovery Protocol

1. **Before compaction**:
```json
{
  "tool": "create_block",
  "arguments": {
    "block_type": "structure",
    "title": "checkpoint/{session-id}",
    "content": "# Session Checkpoint\n\n**Last State**: \n- Working on: {current_task}\n- Next step: {next_action}",
    "tags": ["checkpoint", "session"]
  }
}
```

2. **After compaction**:
```json
{
  "tool": "search_blocks",
  "arguments": {
    "query": "checkpoint/{session-id}",
    "tags": ["checkpoint"],
    "limit": 1
  }
}
```

## Design Principles

1. **Explicit over Implicit**: Nothing happens automatically
2. **Enrichment on Demand**: AI chooses when to get context
3. **Atomic Commits**: Changes are staged until truly ready
4. **Full Context**: AI gets link suggestions, gravity, type hints before committing
5. **Reversible**: Working set allows discarding before commit
6. **Persistent Memory**: Save decisions, patterns, fixes for session survival

## See Also

- [concepts.md](concepts.md) — SDD phases and artifacts
- [architecture.md](architecture.md) — Workflow architecture
- [installation.md](installation.md) — Setup guide
- [persistence.md](persistence.md) — Storage modes
- [token-economics.md](token-economics.md) — Efficiency analysis