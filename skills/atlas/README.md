# `atlas` (`crates/ap-atlas`) — Sub-50ms Tree-sitter Structural Map

[English](./README.md) | [简体中文](./README.zh-CN.md)

**Standalone Rust Crate (`crates/ap-atlas`) & Skill (`skills/atlas/`)** · 6 Tree-sitter grammars + `ignore` walker · `58 tokens` idle skill cost.

## Why You Need It

Cold-starting on an unfamiliar repo or Markdown vault often burns 50k–600k tokens running scattered `ls`/`find`/`grep` calls (`8,952` broad search calls measured in local audit). `ap atlas` embeds official **Tree-sitter C grammars (`Python`, `TypeScript/TSX`, `JavaScript`, `Go`, `Rust`, `C`)** and `ripgrep`'s `.gitignore`-aware walker to scan a 64-module corpus in **`16.3 ms`** (**-99.0% orientation tokens**), ranking modules by inbound import centrality + `[dirty]` git hotspots while flagging `>90d` stale TODO traps.

## Real Commands

```bash
ap atlas map .
ap atlas map . --focus "exporter" --max-files 20
ap atlas map . --json
```
