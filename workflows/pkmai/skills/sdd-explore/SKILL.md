---
name: sdd-explore
description: >
  SDD Explore Phase - Research and gather information about a topic.
  Trigger: When assigned as sdd-explore phase sub-agent.
license: MIT
metadata:
  author: pkm-ai
  version: "1.0"
---

## Purpose

Explore a topic thoroughly to understand the problem space before proposing solutions. This phase:
- Researches the topic using web search
- Identifies key concepts and relationships
- Documents findings as an exploration artifact
- Does NOT propose solutions (that's for sdd-propose)

## What You Receive

- `change-name`: The name of the change being explored
- `topic`: The topic or problem space to explore

## Execution

### Step 1: Load Skills

Load `workflows/pkmai/sdd/_shared/phase-common.md` for return format.
Load `workflows/pkmai/sdd/_shared/pkmai-convention.md` for PKM-AI conventions.

### Step 2: Research Topic

Use web search to gather information:
- Search for existing solutions and approaches
- Identify key concepts and terminology
- Find related projects or case studies
- Note potential challenges or risks

### Step 3: Search for Existing Exploration

Before creating, check if exploration already exists:

```json
{
  "tool": "search_blocks",
  "arguments": {"query": "sdd/{change-name}/explore", "tags": ["sdd-explore"]}
}
```

If found with matching `change-name`, update existing. If not found, create new.

### Step 4: Create Exploration Block

Create a block in PKM-AI:

```json
{
  "tool": "create_block",
  "arguments": {"block_type": "permanent", "title": "sdd/{change-name}/explore", "content": "# Exploration: {topic}\n\n## Summary\n{Brief description of the exploration}\n\n## Key Concepts\n- {concept 1}\n- {concept 2}\n- {concept 3}\n\n## Current State\n{How things work currently}\n\n## Challenges Identified\n- {challenge 1}\n- {challenge 2}\n\n## Opportunities\n- {opportunity 1}\n- {opportunity 2}\n\n## Related Research\n- {source 1}\n- {source 2}\n\n## Metadata\n```json\n{\n  \"change\": \"{change-name}\",\n  \"phase\": \"explore\",\n  \"topic\": \"{topic}\",\n  \"researched\": \"{ISO date}\"\n}\n```\n", "tags": ["sdd", "explore", "sdd-explore", "sdd-{change-name}"]}
}
```

### Step 5: Return Envelope

Return structured summary per phase-common.md format.

## Exploration Template

```markdown
# Exploration: {topic}

## Summary
{2-3 sentences on what this exploration covers}

## Problem Space
{What problem are we trying to solve?}

## Key Concepts
1. **{Concept}**: {brief explanation}
2. **{Concept}**: {brief explanation}

## Current Approaches
{How is this typically solved today?}

## Technologies & Tools
{What tools/technologies are relevant?}

## Challenges
- {challenge description}
- {challenge description}

## Opportunities
- {opportunity description}

## Questions to Resolve
- {question that needs answering}
- {question that needs answering}

## Research Sources
- [Source Title](URL) - {brief note}
- [Source Title](URL) - {brief note}

## Next Steps
- Proceed to sdd-propose with this research
- May need additional research on specific areas
```

## Rules

- Be thorough - explore before proposing
- Use web search to find real information
- Document challenges and opportunities
- Don't propose solutions - that's for the proposal phase
- Save significant findings to PKM-AI as you go
- Return structured envelope with all required fields
- Check for existing exploration before creating new

## Output

Creates: `sdd/{change-name}/explore` block
Links: None (first phase, no parent)

## PKM-AI Tool Reference

```json
{
  "tool": "search_blocks",
  "arguments": {"query": "sdd/{change-name}/explore", "tags": ["sdd-explore"]}
}
```

```json
{
  "tool": "create_block",
  "arguments": {"block_type": "permanent", "title": "sdd/{change-name}/explore", "content": "{markdown content}", "tags": ["sdd", "explore", "sdd-explore", "sdd-{change-name}"]}
}
```
