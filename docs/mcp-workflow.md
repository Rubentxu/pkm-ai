# MCP Intelligent Workflow Guide

## Table of Contents

1. [Overview](#overview)
2. [Core Philosophy: Git-like Workflow](#core-philosophy-git-like-workflow)
3. [Zettelkasten Workflow](#zettelkasten-workflow)
4. [All MCP Tools Reference](#all-mcp-tools-reference)
   - [Knowledge Operations](#knowledge-operations)
   - [Link Operations](#link-operations)
   - [Structure Operations](#structure-operations)
   - [Staging & Commit Operations](#staging--commit-operations)
5. [Block Types](#block-types)
6. [Enrichment Data](#enrichment-data)
7. [Design Principles](#design-principles)

---

## Overview

The MCP server provides AI agents with an intelligent knowledge capture workflow that mirrors Git's philosophy: nothing is committed until truly ready. This approach allows AI agents to make informed decisions about knowledge graph structure before finalizing changes.

---

## Core Philosophy: Git-like Workflow

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
│        │               │               │                    │
│        ▼               ▼               ▼                    │
│   ┌──────────────────────────────────────────┐             │
│   │  AI Analysis: Link suggestions, Gravity,  │             │
│   │  Type detection, Tag recommendations     │             │
│   └──────────────────────────────────────────┘             │
│                                                              │
│  Key Principle: Nothing auto-stages. AI decides when ready. │
└─────────────────────────────────────────────────────────────┘
```

---

## Zettelkasten Workflow

The MCP workflow implements the Zettelkasten method for building a connected knowledge graph:

### Step 1: Capture (Create Block)

Create atomic notes with enrichment data for intelligent linking:

```json
{
  "name": "create_block",
  "arguments": {
    "block_type": "fleeting",
    "title": "Idea about neural networks",
    "content": "Transformer architectures are powerful...",
    "enrich": true
  }
}
```

### Step 2: Think (Analyze Enrichment)

The AI analyzes returned enrichment data:
- **link_suggestions**: Existing blocks that relate to this content
- **tag_suggestions**: Relevant tags for categorization
- **gravity_info**: Connectivity metrics (hub detection)
- **type_suggestion**: Block type recommendation

### Step 3: Connect (Create Links)

Link related blocks using semantic link types:

```json
{
  "name": "create_link",
  "arguments": {
    "from": "01HX5V8F3K7QV9BZEC4N6P0M",
    "to": "01HX5V8F3K7QV9BZEC4N6P0A",
    "link_type": "extends"
  }
}
```

### Step 4: Organize (Structure Operations)

Use structure tools to maintain hierarchy:

```json
{
  "name": "reorder_block",
  "arguments": {
    "block_id": "01HX5V8F3K7QV9BZEC4N6P0M",
    "after_block_id": "01HX5V8F3K7QV9BZEC4N6P0B",
    "position": "after"
  }
}
```

### Step 5: Review (Detect Gaps)

Identify missing connections and sections:

```json
{
  "name": "detect_gaps",
  "arguments": {
    "root_block_id": "01HX5V8F3K7QV9BZEC4N6P0Z"
  }
}
```

### Step 6: Stage (Working Set)

Add processed blocks to staging area:

```json
{
  "name": "stage_block",
  "arguments": {
    "block_id": "01HX5V8F3K7QV9BZEC4N6P0M"
  }
}
```

### Step 7: Commit (Permanent Snapshot)

Finalize changes when truly ready:

```json
{
  "name": "commit_changes",
  "arguments": {
    "message": "Add transformer architecture notes with links to attention mechanisms"
  }
}
```

---

## All MCP Tools Reference

### Knowledge Operations

#### 1. create_block

Creates a new knowledge block with optional AI enrichment.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| block_type | string | Yes | Type: fleeting, literature, permanent, structure, hub, task, reference, outline, ghost |
| title | string | Yes | Block title |
| content | string | No | Block content |
| tags | string[] | No | Initial tags |
| enrich | boolean | No | If true, returns AI enrichment data (default: false) |

**Response (with enrich=true):**

```json
{
  "id": "01HX5V8F3K7QV9BZEC4N6P0M",
  "block_type": "fleeting",
  "title": "Idea about neural networks",
  "content": "Transformer architectures are powerful...",
  "tags": [],
  "created_at": "2026-03-20T10:30:00Z",
  "updated_at": "2026-03-20T10:30:00Z",

  "// AI Enrichment Data (TOP-LEVEL FIELDS, NOT nested under 'enrichment')",

  "link_suggestions": [
    {
      "target_id": "01HX5V8F3K7QV9BZEC4N6P0A",
      "link_type": "extends",
      "confidence": 0.85,
      "reason": "Content extends discussion on attention mechanisms"
    }
  ],
  "tag_suggestions": ["machine-learning", "transformers", "deep-learning"],
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
```

---

#### 2. get_block

Retrieves a block by ID with optional content.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| block_id | string | Yes | ULID of the block to retrieve |
| include_content | boolean | No | Include full content (default: false) |

**Example:**

```json
{
  "name": "get_block",
  "arguments": {
    "block_id": "01HX5V8F3K7QV9BZEC4N6P0M",
    "include_content": true
  }
}
```

---

#### 3. search_blocks

Full-text search across blocks by query, type, or tags.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| query | string | No | Search query text |
| block_type | string | No | Filter by block type |
| tags | string[] | No | Filter by tags (AND logic) |
| limit | number | No | Max results (default: 50) |
| offset | number | No | Pagination offset (default: 0) |

**Example:**

```json
{
  "name": "search_blocks",
  "arguments": {
    "query": "transformer architecture",
    "block_type": "literature",
    "tags": ["machine-learning"],
    "limit": 10
  }
}
```

**Response:**

```json
{
  "blocks": [
    {
      "id": "01HX5V8F3K7QV9BZEC4N6P0M",
      "block_type": "literature",
      "title": "Attention is All You Need",
      "tags": ["machine-learning", "transformers"],
      "created_at": "2026-03-20T10:30:00Z"
    }
  ],
  "total": 1,
  "limit": 10,
  "offset": 0
}
```

---

#### 4. update_block

Updates block content and properties.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| block_id | string | Yes | ULID of block to update |
| title | string | No | New title |
| content | string | No | New content |
| tags | string[] | No | Replace all tags |
| block_type | string | No | Change block type |

**Example:**

```json
{
  "name": "update_block",
  "arguments": {
    "block_id": "01HX5V8F3K7QV9BZEC4N6P0M",
    "title": "Updated: Transformer Architecture",
    "tags": ["machine-learning", "transformers", "nlp"],
    "block_type": "literature"
  }
}
```

---

### Link Operations

#### 5. create_link

Creates a directed link between two blocks with a semantic link type.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| from | string | Yes | Source block ULID |
| to | string | Yes | Target block ULID |
| link_type | string | Yes | Type: extends, refines, contradicts, questions, supports, references, related, similar_to, section_of, next |

**Example:**

```json
{
  "name": "create_link",
  "arguments": {
    "from": "01HX5V8F3K7QV9BZEC4N6P0M",
    "to": "01HX5V8F3K7QV9BZEC4N6P0A",
    "link_type": "extends"
  }
}
```

**Response:**

```json
{
  "success": true,
  "link_id": "link_01HX5V8F3K7QV9BZEC4N6P0Z",
  "from": "01HX5V8F3K7QV9BZEC4N6P0M",
  "to": "01HX5V8F3K7QV9BZEC4N6P0A",
  "link_type": "extends"
}
```

---

#### 6. get_links

Queries links from or to a specific block.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| block_id | string | Yes | ULID of the block |
| direction | string | No | "outgoing", "incoming", or "both" (default: "both") |
| link_type | string | No | Filter by link type |

**Example:**

```json
{
  "name": "get_links",
  "arguments": {
    "block_id": "01HX5V8F3K7QV9BZEC4N6P0M",
    "direction": "outgoing",
    "link_type": "extends"
  }
}
```

---

#### 7. suggest_links

AI-powered link suggestions based on content similarity and semantic analysis.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| block_id | string | Yes | ULID of block to get suggestions for |
| limit | number | No | Max suggestions (default: 10) |

**Example:**

```json
{
  "name": "suggest_links",
  "arguments": {
    "block_id": "01HX5V8F3K7QV9BZEC4N6P0M",
    "limit": 5
  }
}
```

**Response:**

```json
{
  "block_id": "01HX5V8F3K7QV9BZEC4N6P0M",
  "suggestions": [
    {
      "target_id": "01HX5V8F3K7QV9BZEC4N6P0A",
      "link_type": "extends",
      "confidence": 0.85,
      "reason": "Semantic similarity in attention mechanism discussion"
    }
  ]
}
```

---

### Structure Operations

#### 8. traverse_spine

Traverses the hierarchical spine structure of blocks.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| root_block_id | string | Yes | Starting block ULID |
| direction | string | No | "forward" or "backward" (default: "forward") |
| depth | number | No | Max depth to traverse (default: unlimited) |

**Example:**

```json
{
  "name": "traverse_spine",
  "arguments": {
    "root_block_id": "01HX5V8F3K7QV9BZEC4N6P0Z",
    "direction": "forward",
    "depth": 3
  }
}
```

---

#### 9. gravity_check

Checks connectivity metrics for a block (hub detection).

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| block_id | string | Yes | ULID of block to analyze |
| depth | number | No | Analysis depth (default: 1) |

**Example:**

```json
{
  "name": "gravity_check",
  "arguments": {
    "block_id": "01HX5V8F3K7QV9BZEC4N6P0M"
  }
}
```

**Response:**

```json
{
  "block_id": "01HX5V8F3K7QV9BZEC4N6P0M",
  "gravity_score": 3.2,
  "direct_links": {
    "outgoing": 5,
    "incoming": 2
  },
  "network_metrics": {
    "betweenness_centrality": 0.15,
    "page_rank": 0.08
  }
}
```

---

#### 10. reorder_block

Reorders blocks within the spine hierarchy.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| block_id | string | Yes | ULID of block to reorder |
| after_block_id | string | No | ULID of block to place after |
| before_block_id | string | No | ULID of block to place before |
| parent_id | string | No | New parent block ID |

**Example:**

```json
{
  "name": "reorder_block",
  "arguments": {
    "block_id": "01HX5V8F3K7QV9BZEC4N6P0M",
    "after_block_id": "01HX5V8F3K7QV9BZEC4N6P0B",
    "position": "after"
  }
}
```

---

#### 11. get_section_map

Retrieves the hierarchical section tree starting from a block.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| root_block_id | string | No | Starting block (default: root) |
| max_depth | number | No | Maximum tree depth (default: unlimited) |

**Example:**

```json
{
  "name": "get_section_map",
  "arguments": {
    "root_block_id": "01HX5V8F3K7QV9BZEC4N6P0Z",
    "max_depth": 3
  }
}
```

**Response:**

```json
{
  "root_id": "01HX5V8F3K7QV9BZEC4N6P0Z",
  "sections": [
    {
      "id": "01HX5V8F3K7QV9BZEC4N6P0M",
      "title": "Section Title",
      "children": []
    }
  ]
}
```

---

#### 12. detect_gaps

Identifies missing sections in the knowledge graph.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| root_block_id | string | No | Root block to analyze |
| expected_sections | string[] | No | Expected section titles |

**Example:**

```json
{
  "name": "detect_gaps",
  "arguments": {
    "root_block_id": "01HX5V8F3K7QV9BZEC4N6P0Z",
    "expected_sections": ["Introduction", "Methods", "Results", "Conclusion"]
  }
}
```

**Response:**

```json
{
  "root_block_id": "01HX5V8F3K7QV9BZEC4N6P0Z",
  "gaps": [
    {
      "expected": "Methods",
      "status": "missing",
      "suggestion": "Consider adding a Methods section"
    }
  ]
}
```

---

#### 13. list_ghosts

Lists ghost nodes with optional status filtering.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| status | string | No | Filter: predicted, confirmed, resolved |
| block_type | string | No | Filter by block type |

**Example:**

```json
{
  "name": "list_ghosts",
  "arguments": {
    "status": "predicted",
    "block_type": "outline"
  }
}
```

---

#### 14. synthesize

Generates a document from block structure.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| root_block_id | string | Yes | Root block for synthesis |
| format | string | No | Output format: markdown, html, json (default: markdown) |
| include_metadata | boolean | No | Include metadata in output (default: true) |

**Example:**

```json
{
  "name": "synthesize",
  "arguments": {
    "root_block_id": "01HX5V8F3K7QV9BZEC4N6P0Z",
    "format": "markdown",
    "include_metadata": true
  }
}
```

---

#### 15. get_toc

Retrieves the table of contents for a block hierarchy.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| root_block_id | string | No | Root block (default: document root) |
| max_depth | number | No | Maximum heading depth (default: 6) |

**Example:**

```json
{
  "name": "get_toc",
  "arguments": {
    "root_block_id": "01HX5V8F3K7QV9BZEC4N6P0Z",
    "max_depth": 3
  }
}
```

**Response:**

```json
{
  "root_id": "01HX5V8F3K7QV9BZEC4N6P0Z",
  "toc": [
    {
      "id": "01HX5V8F3K7QV9BZEC4N6P0M",
      "title": "Introduction",
      "level": 1,
      "children": [
        {
          "id": "01HX5V8F3K7QV9BZEC4N6P0N",
          "title": "Background",
          "level": 2,
          "children": []
        }
      ]
    }
  ]
}
```

---

### Staging & Commit Operations

#### 16. stage_block

Adds a block to the working set staging area.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| block_id | string | Yes | ULID of block to stage |

**Example:**

```json
{
  "name": "stage_block",
  "arguments": {
    "block_id": "01HX5V8F3K7QV9BZEC4N6P0M"
  }
}
```

**Response:**

```json
{
  "success": true,
  "message": "Block 01HX5V8F3K7QV9BZEC4N6P0M staged for commit"
}
```

---

#### 17. commit_changes

Creates a commit from all staged changes.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| message | string | Yes | Commit message describing the changes |
| author | string | No | Author name (default: "user") |

**Example:**

```json
{
  "name": "commit_changes",
  "arguments": {
    "message": "Add transformer architecture notes with links to attention mechanisms",
    "author": "ai-agent-1"
  }
}
```

**Response:**

```json
{
  "success": true,
  "commit_id": "01HX5V8F3K7QV9BZEC4N6P0Z",
  "message": "Add transformer architecture notes with links to attention mechanisms",
  "blocks_staged": 1,
  "blocks_committed": 1
}
```

---

#### 18. get_working_set_status

Returns current staging area status.

**Parameters:**

None.

**Example:**

```json
{
  "name": "get_working_set_status",
  "arguments": {}
}
```

**Response:**

```json
{
  "staged_blocks": ["01HX5V8F3K7QV9BZEC4N6P0M"],
  "staged_edges": [],
  "removed_blocks": [],
  "removed_edges": [],
  "is_empty": false
}
```

---

## Block Types

| Type | Purpose | Auto-stage? |
|------|---------|------------|
| fleeting | Quick captures, temporary notes | No |
| literature | Processed notes from sources | No |
| permanent | Final, polished knowledge | No |
| structure | Organizing blocks (TOC, index) | No |
| hub | Central topic connectors | No |
| task | Action items, TODOs | No |
| reference | External references, citations | No |
| outline | Hierarchical outlines | No |
| ghost | Placeholder predictions | No |

---

## Enrichment Data

When `enrich=true` is passed to `create_block`, the following top-level fields are returned (NOT nested under "enrichment"):

### link_suggestions

```json
{
  "target_id": "01HX...",
  "link_type": "extends|refines|contradicts|questions|supports|references|related|similar_to|section_of|next",
  "confidence": 0.0-1.0,
  "reason": "Human-readable explanation"
}
```

### tag_suggestions

Array of suggested tags sorted by relevance.

### gravity_info

```json
{
  "gravity_score": 0.0-10.0,
  "outgoing_links": number,
  "incoming_links": number
}
```

Higher gravity = more connected (hub-like).

### type_suggestion

```json
{
  "suggested_type": "literature|permanent|...",
  "confidence": 0.0-1.0,
  "reason": "Why this type is suggested"
}
```

---

## Design Principles

1. **Explicit over Implicit**: Nothing happens automatically
2. **Enrichment on Demand**: AI chooses when to get context
3. **Atomic Commits**: Changes are staged until truly ready
4. **Full Context**: AI gets link suggestions, gravity, type hints before committing
5. **Reversible**: Working set allows discarding before commit
6. **Zettelkasten Method**: Atomic notes, semantic links, organic growth
7. **Ghost Nodes**: Predictive placeholders for future connections

---

## Example: Complete AI Knowledge Construction

```python
# 1. Create a fleeting note with enrichment
result = mcp.call_tool("create_block", {
    "block_type": "fleeting",
    "title": "Attention is all you need",
    "content": "The Transformer architecture...",
    "enrich": True
})

# 2. AI receives suggestions at TOP LEVEL (not under "enrichment"):
# - result.link_suggestions
# - result.tag_suggestions
# - result.gravity_info
# - result.type_suggestion

# 3. AI creates semantic links
mcp.call_tool("create_link", {
    "from": result.id,
    "to": "existing_nn_block_id",
    "link_type": "extends"
})

# 4. Check structure and detect gaps
gaps = mcp.call_tool("detect_gaps", {
    "root_block_id": result.id,
    "expected_sections": ["Abstract", "Introduction", "Methods"]
})

# 5. Stage the block
mcp.call_tool("stage_block", {"block_id": result.id})

# 6. Check staging status
status = mcp.call_tool("get_working_set_status", {})

# 7. Commit when truly ready
mcp.call_tool("commit_changes", {
    "message": "Add Transformer paper notes with neural network links"
})
```
