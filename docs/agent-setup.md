# Agent Setup — PKM-AI Memory Protocol

> Configure AI agents to persist knowledge across sessions using PKM-AI.

**Version:** 1.0
**Date:** 2026-03-20

---

## Quick Reference

| Agent | Setup Complexity | MCP Support | Memory Protocol | Auto-Save |
|-------|-----------------|-------------|-----------------|-----------|
| **Claude Code** | Easy | Native | Full | Session hooks |
| **OpenCode** | Easy | Native | Full | Session hooks |
| **Gemini CLI** | Easy | Native | Full | Session hooks |
| **Codex** | Medium | MCP Only | Partial | Manual |
| **VS Code (Continue)** | Medium | MCP Only | Partial | Manual |
| **Cursor** | Medium | MCP Only | Partial | Manual |
| **Windsurf** | Medium | MCP Only | Partial | Manual |
| **Other MCP Agents** | Medium | Native | Full | Depends |

---

## 1. Prerequisites

### 1.1 System Requirements

- **PKM-AI installed** (`pkmai` or `pkm-ai` binary)
- **MCP server running** (`pkmai mcp` or `pkm-ai mcp`)
- **Project context**: Knowledge base initialized in your project

### 1.2 Verify Installation

```bash
# Verify PKM-AI is installed
pkmai --version

# Verify MCP server starts
pkmai mcp --help

# Initialize project knowledge base (if not exists)
pkmai init
```

---

## 2. Claude Code Setup

### Prerequisites

- Claude Code installed (`npm install -g @anthropic/claude-code`)
- PKM-AI MCP server accessible

### Setup Command

```bash
# Add PKM-AI as a Claude Code feature
claude code --add-feature pkm-memory

# Or via environment variable
export PKM_MCP_SERVER="pkmai mcp"
```

### Manual Configuration

1. Create configuration file: `~/.claude/settings.json`

```json
{
  "mcpServers": {
    "pkm": {
      "command": "pkmai",
      "args": ["mcp"],
      "env": {
        "PKM_PROJECT": "/path/to/your/project"
      }
    }
  },
  "memory": {
    "provider": "pkm",
    "autoSave": true,
    "sessionHooks": true
  }
}
```

### Memory Protocol Integration

```bash
# Start session with automatic tracking
pkmai session start --agent claude-code --project myproject

# Claude Code will now:
# - Save architectural decisions automatically
# - Search knowledge before making major changes
# - Link new findings to existing concepts
```

### Session Lifecycle Hooks

```rust
// .claude/hooks/session_start.sh
#!/bin/bash
pkmai session start \
  --agent "claude-code" \
  --project "$(basename $(pwd))" \
  --session-id "${CLAUDE_SESSION_ID}"

// .claude/hooks/session_end.sh
#!/bin/bash
pkmai session end \
  --session-id "${CLAUDE_SESSION_ID}" \
  --summary "$(cat /dev/stdin)"
```

---

## 3. OpenCode Setup

### Prerequisites

- OpenCode installed
- PKM-AI MCP server running

### Setup Command

```bash
# Initialize OpenCode with PKM memory
opencode --setup-memory pkm

# Or configure manually
opencode config set memory.provider pkm
opencode config set mcp.enabled true
```

### Manual Configuration

1. Create `~/.opencode/config.yaml`:

```yaml
memory:
  provider: pkm
  autoSave: true
  searchOnStart: true

mcp:
  servers:
    - name: pkm
      command: pkmai
      args: [mcp]
      autoConnect: true

agent:
  session:
    trackDecisions: true
    linkToContext: true
```

### Memory Protocol Integration

```bash
# OpenCode will automatically:
# - Save code patterns discovered during session
# - Link bugs fixed to knowledge base
# - Search similar past solutions before attempting fixes
```

---

## 4. Gemini CLI Setup

### Prerequisites

- Google Gemini CLI installed
- PKM-AI MCP server accessible

### Setup Command

```bash
# Add PKM memory to Gemini CLI
gemini --setup pkm-memory

# Configure API key
export GEMINI_API_KEY="your-api-key"
```

### Manual Configuration

1. Create `~/.geminirc`:

```bash
# PKM Integration
GEMINI_MEMORY_PROVIDER=pkm
GEMINI_MCP_SERVER=pkmai:mcp
GEMINI_AUTO_MEMORY=true
```

2. Or via `gemini config`:

```bash
gemini config set memory.provider pkm
gemini config set memory.autoSave true
gemini config set mcp.servers.pkm.command pkmai
gemini config set mcp.servers.pkm.args mcp
```

### Memory Protocol Integration

```bash
# Gemini CLI will now:
# - Persist research findings to PKM
# - Link code reviews to architectural decisions
# - Search past sessions for similar problems
```

---

## 5. Codex Setup

### Prerequisites

- Codex CLI installed (`pip install openai-codex` or binary)
- MCP-aware Codex version

### Setup Command

```bash
# Configure Codex to use PKM MCP server
codex config set mcp.servers.pkm.command "pkmai"
codex config set mcp.servers.pkm.args "mcp"
```

### Manual Configuration

1. Create `~/.codex/config.json`:

```json
{
  "mcpServers": {
    "pkm": {
      "command": "pkmai",
      "args": ["mcp"]
    }
  },
  "memory": {
    "enabled": true,
    "provider": "pkm"
  }
}
```

### Limitations

- Codex has limited MCP tool support
- Manual memory saves required: `pkmai save --message "..."`
- Session compaction not automatic

### Memory Protocol Integration

```bash
# Manual memory saves
pkmai create --type permanent --title "Bug Fix: Auth Token Expiry" --content "
**Problem**: Users logged out after 30 minutes
**Root Cause**: Token expiry not refreshed on refresh
**Solution**: Implemented sliding window expiry
**Files**: src/auth/token.rs, src/middleware/auth.rs
"

# Search before starting
pkmai search "auth token refresh"
```

---

## 6. VS Code (Continue Extension) Setup

### Prerequisites

- VS Code installed
- Continue extension installed (or other MCP-capable extension)

### Setup Command

1. Install Continue extension from marketplace
2. Add PKM MCP server in VS Code settings:

```json
{
  "continue.serverConfigs": {
    "pkm": {
      "command": "pkmai",
      "args": ["mcp"],
      "env": {
        "PKM_PROJECT": "${workspaceFolder}"
      }
    }
  }
}
```

### Manual Configuration

1. Create `~/.continue/config.py`:

```python
from continuedev.src.continuedev.core.config import ContinueConfig

def modify_config(config: ContinueConfig):
    config.mcp_servers = [
        {
            "name": "pkm",
            "command": "pkmai",
            "args": ["mcp"],
            "env": {"PKM_PROJECT": "${workspace_folder}"}
        }
    ]
    return config
```

### Memory Protocol Integration

- Right-click selected code → "Save to PKM"
- Use command palette: "PKM: Search Knowledge Base"
- Automatic capture of chat history (configurable)

---

## 7. Cursor Setup

### Prerequisites

- Cursor IDE installed
- MCP support enabled

### Setup Command

1. Open Cursor Settings → MCP Servers
2. Add new MCP server:

```
Name: PKM-AI
Command: pkmai
Args: mcp
```

### Manual Configuration

1. Edit `~/.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "pkm": {
      "command": "pkmai",
      "args": ["mcp"],
      "env": {
        "PKM_PROJECT": "${workspaceFolder}"
      }
    }
  }
}
```

### Memory Protocol Integration

```bash
# Cursor will use PKM for:
# - Storing code snippets with context
# - Linking related concepts across sessions
# - Searching past solutions
```

---

## 8. Windsurf Setup

### Prerequisites

- Windsurf IDE installed
- MCP-enabled Windsurf version

### Setup Command

1. Open Windsurf Settings → External Tools
2. Add MCP server:

```
Name: PKM-AI Memory
Command: pkmai
Args: mcp
```

### Manual Configuration

1. Create `~/.windsurf/mcp.json`:

```json
{
  "servers": {
    "pkm": {
      "command": "pkmai",
      "args": ["mcp"]
    }
  }
}
```

### Memory Protocol Integration

- Windsurf will use PKM for:
  - Context-aware code suggestions
  - Architectural decision tracking
  - Bug fix history with rationale

---

## 9. Other MCP Agents

### Generic MCP Setup

For any MCP-compatible agent, add the following to their MCP configuration:

```json
{
  "mcpServers": {
    "pkm": {
      "command": "pkmai",
      "args": ["mcp"],
      "env": {
        "PKM_PROJECT": "${WORKSPACE}",
        "PKM_AUTO_SAVE": "true"
      }
    }
  }
}
```

### Tools Available to All MCP Agents

| Tool | Purpose | Key Parameters |
|------|---------|----------------|
| `create_block` | Create knowledge block | `block_type`, `title`, `content`, `tags` |
| `search_blocks` | Search knowledge | `query`, `block_type`, `limit` |
| `get_block` | Retrieve block | `block_id` |
| `update_block` | Update block | `block_id`, `title`, `content`, `tags` |
| `create_link` | Link blocks | `from_id`, `to_id`, `link_type` |
| `get_links` | Get connections | `block_id`, `direction` |
| `suggest_links` | AI link suggestions | `block_id`, `limit` |
| `traverse_spine` | Navigate structure | `root_id`, `depth` |
| `gravity_check` | Check connectivity | `block_id` |
| `synthesize` | Generate document | `structure_id`, `format` |

---

## 10. Surviving Compaction

When agents compact their context window, memories stored in PKM survive.

### What Gets Compacted

| Context Type | Survives? | Storage |
|--------------|-----------|---------|
| Chat history | No | PKM (if saved) |
| Working memory | No | PKM (if saved) |
| Learned patterns | **Yes** | PKM permanent blocks |
| Architectural decisions | **Yes** | PKM permanent blocks |
| Bug fixes & rationale | **Yes** | PKM permanent blocks |
| Code patterns | **Yes** | PKM permanent blocks |
| Project conventions | **Yes** | PKM structure blocks |

### Compaction Recovery Protocol

```bash
# 1. Before compaction, agent should:
pkmai session checkpoint --session-id "${AGENT_SESSION_ID}"

# 2. After compaction, restore context:
pkmai session restore --session-id "${AGENT_SESSION_ID}"

# 3. Search for relevant past knowledge:
pkmai search "similar problem" --limit 10
```

### Block Types for Compaction Survival

| Type | Use Case | TTL |
|------|----------|-----|
| `permanent` | Evergreen knowledge | Infinite |
| `structure` | Project architecture | Infinite |
| `literature` | External references | Infinite |
| `fleeting` | Temporary notes | 7 days |
| `task` | Action items | Until done |

---

## 11. PKM Memory Protocol

### When to Save Sessions

Save to PKM when:

- [ ] **Architectural decisions**: Design choices, trade-offs, rationale
- [ ] **Bug fixes**: Root cause, solution, files affected
- [ ] **Patterns discovered**: Coding patterns, conventions, idioms
- [ ] **Configuration changes**: Environment, build, deployment
- [ ] **New dependencies**: Why added, alternatives considered
- [ ] **Test strategies**: What was tested, what passed, edge cases

### When to Search Knowledge

Search PKM before:

- [ ] **Starting new feature**: "How did we implement similar X?"
- [ ] **Debugging**: "Have we seen this error before?"
- [ ] **Refactoring**: "What's the context around this code?"
- [ ] **Adding dependencies**: "Why do we use X instead of Y?"
- [ ] **Major changes**: "What decisions were made about this?"

### Session Lifecycle Hooks

#### Automatic Hooks (Recommended)

```bash
# .claude/hooks/session_start.sh
#!/bin/bash
export PKM_SESSION_ID="$(uuidgen)"
pkmai session start \
  --agent "claude-code" \
  --project "$(basename $(pwd))" \
  --session-id "$PKM_SESSION_ID" \
  --cwd "$(pwd)"

# .claude/hooks/session_end.sh
#!/bin/bash
pkmai session end \
  --session-id "$PKM_SESSION_ID" \
  --auto-summary true
```

#### Manual Hooks (Fallback)

```bash
# Start session manually
pkmai session start --agent opencode --project myapp

# Work normally...

# End session with summary
pkmai session end --summary "Fixed auth bug, refactored user service"
```

### Recommended Block Types by Content

| Content Type | Block Type | Example Title |
|--------------|------------|---------------|
| Decision | `permanent` | `DECISION: Use async/await for I/O` |
| Bug Fix | `permanent` | `FIX: Race condition in cache` |
| Pattern | `permanent` | `PATTERN: Repository with Unit of Work` |
| Convention | `structure` | `CODE_STYLE: Error handling` |
| Concept | `permanent` | `理解: Event Sourcing vs CRUD` |
| Reference | `literature` | `REF: Rust error handling guide` |
| Task | `task` | `TODO: Migrate to new auth provider` |

### Memory Search Examples

```bash
# Find all bug fixes
pkmai search --type permanent --query "FIX:" | head -20

# Find architectural decisions
pkmai search --type permanent --query "DECISION:" | head -20

# Find patterns by language
pkmai search --type permanent --query "PATTERN: Rust" | head -20

# Find all from current project
pkmai search --project "$(basename $(pwd))" --limit 50
```

---

## 12. Troubleshooting

### MCP Server Not Starting

```bash
# Check if pkmai is in PATH
which pkmai

# Start MCP server manually
pkmai mcp

# Check for errors
pkmai doctor
```

### Agent Not Connecting to MCP

```bash
# Verify MCP server responds
echo '{"jsonrpc":"2.0","method":"tools/list","id":1}' | pkmai mcp

# Check agent's MCP configuration
# Ensure correct path to pkmai binary
```

### Memory Not Persisting

```bash
# Check PKM database location
pkmai config get database.path

# Verify write permissions
touch "$(pkmai config get database.path)/test"

# Check session status
pkmai session list
```

---

## 13. Configuration Reference

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PKM_PROJECT` | Current dir | Project context |
| `PKM_DB_PATH` | `~/.pkm/data.db` | Database path |
| `PKM_AUTO_SAVE` | `false` | Auto-save session |
| `PKM_MCP_TIMEOUT` | `30000` | MCP timeout (ms) |

### CLI Commands

| Command | Description |
|---------|-------------|
| `pkmai session start` | Start knowledge session |
| `pkmai session end` | End and summarize session |
| `pkmai session list` | List recent sessions |
| `pkmai session restore` | Restore session context |
| `pkmai quick "message"` | Quick capture |
| `pkmai search "query"` | Search knowledge base |

---

## 14. See Also

- [MCP README](./mcp/README.md) — MCP server documentation
- [API Reference](./mcp/API.md) — Complete tool reference
- [Concepts](./CONCEPTS.md) — Block types and link types
- [User Manual](./USER_MANUAL.md) — General PKM-AI usage
- [PKM Zettelkasten Analysis](./pkm-zettelkasten-rust-analysis.md) — Design rationale

---

**Last updated:** 2026-03-20
