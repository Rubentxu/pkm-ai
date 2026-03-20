# Installation Guide for PKM-AI Workflow

## Prerequisites

### Required Components

| Component | Version | Purpose |
|-----------|---------|---------|
| Claude Code | Latest | AI coding agent |
| PKM-AI MCP Server | 1.0+ | Persistent artifact store |
| Rust toolchain | 1.75+ | For building projects |

### Optional Components

| Component | Purpose |
|-----------|---------|
| `just` | Build task runner |
| `wasm-pack` | For WASM builds |
| Node.js 18+ | For web frontend |

## PKM-AI MCP Server Setup

### Step 1: Install PKM-AI

```bash
# Clone the repository
git clone https://github.com/your-org/pkm-ai.git
cd pkm-ai

# Build the server
cargo build --release

# Install globally (optional)
cargo install --path .
```

### Step 2: Configure Claude Code MCP

Add PKM-AI to your Claude Code MCP configuration:

```json
// ~/.claude/settings.json (global)
// or
// {project}/.claude/settings.json (project)

{
  "mcpServers": {
    "pkmai": {
      "command": "pkmai",
      "args": ["mcp"],
      "env": {}
    }
  }
}
```

### Step 3: Verify Installation

Test the PKM-AI MCP server:

```bash
# Check if pkmai is installed
pkmai --version

# Test MCP connectivity
pkmai mcp --test
```

Expected output:
```
PKM-AI MCP Server v1.0.0
Status: Connected
Tools available: 15
```

### Step 4: Initialize PKM-AI Database

```bash
# Initialize the database (creates ~/.pkmai if not exists)
pkmai init

# Verify database
pkmai stats
```

Expected output:
```
Database: ~/.pkmai/pkmai.db
Blocks: 0
Edges: 0
```

## Claude Code Configuration

### Step 1: Create Project CLAUDE.md

In your project root, create `.claude/CLAUDE.md`:

```bash
mkdir -p {project}/.claude
```

Add the following content:

```markdown
# PKM-AI Workflow Integration

This project uses PKM-AI for persistent artifact storage.

## Workflow Commands

- `/sdd-init {change-name}` - Initialize new SDD change
- `/sdd-explore {change-name}` - Explore a topic
- `/sdd-propose {change-name}` - Create proposal
- `/sdd-spec {change-name}` - Write specification
- `/sdd-design {change-name}` - Create design
- `/sdd-tasks {change-name}` - Break into tasks
- `/sdd-apply {change-name}` - Implement tasks
- `/sdd-verify {change-name}` - Verify implementation
- `/sdd-archive {change-name}` - Archive completed change

## Persistence Modes

| Mode | Description |
|------|-------------|
| `pkmai` | Default. Artifacts in PKM-AI blocks |
| `openspec` | Artifacts in filesystem |
| `hybrid` | Both PKM-AI and filesystem |
| `none` | Inline only (not recommended) |

## SDD Workflow

See `workflows/pkmai/docs/` for full documentation.
```

### Step 2: Configure Skill Paths

Ensure Claude Code can find the workflow skills:

```json
// In your project's .claude/settings.json
{
  "skills": {
    "searchPaths": [
      "workflows/pkmai/skills",
      "workflows/pkmai/sdd/phases",
      ".claude/skills"
    ]
  }
}
```

### Step 3: Verify Configuration

Restart Claude Code and verify PKM-AI tools are available:

```bash
# In Claude Code, run:
/help mcp

# Or test directly:
mcp search_blocks query="test" limit=1
```

## Project Initialization with sdd-init

### Step 1: Initialize a New Change

```bash
# In Claude Code:
/sdd-init my-new-feature
```

This creates:
- Project context block
- Phase tracker block
- Initial discovery block

### Step 2: Verify Created Blocks

```python
# Search for created blocks
search_blocks(
    query="sdd-my-new-feature",
    tags=["sdd"],
    limit=10
)
```

Expected output:
```json
{
  "blocks": [
    {
      "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
      "block_type": "structure",
      "title": "sdd/my-new-feature/project",
      "tags": ["sdd", "project", "sdd-my-new-feature"]
    },
    {
      "id": "01ARZ3NDEKTSV4RRFFQ69G5FAW",
      "block_type": "outline",
      "title": "sdd/my-new-feature/tracker",
      "tags": ["sdd", "tracker", "sdd-my-new-feature"]
    },
    {
      "id": "01ARZ3NDEKTSV4RRFFQ69G5FAZ",
      "block_type": "permanent",
      "title": "sdd/my-new-feature/discovery",
      "tags": ["sdd", "discovery", "sdd-my-new-feature"]
    }
  ],
  "count": 3
}
```

## Skill Configuration

### Skill Search Paths

The orchestrator looks for skills in these locations:

```
~/.claude/skills/                      # User global skills
{project}/.claude/skills/              # Project skills
{project}/workflows/pkmai/skills/      # PKM-AI skills
{project}/workflows/pkmai/sdd/phases/  # SDD phase skills
{project}/workflows/pkmai/sdd/         # SDD root skills
```

### SDD Skill Files

| Skill | Path | Purpose |
|-------|------|---------|
| Base SDD | `workflows/pkmai/sdd/SKILL.md` | Common SDD conventions |
| sdd-init | `workflows/pkmai/skills/sdd-init/SKILL.md` | Initialize change |
| sdd-explore | `workflows/pkmai/sdd/phases/sdd-explore.md` | Exploration phase |
| sdd-propose | `workflows/pkmai/sdd/phases/sdd-propose.md` | Proposal phase |
| sdd-spec | `workflows/pkmai/sdd/phases/sdd-spec.md` | Specification phase |
| sdd-design | `workflows/pkmai/sdd/phases/sdd-design.md` | Design phase |
| sdd-tasks | `workflows/pkmai/sdd/phases/sdd-tasks.md` | Task breakdown |
| sdd-apply | `workflows/pkmai/sdd/phases/sdd-apply.md` | Implementation |
| sdd-verify | `workflows/pkmai/sdd/phases/sdd-verify.md` | Verification |
| sdd-archive | `workflows/pkmai/sdd/phases/sdd-archive.md` | Archive |

### Shared Conventions

Shared files loaded by all phases:

| File | Path | Purpose |
|------|------|---------|
| PKM-AI Conventions | `workflows/pkmai/sdd/_shared/pkmai-convention.md` | PKM-AI-specific rules |
| Phase Common | `workflows/pkmai/sdd/_shared/phase-common.md` | Return envelope format |

## Running the Full SDD Pipeline

### Option 1: Interactive Mode

```bash
# Start exploration
/sdd-explore my-feature

# When exploration complete, proceed to propose
/sdd-propose my-feature

# Continue through all phases...
/sdd-spec my-feature
/sdd-design my-feature
/sdd-tasks my-feature
/sdd-apply my-feature
/sdd-verify my-feature
/sdd-archive my-feature
```

### Option 2: Full Pipeline Command

```bash
# Run all phases for a new change
/sdd-new my-feature
```

This runs all phases sequentially (or in parallel when possible).

### Option 3: Resume Incomplete Pipeline

```bash
# Continue from where a previous run left off
/sdd-continue my-feature
```

## Troubleshooting

### PKM-AI Tools Not Available

**Symptom**: `Unknown tool: search_blocks`

**Solution**:
1. Check MCP configuration:
   ```json
   {
     "mcpServers": {
       "pkmai": {
         "command": "pkmai",
         "args": ["mcp"]
       }
     }
   }
   ```

2. Restart Claude Code

3. Verify server runs standalone:
   ```bash
   pkmai mcp --test
   ```

### Skills Not Found

**Symptom**: `Skill not found: sdd-explore`

**Solution**:
1. Verify skill paths in settings:
   ```bash
   ls workflows/pkmai/sdd/phases/
   # Should contain sdd-explore.md
   ```

2. Check skill search paths include the workflow directory

### Database Errors

**Symptom**: `Database error: unable to open`

**Solution**:
1. Initialize database:
   ```bash
   pkmai init
   ```

2. Check permissions:
   ```bash
   ls -la ~/.pkmai/
   ```

3. Recreate if corrupted:
   ```bash
   rm -rf ~/.pkmai
   pkmai init
   ```

## Verification Checklist

After installation, verify everything works:

- [ ] `pkmai --version` returns version
- [ ] `pkmai mcp --test` shows connected
- [ ] Claude Code MCP shows pkmai server
- [ ] PKM-AI tools available (search_blocks, create_block, etc.)
- [ ] SDD skills accessible
- [ ] `/sdd-init test-change` creates blocks
- [ ] `search_blocks(query="sdd-test-change", tags=["sdd"])` finds created blocks

## Next Steps

- Read [concepts.md](concepts.md) to understand SDD phases
- Read [persistence.md](persistence.md) to understand storage modes
- Read [sub-agents.md](sub-agents.md) to understand skill execution
- Read [token-economics.md](token-economics.md) for efficiency analysis