# `guard` (`crates/ap-guard`) — Sub-2ms Exit-2 Anti-Slop Rules, Verification Gate & Blast Shield

[English](./README.md) | [简体中文](./README.zh-CN.md)

**Standalone Rust Crate (`crates/ap-guard`) & Skill (`skills/guard/`)** · `<2 ms` native binary · `<65 tokens` idle skill cost.

## Why You Need It

Agents frequently ignore soft prose reminders in `AGENTS.md` or `CLAUDE.md`. `ap guard` turns quality rules and verification into **unbypassable `exit 2` physical gates**:
1. **Persistent Anti-Slop & Custom Rules (`guard rule` + `guard verify`)**: Blocks `: any` / `@ts-ignore`, silent empty `catch {}` error swallowing, and custom forbidden patterns in `.agents/guard.json` (`guard rule add` permanently locks out any past agent mistake), then runs auto-detected tests + `.agents/smoke.ap`.
2. **Dual-Protocol Stop-Gate & PreToolUse Shield (`stop-gate` / `shield` / `run`)**: Enforces Claude/Codex `exit 2` and Antigravity camelCase JSON `deny`/`continue` hooks, blocks exposed secrets (`sk-`, `ghp_`, `xai-`, `AKIA`) & `rm -rf /`, and breaks `3x` identical retry loops.

## Real Commands

```bash
ap guard verify
ap guard rule init
ap guard rule add --id no-slop-csv --glob "src/*.ts" --forbid "old_csv_dump" --msg "Use RFC4180 csv module"
ap guard run --max-repeats 3 -- npm test
ap guard stop-gate
```
