---
name: skill-registry
description: >
  PKM-AI Skill Registry - Discover, index, and register skills for PKM-AI.
  Trigger: When user asks to list skills, register a skill, or find a skill for a task.
license: MIT
metadata:
  author: pkm-ai
  version: "1.0"
---

## Purpose

Discover, index, and register skills within PKM-AI. This skill maintains a centralized registry of all available skills, enabling the orchestrator to select the appropriate skill for any given task.

## When to Use

- User asks to "list skills", "show available skills", or "what skills exist"
- User asks to "register a skill" or "add skill to registry"
- Orchestrator needs to find a skill for a specific task
- User invokes `/skill-registry` or similar
- Starting a new PKM-AI session and loading available skills

## What You Receive

- `action`: The action to perform (`discover`, `register`, `lookup`, `sync`)
- `query`: Optional search query for finding skills
- `skill_path`: Optional path to a skill to register
- `skill_metadata`: Optional metadata for registration

## Execution

### Action: Discover (List All Skills)

Discover all registered skills in PKM-AI:

```json
[
  {
    "tool": "search_blocks",
    "args": {
      "query": "skill-registry",
      "tags": ["skill-registry", "pkmai"],
      "limit": 10
    }
  }
]
```

If no registry exists:

```json
[
  {
    "tool": "create_block",
    "args": {
      "block_type": "permanent",
      "title": "skill-registry",
      "content": "# Skill Registry\n\n## Overview\nCentral registry of all PKM-AI skills.\n\n## Skill Index\n\n| Skill | Path | Purpose | Trigger Phrases |\n|-------|------|---------|-----------------|\n| (empty) | - | - | - |\n\n## Statistics\n- Total Skills: 0\n- Last Updated: {ISO date}\n\n## Notes\nAuto-generated registry. Use `skill-registry` skill to update.",
      "tags": ["skill-registry", "pkmai"]
    }
  }
]
```

Get existing registry:

```json
[
  {
    "tool": "get_block",
    "args": {
      "block_id": "{results.blocks[0].id}",
      "include_content": true
    }
  }
]
```

### Action: Register (Add New Skill)

Register a new skill in PKM-AI:

```python
# Step 1: Validate skill exists
import os
skill_md_path = f"${PKM_AI_SKILLS:-~/.pkm-ai/skills}/{skill_path}/SKILL.md"

if not os.path.exists(skill_md_path):
    raise Exception(f"Skill not found: {skill_path}")

# Step 2: Read skill metadata
with open(skill_md_path, "r") as f:
    content = f.read()

# Extract frontmatter
import re
frontmatter = re.match(r'^---\n(.*?)\n---', content, re.DOTALL)
if frontmatter:
    metadata = yaml.safe_load(frontmatter.group(1))
    name = metadata.get("name", skill_path)
    description = metadata.get("description", "No description")
else:
    name = skill_path
    description = "No description"

# Step 3: Find registry
results = search_blocks(
    query="skill-registry",
    tags=["skill-registry", "pkmai"],
    limit=5
)

# Step 4: Update registry with new skill
# Parse existing content and add new entry
```

### Action: Lookup (Find Skill for Task)

Find the appropriate skill for a given task:

```python
# Step 1: Search for matching skills
results = search_blocks(
    query="skill",
    tags=["skill"],
    limit=20
)

# Step 2: Also check filesystem for skill directories
import os
skill_dirs = []
skills_base = os.path.expanduser(f"${PKM_AI_SKILLS:-~/.pkm-ai/skills}")
for root, dirs, files in os.walk(skills_base):
    if "SKILL.md" in files:
        skill_dirs.append(root.replace(skills_base + "/", ""))

# Step 3: Match query to skills
# Look for trigger phrases or name matches
```

### Action: Sync (Sync Registry with Filesystem)

Sync the PKM-AI registry with actual skill files on disk:

```python
# Step 1: Scan for all SKILL.md files
import os
skills = []
skills_base = os.path.expanduser(f"${PKM_AI_SKILLS:-~/.pkm-ai/skills}")

for root, dirs, files in os.walk(skills_base):
    if "SKILL.md" in files:
        skill_path = root.replace(skills_base + "/", "")
        skills.append(scan_skill(skill_path))

# Step 2: Update or create registry block
```

## Skill Scanning

Scan a skill directory and extract metadata:

```python
def scan_skill(skill_path):
    """Scan a skill directory and return metadata."""
    import re
    import os

    skill_md_path = f"${PKM_AI_SKILLS:-~/.pkm-ai/skills}/{skill_path}/SKILL.md"

    if not os.path.exists(skill_md_path):
        return None

    with open(skill_md_path, "r") as f:
        content = f.read()

    # Extract frontmatter
    frontmatter = re.match(r'^---\n(.*?)\n---', content, re.DOTALL)
    if frontmatter:
        metadata = yaml.safe_load(frontmatter.group(1))
        return {
            "name": metadata.get("name", skill_path),
            "description": metadata.get("description", "No description"),
            "path": skill_path,
            "version": metadata.get("metadata", {}).get("version", "1.0"),
            "author": metadata.get("metadata", {}).get("author", "unknown"),
            "trigger_phrases": extract_trigger_phrases(content),
            "ulid": None  # Will be set when stored in PKM-AI
        }

    return {
        "name": skill_path,
        "description": "No description",
        "path": skill_path,
        "version": "1.0",
        "author": "unknown",
        "trigger_phrases": [],
        "ulid": None
    }

def extract_trigger_phrases(content):
    """Extract trigger phrases from skill content."""
    import re
    phrases = []

    # Look for When to Use section
    when_match = re.search(r'## When to Use\n(.*?)(?:\n##|\Z)', content, re.DOTALL | re.IGNORECASE)
    if when_match:
        section = when_match.group(1)
        # Extract bullet points as trigger phrases
        bullets = re.findall(r'^\s*-\s*(.+)$', section, re.MULTILINE)
        phrases.extend([b.strip() for b in bullets if b.strip()])

    return phrases[:5]  # Limit to 5 trigger phrases
```

## Registry Block Format

```markdown
# Skill Registry

## Overview
Central registry of all PKM-AI skills.

## Skill Index

| Skill | Path | Purpose | Trigger Phrases |
|-------|------|---------|----------------- |
| sdd-init | skills/sdd-init | Initialize SDD project | sdd-init, new change |
| sdd-explore | skills/sdd-explore | Research phase | sdd-explore, research |
| skill-registry | skills/skill-registry | Skill discovery | skill-registry, list skills |
| branch-pr | skills/branch-pr | Create PR | create-pr, branch-pr |
| issue-creation | skills/issue-creation | Create issues | create-issue, issue |

## Statistics
- Total Skills: 5
- Last Updated: 2026-03-20T10:00:00Z

## Directory Structure

### ~/.pkm-ai/skills/
User-defined skills in PKM-AI skills directory.

### ${PKM_AI_SKILLS:-~/.pkm-ai/skills}/
PKM-AI native skills for SDD workflow.

## Notes
This registry is auto-generated and synced with filesystem.
```

## Return Envelope

```markdown
## Skill Registry Complete

**Action**: {action}
**Status**: success | failed

### Skills Found: {count}

| Skill | Path | Purpose |
|-------|------|---------|
| {name} | {path} | {description} |

### Registry Updated
- ULID: {registry_ulid}
- Last Sync: {ISO date}

### Next Actions
- Use `skill-registry` with `lookup` action to find skills for specific tasks
- Use `skill-registry` with `register` action to add new skills
- Use `skill-registry` with `sync` action to resync with filesystem
```

## Search Directories

The skill-registry scans these locations for skills:

| Directory | Priority | Description |
|-----------|----------|-------------|
| `~/.pkm-ai/skills/` | High | User-defined project skills |
| `${PKM_AI_SKILLS:-~/.pkm-ai/skills}/` | High | PKM-AI native skills |
| `skills/` | Medium | Legacy skill directory |

## Rules

- Always validate skill path exists before registering
- Use `permanent` block type for registry
- Tag registry with `skill-registry` and `pkmai`
- Sync registry with filesystem on `sync` action
- Return all found skills in tabular format
- Extract trigger phrases from skill content for better matching

## PKM-AI Tool Reference

```json
[
  {
    "tool": "search_blocks",
    "args": {
      "query": "skill-registry",
      "tags": ["skill-registry", "pkmai"]
    }
  },
  {
    "tool": "create_block",
    "args": {
      "block_type": "permanent",
      "title": "skill-registry",
      "content": "{registry_content}",
      "tags": ["skill-registry", "pkmai"]
    }
  },
  {
    "tool": "update_block",
    "args": {
      "block_id": "{registry_ulid}",
      "content": "{updated_content}"
    }
  },
  {
    "tool": "create_link",
    "args": {
      "source_id": "{registry_ulid}",
      "target_id": "{skill_ulid}",
      "link_type": "contains"
    }
  }
]
```

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-03-20 | Initial version for PKM-AI |
