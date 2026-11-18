# `probe` (`crates/ap-probe`) — Reachability Bundler, Live Server/DOM Smoke Runner & Runtime Microscope

[English](./README.md) | [简体中文](./README.zh-CN.md)

**Standalone Rust Crate (`crates/ap-probe`) & Skill (`skills/probe/`)** · 6 Tree-sitter grammars + pure-Rust HTTP/Browser Smoke + Py/Node-V8/`rustc` probes · `<65 tokens` idle skill cost.

## Why You Need It

1. **Prune Dead Slop Files in 1 Call (`probe slice --bundle`)**: Traces real Tree-sitter imports across Python, TS/JS, Go, Rust & C in **`6.3 ms`**, pruning unreferenced legacy modules so the agent never reads or imitates dead slop.
2. **Eliminate the 70% Verification Token Tax on Windows (`probe smoke`)**: Starts your server on a free ephemeral `PORT` with detached file-backed logs (zero Windows pipe hangs), renders real live HTML+JS DOM via headless **Microsoft Edge / Chrome** (`BROWSER /`), executes UTF-8 Chinese/JSON/CSV HTTP assertions, inspects server `stderr`, and kills the entire Windows process tree (`taskkill /T /F`) in **1 single `<3s` turn**.
3. **Runtime & Differential Microscope (`probe run` / `probe diff`)**: Detects `[ALIAS HAZARD]`, `[IN-PLACE MUTATION]`, `[STATE NO-OP]`, and `-0.0` drift across Python, Node 22 V8 `Proxy`, and native `rustc`.

## Real Commands

```bash
ap probe slice . --bundle
ap probe smoke --cwd . --stdin
ap probe run --cwd . --code "import { Ledger } from './src/ledger.js'; new Ledger().snapshot();"
ap probe diff --lang rust --fn-a "|x: i64| (x * 250 + 5000) / 10000" --fn-b "|x: i64| ((x as f64 / 100.0) * 2.5).round() as i64"
```
