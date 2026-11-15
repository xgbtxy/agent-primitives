---
name: atlas
description: Generate a deterministic <1,000-token structural map of any codebase or personal knowledge vault in <0.5s (entrypoints, AST/symbol signatures, hub modules, recent churn, Markdown outlines, and stale-TODO traps) without reading whole files or setting up vector DBs.
---

# Atlas — Sub-Second Zero-Read Structural Map

Stop burning 50k–600k tokens wandering across directories at session start, and avoid the +77% token trap of chasing stale dead TODOs. `atlas` maps **both codebases** (AST classes/functions, import centrality, entrypoints, git hotspots, stale TODO warnings) and **personal assistant vaults** (Markdown heading trees, active documents, data files) in `<0.5s` with zero dependencies.

## Usage

```bash
# 1. Generate <1,000-token structural overview of current repo or vault
python scripts/atlas.py map .

# 2. Focus on a specific subsystem or query keyword
python scripts/atlas.py map . --focus "exporter" --max-files 20

# 3. Emit machine-readable JSON
python scripts/atlas.py map . --json
```
