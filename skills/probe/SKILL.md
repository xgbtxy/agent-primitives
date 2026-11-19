---
name: probe
description: Attach to live Chrome/Electron CDP ports (`cdp`), run headless UI layout+network audits (`ui`) & smoke tests (`smoke`), API fuzzing (`fuzz`), AST mutation (`mutate`), and slicing (`slice`).
---

# Probe — Live CDP Attacher, Headless UI Auditor, API Fuzzer & AST Mutator

Zero Playwright/Node installs, zero fragile Python WebSocket scripts, zero orphan processes on Windows.

## Usage

```bash
# 1. Attach to an ALREADY-RUNNING Chrome/Edge/Electron CDP port (never kills the host app!)
ap probe cdp --port 9222 --list
ap probe cdp --port 9222 --target "Dashboard" --eval "document.title" --audit

# 2. Desktop (1280x800) + Mobile (390x844) layout, overflow, button-overlap, JS & 4xx/5xx network audit
ap probe ui --cwd . --start "node src/server.mjs"

# 3. Adversarial API chaos + parallel concurrency race fuzzer & Sub-2s AST mutation testing
ap probe fuzz --cwd . --start "node src/server.mjs"
ap probe mutate src/core.mjs --cwd . --max-mutants 8

# 4. Interactive Headless Edge/Chrome DOM + HTTP E2E smoke test
@'
OPEN /
VIEWPORT 390 844
AUDIT_UI
FILL #name Alice
CLICK #submit
EXPECT CONTAINS Alice
'@ | ap probe smoke --cwd . --start "node src/server.mjs" --stdin
```
