# PKM-AI CLI Reference

> Comprehensive command-line interface reference for PKM-AI PKM

## Table of Contents

1. [Global Flags](#1-global-flags)
2. [Init Command](#2-init-command)
3. [Block Commands](#3-block-commands)
   - [create - Create a new block](#31-create)
   - [quick - Quick capture](#32-quick)
   - [list - List blocks](#33-list)
   - [search - Fuzzy search](#34-search)
   - [grep - Search content](#35-grep)
   - [show - Show block details](#36-show)
   - [lint - Validate integrity](#37-lint)
   - [promote - Change block type](#38-promote)
4. [Link Commands](#4-link-commands)
   - [link - Create links](#41-link)
5. [Version Commands](#5-version-control-commands)
   - [status - Working tree status](#51-status)
   - [log - Commit history](#52-log)
   - [diff - Show changes](#53-diff)
   - [add - Stage changes](#54-add)
   - [commit - Create commit](#55-commit)
   - [branch - Branch operations](#56-branch)
   - [checkout - Switch branches](#57-checkout)
   - [merge - Merge branches](#58-merge)
   - [tag - Tag operations](#59-tag)
   - [log-grep - Search commits](#510-log-grep)
   - [orphan - List orphans](#511-orphan)
   - [reset - Reset HEAD](#512-reset)
   - [rebase - Rebase branch](#513-rebase)
   - [push - Push refs](#514-push)
   - [pull - Pull changes](#515-pull)
   - [fetch - Fetch remote](#516-fetch)
   - [clone - Clone repository](#517-clone)
   - [remote-list - List remotes](#518-remote-list)
   - [remote-add - Add remote](#519-remote-add)
6. [Database Commands](#6-database-commands)
   - [db init - Initialize database](#61-db-init)
   - [db stats - Show statistics](#62-db-stats)
   - [db export - Export database](#63-db-export)
   - [db import - Import database](#64-db-import)
7. [Structure Commands](#7-structure-commands)
   - [traverse - Traverse spine](#71-traverse)
   - [toc - Generate TOC](#72-toc)
   - [synthesize - Synthesize document](#73-synthesize)
   - [gravity-check - Check clustering](#74-gravity-check)
8. [Ghost Commands](#8-ghost-commands)
   - [ghost list - List ghosts](#81-ghost-list)
   - [ghost show - Show ghost](#82-ghost-show)
   - [ghost fill - Fill ghost](#83-ghost-fill)
   - [ghost dismiss - Dismiss ghost](#84-ghost-dismiss)
9. [Interactive Commands](#9-interactive-commands)
   - [architect - Interactive TUI](#91-architect)

---

## 1. Global Flags

Global flags can be used with any command and must be placed before the subcommand.

| Flag | Short | Description | Environment Variable |
|------|-------|-------------|---------------------|
| `--db-path <path>` | `-d` | Database path (defaults to `~/.pkmai/data.db`) | `NEXUS_DB_PATH` |
| `--verbose` | `-v` | Enable verbose output (show info logs) | - |
| `--stage` | - | Enable auto-staging after create/link (default: true) | - |
| `--no-stage` | - | Disable auto-staging after create/link operations | - |

### Examples

```bash
# Use verbose mode
pkmai --verbose create -t permanent --title "My Note"

# Specify custom database path
pkmai --db-path /tmp/pkmai.db list

# Disable auto-staging for a single command
pkmai create -t permanent --title "Note" --no-stage

# Disable auto-staging globally (enable with --stage)
pkmai link <src> <dst> --no-stage
```

### Database Path Resolution

The database path is resolved in the following priority:
1. Explicit `--db-path` flag
2. `.pkmai/config.toml` in current directory
3. `~/.pkmai/config.toml` in home directory
4. Default: `~/.pkmai/data.db`

---

## 2. Init Command

### init - Initialize PKM-AI

Initializes the PKM configuration in the filesystem.

```bash
pkmai init [OPTIONS]
```

#### Options

| Flag | Description |
|------|-------------|
| `--home` | Initialize in home directory (`~/.pkmai/`) instead of current directory |
| `--force` | Force overwrite existing config (NOT YET IMPLEMENTED) |

#### Examples

```bash
# Initialize in current directory
pkmai init

# Initialize in home directory
pkmai init --home

# Output:
# Initialized PKM-AI in /home/user/project/
# Config file: /home/user/project/.pkmai/config.toml
# Database: /home/user/project/.pkmai/data.db
```

#### Notes

- Creates `.pkmai/config.toml` with default configuration
- The config file stores the database path relative to the config location
- Fails if config already exists (use `--force` to overwrite when implemented)

---

## 3. Block Commands

### 3.1 create

Create a new block in the knowledge base.

```bash
pkmai create [OPTIONS]
pkmai create -t <type> --title <title> [--content <content>] [--tags <tags>]
```

#### Syntax

```
pkmai create -t <block_type> --title <title> [--content <content>] [-T <tags>] [-i]
```

#### Options

| Flag | Short | Required | Description |
|------|-------|----------|-------------|
| `--title <title>` | - | Yes | Title of the block |
| `--type <type>` | `-t` | Yes | Block type (see below) |
| `--content <content>` | - | No | Content (reads from stdin if not provided) |
| `--tags <tags>` | `-T` | No | Tags (comma-separated) |
| `--interactive` | `-i` | No | Enable interactive AI pre-flight mode with suggestions |

#### Block Types

| Type | Flag Value | Description |
|------|------------|-------------|
| Fleeting | `fleeting` or `f` | Quick capture, temporary notes |
| Literature | `literature` or `l` | Notes from external sources |
| Permanent | `permanent` or `p` | Atomic knowledge notes |
| Structure | `structure` or `s` | Document root, organization (MOC) |
| Hub | `hub` or `h` | Topic overview, index |
| Task | `task` or `t` | Action items, todos |
| Reference | `reference` or `r` | External links, citations |
| Outline | `outline` or `o` | Hierarchical structure |

#### Examples

```bash
# Create a permanent note
pkmai create -t permanent --title "Actor Model Fundamentals" \
  --content "The actor model treats actors as the universal primitives."

# Create a task block
pkmai create -t task --title "TODO: Fix authentication bug" -T "bug,auth"

# Create with tags
pkmai create -t literature --title "Rust Book Notes" \
  --content "Chapter 5: Ownership" -T "rust,programming"

# Interactive mode with AI suggestions
pkmai create -t permanent --title "Rust Concurrency" --interactive

# Create from stdin
echo "Content from stdin" | pkmai create -t fleeting --title "Stdin Note"
```

#### AI Pre-Flight (with `--verbose`)

When `--verbose` is enabled, AI suggestions are shown:

```
🤖 AI Pre-Flight:
⚠️  Similar notes found (possible duplicates):
   - "Rust Ownership Model" (0.94)
📍 Suggested location: "Rust Programming" (affinity: 0.72)
🏷️  Suggested tags: rust, memory, ownership
🔗 Suggested links: 3 notes
```

#### Auto-Detection

Block type can be auto-detected from title keywords:

| Keywords | Detected Type |
|----------|---------------|
| `TODO`, `FIX`, `IMPLEMENT`, `COMPLETE` | `task` |
| `IDEA`, `NOTE`, `OBSERVATION` | `literature` |
| `INDEX`, `MOC`, `OVERVIEW`, `SUMMARY` | `structure` |

#### Notes

- Created blocks are auto-staged by default (use `--no-stage` to disable)
- Interactive mode (`-i`) prompts for duplicate confirmation if similarity > 0.95
- When content is not provided, reads from stdin

---

### 3.2 quick

Quick capture: create + stage + commit in one command.

```bash
pkmai quick <content> [OPTIONS]
pkmai q <content> [OPTIONS]
```

#### Syntax

```
pkmai quick <content> [-t <type>] [-T <tags>]
```

#### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<content>` | Yes | Content of the note |

#### Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--type <type>` | `-t` | `fleeting` | Block type (fleeting, literature, permanent, task, reference) |
| `--tags <tags>` | `-T` | - | Tags (comma-separated) |

#### Examples

```bash
# Quick capture (fleeting by default)
pkmai quick "My quick idea"

# With specific type
pkmai quick "Book reference" -t literature

# With tags
pkmai quick "Important task" -t task -T "work,urgent"

# Using alias
pkmai q "Quick note"
```

#### Behavior

The `quick` command automatically:
1. Creates the block with the specified type
2. Adds it to staging
3. Commits with message: "Quick capture: [first 50 characters of content]"

---

### 3.3 list

List blocks with optional filtering.

```bash
pkmai list [OPTIONS]
pkmai list [-t <type>] [-T <tags>] [-s <search>] [-n <limit>] [-o <format>]
```

#### Syntax

```
pkmai list [-t <type>] [-T <tags>] [-s <search>] [-n <limit>] [-o <format>]
```

#### Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--type <type>` | `-t` | - | Filter by block type |
| `--tags <tags>` | `-T` | - | Filter by tags (comma-separated, OR logic) |
| `--search <search>` | `-s` | - | Fuzzy search by title |
| `--limit <limit>` | `-n` | `50` | Limit number of results |
| `--output <format>` | `-o` | `table` | Output format: `table`, `json`, `simple` |

#### Examples

```bash
# List all blocks (table format)
pkmai list

# List only permanent blocks
pkmai list -t permanent

# List blocks tagged with rust OR python
pkmai list -T rust,python

# Fuzzy search in titles
pkmai list -s ownership

# Limit to 10 results as JSON
pkmai list -n 10 -o json

# Simple output format
pkmai list -o simple
```

#### Output Formats

**table** (default):
```
┌─────────────────────────────────────┬───────────────────────────┬──────────┐
│ ID                                  │ Title                    │ Type     │
├─────────────────────────────────────┼───────────────────────────┼──────────┤
│ 01ARZ3NDEKTSV4RRFFQ69G5FAV         │ Rust Ownership Model     │ permanent│
│ 01ARZ3NDEKTSV4RRFFQ69G5FAW         │ Actor Model Basics       │ permanent│
└─────────────────────────────────────┴───────────────────────────┴──────────┘
```

**json**:
```json
[
  {
    "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
    "title": "Rust Ownership Model",
    "type": "permanent",
    "tags": ["rust", "memory"],
    "created_at": "2024-01-15T10:30:00Z"
  }
]
```

---

### 3.4 search

Fuzzy search blocks by title.

```bash
pkmai search <query> [OPTIONS]
pkmai f <query> [OPTIONS]
```

#### Syntax

```
pkmai search <query> [-n <limit>]
```

#### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<query>` | Yes | Search query (supports fuzzy matching) |

#### Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--limit <limit>` | `-n` | `50` | Limit number of results |

#### Examples

```bash
# Basic fuzzy search
pkmai search "rust own"

# Search with limit
pkmai search "actor model" -n 10

# Using alias
pkmai f "concurr"
```

#### Notes

- Uses fuzzy matching, so "rust own" finds "Rust Ownership Model"
- Case-insensitive
- More permissive than `--search` in `list` command

---

### 3.5 grep

Search block content using regex patterns.

```bash
pkmai grep <pattern> [OPTIONS]
pkmai g <pattern> [OPTIONS]
```

#### Syntax

```
pkmai grep <pattern> [-c] [-i] [-n <limit>]
```

#### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<pattern>` | Yes | Search pattern (regex) |

#### Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--content-only` | `-c` | false | Search in content only (not titles) |
| `--case-sensitive` | `-i` | false | Case sensitive search (default: insensitive) |
| `--limit <limit>` | `-n` | `50` | Limit number of results |

#### Examples

```bash
# Basic grep
pkmai grep "TODO"

# Case-sensitive search
pkmai grep "Rust" -i

# Search only in content
pkmai grep "ownership" -c

# Limit results
pkmai grep "error" -n 20
```

---

### 3.6 show

Show detailed information about a block.

```bash
pkmai show <id> [OPTIONS]
pkmai s <id> [OPTIONS]
```

#### Syntax

```
pkmai show <id> [--related]
```

#### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<id>` | Yes | Block ID (ULID) |

#### Options

| Flag | Short | Description |
|------|-------|-------------|
| `--related` | `-r` | Show related blocks |

#### Examples

```bash
# Show block details
pkmai show 01ARZ3NDEKTSV4RRFFQ69G5FAV

# Show with related blocks
pkmai show 01ARZ3NDEKTSV4RRFFQ69G5FAV --related
```

#### Output Example

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 Block: 01ARZ3NDEKTSV4RRFFQ69G5FAV
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 Title:    Rust Ownership Model
 Type:     permanent
 Tags:     rust, memory, ownership
 Created:  2024-01-15 10:30:00
 Modified: 2024-01-15 14:22:00

 Content:
 ─────────────────────────────────────────────────────────
 Rust's ownership system is a set of rules that the
 compiler enforces at compile time. It manages memory
 without garbage collection.

 Links:
 ─────────────────────────────────────────────────────────
 → 01ARZ3NDEKTSV4RRFFQ69G5FAW (supports)
 → 01ARZ3NDEKTSV4RRFFQ69G5FAX (extends)

 ← 01ARZ3NDEKTSV4RRFFQ69G5FAY (supported_by)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

### 3.7 lint

Validate structural integrity of the knowledge base.

```bash
pkmai lint [OPTIONS]
```

#### Syntax

```
pkmai lint [--fix]
```

#### Options

| Flag | Short | Description |
|------|-------|-------------|
| `--fix` | `-f` | Fix issues automatically |

#### Examples

```bash
# Check for issues
pkmai lint

# Auto-fix issues
pkmai lint --fix
```

#### Notes

- Checks for structural integrity issues
- With `--fix`, attempts to resolve issues automatically
- Reports orphaned blocks, broken links, type inconsistencies

---

### 3.8 promote

Promote a block to a higher order type.

```bash
pkmai promote <id> [OPTIONS]
```

#### Syntax

```
pkmai promote <id> [-t <type>] [--stage]
```

#### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<id>` | Yes | Block ID to promote |

#### Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--type <type>` | `-t` | `permanent` | Target block type |
| `--stage` | - | false | Add to staging automatically |

#### Valid Transitions

| From | To |
|------|-----|
| `fleeting` | `literature`, `permanent`, `ghost` |
| `literature` | `permanent`, `ghost` |
| `ghost` | `permanent` |
| `structure`, `hub`, `task`, `reference`, `outline` | `permanent` |

#### Examples

```bash
# Promote to permanent (default)
pkmai promote 01ARZ3NDEKTSV4RRFFQ69G5FAV

# Promote to literature
pkmai promote 01ARZ3NDEKTSV4RRFFQ69G5FAV -t literature

# Promote with auto-staging
pkmai promote 01ARZ3NDEKTSV4RRFFQ69G5FAV -t permanent --stage
```

---

## 4. Link Commands

### 4.1 link

Create semantic links between blocks.

```bash
pkmai link <from> <to> [OPTIONS]
pkmai ln <from> <to> [OPTIONS]
```

#### Syntax

```
pkmai link <source_id> <target_id> [-t <type>] [-w <weight>] [-c <context>]
```

#### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<from>` | Yes | Source block ID |
| `<to>` | Yes | Target block ID |

#### Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--type <type>` | `-t` | `related` | Link type (see below) |
| `--weight <weight>` | `-w` | `0.0` | Sequence weight (for ordered links) |
| `--context <context>` | `-c` | - | Link context/description |

#### Link Types

| Type | Description | Use Case |
|------|-------------|----------|
| `section_of` | Block is a section of target | Chapter in document |
| `supports` | Supporting evidence | Citations, examples |
| `extends` | Extension of another block | Elaborations |
| `refines` | More specific version | Detailed explanation |
| `contradicts` | Opposite view | Debates, alternatives |
| `questions` | Raises questions about | Challenging assumptions |
| `references` | External citation | Links to sources |
| `related` | Related content | General association |
| `similar_to` | Similar content | Parallel concepts |
| `next` | Sequential relationship | Following block in spine |
| `gravity` | Semantic attraction | Related but not sequential |

#### Examples

```bash
# Basic link
pkmai link 01ARZ3NDEKTSV4RRFFQ69G5FAV 01ARZ3NDEKTSV4RRFFQ69G5FAW

# Section relationship
pkmai link 01ARZ3NDEKTSV4RRFFQ69G5FAV 01ARZ3NDEKTSV4RRFFQ69G5FAW -t section_of

# With weight for ordering
pkmai link 01ARZ3NDEKTSV4RRFFQ69G5FAV 01ARZ3NDEKTSV4RRFFQ69G5FAW \
  -t next -w 1.5

# With context
pkmai link 01ARZ3NDEKTSV4RRFFQ69G5FAV 01ARZ3NDEKTSV4RRFFQ69G5FAW \
  -t supports -c "This provides evidence for the claim"
```

#### Notes

- Links are auto-staged by default (use `--no-stage` to disable)
- Weight is used for structural spine ordering
- Higher weight = appears later in sequence

---

## 5. Version Control Commands

Version control commands use Git-like semantics for blocks.

### 5.1 status

Show working tree status.

```bash
pkmai version status [OPTIONS]
pkmai status [OPTIONS]
```

#### Syntax

```
pkmai version status [-r <repo>]
```

#### Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--repo <repo>` | `-r` | current directory | Repository path |

#### Examples

```bash
# Check status
pkmai version status

# Check specific repo
pkmai version status -r /path/to/repo
```

#### Output Example

```
On branch: main
Status:
  Staged:
    + 01ARZ3NDEKTSV4RRFFQ69G5FAV (new)
    ~ 01ARZ3NDEKTSV4RRFFQ69G5FAW (modified)
  Unstaged:
    ~ 01ARZ3NDEKTSV4RRFFQ69G5FAY (deleted)
```

---

### 5.2 log

Show commit history.

```bash
pkmai version log [OPTIONS]
pkmai log [OPTIONS]
```

#### Syntax

```
pkmai version log [-r <repo>] [--oneline] [--graph] [-n <limit>]
```

#### Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--repo <repo>` | `-r` | current directory | Repository path |
| `--oneline` | - | false | Show one line per commit |
| `--graph` | - | false | Show ASCII graph |
| `--limit <limit>` | `-n` | `50` | Limit number of commits |

#### Examples

```bash
# Full log
pkmai version log

# Compact oneline
pkmai version log --oneline

# With graph
pkmai version log --graph

# Limit results
pkmai version log -n 20
```

---

### 5.3 diff

Show changes between commits or working tree.

```bash
pkmai version diff [OPTIONS]
pkmai diff [OPTIONS]
```

#### Syntax

```
pkmai version diff [-r <repo>] [-b <block_id>]
```

#### Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--repo <repo>` | `-r` | current directory | Repository path |
| `--block <block_id>` | `-b` | - | Block ID to diff |

#### Examples

```bash
# Show all changes
pkmai version diff

# Diff specific block
pkmai version diff -b 01ARZ3NDEKTSV4RRFFQ69G5FAV
```

---

### 5.4 add

Stage changes for commit.

```bash
pkmai version add [OPTIONS]
pkmai add <block_id>
```

#### Syntax

```
pkmai version add [-r <repo>] <block_id>
```

#### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<block_id>` | Yes | Block ID to stage |

#### Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--repo <repo>` | `-r` | current directory | Repository path |

#### Examples

```bash
# Stage a block
pkmai version add 01ARZ3NDEKTSV4RRFFQ69G5FAV

# Stage in specific repo
pkmai version add -r /path/to/repo 01ARZ3NDEKTSV4RRFFQ69G5FAV
```

---

### 5.5 commit

Create a new commit.

```bash
pkmai version commit [OPTIONS]
pkmai commit [OPTIONS]
```

#### Syntax

```
pkmai version commit [-r <repo>] -m <message> [-a <author>] [--amend] [--no-edit]
```

#### Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--repo <repo>` | `-r` | current directory | Repository path |
| `--message <message>` | `-m` | - | Commit message |
| `--author <author>` | `-a` | `user` | Author name |
| `--amend` | - | false | Amend the last commit |
| `--no-edit` | - | false | Amend without changing message |

#### Examples

```bash
# Basic commit
pkmai version commit -m "Add Rust concurrency notes"

# With author
pkmai version commit -m "Update" -a "developer"

# Amend last commit
pkmai version commit --amend --no-edit
```

#### Notes

- `--amend` and `--no-edit` are mutually exclusive
- `--no-edit` uses the last commit message

---

### 5.6 branch

List, create, or delete branches.

```bash
pkmai version branch [OPTIONS]
pkmai branch [OPTIONS]
```

#### Syntax

```
pkmai version branch [-r <repo>] [name] [--delete] [--force-delete]
```

#### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<name>` | No | Branch name (to create) |

#### Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--repo <repo>` | `-r` | current directory | Repository path |
| `--delete` | - | false | Delete a branch |
| `--force-delete` | - | false | Force delete (even if not merged) |

#### Examples

```bash
# List branches
pkmai version branch

# Create branch
pkmai version branch new-feature

# Delete branch
pkmai version branch old-feature --delete

# Force delete
pkmai version branch legacy --force-delete
```

---

### 5.7 checkout

Switch branches or create new branch.

```bash
pkmai version checkout [OPTIONS]
pkmai checkout <name>
```

#### Syntax

```
pkmai version checkout [-r <repo>] <name> [-b]
```

#### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<name>` | Yes | Branch name |

#### Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--repo <repo>` | `-r` | current directory | Repository path |
| `--create-new` | `-b` | false | Create and switch to new branch |

#### Examples

```bash
# Switch to existing branch
pkmai version checkout main

# Create and switch to new branch
pkmai version checkout -b feature-branch
```

---

### 5.8 merge

Merge a branch into current HEAD.

```bash
pkmai version merge [OPTIONS]
pkmai merge <name>
```

#### Syntax

```
pkmai version merge [-r <repo>] -n <name> [-s <strategy>]
```

#### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<name>` | Yes | Branch name to merge |

#### Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--repo <repo>` | `-r` | current directory | Repository path |
| `--strategy <strategy>` | `-s` | `merge` | Merge strategy: `ours`, `theirs`, `merge` |

#### Examples

```bash
# Standard merge
pkmai version merge feature-branch

# theirs strategy (take their changes)
pkmai version merge feature-branch -s theirs
```

---

### 5.9 tag

Tag operations (list, create, delete).

```bash
pkmai version tag [OPTIONS]
pkmai tag [OPTIONS]
```

#### Syntax

```
pkmai version tag [-r <repo>] [name] [-c <commit>] [-m <message>] [--delete]
```

#### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<name>` | No | Tag name |

#### Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--repo <repo>` | `-r` | current directory | Repository path |
| `--commit <commit>` | `-c` | HEAD | Commit ID to tag |
| `--message <message>` | `-m` | - | Tag message (for annotated tags) |
| `--delete` | - | false | Delete a tag |

#### Examples

```bash
# List tags
pkmai version tag

# Create tag
pkmai version tag v1.0.0

# Annotated tag
pkmai version tag -m "Release 1.0.0" v1.0.0

# Tag specific commit
pkmai version tag -c abc123 v0.9.0

# Delete tag
pkmai version tag v0.9.0 --delete
```

---

### 5.10 log-grep

Search commit messages.

```bash
pkmai version log-grep [OPTIONS]
```

#### Syntax

```
pkmai version log-grep [-r <repo>] <pattern> [-n <limit>]
```

#### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<pattern>` | Yes | Pattern to search for |

#### Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--repo <repo>` | `-r` | current directory | Repository path |
| `--limit <limit>` | `-n` | `50` | Limit number of commits |

#### Examples

```bash
# Search commit messages
pkmai version log-grep "fix bug"

# With limit
pkmai version log-grep "feat" -n 20
```

---

### 5.11 orphan

List orphan blocks (blocks without incoming edges).

```bash
pkmai version orphan [OPTIONS]
pkmai orphan [OPTIONS]
```

#### Syntax

```
pkmai version orphan [-r <repo>]
```

#### Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--repo <repo>` | `-r` | current directory | Repository path |

#### Examples

```bash
# List orphan blocks
pkmai version orphan
```

---

### 5.12 reset

Reset HEAD to a previous commit.

```bash
pkmai version reset [OPTIONS]
pkmai reset [OPTIONS]
```

#### Syntax

```
pkmai version reset [-r <repo>] [--soft] [--hard] [-c <commit>]
```

#### Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--repo <repo>` | `-r` | current directory | Repository path |
| `--soft` | - | false | Soft reset: keep changes staged |
| `--hard` | - | false | Hard reset: discard all changes |
| `--commit <commit>` | `-c` | HEAD~1 | Commit to reset to |

#### Examples

```bash
# Soft reset (keep changes staged)
pkmai version reset --soft

# Hard reset (discard changes)
pkmai version reset --hard

# Reset to specific commit
pkmai version reset -c abc123
```

#### Notes

- `--soft` and `--hard` are mutually exclusive
- Default is `--soft` with `--commit` defaulting to HEAD~1

---

### 5.13 rebase

Rebase current branch onto another branch.

```bash
pkmai version rebase [OPTIONS]
pkmai rebase <branch>
```

#### Syntax

```
pkmai version rebase [-r <repo>] <branch>
```

#### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<branch>` | Yes | Branch to rebase onto |

#### Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--repo <repo>` | `-r` | current directory | Repository path |

#### Examples

```bash
# Rebase onto main
pkmai version rebase main
```

---

### 5.14 push

Push refs to a remote.

```bash
pkmai version push [OPTIONS]
pkmai push [OPTIONS]
```

#### Syntax

```
pkmai version push [-r <repo>] [-m <remote>] [-r <refs>] [-f]
```

#### Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--repo <repo>` | `-r` | current directory | Repository path |
| `--remote <remote>` | `-m` | `origin` | Remote name |
| `--refs <refs>` | `-r` | all | Refs to push |
| `--force` | `-f` | false | Force push (skip fast-forward check) |

#### Examples

```bash
# Push to origin
pkmai version push

# Push specific branch
pkmai version push -m origin main

# Force push
pkmai version push --force
```

---

### 5.15 pull

Pull from a remote.

```bash
pkmai version pull [OPTIONS]
pkmai pull [OPTIONS]
```

#### Syntax

```
pkmai version pull [-r <repo>] [-m <remote>] [-b <branch>] [-s <strategy>]
```

#### Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--repo <repo>` | `-r` | current directory | Repository path |
| `--remote <remote>` | `-m` | `origin` | Remote name |
| `--branch <branch>` | `-b` | - | Branch to pull |
| `--strategy <strategy>` | `-s` | `merge` | Merge strategy: `ours`, `theirs`, `merge` |

#### Examples

```bash
# Pull from origin
pkmai version pull

# Pull with strategy
pkmai version pull -s theirs
```

---

### 5.16 fetch

Fetch from a remote without applying changes.

```bash
pkmai version fetch [OPTIONS]
pkmai fetch [OPTIONS]
```

#### Syntax

```
pkmai version fetch [-r <repo>] [-m <remote>]
```

#### Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--repo <repo>` | `-r` | current directory | Repository path |
| `--remote <remote>` | `-m` | `origin` | Remote name |

#### Examples

```bash
# Fetch from origin
pkmai version fetch
```

---

### 5.17 clone

Clone a repository.

```bash
pkmai version clone <source> [OPTIONS]
pkmai clone <source> [OPTIONS]
```

#### Syntax

```
pkmai version clone <source> [-d <destination>] [-b <branch>] [-D <depth>]
```

#### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<source>` | Yes | Source repository path |

#### Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--destination <dest>` | `-d` | source directory name | Destination path |
| `--branch <branch>` | `-b` | `main` | Branch to clone |
| `--depth <depth>` | `-D` | - | Clone depth (for shallow clone) |

#### Examples

```bash
# Clone to default destination
pkmai version clone /path/to/source

# Clone to specific destination
pkmai version clone /path/to/source -d my-clone

# Shallow clone
pkmai version clone /path/to/source -D 10
```

---

### 5.18 remote-list

List configured remotes.

```bash
pkmai version remote list [OPTIONS]
pkmai remote list [OPTIONS]
```

#### Syntax

```
pkmai version remote list [-r <repo>]
```

#### Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--repo <repo>` | `-r` | current directory | Repository path |

#### Examples

```bash
# List remotes
pkmai version remote list
```

---

### 5.19 remote-add

Add a remote.

```bash
pkmai version remote add [OPTIONS]
pkmai remote add <name> <url>
```

#### Syntax

```
pkmai version remote add [-r <repo>] <name> <url>
```

#### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<name>` | Yes | Remote name |
| `<url>` | Yes | Remote path or URL |

#### Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--repo <repo>` | `-r` | current directory | Repository path |

#### Examples

```bash
# Add remote
pkmai version remote add origin /path/to/remote
```

---

## 6. Database Commands

### 6.1 db init

Initialize the database.

```bash
pkmai db init
```

#### Syntax

```
pkmai db init
```

#### Examples

```bash
# Initialize database
pkmai db init
```

#### Notes

- Creates the database file if it doesn't exist
- Fails if database already initialized

---

### 6.2 db stats

Show database statistics.

```bash
pkmai db stats
```

#### Syntax

```
pkmai db stats
```

#### Examples

```bash
# Show statistics
pkmai db stats
```

#### Output Example

```
Database Statistics:
  Total blocks:     1,234
  By type:
    - fleeting:      200
    - literature:    350
    - permanent:     500
    - structure:     50
    - hub:           30
    - task:          80
    - reference:     20
    - outline:       4
  Total links:       3,456
  Ghost nodes:       12
  Last commit:       2024-01-15T14:22:00Z
```

---

### 6.3 db export

Export database to a file.

```bash
pkmai db export [OPTIONS]
```

#### Syntax

```
pkmai db export --format <format>
```

#### Options

| Flag | Required | Description |
|------|----------|-------------|
| `--format <format>` | Yes | Export format |

#### Examples

```bash
# Export to JSON
pkmai db export --format json > backup.json
```

---

### 6.4 db import

Import database from a file.

```bash
pkmai db import <file>
```

#### Syntax

```
pkmai db import <file>
```

#### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<file>` | Yes | File to import |

#### Examples

```bash
# Import from backup
pkmai db import backup.json
```

---

## 7. Structure Commands

### 7.1 traverse

Traverse the structural spine.

```bash
pkmai traverse [OPTIONS]
```

#### Syntax

```
pkmai traverse [--from <id>] [-d <depth>] [-t <link_type>] [-c]
```

#### Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--from <id>` | - | spine root | Starting block ID |
| `--depth <depth>` | `-d` | `10` | Maximum depth |
| `--type <type>` | `-t` | - | Filter by link type |
| `--content` | `-c` | false | Show content |

#### Examples

```bash
# Traverse from root
pkmai traverse

# From specific block
pkmai traverse --from 01ARZ3NDEKTSV4RRFFQ69G5FAV

# Limited depth
pkmai traverse -d 3

# Show content
pkmai traverse -c
```

---

### 7.2 toc

Generate Table of Contents for a structure block.

```bash
pkmai toc <id>
```

#### Syntax

```
pkmai toc <id>
```

#### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<id>` | Yes | Structure block ID |

#### Examples

```bash
# Generate TOC
pkmai toc 01ARZ3NDEKTSV4RRFFQ69G5FAV
```

#### Output Example

```
Table of Contents: Rust Programming Guide
==========================================

1. Introduction
   1.1 What is Rust?
   1.2 Why Rust?
2. Basics
   2.1 Variables
   2.2 Functions
   2.3 Control Flow
3. Ownership
   3.1 Ownership Rules
   3.2 Borrows
```

---

### 7.3 synthesize

Synthesize a document from structure block.

```bash
pkmai synthesize <id> [OPTIONS]
```

#### Syntax

```
pkmai synthesize <id> [-o <output>] [-t <template>] [-f <file>]
```

#### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<id>` | Yes | Structure block ID |

#### Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--output <output>` | `-o` | `pdf` | Output format: `pdf`, `html`, `markdown`, `typst` |
| `--template <template>` | `-t` | - | Template name |
| `--file <file>` | `-f` | - | Output file |

#### Examples

```bash
# Synthesize to Markdown
pkmai synthesize 01ARZ3NDEKTSV4RRFFQ69G5FAV -o markdown

# Synthesize to PDF with template
pkmai synthesize 01ARZ3NDEKTSV4RRFFQ69G5FAV \
  -o pdf --template technical-whitepaper

# Save to file
pkmai synthesize 01ARZ3NDEKTSV4RRFFQ69G5FAV -f output.pdf
```

#### Notes

- PDF output requires `typst` feature
- If `--file` not specified, outputs to stdout

---

### 7.4 gravity-check

Check gravity hooks and semantic clustering.

```bash
pkmai gravity-check <id> [OPTIONS]
```

#### Syntax

```
pkmai gravity-check <id> [-t <threshold>]
```

#### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<id>` | Yes | Block ID to check |

#### Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--threshold <threshold>` | `-t` | `0.7` | Threshold for similarity (0.0-1.0) |

#### Examples

```bash
# Check gravity for block
pkmai gravity-check 01ARZ3NDEKTSV4RRFFQ69G5FAV

# With custom threshold
pkmai gravity-check 01ARZ3NDEKTSV4RRFFQ69G5FAV -t 0.8
```

#### Notes

- Higher threshold = stricter similarity requirement
- Shows blocks with semantic attraction above threshold

---

## 8. Ghost Commands

Ghost nodes are AI-detected gaps in the knowledge base.

### 8.1 ghost list

List all ghost nodes.

```bash
pkmai ghost list
```

#### Syntax

```
pkmai ghost list
```

#### Examples

```bash
# List ghost nodes
pkmai ghost list
```

#### Output Example

```
👻 Ghost Nodes:
   01ARZ3NDEKTSV4RRFFQ69G5FAV "Missing explanation" (confidence: 0.85)
   01ARZ3NDEKTSV4RRFFQ69G5FAW "Unfinished thought" (confidence: 0.72)
```

---

### 8.2 ghost show

Show detailed ghost node information.

```bash
pkmai ghost show <id>
```

#### Syntax

```
pkmai ghost show <id>
```

#### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<id>` | Yes | Ghost node ID |

#### Examples

```bash
# Show ghost details
pkmai ghost show 01ARZ3NDEKTSV4RRFFQ69G5FAV
```

---

### 8.3 ghost fill

Fill a ghost node with real content.

```bash
pkmai ghost fill <id> --content <content>
```

#### Syntax

```
pkmai ghost fill <id> --content <content>
```

#### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<id>` | Yes | Ghost node ID |

#### Options

| Flag | Required | Description |
|------|----------|-------------|
| `--content <content>` | Yes | Content to fill |

#### Examples

```bash
# Fill ghost node
pkmai ghost fill 01ARZ3NDEKTSV4RRFFQ69G5FAV \
  --content "The actor model is a concurrent programming model."
```

---

### 8.4 ghost dismiss

Dismiss a ghost node (mark as not needed).

```bash
pkmai ghost dismiss <id>
```

#### Syntax

```
pkmai ghost dismiss <id>
```

#### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<id>` | Yes | Ghost node ID |

#### Examples

```bash
# Dismiss ghost
pkmai ghost dismiss 01ARZ3NDEKTSV4RRFFQ69G5FAV
```

---

## 9. Interactive Commands

### 9.1 architect

Launch interactive TUI for knowledge graph exploration.

```bash
pkmai architect
```

#### Syntax

```
pkmai architect
```

#### Examples

```bash
# Launch TUI
pkmai architect
```

#### Navigation

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `h` / `←` | Collapse |
| `l` / `→` | Expand |
| `g` | Go to start |
| `G` | Go to end |
| `PageUp` / `PageDown` | Page through results |
| `Enter` | View detail |
| `Esc` | Go back |

#### Detail Mode

| Key | Action |
|-----|--------|
| `b` | Navigate to first backlink |
| `o` | Navigate to first outgoing link |

#### Commands

| Command | Action |
|---------|--------|
| `:` | Enter command mode |
| `:search <query>` | Fuzzy search |
| `:filter <type>` | Filter by type |
| `:filter` | Clear filter |
| `:new` | New note |
| `:all` | Show all |
| `:quit` | Quit |
| `q` | Quit |

#### Additional Shortcuts

| Key | Action |
|-----|--------|
| `?` | Show help |
| `Tab` | Cycle through filters |
| `r` | Reload data |

---

## Appendix A: Block Type Reference

| Type | Flag | Description |
|------|------|-------------|
| `fleeting` | `f` | Quick capture, temporary notes |
| `literature` | `l` | Notes from external sources |
| `permanent` | `p` | Atomic knowledge notes |
| `structure` | `s` | Document root, MOC |
| `hub` | `h` | Topic overview |
| `task` | `t` | Action items |
| `reference` | `r` | External links, citations |
| `outline` | `o` | Hierarchical structure |
| `ghost` | `g` | AI-detected gaps |

## Appendix B: Link Type Reference

| Type | Description |
|------|-------------|
| `section_of` | Block is a section of target |
| `supports` | Supporting evidence |
| `extends` | Extension of another block |
| `refines` | More specific version |
| `contradicts` | Opposite view |
| `questions` | Raises questions |
| `references` | External citation |
| `related` | Related content |
| `similar_to` | Similar content |
| `next` | Sequential relationship |
| `gravity` | Semantic attraction |

## Appendix C: Output Format Reference

| Format | Flag Value | Use Case |
|--------|------------|----------|
| `table` | `-o table` | Default display |
| `json` | `-o json` | Programmatic use |
| `simple` | `-o simple` | Compact display |
