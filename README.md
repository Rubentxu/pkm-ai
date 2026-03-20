# Pkm-ai

> **High-performance PKM with Zettelkasten + SurrealDB + AI**

A Personal Knowledge Management system treating **ORDER and STRUCTURE** as first-class citizens.

## Features

### 🧱 Block-Atom Model
- Every piece of content is a **Block** with ULID (chronologically sortable)
- Block types: Fleeting, Literature, Permanent, Structure, Hub, Task, Reference, Outline, Ghost
- Everything is linkable, everything has metadata

### 📐 Structural Spine (Folgezettel Digital)
- **FractionalIndex**: Lexicographic string ordering (never degrades with insertions)
- **NEXT links**: Deterministic path through knowledge
- **Gravity hooks**: Semantic clustering
- **Structural linting**: Detect gaps, orphans, anachronisms

### 🧠 Smart Sections
- Sections with **Intent**, **Boundary Constraints**, **Semantic Centroid**
- Vacancy tracking: Full, NearlyFull, Partial, Sparse, Empty
- Coherence scoring
- Gravity hooks for semantic attraction

### 👻 Ghost Nodes
- AI-detected content gaps
- Predictive placeholders
- Fill or dismiss workflow

### 📝 Document Synthesis (PRIORITY #1)
- Convert fragments into complete documents
- Generate professional PDFs with Typst
- Table of Contents generation with completion tracking
- Template system with multiple output formats (Markdown, HTML, PDF)
- FractionalIndex for never-degrading order

### 🤖 AI Integration
- Link suggestions (semantic similarity)
- Ghost node detection
- Structure generation
- MCP Protocol for AI agents

## Installation

```bash
# Clone
git clone https://github.com/your-org/pkm-ai
cd pkm-ai

# Build
cargo build --release

# Install to PATH
cargo install --path .
```

## Quick Start

```bash
# Initialize database
pkmai db init

# Create blocks
pkmai create -t permanent --title "Actor Model Fundamentals" \
  --content "The actor model treats actors as the universal primitives..." \
  --tags "actor-model,rust,concurrency"

# Create structure
pkmai create -t structure --title "Nexus-WASM Architecture MOC"

# Link blocks to structure
pkmai link <block-id> <structure-id> --type section_of --weight 1.0

# Generate TOC
pkmai toc <structure-id>

# Synthesize document
pkmai synthesize <structure-id> --template technical-whitepaper --output pdf
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `pkmai create` | Create a new block |
| `pkmai list` | List blocks |
| `pkmai show` | Show block details |
| `pkmai link` | Link blocks |
| `pkmai traverse` | Traverse structural spine |
| `pkmai gravity-check` | Check semantic clustering |
| `pkmai toc` | Generate table of contents |
| `pkmai synthesize` | Synthesize document |
| `pkmai ghost` | Manage ghost nodes |
| `pkmai architect` | Launch interactive TUI |
| `pkmai lint` | Validate structural integrity |

## Architecture

```
pkm-ai/
├── src/
│   ├── models/          # Domain models
│   │   ├── block.rs     # Block model
│   │   ├── edge.rs      # Edge model
│   │   ├── smart_section.rs
│   │   ├── ghost_node.rs
│   │   └── structural_spine.rs
│   ├── db/              # Database layer
│   │   ├── schema.rs    # SurrealDB schema
│   │   ├── repository.rs
│   │   └── connection.rs
│   ├── cli/             # CLI commands
│   │   └── commands/
│   ├── ai/              # AI integration
│   │   ├── embeddings.rs
│   │   ├── link_suggester.rs
│   │   ├── ghost_detector.rs
│   │   └── mcp.rs
│   ├── synthesis/       # Document synthesis
│   │   ├── toc.rs
│   │   ├── template.rs
│   │   └── typst_renderer.rs
│   ├── spine/           # Structural spine
│   │   ├── traversal.rs
│   │   ├── linting.rs
│   │   └── rebalancing.rs
│   └── tui/             # Terminal UI
└── docs/
    └── analysis/
        ├── pkm-zettelkasten-rust-analysis.md
        ├── CLI-STRUCTURAL-SPINE-DESIGN.md
        └── STRUCTURAL-SPINE-SECCIONES-INTELIGENTES.md
```

## Key Concepts

### Structural Spine

The **Structural Spine** is the backbone of ordered knowledge:

```
Block A (weight: 1.0) → Block B (weight: 2.0) → Block C (weight: 3.0)
        ↓                                           ↓
   Block A1 (1.5)                             Block C1 (2.5)
```

- Deterministic traversal
- Flexible insertion (1.5 fits between 1.0 and 2.0)
- No re-numbering required

### Ghost Nodes

AI detects gaps in knowledge:

```
[Block A: Intro] → [Ghost: Missing explanation] → [Block B: Advanced]
```

- Confidence score
- Fill or dismiss workflow
- Expected keywords

### Document Synthesis

Convert Zettelkasten fragments into complete documents:

```
50 Zettels → Structure Note → TOC → Professional PDF
```

The synthesis pipeline:
1. **Generate TOC** - Build hierarchical table of contents from Structure blocks
2. **Order blocks** - Use FractionalIndex for never-degrading sequence order
3. **Apply template** - Render with Mustache-style templates
4. **Export format** - Markdown, HTML, or PDF via Typst

Example:
```bash
# Generate TOC for a structure
pkmai toc 01AR6M3X5FKPB3K5S9R2M4N7PQ

# Synthesize as Markdown
pkmai synthesize 01AR6M3X5FKPB3K5S9R2M4N7PQ --output markdown

# Synthesize as PDF (requires Typst)
pkmai synthesize 01AR6M3X5FKPB3K5S9R2M4N7PQ --output pdf --template technical-whitepaper
```

## Zettelkasten Workflow

The Zettelkasten workflow in PKM-AI follows the **Atomic Notes** principle:

```
┌─────────────────────────────────────────────────────────────────────┐
│                    ZETTELKASTEN WORKFLOW                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  1. CAPTURE (Fleeting Notes)                                        │
│     └─→ Raw ideas, quick captures, meeting notes                    │
│         pkmai create -t fleeting "Maybe we should use SAB..."       │
│                                                                      │
│  2. PROCESS (Literature Notes)                                      │
│     └─→ Processed, linked, categorized ideas                         │
│         pkmai create -t literature --title "SharedArrayBuffer..."    │
│                                                                      │
│  3. ORGANIZE (Permanent Notes)                                      │
│     └─→ Atomic, interconnected knowledge units                       │
│         pkmai create -t permanent --title "Actor Model in WASM"      │
│                                                                      │
│  4. STRUCTURE (MOC/Hub)                                            │
│     └─→ Collections organizing related permanent notes                │
│         pkmai create -t structure --title "Nexus-WASM Architecture"  │
│         pkmai link <note-id> <moc-id> --type section_of              │
│                                                                      │
│  5. SYNTHESIZE (Document)                                          │
│     └─→ Complete documents from Zettelkasten                         │
│         pkmai synthesize <moc-id> --output pdf                       │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### Link Types

| Link Type | Use Case |
|-----------|----------|
| `extends` | Elaborates on an idea |
| `refines` | Improves precision |
| `contradicts` | Opposes or challenges |
| `supports` | Provides evidence |
| `references` | Cites or mentions |
| `section_of` | Block belongs to a Structure (MOC) |
| `ordered_child` | Child with explicit order |
| `next` | Structural Spine: deterministic sequence |

## Configuration

Environment variables:

```bash
PKMAI_DB_PATH=~/.pkm-ai/knowledge.db
PKMAI_LOG_LEVEL=info
```

## Roadmap

- [x] Core models
- [x] SurrealDB schema
- [x] CLI structure
- [x] Document Synthesis (TOC, templates, Typst PDF)
- [x] FractionalIndex ordering
- [x] Structural Spine traversal
- [x] Ghost Nodes
- [ ] AI embeddings integration
- [ ] TUI with ratatui
- [ ] MCP Protocol server

## Documentation

- [Architecture](docs/arquitectura/TECNICA.md) - Technical specification
- [API Design](docs/arquitectura/GIT-LIKE-API.md) - Git-like API design
- [Synthesis Module](docs/arquitectura/SYNTHESIS.md) - Document synthesis guide
- [PRD](docs/arquitectura/PRD.md) - Product requirements

## License

MIT
