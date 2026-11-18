---
name: baton
description: Save and resume compaction-proof task state, verified facts, falsified dead-ends, and next steps across sessions, git worktrees, and heterogeneous agents (Claude Code, Codex, Grok, Antigravity, Pi) in <500 tokens.
---

# Baton — Compaction-Proof State & Cross-Agent Handoff Capsule

Eliminate post-compaction amnesia and cross-agent state loss without cloud accounts, Docker, or token-burning LLM observers. Stores a deterministic `<500`-token state capsule in `.baton/`.

## Usage

```bash
# 1. Start or update active task state
ap baton set --goal "Fix OTLP retry backoff jitter" --next "Run unit test on line 142"

# 2. Record verified ground-truth facts & falsified dead-ends (prevents repeat loops!)
ap baton fact "Timeout originates in exporter/http.rs:88" --ref "exporter/http.rs:88"
ap baton deadend "Increasing pool_maxsize had zero effect on 429 retries"

# 3. Auto-snapshot workspace state (wire into PreCompact / Stop hooks)
ap baton checkpoint --note "Before context compaction"

# 4. Resume cleanly in <500 tokens in any new session or different agent
ap baton resume
```
