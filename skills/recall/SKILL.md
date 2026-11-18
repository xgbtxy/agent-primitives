---
name: recall
description: Store and retrieve path-, command-, or topic-triggered micro-rules in <1ms using bundled Rust SQLite FTS5/BM25 instead of bloating AGENTS.md or MEMORY.md. Injects only the 1-3 matching facts (~120 tokens).
---

# Recall — Sub-Millisecond Triggered Fact Vault

Most agent memory failures are **retrieval** bugs, not storage bugs. Stuffing hundreds of rules into `AGENTS.md` / `MEMORY.md` wastes thousands of tokens on every turn. `recall` uses bundled SQLite FTS5 + glob/keyword triggers to inject **only** the 1–3 rules matching the current file, command, or topic in `<1ms` (English & CJK supported).

## Usage

```bash
# 1. Add a triggered rule (coding gotcha OR personal assistant workflow)
ap recall add \
  --triggers "src/auth/*,jwt,refresh" \
  --rule "Refresh tokens rotate on use; never retry 401 with cached token." \
  --kind gotcha

ap recall add \
  --triggers "invoice,报销,finance" \
  --rule "Use entity 'Independent Lab' and CC finance-archive@example.com." \
  --kind preference

# 2. Match top rules for the active task/files (<1ms, ~120 tokens output)
ap recall match --query "fix 401 loop in token refresh" --files "src/auth/client.rs" --top 3

# 3. Inspect vault health or prune stale unused entries
ap recall stats
```
