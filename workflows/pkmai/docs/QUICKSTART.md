# PKM-AI Quick Start

## What is PKM-AI?

PKM-AI is a skill-based workflow system for AI agents that enables Spec-Driven Development (SDD). It provides portable skills for managing knowledge, running structured development processes, and integrating with GitHub.

## Installation (30 seconds)

```bash
curl -fsSL https://raw.githubusercontent.com/pkm-ai/pkm-ai/main/scripts/install.sh | bash
source ~/.pkm-ai/.envrc
```

## Skills Overview

### SDD Phase Skills (9)

| Skill | Purpose |
|-------|---------|
| `sdd-init` | Initialize a new SDD project |
| `sdd-explore` | Research a topic |
| `sdd-propose` | Create a proposal |
| `sdd-spec` | Write detailed specification |
| `sdd-design` | Make architectural decisions |
| `sdd-tasks` | Break into tasks |
| `sdd-apply` | Implement changes |
| `sdd-verify` | Validate implementation |
| `sdd-archive` | Complete the change |

### Utility Skills (3)

| Skill | Purpose |
|-------|---------|
| `branch-pr` | Create a GitHub PR |
| `issue-creation` | Create GitHub issues |
| `skill-registry` | Discover skills |

## Quick Examples

### Start a New Change

```bash
# In Claude Code or similar AI assistant
/sdd-init my-new-feature
```

### Run the Full SDD Pipeline

```bash
/sdd-new my-feature
```

### Create a GitHub PR

```bash
/branch-pr
```

## Environment Variables

After installation, these are available:

```bash
export PKM_AI_DIR="$HOME/.pkm-ai"
export PKM_AI_SHARED="$PKM_AI_DIR/sdd/_shared"
export PKM_AI_SKILLS="$PKM_AI_DIR/skills"
```

## Next Steps

1. Read [Installation Guide](INSTALLATION.md) for all install options
2. Read [Skill Format](SKILL_FORMAT.md) to understand skill structure
3. Explore `~/.pkm-ai/skills/` for all available skills
4. Read [Shared Conventions](SHARED_CONVENTIONS.md) for SDD patterns
