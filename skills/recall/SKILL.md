---
name: recall
description: Store and retrieve path-, command-, or topic-triggered micro-rules in <5ms using Python stdlib SQLite FTS5/BM25 instead of bloating AGENTS.md or MEMORY.md. Injects only the 1-3 matching facts (~120 tokens) for coding gotchas or personal assistant workflows.
---

# Recall — Sub-5ms Triggered Fact Vault

Most agent memory failures are **retrieval** bugs, not storage bugs. Stuffing hundreds of rules into `AGENTS.md` / `MEMORY.md` wastes thousands of tokens on every turn. `recall` uses Python's built-in `sqlite3` FTS5 + glob/keyword triggers to inject **only** the 1–3 rules matching the current file, command, or topic in `<5ms` (English & CJK supported, zero pip packages).

## Usage

```bash
# 1. Add a triggered rule (coding gotcha OR personal assistant rule)
python scripts/recall.py add \
  --triggers "src/auth/*,jwt,refresh" \
  --rule "Refresh tokens rotate on use; never retry 401 with cached token." \
  --kind gotcha

python scripts/recall.py add \
  --triggers "invoice,报销,finance" \
  --rule "Use entity 'Independent Lab' and CC finance-archive@example.com." \
  --kind preference

# 2. Match top rules for the active task/files (<5ms, ~120 tokens output)
python scripts/recall.py match --query "fix 401 loop in token refresh" --files "src/auth/client.py" --top 3

# 3. Inspect vault health or prune stale unused entries
python scripts/recall.py stats
```
