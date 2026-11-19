---
name: probe
description: Run headless Edge/Chrome UI layout+network audits (`ui`) & E2E smoke tests (`smoke`), API chaos/race fuzzing (`fuzz`), AST mutation testing (`mutate`), dead-file slicing (`slice`), and diff oracles (`diff`).
---

# Probe — Headless UI Auditor, API Fuzzer, AST Mutator & E2E Smoke Runner

Zero Playwright/Node installs, zero orphan processes on Windows.

## Usage

```bash
# 1. Desktop (1280x800) + Mobile (390x844) layout, overflow, button-overlap, JS & 4xx/5xx network audit
ap probe ui --cwd . --start "node src/server.mjs"

# 2. Adversarial API chaos (bad JSON, nulls, negatives, SQLi) + parallel concurrency race fuzzer
ap probe fuzz --cwd . --start "node src/server.mjs"

# 3. Sub-2s Polyglot Tree-sitter AST mutation testing to prove tests actually catch real bugs
ap probe mutate src/core.mjs --cwd . --max-mutants 8

# 4. Interactive Headless Edge/Chrome DOM + HTTP E2E smoke test (supports OPEN, VIEWPORT, FILL, CLICK, EVAL, AUDIT_UI, SCREENSHOT)
@'
GET /api/health
EXPECT STATUS 200
OPEN /
VIEWPORT 390 844
AUDIT_UI
FILL #name Alice
CLICK #submit
WAIT_MS 300
EXPECT CONTAINS Alice
'@ | ap probe smoke --cwd . --start "node src/server.mjs" --stdin
```
