# `probe` (`crates/ap-probe`) — Tree-sitter Reachability Bundler & Causal Runtime Microscope

[English](./README.md) | [简体中文](./README.zh-CN.md)

**Standalone Rust Crate (`crates/ap-probe`) & Skill (`skills/probe/`)** · 6-language Tree-sitter + Py/TS/JS/Rust probes · `59 tokens` idle skill cost.

## Why You Need It

Eliminates two huge sources of agent waste (`4,604` ad-hoc inline probes, `185` `ImportError`s, `131` shell-quoting crashes, and 20k+ prose reasoning loops):
1. **`ap probe slice . --bundle`**: Uses Tree-sitter + import/`#include` graph traversal across **Python, TypeScript/JS, Go, Rust, and C** (`7.1 ms` on 24 TS files!) to prune dead distractor modules and emit root manifest + active numbered source in 1 call.
2. **`ap probe run` & `ap probe diff`**: Executes Python (`sys.settrace`), TypeScript/JS (Node 22 V8 `Proxy`/`Reflect`), or **Rust (`rustc`)** snippets/oracles to catch **`[ALIAS HAZARD]`**, **`[IN-PLACE MUTATION]`**, **`[STATE NO-OP]`**, **`-0`/`NaN`**, cache collisions, and negative integer rounding drift in `<0.2s`.

## Real Commands

```bash
ap probe slice . --bundle
ap probe run --cwd . --code "import { Ledger } from './src/ledger.js'; const l = new Ledger(); l.snapshot();"
ap probe diff --lang rust --fn-a "|x: i64| (x * 250 + 5000) / 10000" --fn-b "|x: i64| ((x as f64 / 100.0) * 2.5).round() as i64"
```
