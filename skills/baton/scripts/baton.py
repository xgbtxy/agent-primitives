#!/usr/bin/env python3
"""baton.py - Compaction-proof state & cross-agent handoff capsule.

Zero external dependencies (Python 3.9+ stdlib only). Zero cloud, zero LLM tax.
Separates Objective, Constraints, Verified Facts, Falsified Dead-Ends ("DO NOT
RETRY"), Next Action, and Workspace Snapshot into a strict <500-token capsule.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path
from typing import Any


def _baton_dir() -> Path:
    env = os.environ.get("BATON_HOME")
    base = Path(env) if env else Path.cwd() / ".baton"
    base.mkdir(parents=True, exist_ok=True)
    (base / "history").mkdir(exist_ok=True)
    gitignore = base / ".gitignore"
    if not gitignore.exists():
        gitignore.write_text("*\n", encoding="utf-8")
    return base


def _now_iso() -> str:
    return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())


def _default_state() -> dict[str, Any]:
    return {
        "goal": "",
        "constraints": [],
        "facts": [],
        "deadends": [],
        "next_step": "",
        "updated_at": _now_iso(),
        "workspace": {},
    }


def _load_state(base: Path) -> dict[str, Any]:
    p = base / "state.json"
    if not p.exists():
        return _default_state()
    try:
        return json.loads(p.read_text(encoding="utf-8"))
    except Exception:
        return _default_state()


def _probe_workspace() -> dict[str, Any]:
    info: dict[str, Any] = {"cwd": str(Path.cwd())}
    try:
        branch = subprocess.run(
            ["git", "rev-parse", "--abbrev-ref", "HEAD"],
            capture_output=True,
            text=True,
            timeout=2,
            encoding="utf-8",
            errors="replace",
        )
        if branch.returncode == 0:
            info["git_branch"] = branch.stdout.strip()
            status = subprocess.run(
                ["git", "status", "--short"],
                capture_output=True,
                text=True,
                timeout=2,
                encoding="utf-8",
                errors="replace",
            )
            dirty = [ln.strip() for ln in status.stdout.splitlines() if ln.strip()]
            info["dirty_count"] = len(dirty)
            info["dirty_files"] = dirty[:10]
            return info
    except Exception:
        pass
    # Personal-assistant / non-git folder fallback: top 8 most recently modified files
    files = [
        p for p in Path.cwd().iterdir()
        if p.is_file() and not p.name.startswith(".")
    ]
    files.sort(key=lambda p: p.stat().st_mtime, reverse=True)
    info["recent_files"] = [p.name for p in files[:8]]
    return info


def _render_markdown(state: dict[str, Any]) -> str:
    lines: list[str] = [f"# BATON HANDOFF ({state.get('updated_at', '')})"]
    if state.get("goal"):
        lines.append(f"**Goal**: {state['goal']}")
    if state.get("constraints"):
        lines.append("**Constraints**: " + " | ".join(state["constraints"][-6:]))
    if state.get("next_step"):
        lines.append(f"**Next Immediate Step**: {state['next_step']}")

    ws = state.get("workspace") or {}
    if ws.get("git_branch"):
        dirty = ws.get("dirty_files") or []
        lines.append(
            f"**Git**: `{ws['git_branch']}` ({ws.get('dirty_count', 0)} modified"
            + (f": {', '.join(dirty[:5])}" if dirty else "")
            + ")"
        )
    elif ws.get("recent_files"):
        lines.append(f"**Recent Files**: {', '.join(ws['recent_files'][:6])}")

    facts = state.get("facts") or []
    if facts:
        lines.append("\n## Verified Facts")
        for f in facts[-8:]:
            ref = f" (`{f['ref']}`)" if f.get("ref") else ""
            lines.append(f"- {f['text']}{ref}")

    deadends = state.get("deadends") or []
    if deadends:
        lines.append("\n## Dead-Ends (DO NOT RETRY)")
        for d in deadends[-8:]:
            why = f" — {d['reason']}" if d.get("reason") else ""
            lines.append(f"- [x] {d['text']}{why}")

    return "\n".join(lines).strip() + "\n"


def _save_state(base: Path, state: dict[str, Any], snapshot_history: bool = False) -> str:
    state["updated_at"] = _now_iso()
    state["workspace"] = _probe_workspace()
    (base / "state.json").write_text(json.dumps(state, ensure_ascii=False, indent=2), encoding="utf-8")
    md = _render_markdown(state)
    (base / "HANDOFF.md").write_text(md, encoding="utf-8")
    if snapshot_history:
        stamp = time.strftime("%Y%m%d-%H%M%S", time.gmtime())
        (base / "history" / f"baton-{stamp}.json").write_text(
            json.dumps(state, ensure_ascii=False, indent=2), encoding="utf-8"
        )
        snaps = sorted((base / "history").glob("baton-*.json"))
        for old in snaps[:-20]:
            old.unlink(missing_ok=True)
    return md


def main(argv: list[str] | None = None) -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
        sys.stderr.reconfigure(encoding="utf-8", errors="replace")

    parser = argparse.ArgumentParser(prog="baton", description="Compaction-proof state & cross-agent handoff capsule")
    sub = parser.add_subparsers(dest="subcmd", required=True)

    p_set = sub.add_parser("set", help="Set goal, constraints, or next step")
    p_set.add_argument("--goal", help="Primary task objective")
    p_set.add_argument("--constraint", action="append", help="Hard constraint (repeatable)")
    p_set.add_argument("--next", dest="next_step", help="Single immediate next step")
    p_set.add_argument("--reset", action="store_true", help="Clear prior state first")

    p_fact = sub.add_parser("fact", help="Append a verified ground-truth fact")
    p_fact.add_argument("text", help="Concise verified fact")
    p_fact.add_argument("--ref", default="", help="File:line, commit, or URL evidence")

    p_dead = sub.add_parser("deadend", help="Record a falsified approach so future turns never repeat it")
    p_dead.add_argument("text", help="Approach/command that failed")
    p_dead.add_argument("--reason", default="", help="Root reason it failed")

    p_chk = sub.add_parser("checkpoint", help="Snapshot workspace & write history (ideal for PreCompact/Stop hooks)")
    p_chk.add_argument("--note", default="", help="Optional checkpoint note")

    p_res = sub.add_parser("resume", help="Emit compact <500-token handoff packet")
    p_res.add_argument("--json", action="store_true", help="Output JSON instead of Markdown")

    sub.add_parser("clear", help="Reset active baton capsule")

    args = parser.parse_args(argv)
    base = _baton_dir()

    if args.subcmd == "clear":
        _save_state(base, _default_state(), snapshot_history=True)
        sys.stdout.write("[baton] active state cleared (previous snapshot archived).\n")
        return 0

    state = _default_state() if getattr(args, "reset", False) else _load_state(base)

    if args.subcmd == "set":
        if args.goal is not None:
            state["goal"] = args.goal.strip()[:240]
        if args.constraint:
            for c in args.constraint:
                c_clean = c.strip()[:160]
                if c_clean and c_clean not in state["constraints"]:
                    state["constraints"].append(c_clean)
        if args.next_step is not None:
            state["next_step"] = args.next_step.strip()[:240]
        md = _save_state(base, state)
        tok = max(1, (len(md) + 3) // 4)
        sys.stdout.write(f"[baton] updated (~{tok} tokens)\n")
        return 0

    if args.subcmd == "fact":
        entry = {"text": args.text.strip()[:220], "ref": args.ref.strip()[:120], "at": _now_iso()}
        if not any(f["text"] == entry["text"] for f in state["facts"]):
            state["facts"].append(entry)
            state["facts"] = state["facts"][-12:]
        md = _save_state(base, state)
        sys.stdout.write(f"[baton] recorded fact ({len(state['facts'])} active, ~{(len(md)+3)//4} tok)\n")
        return 0

    if args.subcmd == "deadend":
        entry = {"text": args.text.strip()[:220], "reason": args.reason.strip()[:160], "at": _now_iso()}
        if not any(d["text"] == entry["text"] for d in state["deadends"]):
            state["deadends"].append(entry)
            state["deadends"] = state["deadends"][-12:]
        md = _save_state(base, state)
        sys.stdout.write(f"[baton] recorded dead-end ({len(state['deadends'])} active, ~{(len(md)+3)//4} tok)\n")
        return 0

    if args.subcmd == "checkpoint":
        if args.note:
            state["next_step"] = args.note.strip()[:240]
        md = _save_state(base, state, snapshot_history=True)
        sys.stdout.write(f"[baton] checkpoint saved (~{(len(md)+3)//4} tokens)\n")
        return 0

    if args.subcmd == "resume":
        state["workspace"] = _probe_workspace()
        if args.json:
            sys.stdout.write(json.dumps(state, ensure_ascii=False, indent=2) + "\n")
        else:
            sys.stdout.write(_render_markdown(state))
        return 0

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
