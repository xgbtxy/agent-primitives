---
name: atlas
description: Sub-50ms Tree-sitter workspace maps (`map`), UI/API product maps (`product`), symptom fault localization (`locate`), and multi-hop call-chain tracing (`trace`).
---

# Atlas — Sub-50ms Structural Map, Fault Localizer & Multi-Hop Call-Chain Tracer

Stop burning turns wandering across directories or writing Python AST call-graph scripts.

## Usage

```bash
# 1. Generate <1,000-token structural overview of code + .xlsx/.csv/.sqlite/archives
ap atlas map .

# 2. Localize a bug symptom directly to Top-5 suspect functions + numbered source (<25ms)
ap atlas locate "voucher percent max cap payable" .

# 3. Trace multi-hop upstream callers (1..3 hops), downstream callees & I/O/exec/SQL sinks (<30ms)
ap atlas trace readDiskAsset . --depth 3

# 4. Extract UI controls, registered API routes, broken frontend<->backend links & slop smells
ap atlas product . --write PRODUCT_MAP.md

# 5. Git-diff-aware Change Impact Brief: modified AST symbols, untouched callers & related tests (<30ms)
ap atlas changes . --base HEAD
```
