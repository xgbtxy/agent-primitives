# `baton` — Compaction-Proof State & Cross-Agent Handoff Capsule

[English](./README.md) | [简体中文](./README.zh-CN.md)

**Standalone Mini-Project (`skills/baton/`)** · Zero third-party dependencies · `70 tokens` idle skill cost.

## Why You Need It

Two failures plague long agent work:
1. **Compaction Amnesia**: Right in the middle of a complex debug or research workflow, automatic context compaction triggers—and the agent wakes up having forgotten *which hypotheses already failed*, immediately repeating broken edits.
2. **Cross-Agent & Worktree Silos**: Switching from Claude Code to Codex CLI, Grok Build, Antigravity, or OpenClaw (or switching git worktrees) loses active task state, while heavy memory plugins require Docker/Bun or paid cloud observers.

`baton` maintains a deterministic, zero-LLM, `< 400-token` capsule in `.baton/` separating four orthogonal slots: **Goal & Constraints**, **Verified Facts (with file/URL evidence)**, **Falsified Dead-Ends (`DO NOT RETRY`)**, and **Next Immediate Step** + automatic Git / workspace file snapshot.

## Copy Just This Folder

```bash
cp -r skills/baton ~/.agents/skills/
# or: npx skills add xgbtxy/agent-primitives --skill baton
```

## Real Commands

```bash
# Set objective and next concrete step
python scripts/baton.py set --goal "Eliminate 429 retry storm in webhook sender" --next "Patch jitter in sender.py:118"

# Record verified ground truth & falsified dead-ends
python scripts/baton.py fact " Upstream resets Retry-After in seconds, not ms" --ref "sender.py:94"
python scripts/baton.py deadend "Doubling worker threads" --reason "Triggered stricter IP rate-limit"

# Auto-snapshot (ideal for PreCompact / SessionEnd hooks)
python scripts/baton.py checkpoint --note "Pre-compaction snapshot"

# Resume in <400 tokens in any session or any other agent
python scripts/baton.py resume
```
