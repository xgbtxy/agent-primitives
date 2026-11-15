#!/usr/bin/env python3
"""sieve.py - Lossless output compressor & flight recorder for AI agents.

Zero external dependencies (Python 3.9+ stdlib only). Cross-platform UTF-8 safe.
Compresses noisy CLI logs, stack traces, JSON payloads, and HTML pages into a
compact deterministic digest while storing the exact raw stream in a bounded
local ring buffer for sub-10ms lossless slice retrieval (`peek`).
"""

from __future__ import annotations

import argparse
import hashlib
import html.parser
import json
import os
import re
import subprocess
import sys
import time
from pathlib import Path
from typing import Any

DEFAULT_MAX_STORE_BYTES = 50 * 1024 * 1024  # 50 MB ring buffer cap
PASSTHROUGH_BYTES = 1200  # Below this size, emit raw output directly (honest behavior)
ERROR_RE = re.compile(
    r"(traceback \(most recent call last\)|^\s*file \".*\", line \d+|"
    r"\b(?:error|exception|failed|failure|fatal|panic|assertionerror|typeerror|"
    r"valueerror|keyerror|syntaxerror|segfault)\b|^E\s+|^FAIL[:\s]|^ERROR[:\s])",
    re.IGNORECASE,
)


def _store_dir() -> Path:
    env = os.environ.get("SIEVE_HOME")
    base = Path(env) if env else Path.cwd() / ".sieve"
    base.mkdir(parents=True, exist_ok=True)
    gitignore = base / ".gitignore"
    if not gitignore.exists():
        gitignore.write_text("*\n", encoding="utf-8")
    return base


def _est_tokens(text: str) -> int:
    # Conservative UTF-8 byte/char hybrid heuristic (~4 chars per token on English/code/JSON)
    return max(1, (len(text) + 3) // 4) if text else 0


def _prune_ring_buffer(store: Path, max_bytes: int = DEFAULT_MAX_STORE_BYTES) -> None:
    raw_files = sorted(store.glob("*.raw"), key=lambda p: p.stat().st_mtime)
    total = sum(p.stat().st_size for p in raw_files)
    while total > max_bytes and len(raw_files) > 1:
        victim = raw_files.pop(0)
        try:
            sz = victim.stat().st_size
            victim.unlink(missing_ok=True)
            victim.with_suffix(".meta.json").unlink(missing_ok=True)
            total -= sz
        except OSError:
            break


def _archive(raw_text: str, label: str, exit_code: int, elapsed_ms: float) -> str:
    store = _store_dir()
    digest_hash = hashlib.sha256(raw_text.encode("utf-8", errors="replace")).hexdigest()[:10]
    raw_path = store / f"{digest_hash}.raw"
    meta_path = store / f"{digest_hash}.meta.json"
    raw_path.write_text(raw_text, encoding="utf-8", errors="replace")
    meta = {
        "id": digest_hash,
        "label": label,
        "exit_code": exit_code,
        "elapsed_ms": round(elapsed_ms, 1),
        "raw_bytes": len(raw_text.encode("utf-8", errors="replace")),
        "raw_lines": raw_text.count("\n") + (1 if raw_text and not raw_text.endswith("\n") else 0),
        "created_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    }
    meta_path.write_text(json.dumps(meta, indent=2), encoding="utf-8")
    _prune_ring_buffer(store)
    return digest_hash


class _HTMLStripper(html.parser.HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self._skip_depth = 0
        self._in_title = False
        self._heading_tag: str | None = None
        self.title: list[str] = []
        self.headings: list[str] = []
        self._cur_heading: list[str] = []
        self.text_chunks: list[str] = []
        self.link_count = 0

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        t = tag.lower()
        if t in ("script", "style", "svg", "noscript"):
            self._skip_depth += 1
        elif t == "title":
            self._in_title = True
        elif t in ("h1", "h2", "h3"):
            self._heading_tag = t.upper()
            self._cur_heading = []
        elif t == "a":
            self.link_count += 1

    def handle_endtag(self, tag: str) -> None:
        t = tag.lower()
        if t in ("script", "style", "svg", "noscript") and self._skip_depth > 0:
            self._skip_depth -= 1
        elif t == "title":
            self._in_title = False
        elif t in ("h1", "h2", "h3") and self._heading_tag:
            h = " ".join("".join(self._cur_heading).split())
            if h:
                self.headings.append(f"{self._heading_tag}: {h[:100]}")
            self._heading_tag = None

    def handle_data(self, data: str) -> None:
        if self._skip_depth > 0:
            return
        if self._in_title:
            self.title.append(data)
        if self._heading_tag:
            self._cur_heading.append(data)
        cleaned = " ".join(data.split())
        if cleaned:
            self.text_chunks.append(cleaned)


def _summarize_json(data: Any) -> str:
    lines: list[str] = []
    if isinstance(data, dict):
        keys = list(data.keys())
        lines.append(f"JSON Object ({len(keys)} top-level keys): {', '.join(keys[:15])}")
        for k in keys[:10]:
            v = data[k]
            if isinstance(v, list):
                sample_keys = (
                    list(v[0].keys())[:8] if v and isinstance(v[0], dict) else type(v[0]).__name__ if v else "empty"
                )
                lines.append(f"  - {k}: Array[{len(v)}] (item schema: {sample_keys})")
            elif isinstance(v, dict):
                lines.append(f"  - {k}: Object({len(v)} keys: {', '.join(list(v.keys())[:8])})")
            else:
                val_s = repr(v)
                lines.append(f"  - {k}: {val_s[:100]}")
        for alert_k in ("error", "errors", "message", "status", "detail", "warnings"):
            if alert_k in data and data[alert_k]:
                lines.append(f"  ! {alert_k}: { json.dumps(data[alert_k], ensure_ascii=False)[:240] }")
    elif isinstance(data, list):
        sample = list(data[0].keys())[:10] if data and isinstance(data[0], dict) else (type(data[0]).__name__ if data else "empty")
        lines.append(f"JSON Array ({len(data)} items, item schema: {sample})")
        if data:
            lines.append(f"  - [0] preview: {json.dumps(data[0], ensure_ascii=False)[:260]}")
    else:
        lines.append(f"JSON Scalar: {repr(data)[:200]}")
    return "\n".join(lines)


def _summarize_html(raw: str) -> str:
    parser = _HTMLStripper()
    try:
        parser.feed(raw)
    except Exception:
        return _summarize_log(raw)
    title = " ".join("".join(parser.title).split())[:140] or "(untitled)"
    body = " ".join(parser.text_chunks)
    out = [f"HTML Document | Title: {title} | Links: {parser.link_count}"]
    if parser.headings:
        out.append("Outline:")
        for h in parser.headings[:12]:
            out.append(f"  - {h}")
    out.append(f"Lead text: {body[:500]}{'...' if len(body) > 500 else ''}")
    return "\n".join(out)


def _summarize_log(raw: str) -> str:
    lines = raw.splitlines()
    n = len(lines)
    # 1. Collapse consecutive identical / near-identical timestamp-stripped lines
    norm_counts: dict[str, int] = {}
    error_windows: list[tuple[int, int]] = []
    for idx, ln in enumerate(lines):
        norm = re.sub(r"\b\d{2,4}[-/:]\d{2}[-/:]\d{2}[T\s]\d{2}:\d{2}:\d{2}(?:\.\d+)?Z?\b", "<TS>", ln).strip()
        if norm:
            norm_counts[norm] = norm_counts.get(norm, 0) + 1
        if ERROR_RE.search(ln):
            start = max(0, idx - 1)
            end = min(n - 1, idx + 3)
            if error_windows and start <= error_windows[-1][1] + 2:
                error_windows[-1] = (error_windows[-1][0], max(error_windows[-1][1], end))
            else:
                error_windows.append((start, end))

    sections: list[str] = []
    # Head anchor (first 4 non-empty lines)
    head = [f"L{i+1}: {lines[i][:140]}" for i in range(min(5, n)) if lines[i].strip()][:4]
    if head:
        sections.append("Head:\n  " + "\n  ".join(head))

    # Deduplicated error blocks (up to 4 distinct blocks)
    if error_windows:
        seen_sigs: set[str] = set()
        kept_blocks: list[str] = []
        for s, e in error_windows:
            sig = "\n".join(re.sub(r"\d+", "#", lines[k])[:80] for k in range(s, min(e + 1, s + 3)))
            if sig in seen_sigs:
                continue
            seen_sigs.add(sig)
            snippet = "\n    ".join(f"L{k+1}: {lines[k][:160]}" for k in range(s, e + 1))
            kept_blocks.append(f"  [L{s+1}-L{e+1}]\n    {snippet}")
            if len(kept_blocks) >= 4:
                break
        omitted = max(0, len(error_windows) - len(kept_blocks))
        hdr = f"High-Signal Failure/Error Blocks ({len(error_windows)} total" + (
            f", {omitted} similar omitted):" if omitted else "):"
        )
        sections.append(hdr + "\n" + "\n".join(kept_blocks))

    # Top repeated noise lines
    noisy = sorted(((cnt, txt) for txt, cnt in norm_counts.items() if cnt >= 5), reverse=True)[:3]
    if noisy:
        sections.append(
            "Collapsed Repetitions:\n  " + "\n  ".join(f"[x{cnt}] {txt[:110]}" for cnt, txt in noisy)
        )

    # Tail anchor (last 5 lines)
    if n > 6:
        tail_start = max(5, n - 5)
        tail = [f"L{i+1}: {lines[i][:140]}" for i in range(tail_start, n) if lines[i].strip()]
        if tail:
            sections.append("Tail:\n  " + "\n  ".join(tail))
    return "\n".join(sections)


def compress_stream(raw_text: str, label: str = "stream", exit_code: int = 0, elapsed_ms: float = 0.0) -> dict[str, Any]:
    raw_bytes = len(raw_text.encode("utf-8", errors="replace"))
    item_id = _archive(raw_text, label=label, exit_code=exit_code, elapsed_ms=elapsed_ms)

    if raw_bytes <= PASSTHROUGH_BYTES:
        rendered = (
            f"{raw_text.rstrip()}\n"
            f"[sieve id={item_id} | exit={exit_code} | {raw_bytes}B passthrough]"
        )
        return {
            "id": item_id,
            "mode": "passthrough",
            "exit_code": exit_code,
            "raw_bytes": raw_bytes,
            "digest_bytes": len(rendered.encode("utf-8")),
            "raw_tokens_est": _est_tokens(raw_text),
            "digest_tokens_est": _est_tokens(rendered),
            "rendered": rendered,
        }

    stripped = raw_text.lstrip()
    body_summary: str
    kind = "log"
    if stripped.startswith(("{", "[")):
        try:
            parsed = json.loads(stripped)
            body_summary = _summarize_json(parsed)
            kind = "json"
        except Exception:
            body_summary = _summarize_log(raw_text)
    elif stripped[:200].lower().startswith("<!doctype html") or "<html" in stripped[:400].lower():
        body_summary = _summarize_html(raw_text)
        kind = "html"
    else:
        body_summary = _summarize_log(raw_text)

    raw_tok = _est_tokens(raw_text)
    footer = (
        f"[sieve id={item_id} | kind={kind} | exit={exit_code} | "
        f"raw={raw_bytes:,}B (~{raw_tok:,} tok) | "
        f"peek: python sieve.py peek {item_id} --lines A:B | --grep REGEX]"
    )
    rendered = f"{body_summary}\n{footer}"
    dig_bytes = len(rendered.encode("utf-8"))
    dig_tok = _est_tokens(rendered)
    saved_pct = round((1.0 - dig_bytes / max(1, raw_bytes)) * 100.0, 1)
    return {
        "id": item_id,
        "mode": kind,
        "exit_code": exit_code,
        "raw_bytes": raw_bytes,
        "digest_bytes": dig_bytes,
        "saved_pct": saved_pct,
        "raw_tokens_est": raw_tok,
        "digest_tokens_est": dig_tok,
        "rendered": rendered,
    }


def _resolve_json_path(obj: Any, path: str) -> Any:
    cur = obj
    for part in [p for p in path.split(".") if p]:
        if isinstance(cur, list):
            cur = cur[int(part)]
        elif isinstance(cur, dict):
            cur = cur[part]
        else:
            raise KeyError(part)
    return cur


def cmd_peek(args: argparse.Namespace) -> int:
    store = _store_dir()
    matches = list(store.glob(f"{args.id}*.raw"))
    if not matches:
        sys.stderr.write(f"[sieve] error: unknown archive id '{args.id}' in {store}\n")
        return 1
    raw = matches[0].read_text(encoding="utf-8", errors="replace")

    if args.json_path:
        data = json.loads(raw)
        sub = _resolve_json_path(data, args.json_path)
        sys.stdout.write(json.dumps(sub, indent=2, ensure_ascii=False)[:8000] + "\n")
        return 0

    lines = raw.splitlines()
    if args.lines:
        parts = args.lines.split(":")
        start = max(1, int(parts[0]))
        end = min(len(lines), int(parts[1]) if len(parts) > 1 and parts[1] else start + 40)
        for idx in range(start - 1, end):
            sys.stdout.write(f"L{idx+1}: {lines[idx]}\n")
        return 0

    if args.grep:
        pat = re.compile(args.grep, re.IGNORECASE)
        ctx = args.context
        shown: set[int] = set()
        hits = 0
        for i, ln in enumerate(lines):
            if pat.search(ln):
                hits += 1
                for j in range(max(0, i - ctx), min(len(lines), i + ctx + 1)):
                    if j not in shown:
                        shown.add(j)
                        marker = ">" if j == i else " "
                        sys.stdout.write(f"{marker}L{j+1}: {lines[j]}\n")
                if hits >= args.max_hits:
                    sys.stdout.write(f"... (capped at {args.max_hits} matches)\n")
                    break
        if hits == 0:
            sys.stdout.write(f"[sieve] 0 matches for /{args.grep}/ in {len(lines)} lines.\n")
        return 0

    # Default preview: first 40 lines
    for idx in range(min(40, len(lines))):
        sys.stdout.write(f"L{idx+1}: {lines[idx]}\n")
    return 0


def main(argv: list[str] | None = None) -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
        sys.stderr.reconfigure(encoding="utf-8", errors="replace")

    parser = argparse.ArgumentParser(prog="sieve", description="Lossless output compressor & flight recorder")
    sub = parser.add_subparsers(dest="subcmd", required=True)

    p_run = sub.add_parser("run", help="Run a command and compress its output losslessly")
    p_run.add_argument("--json", action="store_true", help="Emit structured JSON metadata")
    p_run.add_argument("cmd", nargs=argparse.REMAINDER, help="Command after '--'")

    p_pipe = sub.add_parser("pipe", help="Read stdin and compress losslessly")
    p_pipe.add_argument("--label", default="stdin", help="Label for archived stream")
    p_pipe.add_argument("--json", action="store_true", help="Emit structured JSON metadata")

    p_peek = sub.add_parser("peek", help="Retrieve exact raw slices by id")
    p_peek.add_argument("id", help="Archive id prefix")
    p_peek.add_argument("--lines", help="1-based inclusive range START:END (e.g. 120:160)")
    p_peek.add_argument("--grep", help="Regex pattern to search in raw archive")
    p_peek.add_argument("-C", "--context", type=int, default=3, help="Context lines around --grep")
    p_peek.add_argument("--max-hits", type=int, default=10, help="Max regex matches to expand")
    p_peek.add_argument("--json-path", help="Dot path for JSON archives (e.g. data.items.0)")

    args = parser.parse_args(argv)

    if args.subcmd == "run":
        cmd = args.cmd[1:] if args.cmd and args.cmd[0] == "--" else args.cmd
        if not cmd:
            sys.stderr.write("[sieve] usage: sieve.py run -- <command...>\n")
            return 2
        t0 = time.perf_counter()
        proc = subprocess.run(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        elapsed_ms = (time.perf_counter() - t0) * 1000.0
        res = compress_stream(proc.stdout or "", label=" ".join(cmd[:3]), exit_code=proc.returncode, elapsed_ms=elapsed_ms)
        if args.json:
            sys.stdout.write(json.dumps(res, ensure_ascii=False, indent=2) + "\n")
        else:
            sys.stdout.write(res["rendered"] + "\n")
        return proc.returncode

    if args.subcmd == "pipe":
        raw = sys.stdin.read()
        res = compress_stream(raw, label=args.label, exit_code=0)
        if args.json:
            sys.stdout.write(json.dumps(res, ensure_ascii=False, indent=2) + "\n")
        else:
            sys.stdout.write(res["rendered"] + "\n")
        return 0

    if args.subcmd == "peek":
        return cmd_peek(args)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
