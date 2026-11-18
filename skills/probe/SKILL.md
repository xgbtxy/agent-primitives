---
name: probe
description: Prune dead distractor modules and bundle active reachable code across Py/TS/JS/Go/Rust/C (`slice --bundle`), trace Py/TS/JS aliasing/mutation/state-noop/`-0` bugs (`run`), and run differential oracles (`diff`).
---

# Probe — Tree-sitter Reachability Bundler & Causal Runtime Microscope

Never burn 15+ tool calls reading dead/unreferenced modules or 20,000+ reasoning tokens mentally simulating heap aliasing, partial exception mutations, void-method no-ops, `-0`/`NaN`, or integer/type edge cases.

## Fast 3-Step Debugging Workflow

```bash
# 1. Prune dead distractor files AND emit manifest + numbered source of active reachable modules in 1 call (Py, TS/JS, Go, Rust, C)
ap probe slice . --bundle

# 2a. Trace a Python, TypeScript/JS, or Rust reproducer (detects aliasing, unrolled exception mutations, state no-ops & -0/NaN):
ap probe run --cwd . --code "import { Ledger } from './src/ledger.js'; const l = new Ledger(); l.snapshot();"
ap probe run --cwd . --focus <pkg> --code "<single-line python repro>"

# 2b. Multi-line reproducer (PowerShell `@' ... '@ |` — NEVER use `<<` in PowerShell; POSIX bash `<<'EOF'`):
@'
from pkg import Engine
Engine().run(...)
'@ | ap probe run --cwd . --stdin

# 3. Differential & boundary/cache oracle across Python, TypeScript/JS, OR Rust
ap probe diff --cwd . --setup "import { f, ref } from './src/m.js';" --fn-a "f(x)" --fn-b "ref(x)"
ap probe diff --lang rust --fn-a "|x: i64| (x * 250 + 5000) / 10000" --fn-b "|x: i64| ((x as f64 / 100.0) * 2.5).round() as i64"
```
