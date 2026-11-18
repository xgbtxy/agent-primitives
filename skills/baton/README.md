# `baton` (`crates/ap-baton`) — Compaction-Proof State & Cross-Agent Handoff Capsule

[English](./README.md) | [简体中文](./README.zh-CN.md)

**Standalone Rust Crate (`crates/ap-baton`) & Skill (`skills/baton/`)** · Zero sibling dependencies · `60 tokens` idle skill cost.

## Why You Need It

Two failures plague long agent work:
1. **Compaction Amnesia**: Mid-task context compaction triggers—and the agent forgets *which hypotheses already failed*, immediately repeating broken edits.
2. **Cross-Agent & Worktree Silos**: Switching between Claude Code, Codex CLI, Grok Build, Antigravity, Pi, or OpenClaw loses active state.

`ap baton` maintains a deterministic, zero-LLM, `< 400-token` capsule in `.baton/` separating **Goal & Constraints**, **Verified Facts (`--ref`)**, **Falsified Dead-Ends (`DO NOT RETRY`)**, and **Next Immediate Step** + automatic Git / workspace snapshot.

## Install

```bash
# Single-binary full install (macOS / Linux / Windows):
curl -fsSL https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/install.sh | sh
# Or install ONLY baton into current repo:
ap init --here --only baton
```

## Real Commands

```bash
ap baton set --goal "Eliminate 429 retry storm in webhook sender" --next "Patch jitter in sender.rs:118"
ap baton fact "Upstream resets Retry-After in seconds, not ms" --ref "sender.rs:94"
ap baton deadend "Doubling worker threads" --reason "Triggered stricter IP rate-limit"
ap baton checkpoint --note "Pre-compaction snapshot"
ap baton resume
```
