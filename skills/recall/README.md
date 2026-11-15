# `recall` — Sub-5ms Triggered Fact Vault (SQLite FTS5/BM25)

[English](./README.md) | [简体中文](./README.zh-CN.md)

**Standalone Mini-Project (`skills/recall/`)** · Zero third-party dependencies (`stdlib sqlite3`) · `71 tokens` idle skill cost.

## Why You Need It

Most agent memory failures are **retrieval bugs**, not storage bugs:
- Stuffing 150+ subsystem gotchas or personal SOPs into a monolithic `AGENTS.md` / `CLAUDE.md` / `MEMORY.md` burns `6,000–15,000 tokens` on *every* turn and dilutes attention so the model ignores the exact rule it needed.
- Always-on vector MCP servers consume thousands of tokens just for tool schemas and require external processes/embeddings.

`recall` uses Python's built-in `sqlite3` FTS5 (`BM25` ranking) combined with boundary-aware path/keyword triggers and hybrid English + CJK bigram indexing. On a 200-rule vault, it retrieves the exact top-2 rules (`~40 tokens` total, **-99.4%** vs monolithic file) in **`3.7 ms`** with **100% Top-1 accuracy**.

## Copy Just This Folder

```bash
cp -r skills/recall ~/.agents/skills/
# or: npx skills add xgbtxy/agent-primitives --skill recall
```

## Real Commands

```bash
# Coding rule (triggered by file glob or keyword)
python scripts/recall.py add \
  --triggers "src/auth/*,jwt,refresh" \
  --rule "Refresh tokens rotate on every use; never retry 401 with cached token." \
  --kind gotcha

# Personal Assistant SOP (bilingual English + CJK supported natively)
python scripts/recall.py add \
  --triggers "invoice,报销,travel" \
  --rule "Attach PDF receipt hash and CC finance-archive@example.com." \
  --kind workflow

# Sub-5ms targeted retrieval before acting
python scripts/recall.py match --query "401 loop on token refresh" --files "src/auth/session.py" --top 2

# Vault statistics & stale zero-hit pruning
python scripts/recall.py stats
python scripts/recall.py prune --stale-days 90
```
