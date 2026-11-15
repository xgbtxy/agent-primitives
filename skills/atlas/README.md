# `atlas` — Sub-Second Zero-Read Structural Atlas

[English](./README.md) | [简体中文](./README.zh-CN.md)

**Standalone Mini-Project (`skills/atlas/`)** · Zero third-party dependencies · `73 tokens` idle skill cost.

## Why You Need It

Cold-start sessions routinely waste 10+ tool calls and `50k–130k tokens` reading random files just to figure out where entrypoints and core classes live—and empirical studies show coding agents burn **+77% extra tokens** when distracted by stale old `TODO`/`FIXME` comments nobody asked about.

`atlas` scans a **codebase** (Python AST classes/methods, JS/TS/Go/Rust/C symbols, internal import centrality "hub modules", entrypoints, and `> 90-day` stale-TODO trap warnings) or a **personal knowledge vault** (folder taxonomy + Markdown `H1/H2` outlines + active notes) in **`< 0.1s`**, emitting a `< 800-token` high-density navigation map (-99.4% vs full read).

## Copy Just This Folder

```bash
cp -r skills/atlas ~/.agents/skills/
# or: npx skills add xgbtxy/agent-primitives --skill atlas
```

## Real Commands

```bash
# Map current codebase or personal Markdown vault in <800 tokens
python scripts/atlas.py map .

# Narrow focus to a specific subsystem or folder
python scripts/atlas.py map . --focus "exporter" --max-files 15

# Machine-readable JSON output
python scripts/atlas.py map . --json
```
