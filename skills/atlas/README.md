# `atlas` (`crates/ap-atlas`) — Sub-50ms Tree-sitter Structural & Product Map

[English](./README.md) | [简体中文](./README.zh-CN.md)

**Standalone Rust Crate (`crates/ap-atlas`) & Skill (`skills/atlas/`)** · 6 Tree-sitter grammars + `ignore` walker · `<65 tokens` idle skill cost.

## Why You Need It

1. **Codebase & Vault Map (`atlas map`)**: Scans a 64-module corpus in **`16.3 ms`** (**-99.0% orientation tokens**), ranking modules by inbound Tree-sitter import centrality across `Python`, `TypeScript/TSX`, `JavaScript`, `Go`, `Rust`, and `C`, flagging `[dirty]` git hotspots and `>90d` stale TODO traps.
2. **Product Feature & Anti-Slop Contract Map (`atlas product`)**: Directly extracts HTML pages & interactive UI controls (`#id <button|input|select>`), registered backend HTTP routes, **`[BROKEN]` frontend↔backend API mismatches**, and **`[SLOP RADAR]`** files (`: any`, `@ts-ignore`, silent empty `catch {}`) so the agent never guesses routes or copies legacy slop.

## Real Commands

```bash
ap atlas map .
ap atlas product . --write PRODUCT_MAP.md
ap atlas map . --focus "exporter" --json
```
