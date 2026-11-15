#!/usr/bin/env python3
"""audit_sessions.py - Local Zero-Cloud Agent Process Telemetry Auditor.

Scans your local Coding / Personal Agent session logs (`~/.claude`, `~/.codex`,
`~/.grok`, `~/.gemini/antigravity-cli`) strictly locally (zero network calls,
never prints file contents or prompts) to quantify exact process bottlenecks,
compare historical vs. recent session friction (`--compare`), and recommend ONLY
the `skills/<folder>` primitives your workflow actually needs.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import time
from collections import Counter
from pathlib import Path
from typing import Any


def _classify_cmd(cmd: str) -> list[str]:
    tags: list[str] = []
    if re.search(r"\bpython(?:\.exe|3)?\s+-c\b|@['\"]\s*\n.*import\s+", cmd, re.DOTALL):
        tags.append("adhoc_inline_python")
    if re.search(r"\b(?:Get-Content|cat|head|tail|Select-Object\s+-(?:First|Last|Skip)|sed\s+-n)\b", cmd):
        tags.append("shell_line_slice_read")
    if re.search(r"\b(?:rg|grep|Select-String|Get-ChildItem|find\s+|findstr)\b", cmd):
        tags.append("shell_broad_search")
    return tags


def _classify_err(txt: str) -> list[str]:
    errs: list[str] = []
    if any(k in txt for k in ("ParserError", "Unexpected token", "The '&&' operator", "The '<' operator", "unterminated triple-quoted")):
        errs.append("shell_quoting_collision")
    if ("Target" in txt and ("not found" in txt or "match" in txt)) or "Failed to find expected lines" in txt or ("apply_patch" in txt.lower() and "error" in txt.lower()):
        errs.append("patch_target_mismatch")
    if any(k in txt for k in ("UnicodeDecodeError", "UnicodeEncodeError", "'gbk' codec", "'charmap' codec")):
        errs.append("encoding_utf8_crash")
    if "ModuleNotFoundError" in txt or "ImportError" in txt or "No module named pytest" in txt:
        errs.append("import_syspath_error")
    if any(k in txt for k in ("truncated due to being too long", "output was truncated", "chars truncated")):
        errs.append("output_truncation_spill")
    return errs


def _scan_codex_file(f: Path) -> Counter[str]:
    st: Counter[str] = Counter()
    prev_cmd = ""
    repeat_streak = 0
    for ln in f.read_text(encoding="utf-8", errors="replace").splitlines():
        try:
            o = json.loads(ln)
        except Exception:
            continue
        p = o.get("payload") if isinstance(o.get("payload"), dict) else {}
        pt = p.get("type", "")
        if pt in ("function_call", "custom_tool_call"):
            st["tool_calls"] += 1
            inp = str(p.get("arguments") or p.get("input") or "")
            norm_inp = re.sub(r"\s+", " ", inp.strip())[:160]
            if norm_inp and norm_inp == prev_cmd:
                repeat_streak += 1
                if repeat_streak >= 2:
                    st["identical_retry_loops"] += 1
            else:
                prev_cmd = norm_inp
                repeat_streak = 0
            for tg in _classify_cmd(inp):
                st[tg] += 1
        elif pt in ("function_call_output", "custom_tool_call_output"):
            out_s = str(p.get("output") or "")
            if len(out_s) >= 8192:
                st["outputs_over_8kb"] += 1
            for ek in _classify_err(out_s):
                st[ek] += 1
    return st


def _scan_agy_file(f: Path) -> Counter[str]:
    st: Counter[str] = Counter()
    seen_reads: Counter[str] = Counter()
    for ln in f.read_text(encoding="utf-8", errors="replace").splitlines():
        try:
            o = json.loads(ln)
        except Exception:
            continue
        content = str(o.get("content") or "")
        for ek in _classify_err(content):
            st[ek] += 1
        for tc in o.get("tool_calls") or []:
            st["tool_calls"] += 1
            nm = tc.get("name") or tc.get("tool_name") or ""
            args = tc.get("args") or tc.get("arguments") or {}
            if isinstance(args, str):
                try:
                    args = json.loads(args)
                except Exception:
                    args = {}
            if nm == "read_file" and isinstance(args, dict):
                st["file_reads"] += 1
                pk = str(args.get("Path") or "")
                if pk:
                    seen_reads[pk] += 1
                    if seen_reads[pk] >= 2:
                        st["redundant_same_file_rereads"] += 1
            elif nm == "run_command" and isinstance(args, dict):
                for tg in _classify_cmd(str(args.get("CommandLine") or "")):
                    st[tg] += 1
    return st


def _scan_claude_file(f: Path) -> Counter[str]:
    st: Counter[str] = Counter()
    seen_reads: Counter[str] = Counter()
    for ln in f.read_text(encoding="utf-8", errors="replace").splitlines():
        try:
            o = json.loads(ln)
        except Exception:
            continue
        msg = o.get("message") if isinstance(o.get("message"), dict) else {}
        content = msg.get("content")
        if isinstance(content, list):
            for blk in content:
                if not isinstance(blk, dict):
                    continue
                bt = blk.get("type")
                if bt == "tool_use":
                    st["tool_calls"] += 1
                    nm = str(blk.get("name") or "")
                    inp_d = blk.get("input") if isinstance(blk.get("input"), dict) else {}
                    if nm == "Read":
                        st["file_reads"] += 1
                        fp = str(inp_d.get("file_path") or "")
                        if fp:
                            seen_reads[fp] += 1
                            if seen_reads[fp] >= 2:
                                st["redundant_same_file_rereads"] += 1
                    elif nm == "Bash":
                        for tg in _classify_cmd(str(inp_d.get("command") or "")):
                            st[tg] += 1
                elif bt == "tool_result":
                    rc = str(blk.get("content") or "")
                    if len(rc) >= 8192:
                        st["outputs_over_8kb"] += 1
                    for ek in _classify_err(rc):
                        st[ek] += 1
    return st


def _scan_grok_file(f: Path) -> Counter[str]:
    st: Counter[str] = Counter()
    seen_reads: Counter[str] = Counter()
    with f.open("r", encoding="utf-8", errors="replace") as fh:
        for ln in fh:
            if "tool_call" not in ln:
                continue
            try:
                o = json.loads(ln)
            except Exception:
                continue
            upd = ((o.get("params") or {}).get("update") or {}) if isinstance(o.get("params"), dict) else {}
            su = upd.get("sessionUpdate")
            if su == "tool_call":
                st["tool_calls"] += 1
                nm = str(upd.get("title") or "")
                rinp = upd.get("rawInput") if isinstance(upd.get("rawInput"), dict) else {}
                if nm == "read_file":
                    st["file_reads"] += 1
                    fp = str(rinp.get("file_path") or rinp.get("path") or "")
                    if fp:
                        seen_reads[fp] += 1
                        if seen_reads[fp] >= 2:
                            st["redundant_same_file_rereads"] += 1
                elif nm == "run_terminal_command":
                    for tg in _classify_cmd(str(rinp.get("command") or "")):
                        st[tg] += 1
            elif su == "tool_call_update" and upd.get("status") in ("completed", "failed"):
                raw_out = json.dumps(upd.get("rawOutput") or "", ensure_ascii=False)
                if len(raw_out) >= 8192:
                    st["outputs_over_8kb"] += 1
                for ek in _classify_err(raw_out):
                    st[ek] += 1
    return st


def _compute_half_comparison(per_session: list[Counter[str]]) -> dict[str, Any] | None:
    """`per_session` is ordered newest-first; compares older half vs newer half."""
    n = len(per_session)
    if n < 2:
        return None
    mid = n // 2
    recent = per_session[:mid]
    older = per_session[mid:]

    def _rates(bucket: list[Counter[str]]) -> dict[str, float]:
        cnt = max(1, len(bucket))
        agg: Counter[str] = Counter()
        for c in bucket:
            agg.update(c)
        crashes = (
            agg["shell_quoting_collision"]
            + agg["patch_target_mismatch"]
            + agg["encoding_utf8_crash"]
            + agg["import_syspath_error"]
        )
        return {
            "sessions": len(bucket),
            "calls_per_session": round(agg["tool_calls"] / cnt, 2),
            "rereads_per_session": round(agg["redundant_same_file_rereads"] / cnt, 2),
            "inline_py_per_session": round(agg["adhoc_inline_python"] / cnt, 2),
            "crashes_per_session": round(crashes / cnt, 2),
            "spills_over_8kb_per_session": round((agg["outputs_over_8kb"] + agg["output_truncation_spill"]) / cnt, 2),
        }

    old_r = _rates(older)
    new_r = _rates(recent)
    deltas: dict[str, str] = {}
    for k in ("calls_per_session", "rereads_per_session", "inline_py_per_session", "crashes_per_session", "spills_over_8kb_per_session"):
        ov, nv = old_r[k], new_r[k]
        if ov > 0:
            pct = ((nv - ov) / ov) * 100.0
            deltas[k] = f"{ov} -> {nv} ({pct:+.1f}%)"
        else:
            deltas[k] = f"{ov} -> {nv}"
    return {"older_half": old_r, "recent_half": new_r, "delta": deltas}


def audit_home(home: Path, max_per_harness: int = 50, compare: bool = False) -> dict[str, Any]:
    t0 = time.perf_counter()
    harnesses: dict[str, dict[str, Any]] = {}
    comparisons: dict[str, Any] = {}
    totals: Counter[str] = Counter()

    specs = [
        ("codex_cli", home / ".codex" / "sessions", "rollout-*.jsonl", _scan_codex_file),
        ("antigravity_cli", home / ".gemini" / "antigravity-cli" / "brain", "transcript.jsonl", _scan_agy_file),
        ("claude_code", home / ".claude" / "projects", "*.jsonl", _scan_claude_file),
        ("grok_cli", home / ".grok" / "sessions", "updates.jsonl", _scan_grok_file),
    ]

    for hname, hdir, pattern, scanner in specs:
        if not hdir.is_dir():
            continue
        files = sorted(hdir.rglob(pattern), key=lambda p: p.stat().st_mtime, reverse=True)[:max_per_harness]
        if not files:
            continue
        h_stats: Counter[str] = Counter()
        per_session: list[Counter[str]] = []
        for f in files:
            st = scanner(f)
            h_stats.update(st)
            per_session.append(st)
        harnesses[hname] = {"sessions": len(files), **dict(h_stats)}
        totals.update(h_stats)
        if compare:
            cmp = _compute_half_comparison(per_session)
            if cmp is not None:
                comparisons[hname] = cmp

    recs: list[dict[str, str]] = []
    if totals["adhoc_inline_python"] + totals["import_syspath_error"] + totals["shell_quoting_collision"] > 0:
        recs.append({
            "primitive": "probe",
            "evidence": f"{totals['adhoc_inline_python']} inline `python -c` probes, {totals['import_syspath_error']} ImportError/sys.path failures, {totals['shell_quoting_collision']} shell-quoting crashes",
            "why": "Replaces brittle inline probes & 30k-token prose mental execution with <0.1s reachability slicing (`slice --bundle`), heap/mutation tracing (`run`), and differential testing (`diff`).",
        })
    if totals["redundant_same_file_rereads"] + totals["shell_line_slice_read"] + totals["patch_target_mismatch"] + totals["encoding_utf8_crash"] > 0:
        recs.append({
            "primitive": "splice",
            "evidence": f"{totals['redundant_same_file_rereads']} redundant file re-reads, {totals['shell_line_slice_read']} shell line-slice reads, {totals['patch_target_mismatch']} patch mismatches, {totals['encoding_utf8_crash']} UTF-8/encoding crashes",
            "why": "Extracts exact polyglot symbols/Markdown sections (`view`), applies UTF-8/CRLF-safe atomic edits (`apply`), and auto-runs the right test suite (`check --test`).",
        })
    if totals["outputs_over_8kb"] + totals["output_truncation_spill"] > 0:
        recs.append({
            "primitive": "sieve",
            "evidence": f"{totals['outputs_over_8kb']} tool outputs >8 KB, {totals['output_truncation_spill']} truncation spills",
            "why": "Compresses noisy CI/JSON/HTML outputs by ~98% into structured failure/schema digests while indexing raw streams in local FTS5.",
        })
    if totals["shell_broad_search"] > 5:
        recs.append({
            "primitive": "atlas",
            "evidence": f"{totals['shell_broad_search']} broad shell search/listing calls",
            "why": "Generates a deterministic <800-token structural codebase/vault map + stale-TODO trap warning in <0.1s.",
        })
    if totals["identical_retry_loops"] > 0 or totals["tool_calls"] > 50:
        recs.append({
            "primitive": "guard + baton",
            "evidence": f"{totals['identical_retry_loops']} identical retry loops across {totals['tool_calls']} tool calls",
            "why": "Hard-stops identical failing retries (`exit 2`), verifies tests ran after edits, and preserves verified facts/dead-ends across compaction.",
        })

    payload: dict[str, Any] = {
        "elapsed_ms": round((time.perf_counter() - t0) * 1000.0, 2),
        "harnesses_detected": harnesses,
        "aggregate_friction_counts": dict(totals),
        "recommended_primitives": recs,
    }
    if compare:
        payload["trend_comparison"] = comparisons
    return payload


def main(argv: list[str] | None = None) -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    parser = argparse.ArgumentParser(prog="audit_sessions.py", description="Local zero-cloud agent session telemetry auditor")
    parser.add_argument("--home", default=str(Path.home()), help="Home directory containing .codex / .claude / .gemini / .grok")
    parser.add_argument("--max-sessions", type=int, default=50, help="Max recent sessions per harness (default: 50)")
    parser.add_argument("--compare", action="store_true", help="Compare older half vs recent half of sessions per harness")
    parser.add_argument("--json", action="store_true", help="Emit JSON output")
    args = parser.parse_args(argv)

    rep = audit_home(Path(args.home), max_per_harness=args.max_sessions, compare=args.compare)
    if args.json:
        sys.stdout.write(json.dumps(rep, indent=2, ensure_ascii=False) + "\n")
        return 0

    lines = [f"[agent-primitives audit] Scanned {len(rep['harnesses_detected'])} local agent harness(es) in {rep['elapsed_ms']} ms (100% local, 0 bytes uploaded)"]
    for hname, hdata in rep["harnesses_detected"].items():
        lines.append(f"  - {hname}: {hdata['sessions']} sessions, {hdata.get('tool_calls', 0)} tool calls")
    if args.compare and rep.get("trend_comparison"):
        lines.append("\n## Trend Comparison (Older Half -> Recent Half of Sessions)")
        for hname, cmp in rep["trend_comparison"].items():
            d = cmp["delta"]
            lines.append(
                f"  - **{hname}** ({cmp['older_half']['sessions']} older vs {cmp['recent_half']['sessions']} recent): "
                f"calls/sess `{d['calls_per_session']}` | rereads/sess `{d['rereads_per_session']}` | "
                f"crashes/sess `{d['crashes_per_session']}` | >8KB/sess `{d['spills_over_8kb_per_session']}`"
            )
    if rep["recommended_primitives"]:
        lines.append("\n## Empirically Triggered Primitives For Your Machine")
        for r in rep["recommended_primitives"]:
            lines.append(f"- **`skills/{r['primitive']}`** — Observed: {r['evidence']}")
            lines.append(f"  Fix: {r['why']}")
        lines.append("\nZero-config 1-command onboarding:\n  python ap.py init          # Global auto-detected setup + `ap` CLI\n  python ap.py init --here   # Project-local setup in current repo")
    else:
        lines.append("\nNo local agent session logs found yet. Onboard in 1 command: `python ap.py init`")
    sys.stdout.write("\n".join(lines) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
