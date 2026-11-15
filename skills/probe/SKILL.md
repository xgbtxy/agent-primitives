---
name: probe
description: Prune dead distractor modules and bundle active reachable code across Py/TS/JS/Go/Rust (`slice --bundle`), trace Python heap-aliasing/mutation/state-noop bugs (`run`), and run differential boundary/cache oracles (`diff`).
---

# Probe — Polyglot Reachability Bundler & Causal Runtime Microscope

Never burn 15+ tool calls reading dead/unreferenced modules or 20,000+ reasoning tokens mentally simulating heap aliasing, in-place mutations, void-method no-ops, or `bool`/`int`/`None` edge cases. Run `probe` first (via `python .agents/ap.py probe ...` or `python scripts/probe.py ...`).

## Fast 3-Step Debugging Workflow

```bash
# 1. Prune dead distractor files AND emit numbered source of active reachable modules in 1 call (Py, TS/JS, Go, Rust)
python .agents/ap.py probe slice . --bundle

# 2. Trace a minimal Python reproducer (call-site deduplicated, <3.5 KB bounded output):
python .agents/ap.py probe run --cwd . --focus <pkg> --code "<single-line python repro>"

# Multi-line reproducer (PowerShell — NEVER use `<< 'EOF'` in PowerShell!):
@'
from pkg import Engine
Engine().run(...)
'@ | python .agents/ap.py probe run --cwd . --focus <pkg> --stdin

# Multi-line reproducer (POSIX bash/zsh):
python .agents/ap.py probe run --cwd . --focus <pkg> --stdin <<'EOF'
from pkg import Engine
Engine().run(...)
EOF

# 3. Differential & boundary/cache oracle (checks None, True vs 1, False vs 0, sequential cache drift & input purity)
python .agents/ap.py probe diff --cwd . --setup "<imports>" --fn-a "lambda d: <fast>" --fn-b "lambda d: <ref>"
```
