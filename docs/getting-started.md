# Getting Started with PKM-AI

> Your Personal Knowledge Management System with Zettelkasten + AI

**Estimated time:** 5 minutes to first capture, 30 minutes to productivity.

---

## Table of Contents

1. [Quick Start (5 min)](#1-quick-start-5-min)
2. [Installation](#2-installation)
3. [First Steps](#3-first-steps)
4. [Basic Workflow](#4-basic-workflow)
5. [Next Steps](#5-next-steps)

---

## 1. Quick Start (5 min)

### 1.1 What is PKM-AI?

PKM-AI is a **Knowledge Operating System** that treats knowledge as a graph of interconnected blocks. Unlike traditional note-taking apps, PKM-AI uses:

- **Block-Atom Model**: Every piece of knowledge is an addressable block with a unique ULID identifier
- **Structural Spine**: Order and structure are first-class citizens
- **Git-like Versioning**: Full version control for your knowledge base
- **AI Assistance**: Ghost nodes detect gaps, synthesis generates documents

### 1.2 Your First Capture

Let's capture your first idea:

```bash
# Quick capture (creates + stages + commits in one step)
pkmai quick "My first note about PKM-AI"
```

You should see output similar to:

```
[CREATED] Block 01ARZ3NDEKTSV4RRFFQ69G5FAV
[fleeting] My first note about PKM-AI

[STAGED] Ready to commit
[COMMIT] a1b2c3d - Quick capture: My first note about PKM-AI
```

### 1.3 Verify It Works

```bash
# List all your blocks
pkmai list

# Or search for it
pkmai search "PKM"
```

Expected output:

```
01ARZ3NDEKTSV4RRFFQ69G5FAV  [f] My first note about PKM-AI
                              created: 2026-03-20T10:30:00Z
```

### 1.4 The Core Concept: Block Types

PKM-AI uses the **Zettelkasten methodology** with these block types:

| Type | Alias | Purpose | Example |
|------|-------|---------|---------|
| `fleeting` | `f` | Quick captures, temporary | Meeting notes, TODOs |
| `literature` | `l` | Notes from external sources | Book summaries, article notes |
| `permanent` | `p` | Atomic knowledge, evergreen | Concepts, insights |
| `structure` | `s`, `moc` | Document containers | Index, Map of Content |
| `hub` | `h` | Topic entry points | Subject indexes |
| `task` | `t` | Action items | Deliverables, bugs |
| `reference` | `r` | External references | URLs, citations |
| `outline` | `o` | Hierarchical outlines | Document structure |
| `ghost` | `g` | AI-detected gaps | Missing explanations |

**The Zettelkasten Flow:**

```
Fleeting (capture) → Literature (process) → Permanent (elaborate)
                                                    ↓
                                               Structure (organize)
```

---

## 2. Installation

### Prerequisites

- **Rust 1.75+** (check with `rustc --version`)
- **Cargo** (included with Rust)
- **Git**

### 2.1 Linux

#### Option A: Build from Source (Recommended)

```bash
# 1. Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 2. Clone the repository
git clone https://github.com/rubentxu/pkmai
cd pkmai

# 3. Build
cargo build --release

# 4. Install
sudo cp target/release/pkmai /usr/local/bin/
sudo chmod +x /usr/local/bin/pkmai
```

#### Option B: User-level Installation

```bash
git clone https://github.com/rubentxu/pkmai
cd pkmai
cargo build --release
mkdir -p ~/.local/bin
mv target/release/pkmai ~/.local/bin/
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### 2.2 macOS

#### Option A: Homebrew

```bash
git clone https://github.com/rubentxu/pkmai
cd pkmai
brew install rust
cargo build --release
cargo install --path .
```

#### Option B: Direct Binary

```bash
git clone https://github.com/rubentxu/pkmai
cd pkmai
cargo build --release
mkdir -p ~/bin
mv target/release/pkmai ~/bin/
echo 'export PATH="$HOME/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

### 2.3 Windows

```powershell
# 1. Install Rust from https://rustup.rs

# 2. Clone and build
git clone https://github.com/rubentxu/pkmai
cd pkmai
cargo build --release

# 3. Copy to a directory in PATH
mkdir $env:USERPROFILE\bin
copy target\release\pkmai.exe $env:USERPROFILE\bin\

# 4. Add to PATH (PowerShell)
$env:Path += ";$env:USERPROFILE\bin"
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";$env:USERPROFILE\bin", "User")
```

### 2.4 Verify Installation

```bash
pkmai --version
pkmai --help
```

Expected output:

```
pkmai 1.0.0
PKM-AI - Personal Knowledge Management with AI

USAGE:
    pkmai [OPTIONS] <COMMAND>

COMMANDS:
    quick      Quick capture: create + stage + commit
    create     Create a new block
    list       List blocks
    show       Show block details
    link       Create links between blocks
    ...
```

### 2.5 Initialize Database

```bash
pkmai db init
```

This creates the database at `~/.pkmai/` by default.

---

## 3. First Steps

### 3.1 Quick Capture Workflow

The fastest way to capture ideas:

```bash
# Basic quick capture (fleeting type)
pkmai quick "Meeting with team about Q2 planning"

# With specific type
pkmai quick "Rust async patterns" -t literature

# With tags
pkmai quick "Important insight" -T "rust,architecture"

# Combined
pkmai quick "Book note on design patterns" -t literature -T "books,patterns"
```

### 3.2 Creating Blocks Directly

For more control, use `create`:

```bash
# Create a permanent note
pkmai create -t permanent \
  --title "Actor Model Fundamentals" \
  --content "The actor model treats actors as the universal primitives of concurrent computation. Each actor has a mailbox and communicates via message passing."

# Create with tags
pkmai create -t permanent \
  --title "Rust Ownership" \
  --content "Ownership is Rust's unique memory management feature..." \
  -T "rust,memory,safety"

# Create a structure (Map of Content)
pkmai create -t structure \
  --title "Rust Programming Index" \
  --content "Main index for Rust programming notes"
```

### 3.3 Listing and Searching

```bash
# List all blocks (default limit: 50)
pkmai list

# Filter by type
pkmai list -t permanent
pkmai list -t fleeting

# Search by title (fuzzy)
pkmai search "rust own"

# Search in content (regex)
pkmai grep "ownership"
pkmai grep "TODO|FIXME" -i

# Filter by tags
pkmai list -T "rust,concurrency"
```

### 3.4 Viewing Block Details

```bash
# Show block by ULID
pkmai show 01ARZ3NDEKTSV4RRFFQ69G5FAV

# Show with related links
pkmai show 01ARZ3NDEKTSV4RRFFQ69G5FAV --related
```

Expected output:

```
[PERMANENT] 01ARZ3NDEKTSV4RRFFQ69G5FAV
──────────────────────────────────────
Title:   Actor Model Fundamentals
Type:    permanent
Tags:    concurrency,actors
Created: 2026-03-20T10:30:00Z
Updated: 2026-03-20T10:30:00Z

Content:
The actor model treats actors as the universal primitives
of concurrent computation. Each actor has a mailbox and
communicates via message passing.

Links (3):
  → 01ARZ3NDEKTSV4RRFFQ69G5FA1 (extends)
  → 01ARZ3NDEKTSV4RRFFQ69G5FA2 (supports)
  ← 01ARZ3NDEKTSV4RRFFQ69G5FA3 (extends)
```

### 3.5 Linking Blocks

Create relationships between blocks:

```bash
# Basic link (related type)
pkmai link --from <ULID_1> --to <ULID_2>

# Semantic links
pkmai link --from <ULID_1> --to <ULID_2> -t extends
pkmai link --from <ULID_1> --to <ULID_2> -t supports
pkmai link --from <ULID_1> --to <ULID_2> -t refines

# Structural links
pkmai link --from <ULID_1> --to <ULID_STRUCTURE> -t section_of
pkmai link --from <ULID_1> --to <ULID_2> -t next
```

**Link Types Explained:**

| Type | Meaning | Use Case |
|------|---------|----------|
| `extends` | Block extends another | Elaborations on a concept |
| `refines` | More specific version | Detailed explanation |
| `supports` | Supporting evidence | Citations, examples |
| `contradicts` | Opposite view | Debates |
| `references` | External citation | Sources |
| `section_of` | Block belongs to structure | Chapter in document |
| `next` | Sequential relationship | Following block in spine |
| `related` | Related (default) | General association |

### 3.6 Version Control

PKM-AI has full Git-like version control:

```bash
# Check status
pkmai version status

# Stage changes (auto-done by default)
pkmai version add <ULID>

# Commit with message
pkmai version commit -m "Add actor model notes"

# View history
pkmai version log
pkmai version log --oneline

# Branches
pkmai version branch                        # List
pkmai version branch my-branch             # Create
pkmai version checkout my-branch            # Switch
pkmai version checkout -b new-branch        # Create + switch
```

---

## 4. Basic Workflow

### 4.1 Daily Workflow Example

```bash
# 1. Morning: Quick capture thoughts
pkmai quick "Idea about distributed caching"
pkmai quick "Note from standup meeting"
pkmai quick "Reference to interesting article" -t literature

# 2. Review: Convert fleeting to permanent
pkmai promote <FLEETING_ULID> -t permanent

# 3. Create structure for a project
pkmai create -t structure \
  --title "Project X Index" \
  --content "Main index for Project X documentation"

# 4. Link related blocks
pkmai link --from <NOTE_ULID> --to <STRUCTURE_ULID> -t section_of

# 5. Commit your work
pkmai version commit -m "Daily capture: project notes"
```

### 4.2 The Structural Spine

The **Structural Spine** is the ordered backbone of documents, based on Zettelkasten's Folgezettel principle:

```bash
# Create ordered blocks
pkmai create -t permanent --title "Introduction" -c "..."
pkmai create -t permanent --title "Chapter 1" -c "..."

# Link them in sequence
pkmai link --from <INTRO_ULID> --to <CHAPTER1_ULID> -t next

# Traverse the spine
pkmai traverse -d 5
```

### 4.3 Auto-Staging

By default, all changes are auto-staged:

```bash
# This creates AND stages automatically
pkmai create -t permanent --title "New note" --content "..."

# Disable auto-staging
pkmai create -t permanent --title "New note" --content "..." --no-stage
pkmai link --from <A> --to <B> --no-stage

# Manual staging required then
pkmai version add <ULID>
pkmai version commit -m "message"
```

### 4.4 Zettelkasten Promotion

Promote notes through the knowledge hierarchy:

```bash
# Fleeting → Literature
pkmai promote <ULID> -t literature

# Literature → Permanent
pkmai promote <ULID> -t permanent

# Any → Structure (as index)
pkmai promote <ULID> -t structure
```

### 4.5 Interactive Mode

For guided creation with AI pre-flight checks:

```bash
# Interactive mode detects duplicates
pkmai create -t permanent --title "Rust Ownership" --interactive

# Output:
# 🤖 AI Pre-Flight Check:
# ⚠️  Similar note found: "Rust Ownership Model" (0.94 similarity)
# 📍 Suggested location: "Rust Programming" (affinity: 0.72)
# 🏷️  Suggested tags: rust, memory, ownership
# 🔗 Suggested links: 3 notes
#
# [y]es (use existing) / [n]o (create new) / [e]dit / [a]bort:
```

---

## 5. Next Steps

### 5.1 Zettelkasten Advanced

**Creating an Atomic Note:**

```bash
# 1. Literature note (from a source)
pkmai create -t literature \
  --title "Notes on 'Programming Rust' Ch.3" \
  --content "Chapter 3 covers concurrency patterns..."

# 2. Promote to permanent (your own synthesis)
pkmai create -t permanent \
  --title "Rust Concurrency Patterns" \
  --content "Based on Programming Rust, the main patterns are..."

# 3. Link them
pkmai link --from <PERMANENT_ULID> --to <LITERATURE_ULID> -t supports
```

**Building a Structure (MOC):**

```bash
# Create index structure
pkmai create -t structure \
  --title "Distributed Systems Index" \
  --content "Comprehensive index of distributed systems notes"

# Add sections
pkmai link --from <SECTION1_ULID> --to <INDEX_ULID> -t section_of
pkmai link --from <SECTION2_ULID> --to <INDEX_ULID> -t section_of

# View table of contents
pkmai toc <INDEX_ULID>
```

### 5.2 AI Features

**Ghost Nodes (AI-detected gaps):**

```bash
# List detected gaps
pkmai ghost list

# View ghost details
pkmai ghost show <GHOST_ULID>

# Fill a ghost node
pkmai ghost fill <GHOST_ULID> --content "The explanation goes here..."

# Or promote to permanent
pkmai promote <GHOST_ULID> -t permanent --content "Content"
```

**Gravity Check (Semantic Clustering):**

```bash
# Find related blocks
pkmai gravity-check <ULID>
pkmai gravity-check <ULID> -t 0.8  # Higher threshold = stricter
```

**Document Synthesis:**

```bash
# Generate TOC
pkmai toc <STRUCTURE_ULID>

# Synthesize to Markdown
pkmai synthesize <STRUCTURE_ULID> -o markdown

# Synthesize to HTML
pkmai synthesize <STRUCTURE_ULID> -o html

# Synthesize to PDF (requires typst)
pkmai synthesize <STRUCTURE_ULID> -o pdf
```

### 5.3 Interactive TUI

Launch the visual knowledge graph explorer:

```bash
pkmai architect
```

**Navigation:**

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `h` / `←` | Collapse |
| `l` / `→` | Expand |
| `Enter` | View detail |
| `Esc` | Go back |
| `:` | Command mode |

**Commands in TUI:**

```
:search <query>  Fuzzy search
:filter <type>  Filter by type
:new             New note
:quit            Exit
```

### 5.4 Version Control Advanced

**Merging Branches:**

```bash
# Switch to branch
pkmai version checkout main

# Merge
pkmai version merge feature-branch

# View diff
pkmai version diff <ULID>
```

**Remote Sync:**

```bash
# Add remote
pkmai version remote add origin https://github.com/user/pkm.git

# Push
pkmai version push

# Pull
pkmai version pull
```

### 5.5 Maintenance

**Database Statistics:**

```bash
pkmai db stats
```

**Validate Integrity:**

```bash
# Check for issues
pkmai lint

# Auto-fix
pkmai lint --fix
```

**Export/Import:**

```bash
# Export
pkmai db export --format json > backup.json

# Import
pkmai db import backup.json
```

---

## Quick Reference Card

```bash
# ===== CAPTURE =====
pkmai quick "My idea"                    # Fleeting + stage + commit
pkmai quick "Note" -t literature         # Specific type
pkmai quick "Important" -T "tag1,tag2"   # With tags

# ===== CREATE =====
pkmai create -t permanent -T "Title" -c "Content"
pkmai create -t structure -T "Index"

# ===== SEARCH =====
pkmai list                               # All blocks
pkmai list -t permanent                  # By type
pkmai search "query"                     # Fuzzy search
pkmai grep "pattern"                      # Content search

# ===== VIEW =====
pkmai show <ULID>                        # Block details
pkmai architect                          # Interactive TUI

# ===== LINK =====
pkmai link --from <A> --to <B> -t extends

# ===== VERSION =====
pkmai version status
pkmai version commit -m "message"
pkmai version log --oneline
pkmai version branch

# ===== ORGANIZE =====
pkmai promote <ULID> -t permanent
pkmai toc <STRUCTURE_ULID>
pkmai traverse -d 5

# ===== AI =====
pkmai ghost list
pkmai gravity-check <ULID>
pkmai synthesize <STRUCTURE_ULID> -o markdown

# ===== MAINTENANCE =====
pkmai lint --fix
pkmai db stats
pkmai db export > backup.json
```

---

## Troubleshooting

### Database Issues

```bash
# Reinitialize (WARNING: deletes data)
rm -rf ~/.pkmai/
pkmai db init
```

### Performance

```bash
# Run with verbose logging
pkmai --verbose list
```

### Help

```bash
pkmai --help
pkmai <command> --help
```

---

## Next Steps for Power Users

1. **Read the User Manual:** `docs/USER_MANUAL.md`
2. **Understand Concepts:** `docs/CONCEPTS.md`
3. **CLI Reference:** `docs/cli-cheat-sheet.md`
4. **Architecture:** `docs/SPEC.md`

---

**Last updated:** 2026-03-20
**Version:** 1.0.0
