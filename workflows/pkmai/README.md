# PKM-AI Skills

Portable AI agent skills for Spec-Driven Development (SDD) workflows.

## Overview

PKM-AI provides a set of reusable skills that enable AI agents to run structured development workflows. Skills follow the Agent Skills open standard and work across platforms.

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/pkm-ai/pkm-ai/main/scripts/install.sh | bash
source ~/.pkm-ai/.envrc
```

See [docs/INSTALLATION.md](docs/INSTALLATION.md) for all installation methods.

## Skills

### SDD Phase Skills (9)

| Skill | Purpose |
|-------|---------|
| [sdd-init](skills/sdd-init/) | Initialize SDD project |
| [sdd-explore](skills/sdd-explore/) | Research topic |
| [sdd-propose](skills/sdd-propose/) | Create proposal |
| [sdd-spec](skills/sdd-spec/) | Write specification |
| [sdd-design](skills/sdd-design/) | Architectural decisions |
| [sdd-tasks](skills/sdd-tasks/) | Task breakdown |
| [sdd-apply](skills/sdd-apply/) | Implement changes |
| [sdd-verify](skills/sdd-verify/) | Validate implementation |
| [sdd-archive](skills/sdd-archive/) | Complete change |

### Utility Skills (3)

| Skill | Purpose |
|-------|---------|
| [branch-pr](skills/branch-pr/) | Create GitHub PR |
| [issue-creation](skills/issue-creation/) | Create GitHub issues |
| [skill-registry](skills/skill-registry/) | Discover skills |

## Quick Start

```bash
# Install
curl -fsSL https://raw.githubusercontent.com/pkm-ai/pkm-ai/main/scripts/install.sh | bash

# Source environment
source ~/.pkm-ai/.envrc

# List skills
~/.pkm-ai/scripts/list-skills.sh
```

## Documentation

- [Quick Start](docs/QUICKSTART.md) - Get started in 30 seconds
- [Installation Guide](docs/INSTALLATION.md) - All installation methods
- [Skill Format](docs/SKILL_FORMAT.md) - Understanding skill structure
- [Shared Conventions](docs/SHARED_CONVENTIONS.md) - SDD patterns

## Architecture

```
pkm-ai/
├── skills/              # 12 skill directories
│   ├── sdd-*/          # SDD phase skills
│   └── branch-pr/       # GitHub integration
├── sdd/
│   ├── SKILL.md        # Composite SDD workflow
│   └── _shared/        # Shared conventions
├── scripts/            # Installation scripts
└── docs/               # Documentation
```

## License

MIT License - see LICENSE file.
