#!/usr/bin/env python3
"""guard.py - Exit-2 truth gate, repetition-loop breaker & blast-radius shield.

Returns exit code 2 with actionable diagnostics on stderr whenever:
1. A destructive command or secret/credential leak is detected (`shield`);
2. An agent repeats the same failing command/error signature >= N times (`run` / `record`);
3. A completion invariant fails (merge conflict markers, staged secrets, or `--check` cmd) (`stop-gate`).
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
import time
from pathlib import Path
from typing import Any

EXIT_BLOCK = 2

DESTRUCTIVE_PATTERNS: list[tuple[str, re.Pattern[str]]] = [
    ("recursive root/home delete", re.compile(r"\brm\s+-[a-zA-Z]*r[a-zA-Z]*f[a-zA-Z]*\s+(?:/|~|\$HOME|\.\.)(?:\s|$)")),
    ("forced git history overwrite", re.compile(r"\bgit\s+push\b.*(?:\s--force\b|\s-f\b)")),
    ("hard working-tree wipe", re.compile(r"\bgit\s+(?:reset\s+--hard|clean\s+-[a-zA-Z]*f)")),
    ("destructive SQL DDL", re.compile(r"\b(?:DROP\s+TABLE|DROP\s+DATABASE|TRUNCATE\s+TABLE)\b", re.IGNORECASE)),
    ("unbounded outbound mail/webhook blast", re.compile(r"\b(?:sendmail\s+-t|mass[_-]?send|blast[_-]?all)\b", re.IGNORECASE)),
]

SECRET_PATTERNS: list[tuple[str, re.Pattern[str]]] = [
    ("OpenAI/Anthropic/xAI API key", re.compile(r"\b(?:sk-(?:proj-|ant-)?[A-Za-z0-9_-]{20,}|xai-[A-Za-z0-9_-]{20,})\b")),
    ("GitHub Personal/App token", re.compile(r"\b(?:ghp|gho|ghu|ghs|github_pat)_[A-Za-z0-9_]{24,}\b")),
    ("AWS Access Key ID", re.compile(r"\bAKIA[0-9A-Z]{16}\b")),
    ("PEM Private Key block", re.compile(r"-----BEGIN (?:RSA |EC |OPENSSH |)PRIVATE KEY-----")),
]

CONFLICT_RE = re.compile(r"^(?:<{7}\s|={7}$|>{7}\s)", re.MULTILINE)


def _guard_dir() -> Path:
    env = os.environ.get("GUARD_HOME")
    base = Path(env) if env else Path.cwd() / ".guard"
    base.mkdir(parents=True, exist_ok=True)
    gitignore = base / ".gitignore"
    if not gitignore.exists():
        gitignore.write_text("*\n", encoding="utf-8")
    return base


def inspect_text_safety(text: str) -> list[str]:
    findings: list[str] = []
    for label, pat in DESTRUCTIVE_PATTERNS:
        if pat.search(text):
            findings.append(f"Destructive action blocked: {label}")
    for label, pat in SECRET_PATTERNS:
        if pat.search(text):
            findings.append(f"Credential/secret leak blocked: {label}")
    return findings


def _normalize_error_sig(cmd_str: str, output: str, exit_code: int) -> str:
    norm_cmd = " ".join(cmd_str.strip().split())
    err_lines = [
        re.sub(r"\b0x[0-9a-fA-F]+\b|\b\d{4}-\d{2}-\d{2}[T\s]\S+", "<HEX>", ln.strip())
        for ln in output.splitlines()
        if ln.strip()
    ]
    tail_sig = "\n".join(err_lines[-6:])[:400]
    raw = f"{exit_code}::{norm_cmd}::{tail_sig}"
    return hashlib.sha256(raw.encode("utf-8", errors="replace")).hexdigest()[:12]


def _load_ledger(base: Path) -> list[dict[str, Any]]:
    p = base / "loop_ledger.json"
    if not p.exists():
        return []
    try:
        return json.loads(p.read_text(encoding="utf-8"))
    except Exception:
        return []


def _save_ledger(base: Path, entries: list[dict[str, Any]]) -> None:
    (base / "loop_ledger.json").write_text(
        json.dumps(entries[-40:], ensure_ascii=False, indent=2), encoding="utf-8"
    )


def record_attempt(cmd_str: str, exit_code: int, output: str, max_repeats: int = 3) -> tuple[bool, int, str]:
    base = _guard_dir()
    entries = _load_ledger(base)
    if exit_code == 0:
        entries.append({"sig": "ok", "cmd": cmd_str[:160], "exit": 0, "at": int(time.time())})
        _save_ledger(base, entries)
        return False, 0, "ok"

    sig = _normalize_error_sig(cmd_str, output, exit_code)
    entries.append({"sig": sig, "cmd": cmd_str[:160], "exit": exit_code, "at": int(time.time())})
    _save_ledger(base, entries)

    # Count trailing consecutive or recent window occurrences of the exact failure signature
    recent = entries[-8:]
    repeat_count = sum(1 for e in recent if e.get("sig") == sig)
    return repeat_count >= max_repeats, repeat_count, sig


def _scan_conflict_markers(root: Path) -> list[str]:
    bad_files: list[str] = []
    try:
        proc = subprocess.run(
            ["git", "status", "--porcelain"],
            cwd=str(root),
            capture_output=True,
            text=True,
            timeout=3,
            encoding="utf-8",
            errors="replace",
        )
        candidates = []
        if proc.returncode == 0:
            for ln in proc.stdout.splitlines():
                if len(ln) > 3:
                    candidates.append(root / ln[3:].strip().strip('"'))
        else:
            candidates = [p for p in root.rglob("*") if p.is_file() and p.stat().st_size < 512_000][:150]
        for fp in candidates:
            if fp.is_file() and fp.stat().st_size < 1_000_000:
                txt = fp.read_text(encoding="utf-8", errors="ignore")
                if CONFLICT_RE.search(txt):
                    bad_files.append(str(fp.relative_to(root)))
                for _, spat in SECRET_PATTERNS:
                    if spat.search(txt):
                        bad_files.append(f"{fp.relative_to(root)} (contains exposed secret pattern)")
                        break
    except Exception:
        pass
    return bad_files


def main(argv: list[str] | None = None) -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
        sys.stderr.reconfigure(encoding="utf-8", errors="replace")

    parser = argparse.ArgumentParser(prog="guard", description="Exit-2 truth gate, loop breaker & shield")
    sub = parser.add_subparsers(dest="subcmd", required=True)

    p_shield = sub.add_parser("shield", help="Inspect command/payload or Hook stdin JSON for unsafe actions & secrets")
    p_shield.add_argument("--text", default="", help="Command or outbound payload text (reads stdin if omitted)")

    p_run = sub.add_parser("run", help="Execute command with shield + repetition-loop breaker")
    p_run.add_argument("--max-repeats", type=int, default=3, help="Max identical failures before exit-2 trip")
    p_run.add_argument("cmd", nargs=argparse.REMAINDER, help="Command after '--'")

    p_gate = sub.add_parser("stop-gate", help="Verify completion invariants; exit 2 on stderr if unverified")
    p_gate.add_argument("--check", action="append", default=[], help="Shell verification command (repeatable)")

    sub.add_parser("reset", help="Clear loop ledger")

    args = parser.parse_args(argv)

    if args.subcmd == "reset":
        _save_ledger(_guard_dir(), [])
        sys.stdout.write("[guard] loop ledger reset.\n")
        return 0

    if args.subcmd == "shield":
        payload = args.text
        if not payload and not sys.stdin.isatty():
            payload = sys.stdin.read()
        findings = inspect_text_safety(payload)
        if findings:
            for f in findings:
                sys.stderr.write(f"[GUARD BLOCKED | exit 2] {f}\n")
            sys.stderr.write("Action halted. Request explicit human confirmation or sanitize secrets first.\n")
            return EXIT_BLOCK
        sys.stdout.write("[guard] shield passed.\n")
        return 0

    if args.subcmd == "run":
        cmd = args.cmd[1:] if args.cmd and args.cmd[0] == "--" else args.cmd
        if not cmd:
            sys.stderr.write("[guard] usage: guard.py run -- <command...>\n")
            return EXIT_BLOCK
        cmd_str = " ".join(cmd)
        findings = inspect_text_safety(cmd_str)
        if findings:
            for f in findings:
                sys.stderr.write(f"[GUARD BLOCKED | exit 2] {f}\n")
            return EXIT_BLOCK

        # Check if this exact command already tripped the loop breaker in the immediate window
        prior = _load_ledger(_guard_dir())[-6:]
        prior_fails = [e for e in prior if e.get("cmd") == cmd_str[:160] and e.get("exit", 0) != 0]
        if len(prior_fails) >= args.max_repeats:
            sys.stderr.write(
                f"[GUARD LOOP BREAKER | exit 2] Command '{cmd_str}' already failed {len(prior_fails)}x in a row.\n"
                "STOP retrying the identical command. Diagnose root cause or record a dead-end in `baton` first.\n"
            )
            return EXIT_BLOCK

        proc = subprocess.run(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        out = proc.stdout or ""
        if out:
            sys.stdout.write(out)
        tripped, count, sig = record_attempt(cmd_str, proc.returncode, out, max_repeats=args.max_repeats)
        if tripped:
            sys.stderr.write(
                f"\n[GUARD LOOP BREAKER | exit 2] Failure signature {sig} repeated {count}x.\n"
                "Do NOT re-run the same command unchanged. Formulate a new hypothesis first.\n"
            )
            return EXIT_BLOCK
        return proc.returncode

    if args.subcmd == "stop-gate":
        issues = _scan_conflict_markers(Path.cwd())
        for chk in args.check:
            proc = subprocess.run(
                chk,
                shell=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
                encoding="utf-8",
                errors="replace",
            )
            if proc.returncode != 0:
                tail = "\n".join((proc.stdout or "").splitlines()[-12:])
                issues.append(f"Check failed (`{chk}`, exit {proc.returncode}):\n{tail}")
        if issues:
            sys.stderr.write("[GUARD STOP-GATE BLOCKED | exit 2] Cannot mark task complete:\n")
            for item in issues:
                sys.stderr.write(f"- {item}\n")
            return EXIT_BLOCK
        sys.stdout.write("[guard] stop-gate verified clean.\n")
        return 0

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
