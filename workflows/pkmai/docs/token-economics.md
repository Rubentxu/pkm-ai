# Token Overhead Analysis for PKM-AI

## Overview

The PKM-AI workflow uses sub-agent delegation to manage context window efficiently. This document analyzes the token overhead of sub-agent launches versus inline execution, and identifies the crossover point where delegation becomes more efficient.

## Token Cost Model

### Context Window as Finite Resource

```
┌────────────────────────────────────────────────────────────┐
│                    CONTEXT WINDOW                          │
│                                                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │   System    │  │  Working     │  │   Output     │    │
│  │   Prompt    │  │   Area       │  │   Buffer     │    │
│  │  (fixed)    │  │  (variable)  │  │  (reserved)  │    │
│  └──────────────┘  └──────────────┘  └──────────────┘    │
│                                                            │
│  Total: ~200K tokens (varies by model)                    │
└────────────────────────────────────────────────────────────┘
```

### Token Categories

| Category | Description | When Paid |
|----------|-------------|-----------|
| **Fixed Overhead** | System prompt, skills | Once per session |
| **Per-Message** | Each turn in conversation | Per message |
| **Per-Artifact** | Artifacts stored/retrieved | On save/load |
| **Delegation** | Skill loading, context switch | Per sub-agent |

## Delegation Overhead Analysis

### Inline Execution Model

```
┌─────────────────────────────────────────────────────────────┐
│                    ORCHESTRATOR (INLINE)                    │
│                                                              │
│  1. Read source files ──────► Context grows                │
│  2. Analyze code ──────────► Context grows                 │
│  3. Write code ────────────► Context grows                 │
│  4. Repeat for each task ─► Context accumulates           │
│                                                              │
│  Problem: All work adds to single context                  │
└─────────────────────────────────────────────────────────────┘
```

**Token Cost per Inline Task**:
```
Task_N_cost = Task_1_cost + Task_2_cost + ... + Task_N_cost
             = O(N^2) growth (cumulative)
```

### Delegation Execution Model

```
┌─────────────────────────────────────────────────────────────┐
│                    ORCHESTRATOR + SUB-AGENTS                 │
│                                                              │
│  Orchestrator:                                              │
│    1. Launch sub-agent ────► Context = overhead             │
│    2. Await result ────────► Context = overhead             │
│    3. Synthesize ─────────► Context += result               │
│                                                              │
│  Sub-agent (separate context):                              │
│    1. Load skill ─────────► Own context                     │
│    2. Do work ────────────► Own context                     │
│    3. Return result ───────► Own context discarded          │
│                                                              │
│  Benefit: Work happens in isolated contexts                 │
└─────────────────────────────────────────────────────────────┘
```

**Token Cost per Delegated Task**:
```
Delegated_Task_cost = overhead + result_size
                    = O(1) per task
```

## Sub-Agent Launch Overhead

### Overhead Components

| Component | Token Estimate | Description |
|-----------|----------------|-------------|
| Skill file content | ~500-2000 | Load SKILL.md |
| Shared conventions | ~300-800 | Load pkmai-convention.md |
| Phase-common.md | ~200-400 | Load return format |
| Context packaging | ~100-200 | Change name, mode, args |
| **Total launch overhead** | ~1100-3400 | Per sub-agent launch |

### Skill Loading Example

```markdown
<!-- Skill overhead ~1500 tokens -->
# SDD Exploration Phase Skill
## Purpose: Research and discover...

## PKM-AI Tool Mapping
[~300 tokens of tool documentation]

## Inputs
[~100 tokens]

## Execution
[~800 tokens of instructions]

## Return Envelope
[~300 tokens]
```

### Context Switch Cost

When orchestrator delegates to sub-agent:

```
┌────────────────────┐       ┌────────────────────┐
│    ORCHESTRATOR   │       │    SUB-AGENT        │
│                    │       │                    │
│  Context:          │       │  Context:          │
│  - System (~10K)   │       │  - System (~10K)   │
│  - History (~5K)   │       │  - Skill (~1.5K)   │
│  - State (~2K)     │       │  - Inputs (~1K)    │
│  - Overhead: ~17K  │       │  - Overhead: ~12.5K │
│                    │       │                    │
│  Result received:  │◄──────│  Returns:          │
│  ~500 token result │       │  ~500 token result │
└────────────────────┘       └────────────────────┘

Orchestrator context after: ~17K + 0.5K (result)
Sub-agent context discarded after return
```

## Crossover Point Analysis

### When Delegation Wins

```
┌─────────────────────────────────────────────────────────────────┐
│                     TOKEN GROWTH CURVES                         │
│                                                                  │
│  Inline:     ┌──────────────────────────────────               │
│               │                          ╱╱╱                       │
│               │                       ╱╱                          │
│               │                    ╱╱                             │
│               │                 ╱╱                                │
│               │              ╱╱                                   │
│  Tokens:      │           ╱╱                                     │
│               │        ╱╱                                        │
│               │     ╱╱                                            │
│               │  ╱╱                                               │
│               └─────────────────────────────────────────────────►
│                      Task Count →
│                                                                  │
│  Delegated:   ┌─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─                 │
│               │        (constant ~12.5K per task + overhead)     │
│  Tokens:      │  ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─              │
│               └─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ►
│                      Task Count →
│                                                                  │
│  Cross-over: ~3-5 substantial tasks                             │
└─────────────────────────────────────────────────────────────────┘
```

### Crossover Calculation

Given:
- Inline overhead per task: ~2K tokens
- Delegation overhead per task: ~12.5K tokens (skill load + result)
- Delegation fixed cost: ~3.4K (one-time skill loading)

**Inline execution** for N tasks:
```
Inline_total(N) = 2K * N + 2K * (N-1) + 2K * (N-2) + ... + 2K
               = 2K * N * (N + 1) / 2
               = O(N^2)
```

**Delegated execution** for N tasks:
```
Delegated_total(N) = 3.4K (skill loading, amortized) + 12.5K * N
                    = O(N)
```

**Crossover point** where Delegated_total(N) < Inline_total(N):
```
3.4K + 12.5K * N < 2K * N * (N + 1) / 2
3.4K + 12.5K * N < K * N^2

For N > ~3-5 tasks, delegation wins
```

### Real-World Crossover

| Task Type | Inline Cost | Delegation Cost | Crossover |
|-----------|--------------|------------------|-----------|
| Quick question | 100 tokens | 3.5K tokens | Never (inline wins) |
| Small analysis | 500 tokens | 3.5K tokens | Never (inline wins) |
| Medium task | 2K tokens | 3.5K tokens | ~2 tasks |
| Large task | 5K tokens | 3.5K tokens | ~1 task |
| Substantial feature | 10K+ tokens | 3.5K tokens | Immediate |

### Rule of Thumb

| Situation | Recommendation |
|-----------|----------------|
| Simple question | Inline (answer directly) |
| Small task (<500 tokens) | Inline or delegate |
| Medium task (500-2K tokens) | Consider delegation if >2 tasks |
| Large task (>2K tokens) | Delegate immediately |
| Multi-step feature | Delegate (SDD pipeline) |

## Efficiency by SDD Phase

### Per-Phase Token Analysis

| Phase | Inline Estimate | Delegation Overhead | Recommendation |
|-------|----------------|---------------------|-----------------|
| sdd-init | ~3K | ~3.5K | Inline (once per change) |
| sdd-explore | ~5K | ~3.5K | Inline if quick, delegate if research |
| sdd-propose | ~4K | ~3.5K | Inline for simple, delegate for complex |
| sdd-spec | ~8K | ~3.5K | Delegate (detailed work) |
| sdd-design | ~10K | ~3.5K | Delegate (architectural thinking) |
| sdd-tasks | ~6K | ~3.5K | Inline (mechanical breakdown) |
| sdd-apply | ~20K+ | ~3.5K | Delegate (implementation work) |
| sdd-verify | ~5K | ~3.5K | Inline (straight verification) |
| sdd-archive | ~3K | ~3.5K | Inline (summarization) |

### Phase Complexity Matrix

```
                    Low Complexity          High Complexity
                   ┌────────────────────┬────────────────────┐
   Short Duration  │      Inline        │     Inline         │
                   │   (sdd-archive)    │   (sdd-propose)   │
                   ├────────────────────┼────────────────────┤
   Long Duration   │     Delegate       │     Delegate       │
                   │   (sdd-spec)      │   (sdd-design)    │
                   └────────────────────┴────────────────────┘
```

## Why Delegation is More Efficient

### 1. Context Isolation

```
Without delegation:
  Context = System + History + All_work + All_results
           = 10K + 5K + 20K + 10K = 45K (close to limit)

With delegation:
  Orchestrator: 10K + 5K + 2K + 1K = 18K
  Sub-agents:   12.5K each (discarded after)

  Orchestrator never exceeds threshold
```

### 2. No Repeated Context Scanning

Inline execution accumulates context. Each new read adds to the pile.

Delegation keeps sub-agent work isolated:
- Sub-agent reads are in fresh context
- Results are compressed summaries
- Orchestrator doesn't re-read previous work

### 3. Parallel Phase Execution

Independent phases can run concurrently:

```json
{
  "tool": "parallel_execution",
  "arguments": {"phases": "$phase_times"}
}
```

This reduces wall-clock time even if token overhead is similar.

### 4. Skill Reuse

Skills are loaded once and cached:

```
First sub-agent launch: 3.4K tokens (skill loading)
Subsequent launches:    ~500 tokens (context packaging only)
```

Amortized over many tasks, skill loading is negligible.

## Token Budget Optimization

### Strategies

| Strategy | Token Savings | When to Use |
|----------|---------------|-------------|
| Delegate substantial work | 50-70% | Features, complex analysis |
| Inline quick answers | 100% savings | Simple questions |
| Compress results | 20-40% | When delegation overhead high |
| Batch small tasks | 30-50% | Multiple related small tasks |
| Use summary references | 60-80% | When full context not needed |

### Result Compression

Sub-agents should return concise results:

```markdown
## Phase Complete

**Status**: success
**Artifact ULID**: 01ARZ3NDEKTSV4RRFFQ69G5FAV

### Executive Summary
[2-3 sentences, ~100 tokens]

### Key Decisions
- Decision 1
- Decision 2

### Next Recommended
sdd-spec

### Risks
None identified
```

Not:
```markdown
## Full Artifact Content (500+ tokens)
[Entire artifact dumped]
```

## PKM-AI Overhead

### PKM-AI Operations

```python
# PKM-AI overhead per artifact
create_block(
    block_type="permanent",
    title="sdd/{change}/{phase}",
    content="...",
    tags=["sdd", "sdd-{phase}"]
)
# ~400 tokens per create
```

### Operation Costs

| Operation | PKM-AI |
|-----------|--------|
| Save artifact | 400 |
| Search artifacts | 300 |
| Get full artifact | 300 |
| Create link | 200 |
| **Total per phase** | ~1200 |

**PKM-AI costs more per operation** but provides:
- Graph relationships (links)
- Tag-based faceted search
- Block type semantics
- Zettelkasten methodology

### Break-Even Analysis

PKM-AI overhead is justified when:
- Multiple searches needed (>3)
- Graph relationships matter
- Long-lived project with many artifacts
- Team collaboration

Not justified when:
- Single-session experiments
- Minimal artifact retrieval
- Simple, linear workflow

## Practical Guidelines

### When to Delegate

| Condition | Action |
|-----------|--------|
| Task requires reading multiple files | Delegate |
| Task will produce substantial output | Delegate |
| Task is a SDD phase | Always delegate |
| Task is independent of current work | Delegate |
| User is waiting and you can continue | Delegate (async) |

### When to Inline

| Condition | Action |
|-----------|--------|
| Simple factual question | Answer directly |
| Quick navigation help | Answer directly |
| Reading single file for context | Answer directly |
| SDD-init (once per change) | Inline |

### Delegation Efficiency Checklist

Before delegating, verify:
- [ ] Skill path is correct
- [ ] Required artifacts are accessible
- [ ] Mode is set correctly
- [ ] Sub-agent can complete without orchestrator input
- [ ] Result will fit in output buffer

## Summary

| Aspect | Inline | Delegated |
|--------|--------|-----------|
| Token growth | O(N^2) cumulative | O(N) per task |
| Context isolation | No | Yes |
| Parallel execution | No | Yes |
| Skill reuse | N/A | Yes (cached) |
| Crossover point | N/A | ~3-5 tasks |
| Best for | Simple, quick | Complex, substantial |

### Key Takeaways

1. **Delegation has fixed overhead** (~3.5K tokens per sub-agent)
2. **Inline has cumulative overhead** (grows with each task)
3. **Crossover at ~3-5 tasks** where delegation wins
4. **SDD phases always delegate** (substantial work by definition)
5. **PKM-AI overhead is higher** than previous systems but adds value via links/search

## Next Steps

- See [architecture.md](architecture.md) for delegation patterns
- See [concepts.md](concepts.md) for phase details
- See [installation.md](installation.md) for setup