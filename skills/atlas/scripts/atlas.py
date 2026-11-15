#!/usr/bin/env python3
"""atlas.py - Sub-second zero-read structural map for codebases & personal vaults.

Zero external dependencies (Python 3.9+ stdlib only). Scans code repositories
(Python AST + TS/JS/Go/Rust/C symbol regex + import centrality + stale TODO
trap radar) and personal document vaults (Markdown heading outlines + active
notes) into a strict <1,000-token structural briefing.
"""

from __future__ import annotations

import argparse
import ast
import json
import os
import re
import sys
import time
from pathlib import Path
from typing import Any

SKIP_DIRS = {
    ".git", ".hg", ".svn", "node_modules", "__pycache__", ".venv", "venv",
    "env", "dist", "build", "target", ".tox", ".mypy_cache", ".pytest_cache",
    ".ruff_cache", ".next", ".nuxt", ".sieve", ".baton", ".recall", ".guard",
    ".probe", ".splice", ".agents", ".claude", ".grok", ".codex", ".cursor",
    ".openclaw", ".hermes", "vendor", "coverage", ".idea", ".vscode",
}

CODE_EXTS = {".py", ".ts", ".tsx", ".js", ".jsx", ".go", ".rs", ".c", ".cc", ".cpp", ".h", ".java", ".rb", ".sh"}
DOC_EXTS = {".md", ".mdx", ".rst", ".txt"}
ENTRYPOINT_NAMES = {
    "main.py", "app.py", "cli.py", "__main__.py", "server.py", "manage.py",
    "index.ts", "index.js", "main.ts", "main.go", "main.rs", "lib.rs",
    "pyproject.toml", "package.json", "Cargo.toml", "go.mod", "Makefile",
}

SYM_RE = re.compile(
    r"^\s*(?:export\s+)?(?:pub\s+)?(?:async\s+)?"
    r"(?:def|class|function|interface|type|struct|enum|fn|func)\s+([A-Za-z_][A-Za-z0-9_]*)",
    re.MULTILINE,
)
MD_HEADING_RE = re.compile(r"^(#{1,2})\s+(.+?)\s*$", re.MULTILINE)
TODO_RE = re.compile(r"\b(TODO|FIXME|HACK|XXX)\b[:\s]*(.{0,90})")
IMPORT_PY_RE = re.compile(r"^\s*(?:from\s+([A-Za-z0-9_.]+)\s+import|import\s+([A-Za-z0-9_.]+))", re.MULTILINE)
IMPORT_JS_RE = re.compile(r"""(?:from|require\()\s*['"](\.[^'"]+)['"]""", re.MULTILINE)


def _extract_py_info(text: str) -> tuple[list[str], list[str]]:
    symbols: list[str] = []
    imports: list[str] = []
    try:
        tree = ast.parse(text)
        for node in tree.body:
            if isinstance(node, ast.ClassDef):
                methods = [n.name for n in node.body if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef)) and not n.name.startswith("_")]
                m_str = f"({','.join(methods[:4])})" if methods else ""
                symbols.append(f"class {node.name}{m_str}")
            elif isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                if not node.name.startswith("_") or node.name == "__main__":
                    symbols.append(f"def {node.name}()")
            elif isinstance(node, ast.ImportFrom) and node.module:
                imports.append(node.module)
            elif isinstance(node, ast.Import):
                for alias in node.names:
                    imports.append(alias.name)
    except SyntaxError:
        symbols = [m.group(1) for m in SYM_RE.finditer(text)][:8]
        imports = [m.group(1) or m.group(2) for m in IMPORT_PY_RE.finditer(text) if (m.group(1) or m.group(2))]
    return symbols[:8], imports


def scan_workspace(root: Path, focus: str = "", max_files: int = 18, stale_days: int = 90) -> dict[str, Any]:
    t0 = time.perf_counter()
    root = root.resolve()
    focus_l = focus.lower().strip()
    now = time.time()
    stale_cutoff = now - stale_days * 86400

    code_records: list[dict[str, Any]] = []
    doc_records: list[dict[str, Any]] = []
    entrypoints: list[str] = []
    stale_todos: list[str] = []
    dir_counts: dict[str, int] = {}
    inbound_refs: dict[str, int] = {}
    total_raw_bytes = 0
    total_files = 0

    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS and not d.startswith(".git")]
        rel_dir = Path(dirpath).relative_to(root)
        top_bucket = rel_dir.parts[0] if rel_dir.parts else "."

        for fname in filenames:
            if fname.startswith(".") and fname not in (".env.example",):
                continue
            fpath = Path(dirpath) / fname
            rel_str = str(fpath.relative_to(root)).replace("\\", "/")
            if focus_l and focus_l not in rel_str.lower():
                # Still count top-level layout, skip deep parse if focus is set
                dir_counts[top_bucket] = dir_counts.get(top_bucket, 0) + 1
                continue

            try:
                st = fpath.stat()
            except OSError:
                continue
            if st.st_size > 600_000:
                continue

            ext = fpath.suffix.lower()
            total_files += 1
            total_raw_bytes += st.st_size
            dir_counts[top_bucket] = dir_counts.get(top_bucket, 0) + 1

            if fname in ENTRYPOINT_NAMES:
                entrypoints.append(rel_str)

            if ext in CODE_EXTS:
                txt = fpath.read_text(encoding="utf-8", errors="ignore")
                if ext == ".py":
                    syms, imps = _extract_py_info(txt)
                else:
                    syms = [m.group(1) for m in SYM_RE.finditer(txt)][:8]
                    imps = [m.group(1) for m in IMPORT_JS_RE.finditer(txt)]
                for imp in imps:
                    stem = imp.split(".")[-1].split("/")[-1]
                    if stem:
                        inbound_refs[stem] = inbound_refs.get(stem, 0) + 1

                if st.st_mtime < stale_cutoff and len(stale_todos) < 6:
                    m_todo = TODO_RE.search(txt)
                    if m_todo:
                        age_d = int((now - st.st_mtime) // 86400)
                        stale_todos.append(f"`{rel_str}` ({age_d}d old {m_todo.group(1)}: {m_todo.group(2).strip()[:60]})")

                code_records.append(
                    {
                        "path": rel_str,
                        "stem": fpath.stem,
                        "lines": txt.count("\n") + 1,
                        "mtime": st.st_mtime,
                        "symbols": syms,
                    }
                )
            elif ext in DOC_EXTS:
                txt = fpath.read_text(encoding="utf-8", errors="ignore")
                headings = [m.group(2).strip()[:70] for m in MD_HEADING_RE.finditer(txt)][:4]
                doc_records.append(
                    {
                        "path": rel_str,
                        "lines": txt.count("\n") + 1,
                        "mtime": st.st_mtime,
                        "headings": headings,
                    }
                )

    # Rank code files by hub centrality + symbol richness + recency
    for rec in code_records:
        hub = inbound_refs.get(rec["stem"], 0)
        rec["hub_refs"] = hub
        rec["score"] = hub * 4.0 + min(len(rec["symbols"]), 6) * 1.5 + (1.0 if rec["path"] in entrypoints else 0.0)

    code_records.sort(key=lambda r: (r["score"], r["mtime"]), reverse=True)
    doc_records.sort(key=lambda r: r["mtime"], reverse=True)

    elapsed_ms = (time.perf_counter() - t0) * 1000.0
    return {
        "root": root.name,
        "elapsed_ms": round(elapsed_ms, 1),
        "scanned_files": total_files,
        "raw_bytes": total_raw_bytes,
        "raw_tokens_if_read_all": (total_raw_bytes + 3) // 4,
        "top_dirs": dict(sorted(dir_counts.items(), key=lambda kv: kv[1], reverse=True)[:10]),
        "entrypoints": sorted(entrypoints)[:10],
        "top_code": code_records[:max_files],
        "top_docs": doc_records[: min(8, max_files)],
        "stale_todo_traps": stale_todos,
    }


def render_atlas_md(data: dict[str, Any]) -> str:
    lines: list[str] = [
        f"# ATLAS MAP: `{data['root']}` ({data['scanned_files']} files scanned in {data['elapsed_ms']}ms | "
        f"raw ~{data['raw_tokens_if_read_all']:,} tok)"
    ]
    if data["top_dirs"]:
        dirs_s = ", ".join(f"`{k}/`({v})" if k != "." else f"`root`({v})" for k, v in data["top_dirs"].items())
        lines.append(f"**Layout**: {dirs_s}")
    if data["entrypoints"]:
        lines.append("**Entrypoints/Manifests**: " + ", ".join(f"`{e}`" for e in data["entrypoints"]))

    if data["top_code"]:
        lines.append("\n## Key Code Modules (Ranked by Centrality & Symbols)")
        for c in data["top_code"]:
            hub = f" [imported {c['hub_refs']}x]" if c["hub_refs"] > 0 else ""
            syms = ": " + ", ".join(c["symbols"][:5]) if c["symbols"] else ""
            lines.append(f"- `{c['path']}` ({c['lines']}L){hub}{syms}")

    if data["top_docs"]:
        lines.append("\n## Active Docs / Knowledge Vault")
        for d in data["top_docs"]:
            hd = " -> " + " / ".join(d["headings"][:3]) if d["headings"] else ""
            lines.append(f"- `{d['path']}` ({d['lines']}L){hd}")

    if data["stale_todo_traps"]:
        lines.append("\n## Stale TODO Traps (Ignore unless explicitly requested)")
        for t in data["stale_todo_traps"]:
            lines.append(f"- {t}")

    return "\n".join(lines) + "\n"


def main(argv: list[str] | None = None) -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
        sys.stderr.reconfigure(encoding="utf-8", errors="replace")

    parser = argparse.ArgumentParser(prog="atlas", description="Sub-second zero-read structural map")
    sub = parser.add_subparsers(dest="subcmd", required=True)

    p_map = sub.add_parser("map", help="Generate compact structural map of a repo or personal vault")
    p_map.add_argument("path", nargs="?", default=".", help="Target directory")
    p_map.add_argument("--focus", default="", help="Substring filter on file path")
    p_map.add_argument("--max-files", type=int, default=16, help="Max ranked code files to emit")
    p_map.add_argument("--stale-days", type=int, default=90, help="Age threshold for stale TODO trap warning")
    p_map.add_argument("--json", action="store_true", help="Emit structured JSON")

    args = parser.parse_args(argv)
    if args.subcmd == "map":
        res = scan_workspace(Path(args.path), focus=args.focus, max_files=args.max_files, stale_days=args.stale_days)
        if args.json:
            sys.stdout.write(json.dumps(res, ensure_ascii=False, indent=2) + "\n")
        else:
            md = render_atlas_md(res)
            sys.stdout.write(md)
        return 0
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
