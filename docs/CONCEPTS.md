# PKM-AI Concepts

**Version:** 1.0
**Date:** 2026-03-19
**Status:** Canonical Reference

---

## Overview

This document provides canonical definitions for all PKM-AI concepts. When documentation and implementation disagree, this document defines the truth.

---

## 1. Core Domain Concepts

### 1.1 Block

The fundamental unit of content in PKM-AI.

```rust
// Canonical definition in src/models/block.rs
pub struct Block {
    pub id: Ulid,                           // Timestamp + random, chronologically sortable
    pub block_type: BlockType,              // Semantic type
    pub content: String,                     // Markdown content
    pub properties: Map<String, Value>,      // Flexible metadata
    pub embedding_bloom: Option<[u128; 1]>,  // Semantic search filter
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

| Property | Type | Description |
|----------|------|-------------|
| `id` | `Ulid` | Unique identifier, sortable by creation time |
| `block_type` | `BlockType` | Semantic classification |
| `content` | `String` | Markdown-formatted content |
| `properties` | `Map<String, Value>` | Arbitrary key-value metadata |
| `embedding_bloom` | `Option<[u128; 1]>` | Bloom filter for fast semantic search |
| `created_at` | `DateTime<Utc>` | Creation timestamp |
| `updated_at` | `DateTime<Utc>` | Last modification timestamp |

**Block Types:**

| Type | Alias | Description |
|------|-------|-------------|
| `fleeting` | `f` | Temporary capture notes |
| `literature` | `l` | Reference material from external sources |
| `permanent` | `p` | Atomic Zettelkasten notes (core type) |
| `structure` | `s`, `index`, `moc` | Structural containers |
| `hub` | `h` | Central topic nodes (entry points) |
| `task` | `t` | Action items |
| `reference` | `r` | External references |
| `outline` | `o` | Hierarchical outlines |
| `ghost` | `g` | Placeholder for missing content |

### 1.2 Structure

A container block that organizes other blocks.

```rust
pub struct Structure {
    pub id: Ulid,
    pub name: String,
    pub root_blocks: Vec<Ulid>,                    // Top-level blocks
    pub block_tree: Map<Ulid, Vec<Ulid>>,          // parent -> children mapping
    pub spine_order: Vec<(Ulid, FractionalIndex)>, // Ordered sequence
    pub properties: Map<String, Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**Key Distinction:**
- A `Block` with `block_type = Structure` represents a document or collection
- A `Structure` struct represents the internal ordering of a document

### 1.3 Edge

A typed relationship between two blocks.

```rust
pub struct Edge {
    pub id: Ulid,
    pub link_type: LinkType,
    pub from: Ulid,    // Source block
    pub to: Ulid,      // Target block
    pub properties: Map<String, Value>,
    pub sequence_weight: FractionalIndex,  // Position in sequence
}
```

**Link Types:**

| Type | Category | Description |
|------|----------|-------------|
| `extends` | Semantic | Block extends another |
| `refines` | Semantic | Block refines another |
| `contradicts` | Semantic | Block contradicts another |
| `questions` | Semantic | Block questions another |
| `supports` | Semantic | Block supports another |
| `references` | Semantic | Block references another |
| `related` | Semantic | Blocks are related (default) |
| `similar_to` | Semantic | Blocks are similar |
| `section_of` | Structural | Block is section of a Structure |
| `subsection_of` | Structural | Block is subsection |
| `ordered_child` | Structural | Ordered child in hierarchy |
| `next` | Structural | Next in sequence (Structural Spine) |
| `next_sibling` | Structural | Next sibling |
| `first_child` | Structural | First child |
| `contains` | Structural | Block contains another |
| `parent` | Structural | Block is parent |
| `ai_suggested` | AI | AI-suggested link |

---

## 2. Structural Spine

The **Structural Spine** is the ordered backbone of a document, implemented as linked blocks via `NEXT` edges.

### 2.1 Definition

```
Block A (weight: "a") → Block B (weight: "am") → Block C (weight: "b")
```

**Principles:**
- Every block in a Spine has exactly one `next` edge (except the last)
- Traversal is deterministic and preserves order
- Insertion between two blocks generates a midpoint key

### 2.2 FractionalIndex

**DO NOT use `f32` for sequence ordering.** Use lexicographic fractional indexing.

```rust
// Correct implementation
pub struct FractionalIndex(String);

impl FractionalIndex {
    pub fn between(before: &FractionalIndex, after: &FractionalIndex) -> Self {
        // Lexicographic midpoint
    }

    pub fn first() -> Self {
        FractionalIndex("a".to_string())
    }

    pub fn after(last: &FractionalIndex) -> Self {
        // Append 'a' to extend
    }
}
```

### 2.3 Traversal Rules

1. Start from root block
2. Follow `NEXT` edges in order
3. Respect depth limit (default: 100)
4. Detect and handle cycles

---

## 3. Smart Sections

A `Structure` block enhanced with semantic awareness.

```rust
pub struct SmartSection {
    pub block: Block,              // The underlying structure block
    pub intent: String,            // Purpose description
    pub boundary_constraints: Vec<Constraint>,
    pub semantic_centroid: Vec<f32>,  // Average embedding
    pub medoid_id: Option<Ulid>,   // Most representative block
    pub vacancy_status: Vacancy,   // Capacity indicator
    pub coherence_score: f32,       // 0.0 - 1.0
}

pub enum Vacancy {
    Full,       // >90% capacity
    NearlyFull, // 70-90%
    Partial,    // 30-70%
    Sparse,     // 10-30%
    Empty,      // <10%
}
```

### 3.1 Semantic Centroid Calculation

```rust
pub fn calculate_weighted_centroid(
    blocks: &[Block],
    weights: &[f32],  // Based on incoming links + 1
) -> Vec<f32> {
    let total: f32 = weights.iter().sum();
    let mut centroid = vec![0.0; EMBEDDING_DIM];

    for (block, weight) in blocks.iter().zip(weights) {
        for (i, val) in block.embedding.iter().enumerate() {
            centroid[i] += val * weight / total;
        }
    }
    centroid
}
```

---

## 4. Ghost Nodes

Predictive placeholders for missing content.

```rust
pub struct GhostNode {
    pub id: Ulid,
    pub expected_keywords: Vec<String>,
    pub confidence: f32,           // 0.0 - 1.0
    pub parent_id: Ulid,           // Structure containing this gap
    pub suggested_position: FractionalIndex,
    pub status: GhostStatus,
}

pub enum GhostStatus {
    Pending,   // Not yet addressed
    Filled,    // Content has been added
    Dismissed, // Intentionally left empty
}
```

### 4.1 Detection Algorithm

1. Get all blocks in a Structure
2. Calculate semantic centroids for each section
3. For each consecutive pair, calculate distance
4. If distance > threshold, insert GhostNode

---

## 5. Versioning (Git-like API)

### 5.1 Commit

```rust
pub struct Commit {
    pub id: CommitId,
    pub structure_snapshot: Structure,
    pub parents: Vec<CommitId>,
    pub author: AgentId,
    pub message: String,
    pub created_at: DateTime<Utc>,
    pub blocks_added: Vec<Ulid>,
    pub blocks_removed: Vec<Ulid>,
    pub blocks_modified: Vec<Ulid>,
}
```

### 5.2 View (Ref)

```rust
pub enum View {
    Branch {
        name: ViewName,
        target: Ulid,      // Commit ID
        is_head: bool,
    },
    Tag {
        name: ViewName,
        target: Ulid,
        message: String,   // Annotated tag message
    },
}
```

### 5.3 WorkingSet (Index)

```rust
pub struct WorkingSet {
    pub id: WorkingSetId,
    pub author: AgentId,
    pub staged_blocks: BTreeMap<Ulid, BlockDelta>,
    pub staged_edges: BTreeMap<(Ulid, Ulid), EdgeDelta>,
    pub removed_blocks: Vec<Ulid>,
    pub removed_edges: Vec<(Ulid, Ulid)>,
    pub operations: Vec<Operation>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

---

## 6. Agent Concepts

### 6.1 AgentId

```rust
pub struct AgentId(String);

impl AgentId {
    pub fn new(id: String) -> Self;
    pub fn as_str(&self) -> &str;
}
```

### 6.2 Agent Workflow

1. **Capture**: Agent captures fleeting notes
2. ** Crystallize**: Convert to permanent notes
3. **Link**: Create semantic edges
4. **Structure**: Add to Structural Spine
5. **Synthesize**: Generate documents

---

## 7. Database Layer

### 7.1 Schema Tables

```sql
-- Block table (SCHEMAFULL)
DEFINE TABLE block SCHEMAFULL;
DEFINE FIELD id ON block TYPE ulid PRIMARY KEY;
DEFINE FIELD block_type ON block TYPE string;
DEFINE FIELD content ON block TYPE string;
DEFINE FIELD properties ON block TYPE object;
DEFINE FIELD embedding_bloom ON block TYPE option<array>;
DEFINE FIELD created_at ON block TYPE datetime;
DEFINE FIELD updated_at ON block TYPE datetime;

-- Edge table
DEFINE TABLE edge SCHEMAFULL;
DEFINE FIELD id ON edge TYPE ulid PRIMARY KEY;
DEFINE FIELD link_type ON edge TYPE string;
DEFINE FIELD from ON edge TYPE ulid;
DEFINE FIELD to ON edge TYPE ulid;
DEFINE FIELD properties ON edge TYPE object;
DEFINE FIELD sequence_weight ON edge TYPE string;  -- FractionalIndex as string

-- Indexes
DEFINE INDEX idx_block_type ON block FIELDS block_type;
DEFINE INDEX idx_edge_from ON edge FIELDS from;
DEFINE INDEX idx_edge_link_type ON edge FIELDS link_type;
```

---

## 8. CLI Commands

### 8.1 Core Commands

| Command | Description |
|---------|-------------|
| `pkmai create` | Create a new block |
| `pkmai list` | List blocks with filtering |
| `pkmai show` | Show block details |
| `pkmai link` | Create links between blocks |
| `pkmai grep` | Search block content |
| `pkmai traverse` | Traverse the structural spine |
| `pkmai gravity-check` | Check block connectivity |
| `pkmai toc` | Generate table of contents |
| `pkmai synthesize` | Synthesize document from structure |
| `pkmai ghost` | Manage ghost nodes |
| `pkmai architect` | Launch interactive TUI |
| `pkmai lint` | Validate structural integrity |
| `pkmai db` | Database management |
| `pkmai api` | Start REST API server |

### 8.2 Version Control Commands

| Command | Git Equivalent | Description |
|---------|---------------|-------------|
| `pkmai version status` | `git status` | Show working tree status |
| `pkmai version log` | `git log` | Show commit logs |
| `pkmai version diff` | `git diff` | Show changes |
| `pkmai version add` | `git add` | Stage changes |
| `pkmai version commit` | `git commit` | Create commit |
| `pkmai version branch` | `git branch` | Branch management |
| `pkmai version checkout` | `git checkout` | Switch branches |
| `pkmai version merge` | `git merge` | Merge branches |
| `pkmai version tag` | `git tag` | Tag operations |
| `pkmai version push` | `git push` | Push to remote |
| `pkmai version pull` | `git pull` | Pull from remote |

---

## 9. MCP Tools

### 9.1 Block Tools

| Tool | Parameters | Description |
|------|------------|-------------|
| `create_block` | `block_type`, `title`, `content?`, `tags?` | Create block |
| `get_block` | `id` | Get block by ULID |
| `search_blocks` | `query?`, `block_type?`, `tags?`, `limit?` | Search |
| `update_block` | `id`, `title?`, `content?`, `tags?` | Update |

### 9.2 Link Tools

| Tool | Parameters | Description |
|------|------------|-------------|
| `create_link` | `from_id`, `to_id`, `link_type`, `weight?`, `context?` | Create link |
| `get_links` | `block_id`, `direction?` | Get links |
| `suggest_links` | `block_id`, `limit?` | AI suggestions |

### 9.3 Spine Tools

| Tool | Parameters | Description |
|------|------------|-------------|
| `traverse_spine` | `root_id?`, `depth?`, `link_type?` | Traverse |
| `gravity_check` | `block_id`, `threshold?` | Check connectivity |
| `reorder_block` | `block_id`, `new_position`, `parent_id?` | Reorder |

### 9.4 Structure Tools

| Tool | Parameters | Description |
|------|------------|-------------|
| `get_section_map` | `root_id` | Get hierarchy |
| `detect_gaps` | `section_id` | Detect missing |
| `list_ghosts` | `root_id?` | List placeholders |

### 9.5 Synthesis Tools

| Tool | Parameters | Description |
|------|------------|-------------|
| `synthesize` | `structure_id`, `format?`, `template?` | Generate doc |
| `get_toc` | `structure_id` | Get TOC |

---

## 10. Glossary

| Term | Definition | Canonical Location |
|------|------------|-------------------|
| **Block** | Atomic unit of content with ULID | `src/models/block.rs` |
| **Structure** | Container block for organizing blocks | `src/models/block.rs` |
| **Edge** | Typed relationship between blocks | `src/models/edge.rs` |
| **Structural Spine** | Ordered sequence via NEXT edges | `src/spine/` |
| **Smart Section** | Structure with semantic awareness | `src/models/smart_section.rs` |
| **Ghost Node** | Placeholder for missing content | `src/models/ghost_node.rs` |
| **FractionalIndex** | Lexicographic position key | `src/utils/fractional_index.rs` |
| **WorkingSet** | Index of pending changes | `src/models/working_set.rs` |
| **View** | Named pointer to commit (branch/tag) | `src/models/view.rs` |
| **Commit** | Snapshot of structure state | `src/models/commit.rs` |
| **AgentId** | Unique agent identifier | `src/models/agent.rs` |
| **Semantic Centroid** | Average embedding weighted by importance | `smart_section.rs` |
| **Medoid** | Block closest to centroid | `smart_section.rs` |

---

## 11. References

- Domain Models: `src/models/`
- CLI Commands: `src/cli/commands/`
- MCP Tools: `src/ai/mcp.rs`
- Database Schema: `src/db/schema.rs`
- Tests: `src/tests/`

---

**Last updated:** 2026-03-19
**Canonical source:** `src/models/` and this document