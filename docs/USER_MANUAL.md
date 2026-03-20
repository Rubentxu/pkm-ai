# PKM-AI User Manual

> Personal Knowledge Management with Zettelkasten + Structural Spine

## 1. Installation

### Prerequisites

- **Rust 1.75+** (check with `rustc --version`)
- **Cargo** (included with Rust)
- **Git** (for cloning repository)
- **SurrealDB** (embedded, no separate installation needed)

---

### 1.1 macOS

#### Option A: Homebrew (Recommended)

```bash
# Clone repository
git clone https://github.com/rubentxu/pkmai
cd pkmai

# Install with cargo
cargo install --path .

# Or build and install manually
cargo build --release --locked
sudo mv target/release/pkmai /usr/local/bin/
sudo chmod +x /usr/local/bin/pkmai
```

#### Option B: Direct Binary

```bash
# Create a local bin directory
mkdir -p ~/bin

# Build the binary
git clone https://github.com/rubentxu/pkmai
cd pkmai
cargo build --release --locked

# Move to ~/bin
mv target/release/pkmai ~/bin/

# Add to PATH (add to ~/.zshrc or ~/.bash_profile)
echo 'export PATH="$HOME/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

#### Verify on macOS

```bash
pkmai --version
```

#### Troubleshooting macOS

```bash
# If "pkmai: command not found" after installation
source ~/.zshrc

# If security blocked on macOS
# System Preferences → Security & Privacy → Allow pkmai
# Or run: xattr -d com.apple.quarantine ~/bin/pkmai
```

---

### 1.2 Linux

#### Option A: Build from Source

```bash
# Install Rust if not installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Clone and build
git clone https://github.com/rubentxu/pkmai
cd pkmai
cargo build --release --locked

# Install globally
sudo cp target/release/pkmai /usr/local/bin/
sudo chmod +x /usr/local/bin/pkmai
```

#### Option B: User-level Installation

```bash
# Clone repository
git clone https://github.com/rubentxu/pkmai
cd pkmai

# Build release
cargo build --release --locked

# Install to ~/.local/bin
mkdir -p ~/.local/bin
mv target/release/pkmai ~/.local/bin/

# Add to PATH (add to ~/.bashrc or ~/.zshrc)
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

#### Verify on Linux

```bash
pkmai --version
```

#### Troubleshooting Linux

```bash
# If permission denied on /usr/local/bin
ls -la /usr/local/bin/pkmai

# Check library dependencies
ldd $(which pkmai)

# Install missing libraries if needed
sudo apt install libssl-dev # Debian/Ubuntu
sudo dnf install openssl-devel # Fedora
```

---

### 1.3 Windows

#### Option A: Winget (Recommended)

```powershell
# Install Rust via winget (if not installed)
winget install Rust.Rust

# Clone repository
git clone https://github.com/rubentxu/pkmai
cd pkmai

# Build
cargo build --release --locked

# Copy to a directory in PATH
mkdir $env:USERPROFILE\bin
copy target\release\pkmai.exe $env:USERPROFILE\bin\

# Add to PATH (PowerShell)
$env:Path += ";$env:USERPROFILE\bin"
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";$env:USERPROFILE\bin", "User")
```

#### Option B: Manual Installation

```powershell
# 1. Install Rust from https://rustup.rs

# 2. Clone repository
git clone https://github.com/rubentxu/pkmai
cd pkmai

# 3. Build
cargo build --release --locked

# 4. Copy binary to a convenient location
#    e.g., C:\Users\<YourUser>\bin\

# 5. Add to PATH:
#    Windows Settings → System → About → Advanced system settings
#    → Environment Variables → PATH → Edit → Add C:\Users\<YourUser>\bin
```

#### Verify on Windows

```powershell
pkmai --version
```

#### Troubleshooting Windows

```powershell
# If 'pkmai' is not recognized
# Close and reopen your terminal

# Check PATH
echo $env:Path

# Verify binary exists
Test-Path "$env:USERPROFILE\bin\pkmai.exe"
```

---

### 1.4 Cross-Platform: Cargo Install (All OS)

```bash
# Direct install from git repository
cargo install --git https://github.com/rubentxu/pkmai --locked

# Or from local source
cargo install --path . --locked
```

---

### Verify Installation

```bash
pkmai --version
pkmai --help
```

---

### Configuration

| Setting | Default | Environment Variable |
|---------|---------|---------------------|
| Database path | `~/.pkmai/` | `PKMAI_DB_PATH` |
| Auto-staging | `true` | `--stage` / `--no-stage` |
| Log level | `info` (silent) | `PKMAI_LOG_LEVEL` |
| Verbose mode | `false` | `--verbose` |

### Database Initialization

```bash
pkmai db init
```

---

### Shell Completions

For bash/zsh/fish autocomplete:

```bash
# Bash
source docs/shell-completion/bash/pkmai

# Zsh
source docs/shell-completion/zsh/_pkmai

# Fish
source docs/shell-completion/fish/pkmai.fish
```

---

### Quick Uninstall

```bash
# Remove binary
sudo rm /usr/local/bin/pkmai          # Linux (global)
rm ~/.local/bin/pkmai                  # Linux (user)
rm ~/bin/pkmai                         # macOS

# Windows: Delete from installation folder and PATH
```

---

## 2. Quick Start

### Quick Capture with `quick`

The `quick` command is the fastest way to capture an idea:

```bash
# Quick capture (fleeting by default)
pkmai quick "My idea about Rust ownership"

# With specific type
pkmai quick "Book note" -t literature

# With tags
pkmai quick "Important concept" -T "rust,memory"

# With auto-staging and auto-commit
# (default behavior, use --no-stage to disable)
```

The `quick` command automatically:
1. Creates the block
2. Adds it to staging
3. Commits with message "Quick capture: [first 50 characters]"

### Create Your First Block

```bash
# Create a permanent note
pkmai create -t permanent \
  --title "Actor Model Fundamentals" \
  --content "The actor model treats actors as the universal primitives of concurrent computation."

# Create a fleeting note (quick capture)
pkmai create -t fleeting \
  --title "Meeting Notes" \
  --content "Discussed project timeline and deliverables."

# Create a literature note (from sources)
pkmai create -t literature \
  --title "Rust Concurrency Patterns" \
  --content "Summary from 'Programming Rust' chapter on concurrency."
```

### Auto-Staging

By default, all blocks and links are auto-staged:

```bash
# Default behavior: auto-staging enabled
pkmai create -t permanent -T "New note"

# Disable auto-staging for a single command
pkmai create -t permanent -T "New note" --no-stage

# Globally disable (requires --stage to stage manually)
pkmai create -t permanent -T "New note" --no-stage
pkmai link <src> <dst> --no-stage
```

### List and Search Blocks

```bash
# List all blocks
pkmai list

# Filter by type
pkmai list -t fleeting
pkmai list -t permanent

# Fuzzy search (finds even with typos)
pkmai search "rust own"           # Finds "Rust Ownership Model"
pkmai list --search "prog"        # Finds "Rust Programming"

# Limit results
pkmai list -l 20

# Filter by tags
pkmai list --tags "rust,memory"
```

### AI Pre-Flight

When creating blocks with `--verbose`, AI information is shown:

```bash
# With AI suggestions
pkmai create -t permanent --title "Rust Ownership" --verbose

# Output:
# 🤖 AI Pre-Flight:
# ⚠️  Similar notes found (possible duplicates):
#    - "Rust Ownership Model" (0.94)
# 📍 Suggested location: "Rust Programming" (affinity: 0.72)
# 🏷️  Suggested tags: rust, memory, ownership
# 🔗 Suggested links: 3 notes
```

### Interactive Mode

For a more guided experience:

```bash
# Interactive mode with duplicate confirmation
pkmai create -t permanent --title "Rust Ownership" --interactive

# If >0.95 similarity duplicate found, asks:
# ⚠️ Similar note found: "Rust Ownership Model" (0.97)
# [y]es (use existing) / [n]o (create new) / [e]dit / [a]bort:
```

### Auto-Type Detection

Commands automatically detect block type:

```bash
# Auto-detects 'task' type by keywords
pkmai create "TODO: implement auth"

# Auto-detects 'reference' type by keywords
pkmai create "Quote from Programming Rust..."

# Auto-detects 'structure' type by keywords
pkmai create "Index: Rust Programming"
```

---

## 3. CLI Commands Reference

### Quick Capture Commands

| Command | Description | Example |
|---------|-------------|---------|
| `quick` | Capture + stage + commit | `pkmai quick "My idea"` |
| `quick -t literature` | With specific type | `pkmai quick "Note" -t literature` |
| `quick -T "tag1,tag2"` | With tags | `pkmai quick "Idea" -T "rust,memory"` |

### Block Commands

| Command | Description | Example |
|---------|-------------|---------|
| `create` | Create a new block | `pkmai create -t permanent -T "Title" -c "Content"` |
| `promote` | Change block type | `pkmai promote <id> -t permanent` |
| `list` | List blocks | `pkmai list -t fleeting -l 20` |
| `search` | Fuzzy search | `pkmai search "rust own"` |
| `show` | Show block details | `pkmai show 01ARZ3NDEKTSV4RRFFQ69G5FAV` |
| `grep` | Search content | `pkmai grep "pattern"` |
| `lint` | Validate structural integrity | `pkmai lint` |

### Global Flags

| Flag | Description |
|------|-------------|
| `--stage` | Force staging (default) |
| `--no-stage` | Disable auto-staging |
| `--verbose` | Show AI information |
| `--interactive` / `-i` | Interactive mode |
| `--db-path <path>` | Database path |

### Block Promotion

Valid Zettelkasten transitions:

```bash
# Promote to permanent (default)
pkmai promote 01ABC... -t permanent

# Promote fleeting to literature
pkmai promote 01ABC... -t literature

# Promote ghost to permanent (fill gap)
pkmai promote 01ABC... -t permanent --stage

# Valid transitions:
# fleeting → literature, permanent, ghost
# literature → permanent, ghost
# ghost → permanent
# structure/hub/task/reference/outline → permanent
```

### Link Commands

| Command | Description | Example |
|---------|-------------|---------|
| `link` | Create link between blocks | `pkmai link <block-id> <structure-id> --type section_of --weight 1.0` |

### Version Commands

| Command | Description | Example |
|---------|-------------|---------|
| `version status` | Check current status | `pkmai status` |
| `version commit` | Commit changes | `pkmai version commit -m "message"` |
| `version log` | View history | `pkmai version log` |
| `version log --oneline` | Compact history | `pkmai log --oneline` |
| `version branch` | Manage branches | `pkmai version branch` |

### Database Commands

| Command | Description | Example |
|---------|-------------|---------|
| `db init` | Initialize database | `pkmai db init` |
| `db stats` | Statistics | `pkmai db stats` |
| `db export` | Export to JSON | `pkmai db export > backup.json` |
| `db import` | Import from JSON | `pkmai db import backup.json` |

### Structure Commands

| Command | Description | Example |
|---------|-------------|---------|
| `traverse` | Traverse structural spine | `pkmai traverse -d 3` |
| `toc` | Generate Table of Contents | `pkmai toc <structure-id>` |
| `synthesize` | Synthesize document | `pkmai synthesize <structure-id> --template technical-whitepaper --output pdf` |
| `gravity-check` | Check semantic clustering | `pkmai gravity-check` |

### Ghost Node Commands

| Command | Description | Example |
|---------|-------------|---------|
| `ghost` | Manage ghost nodes | `pkmai ghost list` |
| `ghost fill` | Fill a ghost node | `pkmai ghost fill <ghost-id> --content "New content"` |
| `ghost dismiss` | Dismiss a ghost node | `pkmai ghost dismiss <ghost-id>` |

### Interactive Commands

| Command | Description |
|---------|-------------|
| `architect` | Launch interactive TUI for knowledge graph exploration |

---

## 4. Block Types

| Type | Purpose | Example | Auto-Detection Keywords |
|------|---------|---------|--------------------------|
| `fleeting` | Quick capture, temporary notes | Meeting notes, todos | - |
| `literature` | Notes from external sources | Book summaries, article notes | idea, note, observation |
| `permanent` | Atomic knowledge notes | Key insights, concepts | - |
| `structure` | Document root, organization | MOC (Map of Content) | index, moc, overview, summary |
| `hub` | Topic overview, index | Subject index | - |
| `task` | Action items | Todos, deliverables | TODO, fix, implement, complete |
| `reference` | External links, citations | URLs, citations | quote, book, chapter, author |
| `outline` | Hierarchical structure | Document outline | - |
| `ghost` | AI-detected gaps | Missing explanations | - |

### Zettelkasten Flow

```
Fleeting → Literature → Permanent
              ↓
           Ghost (detected by AI)
              ↓
           Permanent (filled)
```

### Block Type Hierarchy

```
structure
├── hub
│   ├── permanent
│   │   ├── literature
│   │   └── reference
│   └── outline
│       └── permanent
├── task
└── ghost
```

---

## 5. Link Types

| Type | Meaning | Use Case |
|------|---------|----------|
| `section_of` | Block is a section of structure | Chapter in document |
| `supports` | Supporting evidence | Citations, examples |
| `extends` | Extension of another block | Elaborations |
| `refines` | More specific version | Detailed explanation |
| `contradicts` | Opposite view | Debates, alternatives |
| `references` | External citation | Links to sources |
| `next` | Sequential relationship | Following block in spine |
| `gravity` | Semantic attraction | Related but not sequential |

### Link Syntax

```bash
# Basic link (auto-staged by default)
pkmai link <source-id> <target-id> --type supports

# With weight (for structural spine ordering)
pkmai link <block-id> <structure-id> --type section_of --weight 1.5

# Without auto-staging
pkmai link <source-id> <target-id> --type supports --no-stage
```

---

## 6. Structural Spine

The **Structural Spine** is the backbone of ordered knowledge, based on Zettelkasten's Folgezettel principle.

### Sequence Weight System

Weights use float-based ordering for flexible insertion:

```
1.0  → Block A (start)
1.5  → Block A1 (between 1.0 and 2.0)
2.0  → Block B
2.1  → Block B1 (sub-section of B)
2.2  → Block B2
3.0  → Block C
```

### Traverse the Spine

```bash
# Basic traversal
pkmai traverse

# With depth limit
pkmai traverse -d 3

# From specific block
pkmai traverse --from 01ARZ3NDEKTSV4RRFFQ69G5FAV
```

### Gravity Hooks

Semantic clustering connects related blocks across the spine:

```bash
# Check semantic clustering
pkmai gravity-check

# Output shows:
# - Blocks with high gravity attraction
# - Orphaned blocks (no connections)
# - Suggested gravity links
```

---

## 7. Document Synthesis

Transform your knowledge fragments into complete documents.

### Generate Table of Contents

```bash
# Generate TOC for a structure
pkmai toc 01ARZ3NDEKTSV4RRFFQ69G5FAV

# With depth limit
pkmai toc 01ARZ3NDEKTSV4RRFFQ69G5FAV --depth 3
```

### Synthesize Document

```bash
# Synthesize to Markdown
pkmai synthesize 01ARZ3NDEKTSV4RRFFQ69G5FAV --output markdown

# Synthesize to PDF (when typst feature enabled)
pkmai synthesize 01ARZ3NDEKTSV4RRFFQ69G5FAV --template technical-whitepaper --output pdf

# With custom template
pkmai synthesize <structure-id> --template my-template --output pdf
```

### Synthesis Pipeline

```
50 Zettels → Structure Note → TOC → Document
```

---

## 8. Ghost Nodes

Ghost nodes are AI-detected gaps in your knowledge base.

### List Ghost Nodes

```bash
pkmai ghost list
```

### Ghost Node Properties

| Property | Description |
|----------|-------------|
| `confidence` | AI confidence score (0-1) |
| `expected_keywords` | Keywords AI predicts should be there |
| `parent_block` | Block that detected the gap |

### Fill or Dismiss Workflow

```bash
# Fill a ghost node with content
pkmai ghost fill <ghost-id> --content "New explanatory content"

# Or use promote (fill the gap)
pkmai promote <ghost-id> -t permanent --content "Content"

# Dismiss if gap is not relevant
pkmai ghost dismiss <ghost-id>
```

---

## 9. Configuration

### Environment Variables

```bash
# Database location
export PKMAI_DB_PATH=~/.pkmai/

# Log level (trace, debug, info, warn, error)
export PKMAI_LOG_LEVEL=info

# Enable AI features (requires ai-integration feature)
export PKMAI_AI_ENABLED=true
```

### Global Flags

```bash
# Verbose mode (shows debug logs)
pkmai --verbose list

# Database path
pkmai --db-path /tmp/pkmai.db list

# Auto-staging (default: true)
pkmai create -t permanent -T "Note"     # Auto-staged
pkmai create -t permanent -T "Note" --no-stage  # Not staged
pkmai link <src> <dst> --type supports       # Auto-staged
pkmai link <src> <dst> --type supports --no-stage
```

### Feature Flags

Build with specific features:

```bash
# Default (RocksDB storage)
cargo build --release

# In-memory storage (testing)
cargo build --release --features memory

# With AI integration
cargo build --release --features ai-integration

# With Typst PDF rendering
cargo build --release --features typst
```

---

## 10. Interactive TUI

Launch the interactive terminal UI for knowledge graph exploration:

```bash
pkmai architect
```

### Vim-Style Navigation

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

### Links and Navigation

| Key | Action |
|-----|--------|
| `b` | Navigate to first backlink (in detail mode) |
| `o` | Navigate to first outgoing link (in detail mode) |

### Commands

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

### Shortcuts

| Key | Action |
|-----|--------|
| `?` | Show help |
| `Tab` | Cycle through filters |
| `r` | Reload data |

### TUI Features

- Visual block browsing with type emojis
- View backlinks and outgoing links in detail mode
- Quick commands for search and filter
- Built-in help panel
- Colors for better readability

---

## 11. Troubleshooting

### Database Issues

```bash
# Reinitialize database (WARNING: deletes existing data)
rm -rf ~/.pkmai/
pkmai db init
```

### Performance

```bash
# Run with logging to identify slow commands
pkmai --verbose list
```

### Validation

```bash
# Check structural integrity
pkmai lint

# Auto-fix minor issues
pkmai lint --fix
```

### Debug Auto-Staging

```bash
# See what's staged
pkmai status

# Manual staging if disabled
# (use individual commands with --stage)
```

---

## 12. Quick Reference Card

```bash
# ===== QUICK CAPTURE =====
pkmai quick "My idea"              # Fleeting + stage + commit
pkmai quick "Note" -t literature   # Specific type

# ===== CREATE =====
pkmai create -t permanent -T "Title" -c "Content"
pkmai create -t permanent -T "Title" --verbose  # With AI pre-flight
pkmai create -t permanent -T "Title" --interactive  # Interactive mode

# Auto-type detection
pkmai create "TODO: do something"    # → Task
pkmai create "Quote from book..."    # → Reference

# ===== AUTO-STAGING =====
pkmai create -t permanent -T "Note"          # Staged (default)
pkmai create -t permanent -T "Note" --no-stage  # Not staged
pkmai link <src> <dst> --type supports       # Staged (default)
pkmai link <src> <dst> --type supports --no-stage

# ===== SEARCH =====
pkmai list                          # All
pkmai list -t permanent             # By type
pkmai list --tags "rust,memory"   # By tags
pkmai list --search "rust"         # Fuzzy search
pkmai search "rust own"            # Fuzzy search (dedicated)
pkmai grep "pattern"              # Content search

# ===== NAVIGATE =====
pkmai show <id>                    # View block
pkmai architect                    # Interactive TUI

# ===== ORGANIZE =====
pkmai promote <id> -t permanent    # Change type
pkmai link <src> <dst> --type supports
pkmai traverse                    # Walk spine
pkmai toc <structure-id>           # Generate TOC

# ===== VERSION =====
pkmai status                       # Current status
pkmai version commit -m "message"  # Commit
pkmai log --oneline               # Compact history
pkmai version branch              # Branches

# ===== AI =====
pkmai ghost list                  # View gaps
pkmai ghost fill <id> --content "..."  # Fill
pkmai ghost dismiss <id>          # Dismiss
pkmai gravity-check               # Check clustering

# ===== DATABASE =====
pkmai db stats                     # Statistics
pkmai db export > backup.json      # Export
pkmai db import backup.json        # Import
```

---

## 13. Feature Changelog

### v0.x - Roadmap Completed

**Phase 1: Core Workflow**
- `pkmai quick` - Quick capture with auto-stage and commit
- `pkmai promote` - Zettelkasten type transitions
- Global `--stage`/`--no-stage` flags

**Phase 2: AI Pre-Flight**
- Duplicate detection by similarity >0.9
- Location suggestion (semantic affinity)
- Tag suggestions based on similar notes
- Link suggestions

**Phase 3: Search and UI**
- Fuzzy search by title
- Interactive mode with `[y/n/e/a]` options
- Auto-detection of block type

**Phase 4: UX Polish**
- `pkmai status` - Git-like status
- `pkmai log --oneline` - Compact history
- Emojis in list output
- Shell completions (bash/zsh/fish)
- TUI with vim navigation and `:` commands
