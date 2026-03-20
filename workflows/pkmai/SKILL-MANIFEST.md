# SKILL-MANIFEST.md

Complete index of all PKM-AI skills for Spec-Driven Development workflows.

## Overview

| Total Skills | 12 |
|--------------|----|
| SDD Phase Skills | 9 |
| Utility Skills | 3 |

---

## SDD Phase Skills

SDD (Spec-Driven Development) phases execute in sequence:

```
sdd-init → sdd-explore → sdd-propose → sdd-spec → sdd-design → sdd-tasks → sdd-apply → sdd-verify → sdd-archive
```

### 1. sdd-init

| Field | Value |
|-------|-------|
| **Path** | `skills/sdd-init` |
| **Purpose** | Initialize a new SDD change project |
| **Trigger** | `/sdd-init`, new change, initialize project |

### 2. sdd-explore

| Field | Value |
|-------|-------|
| **Path** | `skills/sdd-explore` |
| **Purpose** | Research and gather information |
| **Trigger** | `/sdd-explore`, research, explore topic |

### 3. sdd-propose

| Field | Value |
|-------|-------|
| **Path** | `skills/sdd-propose` |
| **Purpose** | Create proposal defining problem/solution |
| **Trigger** | `/sdd-propose`, create proposal |

### 4. sdd-spec

| Field | Value |
|-------|-------|
| **Path** | `skills/sdd-spec` |
| **Purpose** | Detailed specification with Given/When/Then |
| **Trigger** | `/sdd-spec`, create spec |

### 5. sdd-design

| Field | Value |
|-------|-------|
| **Path** | `skills/sdd-design` |
| **Purpose** | Architectural decisions |
| **Trigger** | `/sdd-design`, architect, design phase |

### 6. sdd-tasks

| Field | Value |
|-------|-------|
| **Path** | `skills/sdd-tasks` |
| **Purpose** | Break into actionable tasks |
| **Trigger** | `/sdd-tasks`, task breakdown |

### 7. sdd-apply

| Field | Value |
|-------|-------|
| **Path** | `skills/sdd-apply` |
| **Purpose** | Execute implementation tasks |
| **Trigger** | `/sdd-apply`, implement |

### 8. sdd-verify

| Field | Value |
|-------|-------|
| **Path** | `skills/sdd-verify` |
| **Purpose** | Validate implementation against spec |
| **Trigger** | `/sdd-verify`, verify |

### 9. sdd-archive

| Field | Value |
|-------|-------|
| **Path** | `skills/sdd-archive` |
| **Purpose** | Summarize completed change |
| **Trigger** | `/sdd-archive`, complete |

---

## Utility Skills

### 10. branch-pr

| Field | Value |
|-------|-------|
| **Path** | `skills/branch-pr` |
| **Purpose** | Create GitHub pull request |
| **Trigger** | `/branch-pr`, `/create-pr` |

### 11. issue-creation

| Field | Value |
|-------|-------|
| **Path** | `skills/issue-creation` |
| **Purpose** | Create GitHub issues |
| **Trigger** | `/create-issue`, create issue |

### 12. skill-registry

| Field | Value |
|-------|-------|
| **Path** | `skills/skill-registry` |
| **Purpose** | Discover and manage skills |
| **Trigger** | `/skill-registry`, list skills |

---

## Skill Index Table

| Skill | Path | Phase | Block Type |
|-------|------|-------|------------|
| sdd-init | skills/sdd-init | 0 | structure/outline |
| sdd-explore | skills/sdd-explore | 1 | permanent |
| sdd-propose | skills/sdd-propose | 2 | permanent |
| sdd-spec | skills/sdd-spec | 3 | structure |
| sdd-design | skills/sdd-design | 4 | permanent |
| sdd-tasks | skills/sdd-tasks | 5 | outline |
| sdd-apply | skills/sdd-apply | 6 | permanent |
| sdd-verify | skills/sdd-verify | 7 | permanent |
| sdd-archive | skills/sdd-archive | 8 | permanent |
| branch-pr | skills/branch-pr | Utility | - |
| issue-creation | skills/issue-creation | Utility | - |
| skill-registry | skills/skill-registry | Utility | permanent |

---

## Phase Flow

```
sdd-init
    ↓
sdd-explore → sdd-propose
                   ↓
              sdd-spec
                   ↓
              sdd-design
                   ↓
              sdd-tasks
                   ↓
              sdd-apply
                   ↓
              sdd-verify
                   ↓
              sdd-archive
                   ↓
              branch-pr
```

---

## Installation

Skills are installed via the install script:

```bash
curl -fsSL https://raw.githubusercontent.com/pkm-ai/pkm-ai/main/scripts/install.sh | bash
source ~/.pkm-ai/.envrc
```

---

## Documentation

- [Quick Start](docs/QUICKSTART.md)
- [Installation Guide](docs/INSTALLATION.md)
- [Skill Format](docs/SKILL_FORMAT.md)
- [Shared Conventions](docs/SHARED_CONVENTIONS.md)
