---
name: atlas
description: Generate a deterministic <1,000-token Tree-sitter structural map of any codebase (Py/TS/JS/Go/Rust/C) or knowledge vault in <50ms (entrypoints, AST symbols, hub modules, git hotspots, and stale-TODO traps).
---

# Atlas — Sub-50ms Tree-sitter Zero-Read Structural Map

Stop burning 50k–600k tokens wandering across directories at session start, and avoid the +77% token trap of chasing stale dead TODOs. `atlas` maps **both codebases** (Tree-sitter AST symbols across Python, TS/JS, Go, Rust & C, import centrality, entrypoints, `[dirty]` git hotspots, stale TODO warnings) and **personal assistant vaults** (Markdown heading trees, active documents) in `<50ms`.

## Usage

```bash
# 1. Generate <1,000-token structural overview of current repo or vault
ap atlas map .

# 2. Focus on a specific subsystem or query keyword
ap atlas map . --focus "exporter" --max-files 20

# 3. Emit machine-readable JSON
ap atlas map . --json
```
