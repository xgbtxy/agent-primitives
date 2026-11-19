---
name: atlas
description: Generate <1,000-token Tree-sitter structural & omni-workspace maps (`map`), UI/API product maps (`product`), and <25ms symptom-to-function fault localization (`locate`).
---

# Atlas — Sub-50ms Tree-sitter Structural, Product & Fault Localization Map

Stop burning 50k–600k tokens wandering across directories or grepping blindly. `atlas` maps **code + `.xlsx`/`.csv`/`.sqlite`/archives** (`map`), **full-stack UI/API contracts** (`product`), and **symptom-to-function Top-5 fault localization** (`locate`) in `<50ms`.

## Usage

```bash
# 1. Generate <1,000-token structural overview of current repo or vault
ap atlas map .

# 2. Localize a bug symptom or error directly to Top-5 suspect functions with numbered lines (<25ms)
ap atlas locate "voucher percent max cap payable" .

# 3. Extract UI controls, registered API routes, broken frontend<->backend links & slop smells
ap atlas product . --write PRODUCT_MAP.md
```
