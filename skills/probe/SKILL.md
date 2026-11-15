---
name: probe
description: Replace slow multi-file hunting and error-prone mental execution with sub-second static reachability slicing (`slice`), runtime heap-aliasing/mutation/state-noop tracing (`run`), and boundary/cache differential testing (`diff`).
---

# Probe — Causal Execution, Reachability & Differential Microscope

Never burn 15+ tool calls reading dead/unreferenced modules or 20,000+ reasoning tokens mentally simulating Python heap aliasing, in-place dict mutations, void-method no-ops, or `bool`/`int`/`None` edge cases. Run `probe.py` first to observe exact static reachability and runtime causality in `<0.1s`.

## Fast 3-Step Debugging Workflow

```bash
# 1. Prune dead distractor files & inspect active cross-file call contracts (<0.05s)
python scripts/probe.py slice . --entry <package_or_entrypoint>

# 2. Trace a minimal reproducer to auto-detect [ALIAS HAZARD], [IN-PLACE MUTATION], and [STATE NO-OP] (<0.1s)
python scripts/probe.py run --cwd . --focus <package> --code "<minimal python repro>"

# 3. Differential & boundary/cache oracle (checks None, True vs 1, False vs 0, sequential cache drift & input purity)
python scripts/probe.py diff --cwd . \
  --setup "<imports and engine setup>" \
  --fn-a "lambda d: <fast_or_cached_call>" \
  --fn-b "lambda d: <reference_call>"
```
