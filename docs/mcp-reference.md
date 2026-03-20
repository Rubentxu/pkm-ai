# MCP Reference for PKM

**Status:** Implemented
**Version:** 1.0
**Date:** 2026-03-20

---

## Overview

The PKM MCP server provides AI agents with access to a Zettelkasten-style knowledge graph via the [Model Context Protocol](https://modelcontextprotocol.io/). It exposes **15 tools** across 5 categories for managing atomic notes, semantic links, and structured documents.

**Transport:** stdio (JSON-RPC)
**Implementation:** `src/ai/mcp.rs` using `rmcp` crate

---

## Quick Start

### Running the Server

```bash
# Via cargo
cargo run --release --bin pkm-ai -- mcp

# Or via CLI alias
pkm-ai mcp
---

## Tool Summary

| Category | Tools | Count |
|----------|-------|-------|
| Block | `create_block`, `get_block`, `search_blocks`, `update_block` | 4 |
| Link | `create_link`, `get_links`, `suggest_links` | 3 |
| Spine | `traverse_spine`, `gravity_check`, `reorder_block` | 3 |
| Structure | `get_section_map`, `detect_gaps`, `list_ghosts` | 3 |
| Synthesis | `synthesize`, `get_toc` | 2 |

---

## Block Tools

### create_block

Creates a new block in the knowledge graph.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `block_type` | string | Yes | Type: `fleeting`, `literature`, `permanent`, `structure`, `hub`, `task`, `reference`, `outline`, `ghost` |
| `title` | string | No | Block title (default: "Untitled") |
| `content` | string | No | Content in Markdown format |
| `tags` | string[] | No | Tags for classification |

**Response:**
```json
{
  "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "block_type": "permanent",
  "title": "My Zettel",
  "created_at": "2026-03-20T10:00:00Z"
}
```

---

### get_block

Retrieves a block by its ULID.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `block_id` | string | Yes | - | Block ULID |
| `include_content` | boolean | No | `true` | Include full content |

**Response:**
```json
{
  "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "block_type": "permanent",
  "title": "My Zettel",
  "content": "# Atomic Note\n\nThis is my atomic thought.",
  "tags": ["idea"],
  "metadata": {},
  "created_at": "2026-03-20T10:00:00Z",
  "updated_at": "2026-03-20T10:00:00Z"
}
```

---

### search_blocks

Searches blocks by query, type, or returns all blocks.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `query` | string | No | - | Full-text search query |
| `block_type` | string | No | - | Filter by block type |
| `limit` | integer | No | `20` | Max results (max 100) |

**Response:**
```json
{
  "blocks": [
    {
      "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
      "block_type": "permanent",
      "title": "My Zettel",
      "content": "First 200 chars...",
      "tags": ["idea"],
      "created_at": "2026-03-20T10:00:00Z"
    }
  ],
  "count": 1
}
```

---

### update_block

Updates a block's content or properties.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `block_id` | string | Yes | Block ULID |
| `content` | string | No | New content |
| `properties` | object | No | Key-value pairs for metadata |

**Response:** Plain text confirmation

```
Block 01ARZ3NDEKTSV4RRFFQ69G5FAV updated successfully
```

---

## Link Tools

### create_link

Creates a directed link between two blocks.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `source_id` | string | Yes | Source block ULID |
| `target_id` | string | Yes | Target block ULID |
| `link_type` | string | Yes | Type of link |

**Link Types:**
- Structural: `section_of`, `subsection_of`, `ordered_child`, `next`, `next_sibling`, `first_child`, `contains`, `parent`
- Semantic: `extends`, `refines`, `contradicts`, `questions`, `supports`, `references`
- Similarity: `related`, `similar_to`, `ai_suggested`

**Response:**
```json
{
  "id": "01ARZ3NDEKTSV4RRFFQ69G5FAW",
  "source_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "target_id": "01ARZ3NDEKTSV4RRFFQ69G5FAZ",
  "link_type": "supports"
}
```

---

### get_links

Gets all links from or to a block.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `block_id` | string | Yes | - | Block ULID |
| `link_types` | string[] | No | - | Filter by link types |
| `direction` | string | No | `both` | Direction: `outgoing`, `incoming`, `both` |

**Response:**
```json
{
  "edges": [
    {
      "id": "01ARZ3NDEKTSV4RRFFQ69G5FAW",
      "from": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
      "to": "01ARZ3NDEKTSV4RRFFQ69G5FAZ",
      "link_type": "supports",
      "sequence_weight": 1.0,
      "created_at": "2026-03-20T10:00:00Z"
    }
  ],
  "count": 1
}
```

---

### suggest_links

Suggests links for a block using AI-powered analysis.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `block_id` | string | Yes | - | Block ULID |
| `confidence_threshold` | number | No | `0.5` | Minimum confidence (0.0-1.0) |
| `limit` | integer | No | `10` | Max suggestions |

**Response:**
```json
{
  "suggestions": [
    {
      "target_id": "01ARZ3NDEKTSV4RRFFQ69G5FAZ",
      "link_type": "related",
      "confidence": 0.85,
      "reason": "Both notes discuss neural network architectures"
    }
  ],
  "count": 1
}
```

---

## Spine Tools

### traverse_spine

Traverses the structural spine from a root block.

**Parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `root_id` | string | No | - | Root structure block ULID (null for full spine) |
| `max_depth` | integer | No | `0` | Max traversal depth (0 = unlimited) |

**Response:**
```json
{
  "root_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "blocks": [
    {
      "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
      "block_type": "structure",
      "title": "My Document"
    }
  ],
  "total_count": 2,
  "depth": 1
}
```

---

### gravity_check

Checks the connectivity (gravity) of a block in the knowledge graph.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `block_id` | string | Yes | Block ULID |

**Response:**
```json
{
  "block_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "gravity_score": 15.0,
  "outgoing_links": 8,
  "incoming_links": 7,
  "total_connections": 15
}
```

---

### reorder_block

Reorders a block in the structural spine.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `block_id` | string | Yes | Block ULID to reorder |
| `after_id` | string | No* | Block ULID to place after |
| `before_id` | string | No* | Block ULID to place before |

*At least one of `after_id` or `before_id` must be provided.

**Response:**
```json
{
  "block_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "after_id": "01ARZ3NDEKTSV4RRFFQ69G5FAY",
  "before_id": "01ARZ3NDEKTSV4RRFFQ69G5FAZ",
  "sequence_weight": 1.5,
  "message": "Block reordered successfully"
}
```

---

## Structure Tools

### get_section_map

Gets the section hierarchy from a root structure block.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `root_id` | string | Yes | Root structure block ULID |

**Response:**
```json
{
  "root_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "sections": [
    {
      "id": "01ARZ3NDEKTSV4RRFFQ69G5FAZ",
      "title": "Chapter 1",
      "block_type": "permanent",
      "sequence_weight": 1.0
    }
  ],
  "count": 1
}
```

---

### detect_gaps

Detects gaps (ghost nodes) in a section using AI analysis.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `section_id` | string | Yes | Section block ULID |

**Response:**
```json
{
  "section_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "detected_gaps": [
    {
      "id": "01ARZ3NDEKTSV4RRFFQ69G5FB0",
      "description": "Missing section on performance optimization",
      "confidence": 0.85,
      "status": "detected",
      "ai_rationale": "Topic mentioned in intro but never expanded",
      "position_hint": {
        "after": "01ARZ3NDEKTSV4RRFFQ69G5FAZ",
        "before": null,
        "parent_section": "01ARZ3NDEKTSV4RRFFQ69G5FAV"
      }
    }
  ],
  "count": 1
}
```

---

### list_ghosts

Lists ghost nodes (content placeholders) in the knowledge graph.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `status` | string | No | Filter: `detected`, `acknowledged`, `in_progress`, `filled`, `dismissed` |
| `confidence_below` | number | No | Filter by confidence threshold |

**Response:**
```json
{
  "ghosts": [
    {
      "id": "01ARZ3NDEKTSV4RRFFQ69G5FB0",
      "title": "Performance Optimization",
      "content": "",
      "ai_confidence": 0.85,
      "status": "detected",
      "created_at": "2026-03-20T10:00:00Z"
    }
  ],
  "count": 1
}
```

---

## Synthesis Tools

### synthesize

Synthesizes a document from a structure.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `structure_id` | string | Yes | Structure block ULID |
| `template` | string | No | Template name (default: "default") |
| `output_path` | string | No | Output file path |

**Response:**
```json
{
  "structure_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "title": "My Document",
  "format": "markdown",
  "blocks_used": 5,
  "blocks_total": 7,
  "content": "# My Document\n\nSynthesized content...",
  "message": "Synthesis completed successfully"
}
```

---

### get_toc

Gets the table of contents for a structure.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `structure_id` | string | Yes | Structure block ULID |

**Response:**
```json
{
  "structure_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "toc": [
    {
      "id": "01ARZ3NDEKTSV4RRFFQ69G5FAZ",
      "title": "Chapter 1",
      "level": 1
    },
    {
      "id": "01ARZ3NDEKTSV4RRFFQ69G5FB0",
      "title": "Section 1.1",
      "level": 2
    }
  ],
  "count": 2
}
```

---

## Block Types Reference

| Type | Description | Aliases |
|------|-------------|---------|
| `fleeting` | Temporary notes (captured quickly) | `f` |
| `literature` | Reference material from external sources | `l` |
| `permanent` | Atomic Zettelkasten notes | `p` |
| `structure` | Structural containers (documents, books) | `s`, `index`, `moc` |
| `hub` | Central topic nodes | `h` |
| `task` | Action items and todos | `t` |
| `reference` | External references | `r` |
| `outline` | Hierarchical outlines | `o` |
| `ghost` | Placeholder for missing content | `g` |

---

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| `-32600` | Parse Error | Invalid JSON |
| `-32601` | Method Not Found | Unknown tool name |
| `-32602` | Invalid Params | Invalid parameters |
| `-32603` | Internal Error | Server-side error |

---

## Agent Configuration Examples

### Claude Desktop (claude_desktop_config.json)

```json
{
  "mcpServers": {
    "pkm": {
      "command": "cargo",
      "args": ["run", "--release", "--bin", "pkm-ai", "--", "mcp"],
      "env": {},
      "workingDirectory": "/path/to/hodei-pkm"
    }
  }
}
```

### Claude Code (settings.json)

```json
{
  "mcpServers": {
    "pkm": {
      "command": "cargo",
      "args": ["run", "--release", "--bin", "pkm-ai", "--", "mcp"],
      "workingDirectory": "/path/to/hodei-pkm"
    }
  }
}
```

### Engram Memory Configuration

```json
{
  "memory": {
    "pkm": {
      "type": "mcp",
      "command": "cargo",
      "args": ["run", "--release", "--bin", "pkm-ai", "--", "mcp"],
      "workingDirectory": "/path/to/hodei-pkm"
    }
  }
}
```

---

## Implementation Status

| Tool | Status | Notes |
|------|--------|-------|
| `create_block` | Implemented | Fully functional |
| `get_block` | Implemented | Fully functional |
| `search_blocks` | Implemented | Fully functional |
| `update_block` | Implemented | Fully functional |
| `create_link` | Implemented | Fully functional |
| `get_links` | Implemented | Fully functional |
| `suggest_links` | Implemented | Uses LinkSuggester |
| `traverse_spine` | Implemented | Fully functional |
| `gravity_check` | Implemented | Fully functional |
| `reorder_block` | Implemented | Fully functional |
| `get_section_map` | Implemented | Fully functional |
| `detect_gaps` | Implemented | Uses GhostDetector |
| `list_ghosts` | Implemented | Fully functional |
| `synthesize` | Implemented | Fully functional |
| `get_toc` | Implemented | Fully functional |

---

## Related Documentation

- [MCP README](docs/mcp/README.md) - Full server documentation
- [MCP API](docs/mcp/API.md) - Detailed API reference
- [MCP SKILL](docs/mcp/SKILL.md) - Agent skill documentation
- [MCP Use Cases](docs/mcp/USE_CASES.md) - Practical usage examples

---

**Last updated:** 2026-03-20
