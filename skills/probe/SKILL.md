---
name: probe
description: Prune dead files & bundle active code (`slice --bundle`), run Windows-safe headless browser/HTTP server smoke journeys (`smoke`), trace heap/mutation bugs (`run`), and run diff oracles (`diff`).
---

# Probe — Reachability Bundler, Live Server/DOM Smoke Runner & Runtime Microscope

Never read unreferenced slop modules, never burn 20+ interactive browser/curl turns verifying a local web server on Windows, and never guess heap aliasing or rounding behavior in prose.

## Usage

```bash
# 1. Prune dead distractor files AND emit numbered source of active reachable modules in 1 call
ap probe slice . --bundle

# 2. Windows-safe ephemeral server + headless Edge/Chrome DOM + UTF-8 HTTP E2E smoke test (1 turn, 0 orphan processes):
@'
BROWSER /
EXPECT CONTAINS 待处理
GET /api/tickets?q=支付
EXPECT STATUS 200
POST /api/tickets/archive
BODY {"confirm":true}
EXPECT JSON stats.done == 0
'@ | ap probe smoke --cwd . --stdin

# 3. Runtime microscope & differential oracle across Python, TypeScript/JS, or Rust
ap probe run --cwd . --code "import { Ledger } from './src/ledger.js'; new Ledger().snapshot();"
ap probe diff --lang rust --fn-a "|x: i64| (x * 250 + 5000) / 10000" --fn-b "|x: i64| ((x as f64 / 100.0) * 2.5).round() as i64"
```
