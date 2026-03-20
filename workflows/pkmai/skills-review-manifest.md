# PKM-AI Skills Manifest Review

**Date**: 2026-03-20
**Reviewer**: Claude Code
**Status**: DRAFT

---

## Executive Summary

The current `SKILL-MANIFEST.md` documents 12 skills (9 SDD phases + 3 utilities), but the PKM-AI system exposes **19 MCP tools** plus **7 session CLI commands** with rich Zettelkasten integration. The manifest does not reflect actual implementation.

---

## Part 1: Current Manifest Issues

### 1.1 Block Types: Manifest vs Implementation

| Manifest Says | Actual Implementation |
|---------------|----------------------|
| `structure` | `Fleeting` |
| `outline` | `Literature` |
| `permanent` | `Permanent` |
| `ghost` | `Structure` |
| (missing) | `Hub` |
| (missing) | `Task` |
| (missing) | `Reference` |
| (missing) | `Outline` |
| (missing) | `Ghost` |

**Actual Block Types** (from `src/models/block.rs`):

```rust
pub enum BlockType {
    Fleeting,     // Quick capture, raw ideas
    Literature,   // Notes from sources
    Permanent,    // Crystallized knowledge (atomic)
    Structure,    // Index/MOC (Map of Content)
    Hub,          // Entry point to domain
    Task,         // Actionable items
    Reference,    // External references
    Outline,      // Skeletons for documents (TOC structure)
    Ghost,        // Predictive placeholder
}
```

### 1.2 Link Types: Manifest vs Implementation

| Manifest Says | Actual Implementation |
|---------------|----------------------|
| `refines` | `Extends` |
| `contains` | `Refines` |
| `related` | `Contradicts` |
| `supports` | `Questions` |
| (missing) | `Supports` |
| (missing) | `References` |
| (missing) | `Related` |
| (missing) | `SimilarTo` |
| (missing) | `SectionOf` |
| (missing) | `SubsectionOf` |
| (missing) | `OrderedChild` |
| (missing) | `Next` |
| (missing) | `NextSibling` |
| (missing) | `FirstChild` |
| (missing) | `Contains` |
| (missing) | `Parent` |
| (missing) | `AiSuggested` |

**Actual Link Types** (from `src/models/edge.rs`):

```rust
pub enum LinkType {
    // Classic Zettelkasten links
    Extends,       // Elaborates on idea
    Refines,       // Improves precision
    Contradicts,   // Opposes or challenges
    Questions,     // Raises questions
    Supports,      // Provides evidence
    References,    // Cites or mentions
    Related,       // General association

    // Similarity
    SimilarTo,

    // Structural links (Document Synthesis)
    SectionOf,       // This block is section of Structure Note
    SubsectionOf,    // This block is subsection
    OrderedChild,     // Child with explicit order
    Next,            // Structural Spine: deterministic sequence
    NextSibling,     // Next sibling in sequence
    FirstChild,      // First child of parent

    // Hierarchy
    Contains,
    Parent,

    // AI-suggested links
    AiSuggested,
}
```

### 1.3 MCP Tools: Manifest vs Implementation

The manifest focuses on SDD "skills" but the system exposes 19 MCP tools:

**Block Tools (5)**:
- `create_block` - Create new block with type, content, tags
- `get_block` - Retrieve block by ULID
- `search_blocks` - Full-text search or type filtering
- `update_block` - Update content and metadata
- `delete_block` - Delete with edge cleanup

**Link Tools (4)**:
- `create_link` - Create edge between blocks
- `delete_link` - Remove edge
- `get_links` - Get incoming/outgoing links
- `suggest_links` - AI-powered link suggestions

**Spine Tools (3)**:
- `traverse_spine` - Traverse structural hierarchy
- `gravity_check` - Check block connectivity
- `reorder_block` - Reorder using fractional indexing

**Structure Tools (3)**:
- `get_section_map` - Get children of a structure
- `detect_gaps` - Detect ghost placeholders in section
- `list_ghosts` - List ghost blocks by status

**Synthesis Tools (2)**:
- `synthesize` - Generate document from structure
- `get_toc` - Get table of contents

**Versioning Tools (Git-like) (5)**:
- `stage_block` - Add to staging area
- `commit_changes` - Commit with message
- `get_working_set_status` - Show staging area
- `unstage_block` - Remove from staging
- `discard_working_set` - Clear staging

**Rate Limiting**: 100 requests per 60 seconds per agent

### 1.4 Session Commands Not Documented

The manifest does not mention the 7 session CLI commands:

| Command | Purpose |
|---------|---------|
| `session start` | Start tracking a knowledge session |
| `session end` | End session, optionally promote blocks |
| `session list` | List all sessions |
| `session restore` | Restore a session context |
| `session checkpoint` | Create checkpoint before compaction |
| `session capture` | Capture fleeting note to session |
| `session blocks` | List blocks created in session |

### 1.5 Zettelkasten Workflow Not Documented

The manifest doesn't document the Zettelkasten pipeline:

```
Fleeting (raw capture)
    ↓ session promote_to
Literature (source notes)
    ↓ refinement
Permanent (atomic knowledge)
```

Session blocks have `session/active` → `session/ended` status tracking.

### 1.6 Enrichment Feature Missing

The `create_block` tool supports `enrich: true` which returns:
- Link suggestions
- Tag suggestions
- Gravity info
- Type mismatch detection

This is not documented.

### 1.7 Ghost Detection System Not Documented

The `GhostDetector` system with status lifecycle (`detected` → `acknowledged` → `in_progress` → `filled` → `dismissed`) is not documented in the manifest.

---

## Part 2: Recommended Manifest Structure

### 2.1 Revised Skill Categories

| Category | Count | Description |
|----------|-------|-------------|
| SDD Phase Skills | 9 | Spec-Driven Development workflow |
| Zettelkasten Skills | 4 | Block lifecycle management |
| Session Skills | 3 | Session tracking and capture |
| Utility Skills | 4 | Versioning, synthesis, structure |

**Total**: 20 skills

### 2.2 Proposed Skill Structure

#### SDD Phase Skills (9) - Keep as-is
- sdd-init, sdd-explore, sdd-propose, sdd-spec, sdd-design, sdd-tasks, sdd-apply, sdd-verify, sdd-archive

#### Zettelkasten Skills (4)

**zettel/fleeting**

| Field | Value |
|-------|-------|
| **Purpose** | Quick capture of raw ideas |
| **Trigger** | `/fleeting`, `/capture`, `new idea` |
| **Block Type** | `fleeting` |
| **Session Integration** | Auto-tracked in `created_blocks` |

**zettel/literature**

| Field | Value |
|-------|-------|
| **Purpose** | Notes from sources (books, articles, videos) |
| **Trigger** | `/literature`, `/source`, `reading notes` |
| **Block Type** | `literature` |

**zettel/permanent**

| Field | Value |
|-------|-------|
| **Purpose** | Atomic crystallized knowledge |
| **Trigger** | `/permanent`, `/atomic`, `permanent note` |
| **Block Type** | `permanent` |

**zettel/promote**

| Field | Value |
|-------|-------|
| **Purpose** | Transition blocks through Zettelkasten stages |
| **Trigger** | `/promote`, `promote to permanent` |
| **Block Type** | N/A (operation) |
| **Usage** | `session end --promote-to permanent` |

#### Session Skills (3)

**session/start**

| Field | Value |
|-------|-------|
| **Purpose** | Initialize session for block tracking |
| **Trigger** | `/session-start`, `/begin-session` |
| **Output** | Session block with `session/{id}` title |

**session/capture**

| Field | Value |
|-------|-------|
| **Purpose** | Quick capture during active session |
| **Trigger** | `/capture`, `quick note` |
| **Block Type** | `fleeting` with session metadata |

**session/end**

| Field | Value |
|-------|-------|
| **Purpose** | Finalize session, promote blocks |
| **Trigger** | `/session-end`, `/end-session` |
| **Options** | `--promote-to` (fleeting/literature/permanent) |

#### Utility Skills (4)

**version/stage**

| Field | Value |
|-------|-------|
| **Purpose** | Stage blocks for commit (git-like) |
| **Trigger** | `/stage`, `stage for commit` |

**version/commit**

| Field | Value |
|-------|-------|
| **Purpose** | Commit staged changes with message |
| **Trigger** | `/commit`, `commit changes` |

**synthesize/document**

| Field | Value |
|-------|-------|
| **Purpose** | Generate document from structure |
| **Trigger** | `/synthesize`, `generate document` |

**structure/section-map**

| Field | Value |
|-------|-------|
| **Purpose** | Get section hierarchy |
| **Trigger** | `/section-map`, `show structure` |

---

## Part 3: Block Type Conventions

### 3.1 Block Type Selection Guide

| Type | Use When | Examples |
|------|----------|----------|
| `fleeting` | Raw, unprocessed ideas | Quick captures, brainstorming |
| `literature` | Notes from external sources | Book highlights, article notes, video transcripts |
| `permanent` | Atomic, crystallized knowledge | Concepts, explanations, decisions |
| `structure` | Index or Map of Content | MOCs, dashboards, overview documents |
| `hub` | Entry point to a domain | Project home, topic landing page |
| `task` | Actionable items | TODOs, action items, deliverables |
| `reference` | External resource links | URLs, citations, bookmarks |
| `outline` | Document skeleton | TOC structure, section placeholders |
| `ghost` | Predictive placeholder | Expected but not yet created content |

### 3.2 Block Type State Machine

```
                    ┌──────────────┐
                    │   fleeting   │ ← Session captures
                    └──────┬───────┘
                           │ promote_to (end session)
                           ↓
                    ┌──────────────┐
                    │  literature  │ ← Source notes
                    └──────┬───────┘
                           │ refinement
                           ↓
                    ┌──────────────┐
                    │  permanent   │ ← Atomic knowledge
                    └──────────────┘
```

### 3.3 Required Tags by Type

| Block Type | Required Tags | Optional Tags |
|------------|---------------|---------------|
| `fleeting` | `fleeting` | `session/{id}`, `session-capture` |
| `literature` | `literature` | `source`, `author`, `type/{book,article,video}` |
| `permanent` | `permanent` | `atomic`, `concept`, `decision` |
| `structure` | `structure`, `moc` | `domain`, `project` |
| `hub` | `hub` | `domain`, `entry-point` |
| `task` | `task` | `todo`, `in-progress`, `done` |
| `reference` | `reference` | `url`, `citation` |
| `outline` | `outline` | `draft`, `toc` |
| `ghost` | `ghost` | `placeholder` |

---

## Part 4: Link Type Conventions

### 4.1 Link Type Taxonomy

**Semantic Links** (Zettelkasten meaning):

| Link Type | Meaning | Example |
|-----------|---------|---------|
| `extends` | Builds upon, adds to | "This note extends the idea from..." |
| `refines` | Makes more precise | "This clarifies point X" |
| `contradicts` | Opposes or challenges | "However, this view suggests..." |
| `questions` | Raises questions about | "This raises concerns about..." |
| `supports` | Provides evidence for | "Study X supports this claim" |
| `references` | Cites or mentions | "See also paper Y" |
| `related` | General association | "Related to topic Z" |
| `similar_to` | Resemblance | "Similar pattern in..." |

**Structural Links** (Document Synthesis):

| Link Type | Meaning | Use |
|-----------|---------|-----|
| `section_of` | Block is a section of structure | `Structure` → `block` |
| `subsection_of` | Block is a subsection | Parent section → child section |
| `ordered_child` | Ordered child in hierarchy | Numbered sections |
| `next` | Next in sequence (spine) | Sequential reading order |
| `next_sibling` | Next sibling in sequence | Peer sections |
| `first_child` | First child of parent | Entry point to subtree |

**Hierarchy Links**:

| Link Type | Meaning | Use |
|-----------|---------|-----|
| `contains` | Parent contains children | MOC contains topics |
| `parent` | Child's parent | Back-reference |

**AI-Suggested Links**:

| Link Type | Meaning | Use |
|-----------|---------|-----|
| `ai_suggested` | AI-detected relationship | Requires human confirmation |

### 4.2 Link Direction Convention

Links are **directional**:
- `source_id` → `target_id`
- Source "points to" target
- Source provides evidence/relationship TO target

Example:
```
Block A (source) --refines--> Block B (target)
Block A refines Block B
```

### 4.3 Link Type Selection Guide

```
Is this a Zettelkasten semantic relationship?
├── Yes: extends, refines, contradicts, questions, supports, references, related, similar_to
└── No: Is this document structure?
    ├── Yes: section_of, subsection_of, ordered_child, next, next_sibling, first_child
    └── No: Is this hierarchy?
        ├── Yes: contains, parent
        └── No: Is this AI-suggested?
            └── Yes: ai_suggested
```

---

## Part 5: Session Conventions

### 5.1 Session Lifecycle

```
┌─────────────┐
│   ACTIVE    │ ← session start
└──────┬──────┘
       │ session end
       ↓
┌─────────────┐
│   ENDED     │ ← Final state
└─────────────┘

       ↑ OR

┌─────────────┐
│  RESTORED   │ ← session restore (from ended)
└─────────────┘
```

### 5.2 Session Block Structure

Sessions are stored as `BlockType::Task` blocks with:

**Title**: `session/{session_id}`

**Tags**:
- `session`
- `session-active` | `session-ended` | `session-restored`

**Metadata**:
```json
{
  "session": {
    "session_id": "...",
    "agent": "claude-code",
    "project": "my-project",
    "started_at": "2026-03-20T10:00:00Z",
    "description": "Optional description"
  },
  "session_id": "...",
  "session_status": "active"
}
```

### 5.3 Session Capture Flow

```
User: /capture "Interesting idea about X"
           ↓
    Creates BlockType::Fleeting
           ↓
    Tags: session/{session_id}, session-capture
           ↓
    Metadata: { session_id, captured_at }
           ↓
    Session.created_blocks += [block_id]
```

### 5.4 Session Promote Flow

```
session end --promote-to permanent
           ↓
    For each block in session.created_blocks:
           ↓
    block.block_type = Permanent
    block.tags += "from-session/{id}", "promoted"
           ↓
    Session.ended_at = now
    Session.status = Ended
```

---

## Part 6: Ghost Detection Conventions

### 6.1 Ghost Status Lifecycle

```
detected → acknowledged → in_progress → filled
                                          ↓
                                    dismissed (if invalid)
```

### 6.2 Ghost Confidence

Ghost blocks have `ai_confidence` metadata:
- `> 0.8`: High confidence placeholder needed
- `0.5-0.8`: Medium confidence
- `< 0.5`: Low confidence, may be dismissed

### 6.3 Ghost Detection Triggers

Gaps detected when:
- Structure block has no children
- Expected sections missing based on patterns
- Reference block links to non-existent content

---

## Part 7: Recommended Manifest Changes

### 7.1 Add Zettelkasten Workflow Section

```markdown
## Zettelkasten Workflow

### Block Lifecycle

Blocks transition through stages:

1. **Fleeting** - Raw capture (session captures, quick notes)
2. **Literature** - Source notes (from reading, research)
3. **Permanent** - Crystallized atomic knowledge

Use `session end --promote-to <type>` to transition blocks.

### Session Integration

Sessions track all blocks created during their lifetime.
Capture notes during a session with `/capture <content>`.
End session to promote blocks to permanent knowledge.
```

### 7.2 Add Block Type Reference

```markdown
## Block Types Reference

| Type | Purpose | Tags |
|------|---------|------|
| fleeting | Raw ideas | fleeting |
| literature | Source notes | literature, source |
| permanent | Atomic knowledge | permanent, atomic |
| structure | MOC/Index | structure, moc |
| hub | Domain entry | hub, domain |
| task | Action items | task, todo |
| reference | External links | reference |
| outline | Document skeleton | outline, draft |
| ghost | Placeholder | ghost, placeholder |
```

### 7.3 Add Link Type Reference

```markdown
## Link Types Reference

### Semantic (Zettelkasten)
- extends, refines, contradicts, questions, supports, references, related, similar_to

### Structural (Document Synthesis)
- section_of, subsection_of, ordered_child, next, next_sibling, first_child

### Hierarchy
- contains, parent

### AI-Suggested
- ai_suggested
```

### 7.4 Update Tool Documentation

Add MCP tool reference:

```markdown
## MCP Tools (19)

### Block Tools
- create_block, get_block, search_blocks, update_block, delete_block

### Link Tools
- create_link, delete_link, get_links, suggest_links

### Spine Tools
- traverse_spine, gravity_check, reorder_block

### Structure Tools
- get_section_map, detect_gaps, list_ghosts

### Synthesis Tools
- synthesize, get_toc

### Versioning Tools
- stage_block, commit_changes, get_working_set_status, unstage_block, discard_working_set
```

---

## Appendix A: MCP Tool Detailed Schemas

### create_block

```json
{
  "block_type": "fleeting|literature|permanent|structure|hub|task|reference|outline|ghost",
  "content": "Markdown content",
  "title": "Optional title",
  "tags": ["tag1", "tag2"],
  "enrich": false  // If true, returns link/tag/gravity suggestions
}
```

### create_link

```json
{
  "source_id": "ULID",
  "target_id": "ULID",
  "link_type": "extends|refines|contradicts|questions|supports|references|related|similar_to|section_of|subsection_of|ordered_child|next|next_sibling|first_child|contains|parent|ai_suggested"
}
```

### detect_gaps

Returns ghost blocks for a section with:
- `id`, `description`, `confidence`
- `status`: detected|acknowledged|in_progress|filled|dismissed
- `position_hint`: { after, before, parent_section }

---

## Appendix B: Session Command Reference

```bash
# Start session
pkm session start --agent claude-code --project my-project --description "Working on feature X"

# Capture during session
pkm session capture --session-id <id> --project my-project "Quick idea about Y"

# End session with promotion
pkm session end --session-id <id> --project my-project --promote-to permanent --summary "Completed feature X"

# List session blocks
pkm session blocks --session-id <id> --project my-project

# Restore session
pkm session restore --session-id <id> --project my-project

# Create checkpoint
pkm session checkpoint --session-id <id> --project my-project
```

---

*End of Report*