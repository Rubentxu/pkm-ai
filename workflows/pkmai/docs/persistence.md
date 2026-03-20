# PKM-AI Persistence Modes

## Overview

The PKM-AI workflow supports four persistence modes that determine where and how SDD artifacts are stored. The mode affects storage location, retrieval strategy, and recovery behavior.

| Mode | PKM-AI Storage | Filesystem Storage | Use Case |
|------|---------------|-------------------|----------|
| `pkmai` | Primary | None | Default, recommended |
| `openspec` | None | Primary | Filesystem-first workflows |
| `hybrid` | Primary | Secondary | Backup + searchability |
| `none` | None | None | Temporary, not recommended |

## Mode Selection

### Configuration

Set mode in project context:

```json
{
  "change": "mcp-workflow",
  "mode": "pkmai",
  "project": "hodei-pkm"
}
```

### Mode Decision Guide

| Condition | Recommended Mode |
|-----------|------------------|
| Default, most projects | `pkmai` |
| Want filesystem backup | `hybrid` |
| Need filesystem artifacts | `openspec` |
| Temporary experiment | `none` |
| Team prefers git-based artifacts | `openspec` or `hybrid` |
| Solo development | `pkmai` or `hybrid` |

---

## Mode: pkmai (Default)

**Default mode.** Artifacts stored exclusively in PKM-AI as blocks with tags.

### Characteristics

| Aspect | Behavior |
|--------|----------|
| Storage | PKM-AI blocks only |
| Filesystem | Not used |
| Search | PKM-AI `search_blocks` |
| Recovery | PKM-AI lookup |
| Git tracking | Not automatic |

### Artifact Format

```python
# All artifacts are PKM-AI blocks
create_block(
    block_type="permanent",  # or structure/outline
    title="sdd/{change}/{phase}",
    content="# {Phase Title}\n\n{content}",
    tags=["sdd", "sdd-{phase}", "sdd-{change}"]
)
```

### Link Relationships

```python
# Links establish artifact relationships
create_link(
    source_id="{parent-ulid}",
    target_id="{child-ulid}",
    link_type="refines"  # or contains, supports, etc.
)
```

### Search Patterns

```python
# Find all artifacts for a change
search_blocks(
    query="sdd-{change}",
    tags=["sdd"],
    limit=50
)

# Find specific phase artifact
search_blocks(
    query="sdd/{change}/{phase}",
    tags=["sdd-{phase}"],
    limit=5
)

# Find by block type
search_blocks(
    block_type="structure",
    tags=["sdd", "spec"],
    limit=20
)
```

### Retrieval Pattern

```python
# Step 1: Search for artifact
results = search_blocks(
    query="sdd/mcp-workflow/spec",
    tags=["sdd-spec"]
)

# Step 2: Get full content
if results.blocks:
    spec = get_block(
        block_id=results.blocks[0].id,
        include_content=True
    )
```

### Recovery Protocol

```python
def recover_change(change_name: str) -> dict:
    """Recover all artifacts for a change from PKM-AI."""

    # Find all change artifacts
    artifacts = search_blocks(
        query=f"sdd-{change_name}",
        tags=["sdd"],
        limit=100
    )

    # Find project context
    projects = search_blocks(
        query=f"sdd/{change_name}/project",
        tags=["sdd", "project"]
    )

    # Find tracker
    trackers = search_blocks(
        query=f"sdd/{change_name}/tracker",
        tags=["sdd", "tracker"]
    )

    return {
        "artifacts": artifacts.blocks,
        "project": projects.blocks[0] if projects.blocks else None,
        "tracker": trackers.blocks[0] if trackers.blocks else None
    }
```

### Advantages

- Single source of truth
- Graph relationships via links
- Tag-based faceted search
- Semantic block types
- PKM-AI Zettelkasten methodology

### Disadvantages

- Not directly in git
- Requires PKM-AI server
- Separate backup needed

---

## Mode: openspec

**Filesystem-first mode.** Artifacts stored in filesystem structure, not in PKM-AI.

### Characteristics

| Aspect | Behavior |
|--------|----------|
| Storage | Filesystem only |
| PKM-AI | Not used for SDD |
| Search | File-based search (ripgrep) |
| Recovery | File reading |
| Git tracking | Automatic |

### Directory Structure

```
openspec/
└── changes/
    └── {change-name}/
        ├── state.yaml           # Current state
        ├── explore.md           # Exploration
        ├── proposal.md         # Proposal
        ├── spec.md             # Specification
        ├── design.md           # Design
        ├── tasks.md            # Task list
        ├── progress.md         # Progress reports
        ├── verify.md           # Verification
        └── archive.md          # Archive
```

### State File Format

```yaml
# openspec/changes/{change}/state.yaml
change: mcp-workflow
project: hodei-pkm
mode: openspec
status: active

phases:
  explore:
    status: completed
    file: explore.md
    completed_at: "2026-03-20T10:00:00Z"
  proposal:
    status: completed
    file: proposal.md
    completed_at: "2026-03-20T11:00:00Z"
  spec:
    status: in_progress
    file: spec.md
  design:
    status: pending
  tasks:
    status: pending
  apply:
    status: pending
  verify:
    status: pending
  archive:
    status: pending

created_at: "2026-03-20T09:00:00Z"
updated_at: "2026-03-20T11:00:00Z"
```

### Artifact File Format

```markdown
# openspec/changes/{change}/{phase}.md

<!---
SDD Artifact: {phase}
Change: {change}
ULID: {ulid or "none"}
Created: {ISO 8601}
Mode: openspec
-->

# {Phase Title}

## Summary
{content}

## Metadata
```json
{
  "change": "{change}",
  "phase": "{phase}",
  "created": "{timestamp}",
  "ulid": "{ulid or generated}",
  "mode": "openspec"
}
```
```

### Search Patterns

```bash
# Find all artifacts for a change
find openspec/changes/{change} -type f -name "*.md"

# Search within artifacts
grep -r "sdd-spec" openspec/changes/

# Find specific artifact
cat openspec/changes/{change}/spec.md
```

### Retrieval Pattern

```python
import yaml
from pathlib import Path

def get_artifact(change: str, phase: str) -> dict:
    """Read artifact from filesystem."""

    # Read state file
    state_path = Path(f"openspec/changes/{change}/state.yaml")
    with open(state_path) as f:
        state = yaml.safe_load(f)

    # Get file path from state
    phase_info = state['phases'].get(phase)
    if not phase_info or phase_info['status'] == 'pending':
        return None

    # Read artifact file
    artifact_path = Path(f"openspec/changes/{change}/{phase_info['file']}")
    with open(artifact_path) as f:
        content = f.read()

    return {
        'change': change,
        'phase': phase,
        'content': content,
        'file': str(artifact_path),
        'completed_at': phase_info.get('completed_at')
    }
```

### Recovery Protocol

```python
def recover_change(change_name: str) -> dict:
    """Recover all artifacts for a change from filesystem."""

    base_path = Path(f"openspec/changes/{change_name}")

    # Read state file
    with open(base_path / "state.yaml") as f:
        state = yaml.safe_load(f)

    # Collect all completed phases
    artifacts = {}
    for phase, info in state['phases'].items():
        if info['status'] in ('completed', 'in_progress'):
            with open(base_path / info['file']) as f:
                artifacts[phase] = f.read()

    return {
        'state': state,
        'artifacts': artifacts
    }
```

### Advantages

- Git tracking automatic
- No PKM-AI dependency
- Human-readable files
- Standard diff tools

### Disadvantages

- No graph relationships
- No tag-based search
- Manual link tracking
- No semantic block types

---

## Mode: hybrid

**Dual storage mode.** Primary storage in PKM-AI with filesystem backup.

### Characteristics

| Aspect | Behavior |
|--------|----------|
| PKM-AI | Primary, for search/navigation |
| Filesystem | Secondary, for backup/persistence |
| Search | PKM-AI primary, ripgrep backup |
| Recovery | PKM-AI primary, filesystem fallback |
| Git tracking | Automatic (via filesystem) |

### Storage Strategy

```python
# When creating artifact:
def create_hybrid_artifact(change: str, phase: str, content: str):
    """Create artifact in both PKM-AI and filesystem."""

    # 1. Create in PKM-AI (primary)
    block = create_block(
        block_type="permanent",
        title=f"sdd/{change}/{phase}",
        content=content,
        tags=["sdd", f"sdd-{phase}", f"sdd-{change}"]
    )

    # 2. Write to filesystem (backup)
    file_path = f"openspec/changes/{change}/{phase}.md"
    with open(file_path, 'w') as f:
        f.write(content)

    # 3. Update state file
    update_state(change, phase, block.id, file_path)

    return block
```

### Directory Structure

```
openspec/
└── changes/
    └── {change-name}/
        ├── state.yaml           # Current state with ULIDs
        ├── explore.md           # Exploration (backup)
        ├── proposal.md          # Proposal (backup)
        ├── spec.md              # Specification (backup)
        ├── design.md           # Design (backup)
        ├── tasks.md            # Task list (backup)
        ├── progress.md         # Progress reports (backup)
        ├── verify.md           # Verification (backup)
        └── archive.md          # Archive (backup)
```

### State File Format (Hybrid)

```yaml
# openspec/changes/{change}/state.yaml
change: mcp-workflow
project: hodei-pkm
mode: hybrid
status: active

phases:
  explore:
    status: completed
    file: explore.md
    ulid: 01ARZ3NDEKTSV4RRFFQ69G5FAV
    completed_at: "2026-03-20T10:00:00Z"
  proposal:
    status: completed
    file: proposal.md
    ulid: 01ARZ3NDEKTSV4RRFFQ69G5FAW
    completed_at: "2026-03-20T11:00:00Z"
  spec:
    status: in_progress
    file: spec.md
    ulid: 01ARZ3NDEKTSV4RRFFQ69G5FAZ
```

### PKM-AI Search (Primary)

```python
# Primary search via PKM-AI
def search_primary(change: str, phase: str = None):
    tags = ["sdd"]
    if phase:
        tags.append(f"sdd-{phase}")

    results = search_blocks(
        query=f"sdd-{change}",
        tags=tags,
        limit=50
    )
    return results
```

### Filesystem Search (Backup)

```bash
# Search via ripgrep
grep -r "sdd/{change}" openspec/changes/

# Or use find + cat
find openspec/changes/{change} -type f -name "*.md" | xargs grep "sdd"
```

### Retrieval Pattern

```python
def get_artifact(change: str, phase: str, prefer_pkmai: bool = True) -> dict:
    """Retrieve artifact, preferring PKM-AI."""

    if prefer_pkmai:
        # Try PKM-AI first
        results = search_blocks(
            query=f"sdd/{change}/{phase}",
            tags=["sdd", f"sdd-{phase}"]
        )
        if results.blocks:
            return get_block(
                block_id=results.blocks[0].id,
                include_content=True
            )

        # Fallback to filesystem
        return get_from_filesystem(change, phase)

    else:
        # Try filesystem first
        return get_from_filesystem(change, phase)
```

### Recovery Protocol (Hybrid)

```python
def recover_change(change: str) -> dict:
    """Recover change from PKM-AI with filesystem fallback."""

    # Try PKM-AI first
    pkmai_artifacts = search_blocks(
        query=f"sdd-{change}",
        tags=["sdd"],
        limit=100
    )

    if len(pkmai_artifacts.blocks) >= expected_count:
        # PKM-AI complete, use it
        return {
            'source': 'pkmai',
            'artifacts': pkmai_artifacts.blocks
        }

    # Fallback to filesystem
    fs_state = read_filesystem_state(change)
    return {
        'source': 'filesystem',
        'state': fs_state,
        'artifacts': read_filesystem_artifacts(change)
    }
```

### Advantages

- Best of both worlds
- PKM-AI search + git tracking
- Recovery from either source
- Redundancy

### Disadvantages

- Dual maintenance
- More complex
- Potential sync issues

---

## Mode: none

**Inline mode.** Artifacts returned inline, not persisted.

### Characteristics

| Aspect | Behavior |
|--------|----------|
| Storage | None |
| PKM-AI | Not used |
| Filesystem | Not used |
| Search | Not applicable |
| Recovery | Not possible |

### Use Cases

| Use Case | Why none |
|----------|----------|
| Quick exploration | Don't need persistence |
| Testing SDD workflow | No side effects |
| Demonstrations | Clean slate each time |
| Temporary analysis | Won't need to recover |

### Artifact Handling

```python
# In 'none' mode, artifacts are returned inline only

# Phase execution returns artifact in response:
{
    "status": "success",
    "artifact": {
        "title": "sdd/test/explore",
        "content": "# Exploration\n\n{content}",
        "mode": "none"
    }
}

# No create_block call is made
```

### Example Phase Response

```markdown
## sdd-explore Complete

**Status**: success
**Mode**: none

### Artifact (Inline)

```markdown
# Exploration: test-change

## Research Questions
- What is the scope?

## Findings
- Initial findings here

## Next Steps
Proceed to propose phase
```

### When Not to Use

- Production work
- Multi-session changes
- Anything you'll need later
- Complex features

---

## Recovery with PKM-AI Search

### Recovery Flow

```
Session Starts
     │
     ▼
┌─────────────────┐
│ Load PKM-AI    │
│ Context         │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Search for      │
│ Change Project  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Find All        │
│ Change Artifacts│
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Read Tracker    │
│ Determine Next  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Resume from     │
│ Next Phase      │
└─────────────────┘
```

### Recovery Code

```python
def recover_and_resume(change_name: str):
    """Recover change state and determine next action."""

    # 1. Find project context
    projects = search_blocks(
        query=f"sdd/{change_name}/project",
        tags=["sdd", "project"]
    )

    if not projects.blocks:
        print(f"No project found for '{change_name}'")
        return None

    project = get_block(projects.blocks[0].id, include_content=True)

    # 2. Find tracker
    trackers = search_blocks(
        query=f"sdd/{change_name}/tracker",
        tags=["sdd", "tracker"]
    )

    if trackers.blocks:
        tracker = get_block(trackers.blocks[0].id, include_content=True)
        # Parse tracker content to find current phase
        current_phase = parse_tracker_current_phase(tracker.content)
    else:
        current_phase = None

    # 3. Find all artifacts
    all_artifacts = search_blocks(
        query=f"sdd-{change_name}",
        tags=["sdd"],
        limit=100
    )

    # 4. Determine next phase
    next_phase = determine_next_phase(current_phase, all_artifacts.blocks)

    return {
        "project": project,
        "tracker": tracker if trackers.blocks else None,
        "artifacts": all_artifacts.blocks,
        "current_phase": current_phase,
        "next_phase": next_phase
    }
```

### Gap Detection

```python
def detect_artifacts_gaps(change_name: str) -> dict:
    """Find missing or incomplete artifacts."""

    # Expected artifacts
    expected_phases = [
        'explore', 'proposal', 'spec', 'design',
        'tasks', 'apply', 'verify', 'archive'
    ]

    # Find existing
    existing = search_blocks(
        query=f"sdd-{change_name}",
        tags=["sdd"],
        limit=100
    )

    existing_phases = set()
    for block in existing.blocks:
        # Parse phase from title
        title = block.title  # e.g., "sdd/mcp-workflow/spec"
        if '/' in title:
            phase = title.split('/')[-1]
            existing_phases.add(phase)

    # Find gaps
    missing = [p for p in expected_phases if p not in existing_phases]

    return {
        "existing": list(existing_phases),
        "missing": missing,
        "complete": len(missing) == 0
    }
```

## Summary Table

| Feature | pkmai | openspec | hybrid | none |
|---------|-------|---------|--------|------|
| PKM-AI storage | Yes | No | Yes | No |
| Filesystem storage | No | Yes | Yes | No |
| Graph links | Yes | No | Yes | No |
| Tag search | Yes | No | Yes | No |
| Git tracking | No | Yes | Yes | No |
| Recovery | PKM-AI | Filesystem | Either | None |
| Complexity | Low | Low | Medium | None |

## Next Steps

- See [concepts.md](concepts.md) for artifact lifecycle
- See [sub-agents.md](sub-agents.md) for phase execution
- See [token-economics.md](token-economics.md) for efficiency analysis