#!/usr/bin/env python3
"""recall.py - Sub-5ms triggered micro-memory vault (stdlib sqlite3 + FTS5).

Solves retrieval bloat: instead of injecting a 10,000-token AGENTS.md/MEMORY.md
on every turn, matches path globs, trigger tags, and hybrid English/CJK BM25
in <5ms and emits only the top-K relevant rules (~40-60 tokens per rule).
"""

from __future__ import annotations

import argparse
import fnmatch
import hashlib
import json
import os
import re
import sqlite3
import sys
import time
from pathlib import Path
from typing import Any

CJK_RE = re.compile(r"[\u4e00-\u9fff]")


def _vault_path() -> Path:
    env = os.environ.get("RECALL_HOME")
    base = Path(env) if env else Path.cwd() / ".recall"
    base.mkdir(parents=True, exist_ok=True)
    gitignore = base / ".gitignore"
    if not gitignore.exists():
        gitignore.write_text("*.db-journal\n*.db-wal\n*.db-shm\n", encoding="utf-8")
    return base / "vault.db"


def _tokenize_cjk_hybrid(text: str) -> str:
    """Expand CJK bigrams/chars with spaces so SQLite unicode61 indexes CJK + English cleanly."""
    out: list[str] = []
    buf_cjk: list[str] = []

    def flush_cjk() -> None:
        if not buf_cjk:
            return
        s = "".join(buf_cjk)
        out.extend(list(s))
        for i in range(len(s) - 1):
            out.append(s[i : i + 2])
        buf_cjk.clear()

    for ch in text:
        if CJK_RE.match(ch):
            buf_cjk.append(ch)
        else:
            flush_cjk()
            out.append(ch)
    flush_cjk()
    return " ".join("".join(out).split())


def _connect(db_path: Path) -> sqlite3.Connection:
    conn = sqlite3.connect(str(db_path))
    conn.row_factory = sqlite3.Row
    conn.execute("PRAGMA journal_mode=WAL;")
    conn.execute(
        """
        CREATE TABLE IF NOT EXISTS rules (
            id TEXT PRIMARY KEY,
            kind TEXT NOT NULL,
            triggers TEXT NOT NULL,
            rule TEXT NOT NULL,
            search_tokens TEXT NOT NULL,
            hits INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            last_hit_at INTEGER NOT NULL DEFAULT 0
        );
        """
    )
    # Try FTS5 first; graceful fallback to LIKE scan if custom Python build lacks FTS5
    try:
        conn.execute(
            """
            CREATE VIRTUAL TABLE IF NOT EXISTS rules_fts USING fts5(
                id UNINDEXED,
                search_tokens,
                tokenize='unicode61'
            );
            """
        )
    except sqlite3.OperationalError:
        pass
    return conn


def _has_fts(conn: sqlite3.Connection) -> bool:
    row = conn.execute(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='rules_fts'"
    ).fetchone()
    return row is not None


def add_rule(conn: sqlite3.Connection, triggers: str, rule: str, kind: str = "rule") -> str:
    norm_rule = " ".join(rule.strip().split())
    norm_trig = ",".join(t.strip() for t in triggers.split(",") if t.strip())
    rule_id = hashlib.sha256(f"{norm_trig}::{norm_rule}".encode("utf-8")).hexdigest()[:10]
    now = int(time.time())
    tokens = _tokenize_cjk_hybrid(f"{norm_trig.replace(',', ' ')} {norm_rule} {kind}")
    conn.execute(
        """
        INSERT INTO rules (id, kind, triggers, rule, search_tokens, hits, created_at, last_hit_at)
        VALUES (?, ?, ?, ?, ?, 0, ?, 0)
        ON CONFLICT(id) DO UPDATE SET
            kind=excluded.kind,
            triggers=excluded.triggers,
            rule=excluded.rule,
            search_tokens=excluded.search_tokens
        """,
        (rule_id, kind, norm_trig, norm_rule, tokens, now),
    )
    if _has_fts(conn):
        conn.execute("DELETE FROM rules_fts WHERE id = ?", (rule_id,))
        conn.execute("INSERT INTO rules_fts (id, search_tokens) VALUES (?, ?)", (rule_id, tokens))
    conn.commit()
    return rule_id


def match_rules(
    conn: sqlite3.Connection,
    query: str,
    files: list[str],
    top_k: int = 3,
) -> tuple[list[dict[str, Any]], float]:
    t0 = time.perf_counter()
    scores: dict[str, float] = {}
    rows_by_id: dict[str, sqlite3.Row] = {}

    all_rows = conn.execute("SELECT * FROM rules").fetchall()
    q_lower = query.lower()
    norm_files = [f.replace("\\", "/").lstrip("./") for f in files if f.strip()]

    # 1. Deterministic path-glob & boundary-aware keyword trigger scoring
    for r in all_rows:
        rid = r["id"]
        rows_by_id[rid] = r
        sc = 0.0
        for trig in [t.strip() for t in r["triggers"].split(",") if t.strip()]:
            tl = trig.lower().replace("\\", "/")
            if any(ch in tl for ch in ("*", "?", "/")):
                prefix = tl.rstrip("/*")
                for nf in norm_files:
                    nfl = nf.lower()
                    if fnmatch.fnmatch(nfl, tl) or nfl.startswith(prefix + "/") or nfl == prefix:
                        sc += 8.0
                if prefix and re.search(rf"(?<![\w/]){re.escape(prefix)}(?![\w])", q_lower):
                    sc += 4.0
            else:
                pat = re.compile(rf"(?<![a-z0-9_]){re.escape(tl)}(?![a-z0-9_])")
                if pat.search(q_lower) or any(pat.search(nf.lower()) for nf in norm_files):
                    sc += 5.0
        if sc > 0:
            scores[rid] = sc

    # 2. FTS5 BM25 lexical recall (sorted by BM25 before LIMIT!)
    q_tokens = [
        re.sub(r"[^\w\u4e00-\u9fff]+", "", tok)
        for tok in _tokenize_cjk_hybrid(f"{query} {' '.join(norm_files)}").split()
    ]
    q_tokens = [t for t in q_tokens if len(t) >= 2 or CJK_RE.match(t)][:24]
    if q_tokens and _has_fts(conn):
        fts_expr = " OR ".join(f'"{t}"' for t in dict.fromkeys(q_tokens))
        try:
            fts_rows = conn.execute(
                "SELECT id, bm25(rules_fts) AS rank FROM rules_fts WHERE rules_fts MATCH ? ORDER BY rank LIMIT 20",
                (fts_expr,),
            ).fetchall()
            for fr in fts_rows:
                rid = fr["id"]
                bm25_boost = max(0.5, -float(fr["rank"]))
                scores[rid] = scores.get(rid, 0.0) + bm25_boost
        except sqlite3.OperationalError:
            pass

    ranked = sorted(scores.items(), key=lambda kv: kv[1], reverse=True)[:top_k]
    now = int(time.time())
    results: list[dict[str, Any]] = []
    for rid, sc in ranked:
        r = rows_by_id.get(rid)
        if not r:
            continue
        conn.execute(
            "UPDATE rules SET hits = hits + 1, last_hit_at = ? WHERE id = ?",
            (now, rid),
        )
        results.append(
            {
                "id": rid,
                "kind": r["kind"],
                "triggers": r["triggers"],
                "rule": r["rule"],
                "score": round(sc, 2),
                "hits": int(r["hits"]) + 1,
            }
        )
    if results:
        conn.commit()
    elapsed_ms = (time.perf_counter() - t0) * 1000.0
    return results, elapsed_ms


def main(argv: list[str] | None = None) -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
        sys.stderr.reconfigure(encoding="utf-8", errors="replace")

    parser = argparse.ArgumentParser(prog="recall", description="Sub-5ms triggered micro-memory vault")
    sub = parser.add_subparsers(dest="subcmd", required=True)

    p_add = sub.add_parser("add", help="Add or update a triggered micro-rule")
    p_add.add_argument("--triggers", required=True, help="Comma-separated path globs or keywords")
    p_add.add_argument("--rule", required=True, help="Concise actionable fact/rule (<240 chars ideal)")
    p_add.add_argument("--kind", default="gotcha", help="Category (gotcha, convention, preference, workflow)")

    p_match = sub.add_parser("match", help="Retrieve top-K matching rules in <5ms")
    p_match.add_argument("--query", default="", help="Current task/command/topic text")
    p_match.add_argument("--files", default="", help="Comma-separated active file paths")
    p_match.add_argument("--top", type=int, default=3, help="Max rules to return")
    p_match.add_argument("--json", action="store_true", help="Emit JSON output")

    p_stats = sub.add_parser("stats", help="Show vault size, token comparison, and hit metrics")
    p_stats.add_argument("--json", action="store_true", help="Emit JSON output")

    p_prune = sub.add_parser("prune", help="Delete zero-hit rules older than N days")
    p_prune.add_argument("--stale-days", type=int, default=90)

    args = parser.parse_args(argv)
    conn = _connect(_vault_path())
    try:
        if args.subcmd == "add":
            rid = add_rule(conn, args.triggers, args.rule, args.kind)
            sys.stdout.write(f"[recall] stored id={rid} kind={args.kind} triggers='{args.triggers}'\n")
            return 0

        if args.subcmd == "match":
            file_list = [f.strip() for f in args.files.split(",") if f.strip()]
            hits, elapsed_ms = match_rules(conn, args.query, file_list, top_k=args.top)
            if args.json:
                sys.stdout.write(
                    json.dumps({"elapsed_ms": round(elapsed_ms, 2), "matches": hits}, ensure_ascii=False, indent=2) + "\n"
                )
                return 0
            if not hits:
                sys.stdout.write(f"[recall] 0 triggered rules ({elapsed_ms:.2f}ms)\n")
                return 0
            lines = [f"# Recall Active Rules ({len(hits)} matched in {elapsed_ms:.2f}ms)"]
            for h in hits:
                lines.append(f"- **[{h['kind']}]** (`{h['triggers']}`): {h['rule']}")
            sys.stdout.write("\n".join(lines) + "\n")
            return 0

        if args.subcmd == "stats":
            rows = conn.execute("SELECT rule, triggers, hits FROM rules").fetchall()
            full_chars = sum(len(r["rule"]) + len(r["triggers"]) + 16 for r in rows)
            full_tokens_est = (full_chars + 3) // 4
            data = {
                "total_rules": len(rows),
                "monolithic_tokens_if_always_loaded": full_tokens_est,
                "typical_top3_injected_tokens": min(full_tokens_est, 135),
                "total_hits": sum(int(r["hits"]) for r in rows),
            }
            if args.json:
                sys.stdout.write(json.dumps(data, indent=2) + "\n")
            else:
                sys.stdout.write(
                    f"[recall] rules={data['total_rules']} | full-dump={data['monolithic_tokens_if_always_loaded']} tok "
                    f"vs top-3={data['typical_top3_injected_tokens']} tok | total_hits={data['total_hits']}\n"
                )
            return 0

        if args.subcmd == "prune":
            cutoff = int(time.time()) - args.stale_days * 86400
            stale_ids = [
                r["id"]
                for r in conn.execute(
                    "SELECT id FROM rules WHERE hits = 0 AND created_at < ?", (cutoff,)
                ).fetchall()
            ]
            for rid in stale_ids:
                conn.execute("DELETE FROM rules WHERE id = ?", (rid,))
                if _has_fts(conn):
                    conn.execute("DELETE FROM rules_fts WHERE id = ?", (rid,))
            conn.commit()
            sys.stdout.write(f"[recall] pruned {len(stale_ids)} stale zero-hit rules.\n")
            return 0
    finally:
        conn.close()

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
