#!/usr/bin/env python3
"""audit_sessions.py - Local Zero-Cloud Agent Process Telemetry Auditor.

Scans your local Coding / Personal Agent session logs (`~/.claude`, `~/.codex`,
`~/.grok`, `~/.gemini/antigravity-cli`) strictly locally (zero network calls,
never prints file contents or prompts) to quantify exact process bottlenecks
and recommend ONLY the `skills/<folder>` primitives your workflow actually needs.
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
    if "ModuleNotFoundError" in txt or "ImportError" in txt:
        errs.append("import_syspath_error")
    if any(k in txt for k in ("truncated due to being too long", "output was truncated", "chars truncated")):
        errs.append("output_truncation_spill")
    return errs


def audit_home(home: Path, max_per_harness: int = 50) -> dict[str, Any]:
    t0 = time.perf_counter()
    harnesses: dict[str, dict[str, Any]] = {}
    totals: Counter[str] = Counter()

    # 1. OpenAI Codex CLI (~/.codex/sessions/**/rollout-*.jsonl)
    codex_dir = home / ".codex" / "sessions"
    if codex_dir.is_dir():
        files = sorted(codex_dir.rglob("rollout-*.jsonl"), key=lambda p: p.stat().st_mtime, reverse=True)[:max_per_harness]
        c_stats: Counter[str] = Counter()
        for f in files:
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
                    c_stats["tool_calls"] += 1
                    inp = str(p.get("arguments") or p.get("input") or "")
                    norm_inp = re.sub(r"\s+", " ", inp.strip())[:160]
                    if norm_inp and norm_inp == prev_cmd:
                        repeat_streak += 1
                        if repeat_streak >= 2:
                            c_stats["identical_retry_loops"] += 1
                    else:
                        prev_cmd = norm_inp
                        repeat_streak = 0
                    for tg in _classify_cmd(inp):
                        c_stats[tg] += 1
                elif pt in ("function_call_output", "custom_tool_call_output"):
                    out_s = str(p.get("output") or "")
                    if len(out_s) >= 8192:
                        c_stats["outputs_over_8kb"] += 1
                    for ek in _classify_err(out_s):
                        c_stats[ek] += 1
        if files:
            harnesses["codex_cli"] = {"sessions": len(files), **dict(c_stats)}
            totals.update(c_stats)

    # 2. Google Antigravity CLI (~/.gemini/antigravity-cli/brain/**/transcript.jsonl)
    agy_dir = home / ".gemini" / "antigravity-cli" / "brain"
    if agy_dir.is_dir():
        files = sorted(agy_dir.rglob("transcript.jsonl"), key=lambda p: p.stat().st_mtime, reverse=True)[:max_per_harness]
        a_stats: Counter[str] = Counter()
        for f in files:
            seen_reads: Counter[str] = Counter()
            for ln in f.read_text(encoding="utf-8", errors="replace").splitlines():
                try:
                    o = json.loads(ln)
                except Exception:
                    continue
                content = str(o.get("content") or "")
                for ek in _classify_err(content):
                    a_stats[ek] += 1
                for tc in o.get("tool_calls") or []:
                    a_stats["tool_calls"] += 1
                    nm = tc.get("name") or tc.get("tool_name") or ""
                    args = tc.get("args") or tc.get("arguments") or {}
                    if isinstance(args, str):
                        try:
                            args = json.loads(args)
                        except Exception:
                            args = {}
                    if nm == "read_file" and isinstance(args, dict):
                        a_stats["file_reads"] += 1
                        pk = str(args.get("Path") or "")
                        if pk:
                            seen_reads[pk] += 1
                            if seen_reads[pk] >= 2:
                                a_stats["redundant_same_file_rereads"] += 1
                    elif nm == "run_command" and isinstance(args, dict):
                        for tg in _classify_cmd(str(args.get("CommandLine") or "")):
                            a_stats[tg] += 1
        if files:
            harnesses["antigravity_cli"] = {"sessions": len(files), **dict(a_stats)}
            totals.update(a_stats)

    # 3. Anthropic Claude Code (~/.claude/projects/**/*.jsonl)
    claude_dir = home / ".claude" / "projects"
    if claude_dir.is_dir():
        files = sorted(claude_dir.rglob("*.jsonl"), key=lambda p: p.stat().st_mtime, reverse=True)[:max_per_harness]
        cl_stats: Counter[str] = Counter()
        for f in files:
            seen_reads = Counter()
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
                            cl_stats["tool_calls"] += 1
                            nm = str(blk.get("name") or "")
                            inp_d = blk.get("input") if isinstance(blk.get("input"), dict) else {}
                            if nm == "Read":
                                cl_stats["file_reads"] += 1
                                fp = str(inp_d.get("file_path") or "")
                                if fp:
                                    seen_reads[fp] += 1
                                    if seen_reads[fp] >= 2:
                                        cl_stats["redundant_same_file_rereads"] += 1
                            elif nm == "Bash":
                                for tg in _classify_cmd(str(inp_d.get("command") or "")):
                                    cl_stats[tg] += 1
                        elif bt == "tool_result":
                            rc = str(blk.get("content") or "")
                            if len(rc) >= 8192:
                                cl_stats["outputs_over_8kb"] += 1
                            for ek in _classify_err(rc):
                                cl_stats[ek] += 1
        if files:
            harnesses["claude_code"] = {"sessions": len(files), **dict(cl_stats)}
            totals.update(cl_stats)

    # 4. xAI Grok Build CLI (~/.grok/sessions/**/updates.jsonl)
    grok_dir = home / ".grok" / "sessions"
    if grok_dir.is_dir():
        files = sorted(grok_dir.rglob("updates.jsonl"), key=lambda p: p.stat().st_mtime, reverse=True)[:max_per_harness]
        g_stats: Counter[str] = Counter()
        for f in files:
            seen_reads = Counter()
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
                        g_stats["tool_calls"] += 1
                        nm = str(upd.get("title") or "")
                        rinp = upd.get("rawInput") if isinstance(upd.get("rawInput"), dict) else {}
                        if nm == "read_file":
                            g_stats["file_reads"] += 1
                            fp = str(rinp.get("file_path") or rinp.get("path") or "")
                            if fp:
                                seen_reads[fp] += 1
                                if seen_reads[fp] >= 2:
                                    g_stats["redundant_same_file_rereads"] += 1
                        elif nm == "run_terminal_command":
                            for tg in _classify_cmd(str(rinp.get("command") or "")):
                                g_stats[tg] += 1
                    elif su == "tool_call_update" and upd.get("status") in ("completed", "failed"):
                        raw_out = json.dumps(upd.get("rawOutput") or "", ensure_ascii=False)
                        if len(raw_out) >= 8192:
                            g_stats["outputs_over_8kb"] += 1
                        for ek in _classify_err(raw_out):
                            g_stats[ek] += 1
        if files:
            harnesses["grok_cli"] = {"sessions": len(files), **dict(g_stats)}
            totals.update(g_stats)

    # Recommend exact primitives triggered by observed waste
    recs: list[dict[str, str]] = []
    if totals["adhoc_inline_python"] + totals["import_syspath_error"] + totals["shell_quoting_collision"] > 0:
        recs.append({
            "primitive": "probe",
            "evidence": f"{totals['adhoc_inline_python']} inline `python -c` probes, {totals['import_syspath_error']} ImportError/sys.path failures, {totals['shell_quoting_collision']} shell-quoting crashes",
            "why": "Replaces brittle inline probes & 30k-token prose mental execution with <0.1s reachability slicing (`slice`), heap/mutation tracing (`run --stdin`), and differential testing (`diff`).",
        })
    if totals["redundant_same_file_rereads"] + totals["shell_line_slice_read"] + totals["patch_target_mismatch"] + totals["encoding_utf8_crash"] > 0:
        recs.append({
            "primitive": "splice",
            "evidence": f"{totals['redundant_same_file_rereads']} redundant file re-reads, {totals['shell_line_slice_read']} shell line-slice reads, {totals['patch_target_mismatch']} patch mismatches, {totals['encoding_utf8_crash']} UTF-8/encoding crashes",
            "why": "Extracts exact AST methods/Markdown sections (`view`) and applies UTF-8/CRLF-safe syntax-verified atomic transactions (`apply`).",
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

    return {
        "elapsed_ms": round((time.perf_counter() - t0) * 1000.0, 2),
        "harnesses_detected": harnesses,
        "aggregate_friction_counts": dict(totals),
        "recommended_primitives": recs,
    }


def main(argv: list[str] | None = None) -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    parser = argparse.ArgumentParser(prog="audit_sessions.py", description="Local zero-cloud agent session telemetry auditor")
    parser.add_argument("--home", default=str(Path.home()), help="Home directory containing .codex / .claude / .gemini")
    parser.add_argument("--max-sessions", type=int, default=50, help="Max recent sessions per harness (default: 50)")
    parser.add_argument("--json", action="store_true", help="Emit JSON output")
    args = parser.parse_args(argv)

    rep = audit_home(Path(args.home), max_per_harness=args.max_sessions)
    if args.json:
        sys.stdout.write(json.dumps(rep, indent=2, ensure_ascii=False) + "\n")
        return 0

    lines = [f"[agent-primitives audit] Scanned {len(rep['harnesses_detected'])} local agent harness(es) in {rep['elapsed_ms']} ms (100% local, 0 bytes uploaded)"]
    for hname, hdata in rep["harnesses_detected"].items():
        lines.append(f"  - {hname}: {hdata['sessions']} sessions, {hdata.get('tool_calls', 0)} tool calls")
    if rep["recommended_primitives"]:
        lines.append("\n## Empirically Triggered Primitives For Your Machine")
        for r in rep["recommended_primitives"]:
            lines.append(f"- **`skills/{r['primitive']}`** — Observed: {r['evidence']}")
            lines.append(f"  Fix: {r['why']}")
        names_csv = ",".join(
            p.strip()
            for r in rep["recommended_primitives"]
            for p in r["primitive"].split("+")
        )
        lines.append(f"\nInstall only what your telemetry needs:\n  python install.py --agent agents --only {names_csv}")
    else:
        lines.append("\nNo local agent session logs found yet. Install starter pack: `python install.py --agent agents`")
    sys.stdout.write("\n".join(lines) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
