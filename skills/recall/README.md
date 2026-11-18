# `recall` (`crates/ap-recall`) — Sub-Millisecond Triggered Fact Vault (Bundled SQLite FTS5)

[English](./README.md) | [简体中文](./README.zh-CN.md)

**Standalone Rust Crate (`crates/ap-recall`) & Skill (`skills/recall/`)** · Bundled static SQLite FTS5/BM25 · `58 tokens` idle skill cost.

## Why You Need It

Stuffing 200+ gotchas or personal SOPs into `AGENTS.md` / `CLAUDE.md` / `MEMORY.md` burns `6,880+ tokens` on *every* turn and dilutes attention. `ap recall` embeds static SQLite 3 (`FTS5` + `BM25`) + deterministic path-glob scoring + hybrid English/CJK bigram indexing into the `ap` binary—retrieving the exact top-2 rules (`~47 tokens`, **-99.3%**) in **`3.3 ms`** with **100% Top-1 accuracy** (`20/20`).

## Real Commands

```bash
ap recall add \
  --triggers "src/auth/*,jwt,refresh" \
  --rule "Refresh tokens rotate on every use; never retry 401 with cached token." \
  --kind gotcha

ap recall add \
  --triggers "invoice,报销,travel" \
  --rule "Attach PDF receipt hash and CC finance-archive@example.com." \
  --kind workflow

ap recall match --query "401 loop on token refresh" --files "src/auth/session.rs" --top 2
ap recall stats
ap recall prune --stale-days 90
```
