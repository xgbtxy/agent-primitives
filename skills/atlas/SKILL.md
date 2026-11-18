---
name: atlas
description: Generate <1,000-token Tree-sitter structural maps (`map`) and UI/API product contract & slop maps (`product`) across Py/TS/JS/Go/Rust/C codebases and vaults in <50ms.
---

# Atlas — Sub-50ms Tree-sitter Structural & Product Feature Map

Stop burning 50k–600k tokens wandering across directories or guessing UI/API routes. `atlas` maps **codebases & vaults** (`map`) AND **full-stack product surfaces** (`product`: HTML controls, backend HTTP routes, `[BROKEN]` frontend↔backend mismatches, and `[SLOP RADAR]` hygiene warnings) in `<50ms`.

## Usage

```bash
# 1. Generate <1,000-token structural overview of current repo or vault
ap atlas map .

# 2. Extract UI controls, registered API routes, broken frontend<->backend links & slop smells
ap atlas product . --write PRODUCT_MAP.md

# 3. Focus on a subsystem or emit JSON
ap atlas map . --focus "exporter" --json
```
