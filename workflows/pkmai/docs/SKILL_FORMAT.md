# PKM-AI Skill Format

## Overview

PKM-AI skills follow the **Agent Skills open standard** - a universal format for AI agent capabilities. Skills are stored as Markdown files with YAML frontmatter.

## File Structure

```
skill-name/
└── SKILL.md          # Single file containing all skill metadata
```

## SKILL.md Format

```markdown
---
name: skill-name
description: >
  Brief description of what this skill does.
  Trigger: When user asks to do X or Y.
license: MIT
version: "1.0"
metadata:
  author: pkm-ai
  conventions:
    required:
      - phase-common
      - pkmai-convention
triggers:
  - trigger phrase 1
  - trigger phrase 2
---

## Purpose

{What this skill accomplishes}

## When to Use

{Trigger phrases and scenarios that invoke this skill}

## What You Receive

{Inputs/parameters the skill receives}

## Execution

### Step 1: [Action]
{Step description with code examples}

### Step 2: [Action]
{More steps...}

## Output

{Created artifacts and relationships}

## Rules

- Rule 1
- Rule 2

## PKM-AI Tool Reference

```json
{JSON examples of MCP tool usage}
```
```

## Frontmatter Fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Unique skill identifier (kebab-case) |
| `description` | Yes | Trigger description + purpose |
| `license` | Yes | License (MIT recommended) |
| `version` | Yes | Semantic version (1.0, 1.0.0) |
| `metadata` | No | Author, dependencies |
| `conventions` | No | Required shared conventions |
| `triggers` | No | Explicit trigger phrases |

## Required Sections

### Purpose

Clear explanation of what the skill does.

### When to Use

Trigger phrases and scenarios that should invoke this skill.

### Execution

Step-by-step instructions with code examples.

## Optional Sections

### Output

Description of artifacts created.

### Rules

Constraints and guidelines for skill execution.

### PKM-AI Tool Reference

JSON examples of MCP tool calls.

## Shared Conventions

Skills can declare dependencies on shared conventions:

```yaml
conventions:
  required:
    - phase-common
    - pkmai-convention
```

These are loaded from `${PKM_AI_SHARED}` at runtime.

## Versioning

| Version | When to Increment |
|---------|-------------------|
| Major | Breaking changes to format or conventions |
| Minor | New features, new skills |
| Patch | Bug fixes, documentation |

## Examples

See installed skills at `~/.pkm-ai/skills/` for complete examples.
