# Session Lifecycle and Workflow

**Version:** 1.0.0
**Date:** 2026-03-20
**Project:** hodei-pkm
**Status:** Active

---

## Table of Contents

1. [Overview](#1-overview)
2. [Session Lifecycle](#2-session-lifecycle)
3. [Git-like Workflow](#3-git-like-workflow)
4. [Zettelkasten Workflow](#4-zettelkasten-workflow)
5. [AI-Assisted Workflow](#5-ai-assisted-workflow)
6. [Block Type Transitions](#6-block-type-transitions)
7. [ASCII Diagrams](#7-ascii-diagrams)
8. [Best Practices](#8-best-practices)
9. [Examples](#9-examples)

---

## 1. Overview

This document describes the session lifecycle and workflows in hodei-pkm, a personal knowledge management system with Git-like versioning semantics. The system combines Zettelkasten methodology with AI-assisted knowledge graph management.

### 1.1 Core Concepts

| Concept | Description |
|---------|-------------|
| **Block** | Atomic unit of knowledge (like a Git blob) |
| **WorkingSet** | Staging area for pending changes (like Git index) |
| **Commit** | Immutable snapshot of knowledge state |
| **View** | Named pointer to commits (branch/tag) |
| **Structure** | Ordered collection of blocks (like Git tree) |

### 1.2 Design Principles

```
┌─────────────────────────────────────────────────────────────────┐
│                    DESIGN PRINCIPLES                            │
├─────────────────────────────────────────────────────────────────┤
│  1. Everything is a Commit    - Atomic knowledge snapshots      │
│  2. Branches are Views       - Named pointers to commits       │
│  3. Index is WorkingSet      - Staging area for changes        │
│  4. Distributed is Normal    - Each agent has local state      │
│  5. ULID for Identification  - Time-ordered unique identifiers  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. Session Lifecycle

A **Session** represents a continuous period of knowledge work. It tracks the evolution of ideas from capture through refinement to permanent storage.

### 2.1 Session States

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   ACTIVE    │───▶│  STAGING    │───▶│  COMMITTED  │───▶│  ARCHIVED   │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
      │                  │                  │                  │
      │                  │                  │                  │
      ▼                  ▼                  ▼                  ▼
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│  Creating   │    │  Reviewing  │    │  Snapshot   │    │  Historical │
│  blocks     │    │  changes    │    │  saved      │    │  reference  │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
```

### 2.2 Session Lifecycle Flow

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                         SESSION LIFECYCLE                                    │
└──────────────────────────────────────────────────────────────────────────────┘

    ┌─────────────────────────────────────────────────────────────────────┐
    │                         NEW SESSION                                 │
    │  ┌───────────────────────────────────────────────────────────────┐  │
    │  │  1. Create WorkingSet (staging area)                         │  │
    │  │  2. Load WorkingSet ID (ULID-based)                          │  │
    │  │  3. Initialize operation log                                 │  │
    │  └───────────────────────────────────────────────────────────────┘  │
    └─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
    ┌─────────────────────────────────────────────────────────────────────┐
    │                         WORK IN PROGRESS                             │
    │  ┌───────────────────────────────────────────────────────────────┐  │
    │  │  • Create/edit/delete blocks                                  │  │
    │  │  • Each change generates a Delta (BlockDelta/EdgeDelta)       │  │
    │  │  • Deltas staged to WorkingSet                               │  │
    │  │  • Operation log records all changes                          │  │
    │  └───────────────────────────────────────────────────────────────┘  │
    └─────────────────────────────────────────────────────────────────────┘
                                    │
                    ┌───────────────┴───────────────┐
                    │                               │
                    ▼                               ▼
    ┌───────────────────────────┐    ┌───────────────────────────────┐
    │     STASH (optional)      │    │         COMMIT                 │
    │  ┌─────────────────────┐  │    │  ┌─────────────────────────┐  │
    │  │  save_state()       │  │    │  │  create_commit()       │  │
    │  │  - Pause session    │  │    │  │  - Snapshot structure  │  │
    │  │  - Keep WorkingSet  │  │    │  │  - Record author       │  │
    │  │  - Can resume later  │  │    │  │  - Add commit message  │  │
    │  └─────────────────────┘  │    │  │  - Clear WorkingSet    │  │
    └───────────────────────────┘    │  └─────────────────────────┘  │
                                     └───────────────────────────────┘
                                                        │
                                                        ▼
                                     ┌───────────────────────────────┐
                                     │      UPDATE VIEW (HEAD)        │
                                     │  ┌─────────────────────────┐  │
                                     │  │ View now points to      │  │
                                     │  │ new commit             │  │
                                     │  └─────────────────────────┘  │
                                     └───────────────────────────────┘
```

### 2.3 Session Data Structures

#### WorkingSet (Staging Area)

```rust
pub struct WorkingSet {
    pub id: WorkingSetId,              // Unique ULID-based identifier
    pub author: AgentId,               // Agent owning this working set
    pub staged_blocks: BTreeMap<Ulid, BlockDelta>,   // Pending block changes
    pub staged_edges: BTreeMap<(Ulid, Ulid), EdgeDelta>, // Pending edge changes
    pub removed_blocks: Vec<Ulid>,      // Blocks marked for deletion
    pub removed_edges: Vec<(Ulid, Ulid)>, // Edges marked for deletion
    pub operations: Vec<Operation>,     // Operation log for replay/undo
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

#### Operation Log Entry

```rust
pub struct Operation {
    pub id: Ulid,                       // Unique operation identifier
    pub delta: OperationDelta,         // The delta applied
    pub timestamp: DateTime<Utc>,       // When recorded
}

pub enum OperationDelta {
    Block(BlockDelta),
    Edge(EdgeDelta),
}
```

---

## 3. Git-like Workflow

The PKM follows a Git-inspired workflow for knowledge management.

### 3.1 State Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         KNOWLEDGE STATES                                    │
└─────────────────────────────────────────────────────────────────────────────┘

                         ┌──────────────────┐
                         │      DRAFT       │
                         │  (memory/ephemeral)│
                         └────────┬─────────┘
                                  │ create_block()
                                  │ with_content()
                                  ▼
                         ┌──────────────────┐
          ┌──────────────│     STAGED       │──────────────┐
          │              │  (WorkingSet)    │              │
          │              └────────┬─────────┘              │
          │                       │ stage()                │
          │                       ▼                        │
          │              ┌──────────────────┐               │
          │              │    COMMITTED     │               │
          │              │  (Repository)    │               │
          │              └────────┬─────────┘               │
          │                       │                         │
          │    ┌────────────────────┼────────────────────┐   │
          │    │                    │                    │   │
          ▼    ▼                    ▼                    ▼   ▼
   ┌──────────────┐      ┌──────────────┐      ┌──────────────┐
   │   DISCARDED  │      │    MERGED    │      │   BRANCHED   │
   │  (unstage)   │      │  (synthesis) │      │   (view)     │
   └──────────────┘      └──────────────┘      └──────────────┘
```

### 3.2 Workflow Commands

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         EQUIVALENT COMMANDS                                 │
├──────────────────────┬──────────────────────────────────────────────────────┤
│       Git           │                    PKM                                 │
├──────────────────────┼──────────────────────────────────────────────────────┤
│  git init           │  pkm init                                             │
│  git status         │  pkm status                                           │
│  git add <file>     │  pkm stage <block-id>                                 │
│  git reset <file>   │  pkm unstage <block-id>                               │
│  git diff --cached  │  pkm diff --staged                                    │
│  git diff           │  pkm diff                                             │
│  git commit         │  pkm commit "message"                                 │
│  git branch         │  pkm view list                                        │
│  git checkout -b    │  pkm view create <name>                               │
│  git checkout       │  pkm view switch <name>                               │
│  git tag            │  pkm tag create <name>                                │
│  git log            │  pkm log                                              │
│  git stash          │  pkm stash                                            │
│  git stash pop      │  pkm stash pop                                       │
│  git merge          │  pkm merge <view>                                     │
└──────────────────────┴──────────────────────────────────────────────────────┘
```

### 3.3 Detailed Workflow States

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         DETAILED WORKFLOW                                   │
└─────────────────────────────────────────────────────────────────────────────┘

    ┌─────────────────────────────────────────────────────────────────────────┐
    │  STEP 1: CREATE (Draft State)                                           │
    │                                                                         │
    │  ┌──────────────────────────────────────────────────────────────────┐   │
    │  │  block = Block::fleeting("Raw idea capture")                     │   │
    │  │  block.stage()  ──▶ WorkingSet.staged_blocks[block.id]          │   │
    │  └──────────────────────────────────────────────────────────────────┘   │
    └─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
    ┌─────────────────────────────────────────────────────────────────────────┐
    │  STEP 2: STAGE (Staged State)                                          │
    │                                                                         │
    │  ┌──────────────────────────────────────────────────────────────────┐   │
    │  │  WorkingSet.stage_block(BlockDelta::Created {                    │   │
    │  │      block_id: block.id,                                         │   │
    │  │      title: block.title,                                         │   │
    │  │      content: block.content,                                     │   │
    │  │      block_type: block.block_type,                               │   │
    │  │  })                                                               │   │
    │  │                                                                   │   │
    │  │  # Deltas recorded in operation log                               │   │
    │  │  # Can be unstaged with unstage_block(block_id)                   │   │
    │  └──────────────────────────────────────────────────────────────────┘   │
    └─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
    ┌─────────────────────────────────────────────────────────────────────────┐
    │  STEP 3: COMMIT (Committed State)                                       │
    │                                                                         │
    │  ┌──────────────────────────────────────────────────────────────────┐   │
    │  │  commit = repo.commit(                                            │   │
    │  │      working_set,                                                 │   │
    │  │      "Add fleeting note about Rust async"                         │   │
    │  │  )                                                                 │   │
    │  │                                                                   │   │
    │  │  # Creates StructureSnapshot                                       │   │
    │  │  # Records parent commits                                          │   │
    │  │  # Updates View (HEAD)                                             │   │
    │  └──────────────────────────────────────────────────────────────────┘   │
    └─────────────────────────────────────────────────────────────────────────┘
```

---

## 4. Zettelkasten Workflow

The Zettelkasten methodology is implemented through block type transitions.

### 4.1 Block Types

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         BLOCK TYPE TAXONOMY                                  │
└─────────────────────────────────────────────────────────────────────────────┘

                    ┌─────────────────────┐
                    │   INPUT (Capture)    │
                    └──────────┬────────────┘
                               │
              ┌────────────────┼────────────────┐
              ▼                ▼                ▼
    ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
    │    FLEETING     │ │   LITERATURE    │ │    REFERENCE    │
    │  (raw ideas)    │ │ (source notes)  │ │ (external refs) │
    └────────┬────────┘ └────────┬────────┘ └─────────────────┘
             │                   │
             │   process()       │   process()
             ▼                   ▼
    ┌─────────────────┐ ┌─────────────────┐
    │   PERMANENT     │ │    PERMANENT    │
    │  (crystallized) │ │  (synthesized)  │
    └────────┬────────┘ └────────┬────────┘
             │                   │
             └─────────┬─────────┘
                       │
                       ▼
             ┌─────────────────┐
             │    STRUCTURE    │
             │   (MOC/Index)   │
             └─────────────────┘
```

### 4.2 Block Type Definitions

| Type | Purpose | Characteristics |
|------|---------|-----------------|
| **Fleeting** | Quick capture | Raw ideas, unprocessed |
| **Literature** | Source notes | Citations, summaries |
| **Permanent** | Atomic knowledge | Dense, linked, standalone |
| **Structure** | Index/MOC | Entry points, groupings |
| **Hub** | Domain entry | Topic anchors |
| **Task** | Actions | Executable items |
| **Reference** | External links | Citations, URLs |
| **Outline** | Document skeleton | TOC structures |
| **Ghost** | Predictive placeholder | Future concepts |

### 4.3 Zettelkasten Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      ZETTELKASTEN WORKFLOW                                   │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│  FLEETING NOTES (Inbox)                                                      │
│  ─────────────────────────────────                                           │
│  • Quick capture without structure                                           │
│  • Raw ideas captured via voice or quick typing                             │
│  • Timestamped with ULID                                                    │
│                                                                             │
│  Example:                                                                    │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  Fleeting: "Maybe async/await in Rust could simplify the worker     │    │
│  │            pattern mentioned in the Nexus architecture"              │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ process()
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  LITERATURE NOTES (Processing)                                               │
│  ────────────────────────────────────                                        │
│  • Expand and clarify fleeting notes                                         │
│  • Add context from external sources                                         │
│  • Maintain link to source material                                         │
│                                                                             │
│  Example:                                                                    │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  Literature: "Rust async/await simplifies concurrent worker patterns │    │
│  │              - Tokio runtime for async tasks                         │    │
│  │              - Shared state via Arc<Mutex<T>>                        │    │
│  │              - Source: Rust Async Book Chapter 3                      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ synthesize()
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  PERMANENT NOTES (Knowledge)                                                 │
│  ───────────────────────────────                                             │
│  • Atomic, self-contained knowledge units                                   │
│  • Linked to other permanent notes via edges                               │
│  • Your own words, synthesized understanding                               │
│                                                                             │
│  Example:                                                                   │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  Permanent: "Async Worker Pattern in Rust                            │    │
│  │                                                                          │    │
│  │  The async/await syntax in Rust enables a clean worker pattern where │    │
│  │  Tokio manages scheduling. Key insight: bounded message passing      │    │
│  │  with channels prevents resource exhaustion.                         │    │
│  │                                                                          │    │
│  │  Links: → Rust Async Worker Pattern (self)                           │    │
│  │          → Nexus Actor Model (related)                                │    │
│  │          → Rust Async Book (source)                                    │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ organize()
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  STRUCTURE NOTES (MOC/Index)                                                 │
│  ─────────────────────────────────                                         │
│  • Hub for related permanent notes                                           │
│  • Provides navigation entry points                                          │
│  • Maintains overview of topic area                                         │
│                                                                             │
│  Example:                                                                   │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  Structure: "Rust Concurrency Patterns"                               │    │
│  │                                                                          │    │
│  │  ## Index                                                              │    │
│  │  • [[Async Worker Pattern]]        - Production-ready pattern        │    │
│  │  • [[Actor Model]]                 - Nexus architecture basis        │    │
│  │  • [[Lock-Free Data Structures]]  - Advanced optimization           │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 5. AI-Assisted Workflow

The AI subsystem enhances knowledge management through embeddings, linking suggestions, and ghost detection.

### 5.1 AI Integration Points

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      AI-ASSISTED WORKFLOW                                    │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                           CAPTURE                                            │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │  AI Assistance:                                                        │  │
│  │  • Auto-classify block type based on content                          │  │
│  │  • Suggest initial tags                                                │  │
│  │  • Detect duplicate ideas                                              │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                          PROCESS                                             │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │  AI Assistance:                                                        │  │
│  │  • Semantic clustering (group related blocks)                         │  │
│  │  • Embedding-based similarity search                                   │  │
│  │  • Link suggestion (find related blocks)                              │  │
│  │  • Ghost detection (identify predictive placeholders)                 │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                          SYNTHESIZE                                          │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │  AI Assistance:                                                        │  │
│  │  • Structure generation (create MOC from related blocks)              │  │
│  │  • Summary generation for literature notes                             │  │
│  │  • Confidence scoring for AI-generated content                        │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         RETRIEVE                                            │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │  AI Assistance:                                                        │  │
│  │  • Semantic search (natural language queries)                         │  │
│  │  • Context-aware retrieval                                            │  │
│  │  • Graph traversal suggestions                                        │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 5.2 AI Capabilities

| Component | Function | Description |
|-----------|----------|-------------|
| `Embeddings` | Semantic vectors | Generate embeddings for blocks |
| `LinkSuggester` | Relationship discovery | Find potential links between blocks |
| `SemanticClustering` | Topic grouping | Cluster blocks by semantic similarity |
| `GhostDetector` | Placeholder identification | Detect incomplete or predictive blocks |
| `StructureGenerator` | MOC creation | Generate structure notes from related blocks |

---

## 6. Block Type Transitions

### 6.1 Transition Matrix

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      BLOCK TYPE TRANSITIONS                                 │
└─────────────────────────────────────────────────────────────────────────────┘

    FROM ╲ TO  │ Fleeting │ Literature │ Permanent │ Structure │ Hub │ Task │
    ───────────┼──────────┼────────────┼───────────┼───────────┼─────┼──────│
    Fleeting   │    ─     │    ✓       │    ✓      │     ✓     │  ✓  │  ✓   │
    Literature  │    ✗     │    ─       │    ✓      │     ✓     │  ✓  │  ✓   │
    Permanent   │    ✗     │    ✗       │    ─      │     ✓     │  ✓  │  ✓   │
    Structure   │    ✗     │    ✗       │    ✗      │     ─     │  ✗  │  ✗   │
    Hub         │    ✗     │    ✗       │    ✗      │     ✓     │  ✗  │  ✗   │
    Ghost       │    ✓     │    ✓       │    ✓      │     ✓     │  ✓  │  ✓   │

    Legend: ✓ = allowed transition, ✗ = not allowed, ─ = same type
```

### 6.2 Transition Rules

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         TRANSITION RULES                                     │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│  RULE 1: Fleeting → Literature                                               │
│  ─────────────────────────────────                                          │
│  Condition: User expands idea with source citation                          │
│  Action:    Convert block, add source metadata                               │
│  Example:   "Rust is fast" → "Rust is fast (Source: rust-lang.org)"         │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│  RULE 2: Fleeting/Literature → Permanent                                    │
│  ─────────────────────────────────────────                                  │
│  Condition: Knowledge is synthesized, atomic, self-contained                │
│  Action:    Convert to Permanent, create links to related blocks          │
│  Validation: Must have at least one outgoing edge                          │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│  RULE 3: Any → Ghost                                                         │
│  ───────────────────                                                        │
│  Condition: Block content is speculative or placeholder                    │
│  Action:    Convert to Ghost, mark as predictive                            │
│  Example:   "Future: Multi-region support" → Ghost block                   │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│  RULE 4: Multiple Permanents → Structure                                     │
│  ────────────────────────────────────────                                   │
│  Condition: Indexing related permanent notes                                │
│  Action:    Create Structure block with links to related blocks             │
│  AI Assist: StructureGenerator can suggest structure from clusters          │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 6.3 Delta Types for Transitions

```rust
// BlockDelta variants for type transitions
pub enum BlockDelta {
    Created { block_id, title, content, block_type },
    Modified { block_id, old_title, new_title, old_content, new_content },
    Deleted { block_id, title },
    Reorganized { block_id, old_predecessor, new_predecessor },
    TypeChanged { block_id, old_type, new_type },  // For type transitions
    TagAdded { block_id, tag },
    TagRemoved { block_id, tag },
}
```

---

## 7. ASCII Diagrams

### 7.1 Complete Workflow Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    COMPLETE PKM WORKFLOW DIAGRAM                              │
└─────────────────────────────────────────────────────────────────────────────┘

    ╔═══════════════════════════════════════════════════════════════════════╗
    ║                         USER INPUT                                       ║
    ╠═══════════════════════════════════════════════════════════════════════╣
    ║                                                                        ║
    ║   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐                ║
    ║   │  Fleeting   │    │  Literature │    │  Reference  │                ║
    ║   │   Note      │    │    Note     │    │   Note      │                ║
    ║   └──────┬──────┘    └──────┬──────┘    └──────┬──────┘                ║
    ║          │                  │                  │                       ║
    ║          │ process()        │ process()        │ link()                ║
    ║          ▼                  ▼                  ▼                       ║
    ║   ┌──────────────────────────────────────────────────────┐               ║
    ║   │              PERMANENT NOTES                          │               ║
    ║   │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  │               ║
    ║   │  │ Block 1 │  │ Block 2 │  │ Block 3 │  │ Block N │  │               ║
    ║   │  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘  │               ║
    ║   │       │    links   │    links   │    links   │        │               ║
    ║   │       └────────────┼────────────┼────────────┘        │               ║
    ║   │                    │            │                    │               ║
    ║   │              ┌──────┴────────────┴──────┐              │               ║
    ║   │              │                         │              │               ║
    ║   │              ▼                         ▼              │               ║
    ║   │       ┌───────────┐             ┌───────────┐           │               ║
    ║   │       │   Hub     │             │ Structure │           │               ║
    ║   │       │ (domain)  │             │   (MOC)   │           │               ║
    ║   │       └───────────┘             └───────────┘           │               ║
    ║   └──────────────────────────────────────────────────────┘               ║
    ║                               │                                         ║
    ║                               │ synthesize()                            ║
    ║                               ▼                                         ║
    ║   ┌──────────────────────────────────────────────────────┐               ║
    ║   │                 SYNTHESIS OUTPUT                     │               ║
    ║   │  • Document exports    • Typst renders               │               ║
    ║   │  • Graph visualizations • Knowledge maps            │               ║
    ║   └──────────────────────────────────────────────────────┘               ║
    ║                                                                       ║
    ╚═══════════════════════════════════════════════════════════════════════╝
```

### 7.2 Version Control Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      VERSION CONTROL FLOW                                    │
└─────────────────────────────────────────────────────────────────────────────┘

        ┌─────────────────────────────────────────────────────────────────┐
        │                        WORKING DIRECTORY                         │
        │  ┌─────────────────────────────────────────────────────────┐    │
        │  │  Blocks in memory / being edited                        │    │
        │  │  • Created via Block::fleeting(), Block::permanent()   │    │
        │  │  • Modified via block.with_content(), block.with_tag()  │    │
        │  └─────────────────────────┬───────────────────────────────┘    │
        └────────────────────────────┼────────────────────────────────────┘
                                     │ stage()
                                     ▼
        ┌─────────────────────────────────────────────────────────────────┐
        │                         WORKING SET                              │
        │  ┌─────────────────────────────────────────────────────────┐    │
        │  │  STAGED CHANGES                                         │    │
        │  │  ┌────────────────┐  ┌────────────────┐                │    │
        │  │  │ BlockDelta    │  │ EdgeDelta      │                │    │
        │  │  │ - Created     │  │ - Created      │                │    │
        │  │  │ - Modified    │  │ - Deleted      │                │    │
        │  │  │ - Deleted     │  │                │                │    │
        │  │  └────────────────┘  └────────────────┘                │    │
        │  │                                                        │    │
        │  │  REMOVED PENDING                                       │    │
        │  │  • removed_blocks: Vec<Ulid>                           │    │
        │  │  • removed_edges: Vec<(Ulid, Ulid)>                    │    │
        │  │                                                        │    │
        │  │  OPERATION LOG                                          │    │
        │  │  • Vec<Operation> for replay/undo                      │    │
        │  └─────────────────────────────────────────────────────────┘    │
        └────────────────────────────┬────────────────────────────────────┘
                                     │ commit()
                                     ▼
        ┌─────────────────────────────────────────────────────────────────┐
        │                          COMMIT                                  │
        │  ┌─────────────────────────────────────────────────────────┐    │
        │  │  Commit {                                                │    │
        │  │    id: CommitId(Ulid),                                    │    │
        │  │    structure_snapshot: StructureSnapshot,                 │    │
        │  │    parents: Vec<CommitId>,                                 │    │
        │  │    author: AgentId,                                        │    │
        │  │    message: String,                                       │    │
        │  │    created_at: DateTime,                                   │    │
        │  │    blocks_added/modified/removed: Vec<Ulid>,              │    │
        │  │  }                                                         │    │
        │  └─────────────────────────┬───────────────────────────────┘    │
        └────────────────────────────┼────────────────────────────────────┘
                                     │ update View
                                     ▼
        ┌─────────────────────────────────────────────────────────────────┐
        │                           REPOSITORY                              │
        │  ┌─────────────────────────────────────────────────────────┐    │
        │  │  VIEWS (Branches/Tags)                                   │    │
        │  │  ┌──────────┐  ┌──────────┐  ┌──────────┐                │    │
        │  │  │ main    │  │ work     │  │ v1.0.0  │ (tag)          │    │
        │  │  │  ↓      │  │   ↓     │  │   ↓     │                │    │
        │  │  │ Commit  │  │ Commit  │  │ Commit  │                │    │
        │  │  └──────────┘  └──────────┘  └──────────┘                │    │
        │  │                                                        │    │
        │  │  COMMITS (Chain)                                        │    │
        │  │  ●────●────●────●────● (HEAD/main)                      │    │
        │  │                                                                    │    │
        │  │  STRUCTURE SNAPSHOT                                        │    │
        │  │  • block_order: Vec<Ulid> (FOLLOWZETTEL ordering)          │    │
        │  │  • edges: Vec<EdgeSnapshot>                               │    │
        │  └─────────────────────────────────────────────────────────┘    │
        └─────────────────────────────────────────────────────────────────┘
```

### 7.3 Session Cycle

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          SESSION CYCLE                                      │
└─────────────────────────────────────────────────────────────────────────────┘

                              ┌───────────────────┐
                              │   SESSION START   │
                              │   Create WorkingSet│
                              └─────────┬─────────┘
                                        │
                                        ▼
        ┌───────────────────────────────────────────────────────────────────┐
        │                        SESSION ACTIVE                              │
        │                                                                    │
        │   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐               │
        │   │   CREATE    │  │   EDIT      │  │   DELETE    │               │
        │   │   BLOCK     │──│   BLOCK     │──│   BLOCK     │               │
        │   └─────────────┘  └─────────────┘  └─────────────┘               │
        │         │                │                │                        │
        │         └────────────────┼────────────────┘                        │
        │                          │                                         │
        │                          ▼                                         │
        │   ┌─────────────────────────────────────────────────────────┐     │
        │   │                    STAGE CHANGES                          │     │
        │   │  • stage_block(delta)     • stage_edge(delta)           │     │
        │   │  • mark_block_removed(id) • mark_edge_removed(s,t)      │     │
        │   └─────────────────────────────────────────────────────────┘     │
        │                          │                                         │
        │           ┌──────────────┴──────────────┐                         │
        │           │                             │                         │
        │           ▼                             ▼                         │
        │   ┌───────────────┐             ┌───────────────┐                │
        │   │    COMMIT     │             │     STASH     │                │
        │   │  create_commit│             │   save_state  │                │
        │   └───────┬───────┘             └───────────────┘                │
        │           │                                                          │
        │           ▼                                                          │
        │   ┌───────────────┐                                                 │
        │   │ UPDATE VIEW   │                                                 │
        │   │ (HEAD moves)  │                                                 │
        │   └───────────────┘                                                 │
        │                                                                    │
        └────────────────────────────────────────────────────────────────────┘
                                        │
                                        ▼
                              ┌───────────────────┐
                              │   SESSION END     │
                              │   Close/Archive   │
                              └───────────────────┘
```

---

## 8. Best Practices

### 8.1 Session Management

| Practice | Description |
|----------|-------------|
| **Atomic Commits** | Group related changes in a single commit with a descriptive message |
| **Frequent Staging** | Stage changes regularly to avoid losing work |
| **View Discipline** | Keep main view clean; use feature views for experimental work |
| **Meaningful Messages** | Write commit messages that explain "why", not just "what" |

### 8.2 Block Creation

| Practice | Description |
|----------|-------------|
| **One Idea Per Block** | Keep blocks atomic; if content grows beyond 2-3 paragraphs, consider splitting |
| **Use ULID Timestamps** | Leverage ULID's time-ordering for automatic chronological sorting |
| **Link Generously** | Create edges between related blocks to build the knowledge graph |
| **Tag Consistently** | Establish a controlled vocabulary for tags |

### 8.3 Zettelkasten Principles

| Practice | Description |
|----------|-------------|
| **Process Fleeting Notes** | Review and process fleeting notes within 24-48 hours |
| **Write in Your Own Words** | Literature notes should be synthesized, not copied |
| **Permanent = Linked** | Permanent notes must have outgoing edges to other notes |
| **Build Structures Last** | Create MOC/Hub blocks after accumulating related permanent notes |

### 8.4 Version Control

| Practice | Description |
|----------|-------------|
| **Commit Early, Commit Often** | Small commits are easier to understand and revert |
| **Reference External Context** | Include links to issues, PRs, or documents in commit messages |
| **Branch for Major Work** | Use views for major reorganizations or experimental restructuring |
| **Tag Milestones** | Tag significant knowledge states (e.g., "completed-course-2026") |

---

## 9. Examples

### 9.1 Complete Workflow Example

```rust
use hodei_pkm::{
    versioning::{WorkingSet, Commit, View},
    models::{Block, BlockType},
    ai::{Embeddings, LinkSuggester},
};

// 1. Start a new session
let author = AgentId::new("researcher");
let mut working_set = WorkingSet::new(author.clone());

// 2. Create fleeting note
let mut fleeting = Block::fleeting(
    "Rust async patterns could simplify Nexus worker design"
);
fleeting = fleeting.with_tag("rust").with_tag("async");

// 3. Stage the block
let delta = BlockDelta::Created {
    block_id: fleeting.id,
    title: fleeting.title.clone(),
    content: fleeting.content.clone(),
    block_type: "fleeting".to_string(),
};
working_set.stage_block(delta);

println!("Staged {} block(s)", working_set.staged_blocks_count());

// 4. Process: expand to literature note
let literature = Block::new(BlockType::Literature, "Rust Async Worker Patterns");
let literature_content = r#"
From: Rust Async Book

Key patterns:
1. Tokio runtime for async tasks
2. Shared state via Arc<Mutex<T>>
3. Channels for message passing
4. Bounded channels prevent resource exhaustion

See also: Actor Model in Nexus architecture
"#;
let literature = literature.with_content(literature_content);

// 5. Stage literature note
working_set.stage_block(BlockDelta::Created {
    block_id: literature.id,
    title: literature.title.clone(),
    content: literature.content.clone(),
    block_type: "literature".to_string(),
});

// 6. Stage the edge linking literature to fleeting
working_set.stage_edge(EdgeDelta::Created {
    source: literature.id,
    target: fleeting.id,
    relation: "processes".to_string(),
});

// 7. Commit the session
let commit = Commit::new(
    StructureSnapshot {
        id: Ulid::new(),
        block_order: vec![fleeting.id, literature.id],
        edges: vec![
            EdgeSnapshot {
                source: literature.id,
                target: fleeting.id,
                relation: "processes".to_string(),
            }
        ],
    },
    author.clone(),
    "Process fleeting note into literature note".to_string(),
    Vec::new(),
    vec![fleeting.id, literature.id],
    Vec::new(),
    Vec::new(),
);

// 8. Update main view
let main_view = View::branch_head("main", commit.id);
```

### 9.2 Zettelkasten Progression Example

```rust
// STEP 1: Fleeting (capture)
let fleeting_note = Block::fleeting(
    "Agent supervision patterns in distributed systems"
);

// STEP 2: Literature (expand with sources)
let literature_note = Block::new(BlockType::Literature, "Agent Supervision Patterns");
let literature_content = r#"
Key concepts from "Designing Distributed Systems" by Brendan Burns:

SUPERVISION TREES:
- One-for-one: Restart failed child only
- One-for-all: Restart all children if one fails
- Custom strategies for specific failure modes

NEXUS APPLICATION:
The Nexus architecture uses supervision for worker lifecycle:
- SensorWorker supervised by main thread
- LogicWorker supervised independently
- RenderWorker has its own recovery domain

Source: Designing Distributed Systems, Brendan Burns
"#;
let literature = literature_note.with_content(literature_content);

// STEP 3: Permanent (synthesize atomic knowledge)
let permanent_note = Block::permanent(
    "Nexus Agent Supervision Pattern",
    r#"
The Nexus WASM architecture implements hierarchical supervision:

RUNTIME SUPERVISION:
Main Thread (React UI)
    └── SensorWorker (actor)
    └── LogicWorker (actor)
    └── RenderWorker (actor)

SUPERVISION POLICY:
- Heartbeat monitoring (control-plane.sab)
- Epoch-gated recovery
- Replace-on-crash semantics
- No shared state; SAB as single source of truth

This enables:
- Fault isolation between workers
- Independent recovery without system-wide restart
- Deterministic restart behavior

Related: [[Actor Model]], [[Nexus Control Plane]]
"#
).with_tag("nexus").with_tag("architecture").with_tag("fault-tolerance");

// STEP 4: Structure (create MOC)
let structure_note = Block::structure("Nexus Architecture Index");
let structure_content = r#"
# Nexus Architecture Topics

## Core Concepts
- [[Nexus Agent Supervision Pattern]] - Worker lifecycle management
- [[Nexus Control Plane]] - SAB-based synchronization
- [[Nexus Actor Model]] - Actor semantics over shared memory

## Patterns
- [[Zarfian Determinism]] - Rule specificity ordering
- [[Ghost Principle]] - Ephemeral worker context recovery

## Implementation
- [[Nexus Draw Architecture]] - Reference application
- [[Nexus WASM Workers]] - Worker implementation
"#;
let moc = structure_note.with_content(structure_content);
```

### 9.3 Session Stash/Resume Example

```rust
// INTERRUPT: Switch context mid-session
let stash_id = repo.stash_working_set(working_set, "WIP: researching supervision")?;
println!("Stashed working set: {}", stash_id);

// ... later, resume work ...

let (resumed_working_set, stash) = repo.pop_stash(stash_id)?;
println!("Resumed stash: {}", stash.message());
// Continue editing...
```

---

## Appendix: Command Reference

| CLI Command | Description |
|-------------|-------------|
| `pkm init` | Initialize new PKM repository |
| `pkm status` | Show working set status |
| `pkm stage <id>` | Stage block for commit |
| `pkm unstage <id>` | Remove from staging |
| `pkm diff [--staged]` | Show changes |
| `pkm commit <msg>` | Commit staged changes |
| `pkm log [--graph]` | Show commit history |
| `pkm view list` | List all views |
| `pkm view create <name>` | Create new view |
| `pkm view switch <name>` | Switch HEAD to view |
| `pkm tag create <name>` | Create annotated tag |
| `pkm merge <view>` | Merge view into current |
| `pkm stash` | Stash current working set |
| `pkm stash pop` | Restore stashed working set |

---

*Document version: 1.0.0 | Last updated: 2026-03-20*
