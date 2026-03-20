# PKM-AI Specification

**Version:** 1.0
**Date:** 2026-03-19
**Status:** Canonical
**Project:** hodei-pkm

---

## Table of Contents

1. [Vision and Principles](#1-vision-and-principles)
2. [Domain Models](#2-domain-models)
3. [Structural Spine](#3-structural-spine)
4. [Smart Sections](#4-smart-sections)
5. [Ghost Nodes](#5-ghost-nodes)
6. [Git-like API](#6-git-like-api)
7. [CLI Commands](#7-cli-commands)
8. [MCP Tools](#8-mcp-tools)
9. [REST API](#9-rest-api)
10. [Architecture](#10-architecture)
11. [Implementation Status](#11-implementation-status)

---

## 1. Vision and Principles

### 1.1 Core Vision

PKM-AI is a **Knowledge Operating System** for teams working with AI agent swarms. While tools like Obsidian or Logseq are individual note zoos, PKM-AI is built for:

- Multiple AI agents operating concurrently on the same knowledge base
- Automatic synthesis of Zettelkasten fragments into professional technical documents
- Deterministic ordering emerging from structure, not hierarchically imposed
- High performance (target: 65,000 blocks with graph operations in <16ms)

### 1.2 Fundamental Principles

| Principle | Description |
|-----------|-------------|
| **Block-Atom Model** | Every piece of knowledge is an addressable block with ULID |
| **Structural Spine First** | Order and structure are first-class citizens |
| **Semantic/Structural Separation** | Semantic edges (links) vs structural edges (ordering) |
| **Ghost Nodes as Predicates** | Gaps are constraints describing ideal content |
| **Performance as Requirement** | O(Δblocks) not O(N) for gravity hooks |

### 1.3 Project Relationship

PKM-AI and Nexus-WASM are sibling projects under "hodei-pkm":

| Aspect | Nexus-WASM | PKM-AI |
|--------|-----------|---------|
| **Domain** | High-performance WASM runtime | Knowledge operating system |
| **Target** | 65,536 entities @ 60 FPS | 65,000+ blocks with concurrent agents |
| **Architecture** | Actor model over SAB | Actor model over SurrealDB daemon |
| **Coordination** | SharedArrayBuffer + Atomics | Unix socket + LeaseManager |

---

## 2. Domain Models

### 2.1 Block

```rust
// Canonical: src/models/block.rs
pub struct Block {
    pub id: Ulid,                           // Chronologically sortable
    pub block_type: BlockType,
    pub content: String,                    // Markdown content
    pub properties: Map<String, Value>,    // Flexible metadata
    pub embedding_bloom: Option<[u128; 1]>, // Semantic search filter
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**Block Types:**

| Type | Alias | Description |
|------|-------|-------------|
| `fleeting` | `f` | Temporary capture notes |
| `literature` | `l` | Reference material from external sources |
| `permanent` | `p` | Atomic Zettelkasten notes |
| `structure` | `s`, `moc` | Structural containers |
| `hub` | `h` | Central topic entry points |
| `task` | `t` | Action items |
| `reference` | `r` | External references |
| `outline` | `o` | Hierarchical outlines |
| `ghost` | `g` | Placeholder for missing content |

### 2.2 Edge

```rust
pub struct Edge {
    pub id: Ulid,
    pub link_type: LinkType,
    pub from: Ulid,    // Source block
    pub to: Ulid,      // Target block
    pub properties: Map<String, Value>,
    pub sequence_weight: FractionalIndex,  // Position key (NOT f32!)
    pub updated_at: DateTime<Utc>,
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
| `related` | Semantic | Related (default) |
| `similar_to` | Semantic | Similar to another |
| `section_of` | Structural | Block is section of a Structure |
| `subsection_of` | Structural | Subsection |
| `ordered_child` | Structural | Ordered child |
| `next` | Structural | Next in sequence |
| `next_sibling` | Structural | Next sibling |
| `first_child` | Structural | First child |
| `contains` | Structural | Contains another |
| `parent` | Structural | Parent of |
| `ai_suggested` | AI | AI-suggested link |

### 2.3 FractionalIndex (CRITICAL: NOT f32)

**DO NOT use `f32` for sequence ordering.** Use lexicographic fractional indexing.

```rust
pub struct FractionalIndex(String);

impl FractionalIndex {
    pub fn first() -> Self;
    pub fn after(last: &FractionalIndex) -> Self;
    pub fn between(before: &FractionalIndex, after: &FractionalIndex) -> Self;
}
```

### 2.4 SmartSection

```rust
pub struct SmartSection {
    pub block: Block,
    pub intent: String,
    pub boundary_constraints: Vec<Constraint>,
    pub semantic_centroid: Vec<f32>,
    pub medoid_id: Option<Ulid>,
    pub vacancy_status: Vacancy,
    pub coherence_score: f32,
}

pub enum Vacancy {
    Full,       // >90%
    NearlyFull, // 70-90%
    Partial,    // 30-70%
    Sparse,     // 10-30%
    Empty,      // <10%
}
```

### 2.5 GhostNode

```rust
pub struct GhostNode {
    pub id: Ulid,
    pub expected_keywords: Vec<String>,
    pub confidence: f32,
    pub parent_id: Ulid,
    pub suggested_position: FractionalIndex,
    pub status: GhostStatus,
}

pub enum GhostStatus {
    Pending,
    Filled,
    Dismissed,
}
```

---

## 3. Structural Spine

### 3.1 Definition

The **Structural Spine** is the ordered backbone of a document, implemented as linked blocks via `NEXT` edges.

```
Block A (weight: "a") → Block B (weight: "am") → Block C (weight: "b")
```

### 3.2 Traversal Rules

1. Start from root block
2. Follow `NEXT` edges in order
3. Respect depth limit (default: 100)
4. Detect and handle cycles

### 3.3 Traverse Algorithm

```rust
async fn traverse_spine(
    db: &SurrealDb,
    root: Ulid,
    max_depth: usize,
) -> Result<Vec<Block>> {
    let mut visited = HashSet::new();
    traverse_recursive(db, root, max_depth, 0, &mut visited).await
}

#[async_recursion::async_recursion]
async fn traverse_recursive(
    db: &SurrealDb,
    node: Ulid,
    max_depth: usize,
    current_depth: usize,
    visited: &mut HashSet<Ulid>,
) -> Result<Vec<Block>> {
    if current_depth >= max_depth || visited.contains(&node) {
        return Ok(vec![]);
    }
    visited.insert(node);

    let children = db.query("
        SELECT out.*, sequence_weight
        FROM edge
        WHERE in = $node AND link_type = 'next'
        ORDER BY sequence_weight ASC
    ")
    .bind(("node", node))
    .await?;

    let mut result = Vec::new();
    for child in children {
        if child.block_type == BlockType::Structure(_) {
            let nested = traverse_recursive(db, child.id, max_depth, current_depth + 1, visited).await?;
            result.extend(nested);
        } else {
            result.push(child);
        }
    }
    Ok(result)
}
```

---

## 4. Smart Sections

### 4.1 Semantic Centroid

Calculate using **weighted mean** by importance (incoming links + 1):

```rust
pub fn calculate_weighted_centroid(
    blocks: &[Block],
    weights: &[f32],
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

### 4.2 Medoid

The **medoid** is the block closest to the centroid (most representative block).

---

## 5. Ghost Nodes

### 5.1 Detection Algorithm

1. Get all blocks in a Structure
2. Calculate semantic centroids for each section
3. For each consecutive pair, calculate distance
4. If distance > threshold, insert GhostNode

---

## 6. Git-like API

### 6.1 Commit

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

### 6.2 View (Ref)

```rust
pub enum View {
    Branch {
        name: ViewName,
        target: Ulid,
        is_head: bool,
    },
    Tag {
        name: ViewName,
        target: Ulid,
        message: String,
    },
}
```

### 6.3 WorkingSet (Index)

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

## 7. CLI Commands

### 7.1 Core Commands

| Command | Status | Description |
|---------|--------|-------------|
| `pkmai create` | ✅ | Create a new block |
| `pkmai list` | ✅ | List blocks with filtering |
| `pkmai show` | ✅ | Show block details |
| `pkmai link` | ✅ | Create links between blocks |
| `pkmai grep` | ✅ | Search block content |
| `pkmai traverse` | ✅ | Traverse the structural spine |
| `pkmai gravity-check` | ✅ | Check block connectivity |
| `pkmai toc` | ✅ | Generate table of contents |
| `pkmai synthesize` | ✅ | Synthesize document from structure |
| `pkmai ghost` | ✅ | Manage ghost nodes |
| `pkmai architect` | ✅ | Launch interactive TUI |
| `pkmai lint` | ✅ | Validate structural integrity |
| `pkmai db` | ✅ | Database management |
| `pkmai api` | ✅ | Start REST API server |

### 7.2 Version Control Commands

| Command | Git Equivalent | Status |
|---------|---------------|--------|
| `version status` | `git status` | ✅ |
| `version log` | `git log` | ✅ |
| `version diff` | `git diff` | ✅ |
| `version add` | `git add` | ✅ |
| `version commit` | `git commit` | ✅ |
| `version branch` | `git branch` | ✅ |
| `version checkout` | `git checkout` | ✅ |
| `version merge` | `git merge` | ✅ |
| `version tag` | `git tag` | ✅ |
| `version push` | `git push` | ✅ |
| `version pull` | `git pull` | ✅ |

**Total CLI Commands: 34 (100% implemented)**

---

## 8. MCP Tools

### 8.1 Block Tools (4)

| Tool | Parameters | Description |
|------|------------|-------------|
| `create_block` | `block_type`, `title`, `content?`, `tags?` | Create block |
| `get_block` | `id` | Get block by ULID |
| `search_blocks` | `query?`, `block_type?`, `tags?`, `limit?` | Search |
| `update_block` | `id`, `title?`, `content?`, `tags?` | Update |

### 8.2 Link Tools (3)

| Tool | Parameters | Description |
|------|------------|-------------|
| `create_link` | `from_id`, `to_id`, `link_type`, `weight?`, `context?` | Create link |
| `get_links` | `block_id`, `direction?` | Get links |
| `suggest_links` | `block_id`, `limit?` | AI suggestions |

### 8.3 Spine Tools (3)

| Tool | Parameters | Description |
|------|------------|-------------|
| `traverse_spine` | `root_id?`, `depth?`, `link_type?` | Traverse |
| `gravity_check` | `block_id`, `threshold?` | Check connectivity |
| `reorder_block` | `block_id`, `new_position`, `parent_id?` | Reorder |

### 8.4 Structure Tools (3)

| Tool | Parameters | Description |
|------|------------|-------------|
| `get_section_map` | `root_id` | Get hierarchy |
| `detect_gaps` | `section_id` | Detect missing |
| `list_ghosts` | `root_id?` | List placeholders |

### 8.5 Synthesis Tools (2)

| Tool | Parameters | Description |
|------|------------|-------------|
| `synthesize` | `structure_id`, `format?`, `template?` | Generate doc |
| `get_toc` | `structure_id` | Get TOC |

**Total MCP Tools: 15 (100% implemented)**

---

## 9. REST API

### 9.1 Endpoints

```
Base URL: /api/v1

HEALTH
  GET    /health              Health check

BLOCKS
  GET    /blocks              List blocks
  GET    /blocks/:id          Get block
  POST   /blocks              Create block
  PUT    /blocks/:id          Update block
  DELETE /blocks/:id          Delete block
  GET    /blocks/:id/history  Block history

STRUCTURES
  GET    /structures              List structures
  GET    /structures/:id          Get structure
  POST   /structures              Create structure
  PUT    /structures/:id          Update structure
  DELETE /structures/:id          Delete structure
  GET    /structures/:id/spine    Get spine

COMMITS
  GET    /commits                  List commits
  GET    /commits/:id              Get commit
  POST   /commits                  Create commit
  GET    /commits/:id/diff         View diff

VIEWS (REFS)
  GET    /views               List views
  GET    /views/:name         Get view
  POST   /views               Create view
  PUT    /views/:name         Update view
  DELETE /views/:name         Delete view

WORKINGSET (INDEX)
  GET    /working-set             Get working set
  POST   /working-set/stage       Add to staging
  POST   /working-set/unstage     Remove from staging
  POST   /working-set/commit      Create commit from staging
  DELETE /working-set             Discard

SYNC
  POST   /sync/push           Push to remote
  POST   /sync/pull           Pull from remote
  POST   /sync/fetch          Fetch metadata
  GET    /sync/status          Sync status
```

**Status: ~70% implemented (all major endpoints)**

---

## 10. Architecture

### 10.1 High-Level

```
┌─────────────────────────────────────────────────────────────┐
│                     CLI / TUI / MCP / API                    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      PKM-AI CORE LIBRARY                     │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐  │
│  │ Block CRUD  │ │ Edge Manager│ │ FractionalIndex     │  │
│  └─────────────┘ └─────────────┘ └─────────────────────┘  │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐  │
│  │ Spine       │ │ Lint Engine │ │ Ghost System        │  │
│  │ Traversal   │ │             │ │                     │  │
│  └─────────────┘ └─────────────┘ └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   SURREALDB DAEMON                           │
│   Unix socket: /tmp/pkmai-surreal.sock                     │
│   Tables: block, edge, commit, view, working_set           │
└─────────────────────────────────────────────────────────────┘
```

### 10.2 DAEMON Mode Pattern

SurrealDB runs as a server process, not embedded, to support multi-process concurrent access.

```rust
pub async fn get_daemon_connection() -> Result<Surreal<Unix>> {
    let socket_path = "/tmp/pkmai-surreal.sock";

    // Try to connect to existing daemon
    if Path::new(socket_path).exists() {
        if let Ok(db) = connect_to_socket(socket_path).await {
            if db.health().await.is_ok() {
                return Ok(db);
            }
        }
    }

    // Spawn new daemon
    Command::new("surrealdb")
        .args(["start", "--bind", &format!("unix:{}", socket_path), ...])
        .spawn()?;

    wait_for_socket(socket_path, Duration::from_secs(5)).await?;
    connect_to_socket(socket_path).await
}
```

---

## 10.5 Storage Layer Architecture

### 10.5.1 Strategy Pattern for Databases

PKM-AI uses the **Ports & Adapters (Hexagonal Architecture)** pattern to abstract storage:

```
┌─────────────────────────────────────────────────────────────┐
│                    APPLICATION CORE                         │
│  Domain Models (Block, Edge, Commit, View, WorkingSet)     │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   PORTS (Traits)                            │
│  ┌─────────────────────────────────────────────────────┐  │
│  │ BlockRepository: save, find_by_id, find_all, delete│  │
│  │ EdgeRepository: save, find_by_from, find_by_to      │  │
│  │ CommitRepository: save, find_by_id, find_by_author │  │
│  │ ViewRepository: save, find_by_name, find_all       │  │
│  └─────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   ADAPTERS (Implementations)                │
│                                                              │
│  ┌─────────────────┐  ┌─────────────────┐                  │
│  │ SurrealDBAdapter│  │ KuzuAdapter     │                  │
│  │ (Current)       │  │ (Planned)       │                  │
│  │                 │  │                 │                  │
│  │ Unix socket     │  │ Embedded WASM   │                  │
│  │ RocksDB         │  │ Cypher-like     │                  │
│  └─────────────────┘  └─────────────────┘                  │
│                                                              │
│  ┌─────────────────┐  ┌─────────────────┐                  │
│  │ InMemoryAdapter │  │ MockAdapter     │                  │
│  │ (Development)   │  │ (Testing)       │                  │
│  │                 │  │                 │                  │
│  │ HashMap-based   │  │ No DB required  │                  │
│  └─────────────────┘  └─────────────────┘                  │
└─────────────────────────────────────────────────────────────┘
```

### 10.5.2 Repository Traits

```rust
#[async_trait]
pub trait BlockRepository: Send + Sync {
    async fn save(&self, block: &Block) -> Result<Ulid, StorageError>;
    async fn find_by_id(&self, id: Ulid) -> Result<Option<Block>, StorageError>;
    async fn find_all(&self, filter: BlockFilter, pagination: Pagination) -> Result<Vec<Block>, StorageError>;
    async fn delete(&self, id: Ulid) -> Result<(), StorageError>;
    async fn find_by_type(&self, block_type: BlockType) -> Result<Vec<Block>, StorageError>;
    async fn search(&self, query: &str) -> Result<Vec<Block>, StorageError>;
}

#[async_trait]
pub trait EdgeRepository: Send + Sync {
    async fn save(&self, edge: &Edge) -> Result<Ulid, StorageError>;
    async fn find_by_id(&self, id: Ulid) -> Result<Option<Edge>, StorageError>;
    async fn find_by_from(&self, from: Ulid) -> Result<Vec<Edge>, StorageError>;
    async fn find_by_to(&self, to: Ulid) -> Result<Vec<Edge>, StorageError>;
    async fn find_by_type(&self, link_type: LinkType) -> Result<Vec<Edge>, StorageError>;
    async fn delete(&self, id: Ulid) -> Result<(), StorageError>;
    async fn delete_by_block(&self, block_id: Ulid) -> Result<u32, StorageError>;
}
```

### 10.5.3 Storage Error Type

```rust
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Query failed: {0}")]
    QueryFailed(String),
    #[error("Entity not found: {0}")]
    NotFound(String),
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
}
```

### 10.5.4 Factory Pattern

```rust
pub enum StorageBackend {
    SurrealDB { socket_path: String },
    Kuzu { path: String },
    InMemory,
    Mock,
}

pub struct StorageContainer {
    pub blocks: Arc<dyn BlockRepository>,
    pub edges: Arc<dyn EdgeRepository>,
    pub commits: Arc<dyn CommitRepository>,
    pub views: Arc<dyn ViewRepository>,
}

impl StorageFactory {
    pub async fn create(backend: StorageBackend) -> Result<StorageContainer, StorageError> {
        match backend {
            StorageBackend::SurrealDB { socket_path } => {
                let adapter = SurrealDBAdapter::new(&socket_path).await?;
                Ok(StorageContainer::new(adapter))
            }
            // ... other adapters
        }
    }
}
```

### 10.5.5 Current Status

| Adapter | Status | Notes |
|---------|--------|-------|
| SurrealDB | ✅ Default | Current production backend |
| Kuzu | 🔲 Planned | Embedded graph DB, WASM-ready |
| InMemory | 🔲 Planned | For development without DB |
| Mock | 🔲 Planned | For unit testing |

### 10.5.6 Migration Path

See [docs/reports/MIGRATION_PLAN.md](../reports/MIGRATION_PLAN.md) for detailed migration plan.

---

## 11. Implementation Status

### 11.1 Test Coverage

| Metric | Value |
|--------|-------|
| Total Tests | 229 |
| Passing | 229 |
| Failed | 0 |

### 11.2 Module Status

| Module | Tests | Status |
|--------|-------|--------|
| FractionalIndex | 3 | ✅ |
| Block Model | 5 | ✅ |
| GhostNode | 5+ | ✅ |
| SmartSection + Bloom | 10 | ✅ |
| GravityHooks | 10 | ✅ |
| Commit | 8 | ✅ |
| View | 10 | ✅ |
| WorkingSet | 12 | ✅ |
| LinkSuggester | 3 | ✅ |
| Synthesis | 5 | ✅ |
| MCP | 16 | ✅ |
| Traverse | 4 | ✅ |
| CLI Create | 11 | ✅ |
| CLI Link | 18 | ✅ |
| Embeddings | 5 | ✅ |
| Delta | 6 | ✅ |

### 11.3 Known Issues

| Issue | Severity | Workaround |
|-------|----------|------------|
| 11 compiler warnings | Minor | Cleanup before v1.0 |
| REST API validation incomplete | Medium | Add error handling |
| Weighted centroid not implemented | Medium | Use simple mean |

---

## Appendix A: Critical Design Decisions

### A.1 FractionalIndex over f32

Using `f32` for sequence ordering causes precision degradation. FractionalIndex with lexicographic strings never degrades.

### A.2 section_of Direction

**CORRECT:** Zettel → Structure (content points to container)
```sql
RELATE block:01HABC1->edge:section_of->block:01HSTRUCT
```

### A.3 traverse_spine Must Be Async

```rust
async fn traverse_spine(
    db: &SurrealDb,
    root: Ulid,
    max_depth: usize,
) -> Result<Vec<Block>>
```

---

**Last updated:** 2026-03-19
**Canonical source:** `src/models/` and this document