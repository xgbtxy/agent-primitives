#!/usr/bin/env python3
"""probe.py - Zero-dependency Causal Execution, Reachability & Differential Microscope.

Upgrades an LLM coding agent's foundational process from "guessing heap state and
call graphs in 20,000 prose reasoning tokens" to deterministic <0.2s verification:

  1. `slice` : Static AST + import reachability slice from any entrypoint/test,
               pruning dead distractor modules and juxtaposing cross-file call contracts.
  2. `run`   : Scoped `sys.settrace` runtime microscope that automatically flags:
               - `[ALIAS HAZARD]` (shallow-copy / shared mutable nested heap refs),
               - `[IN-PLACE MUTATION]` & `[MUTATED ON EXCEPTION]` (caller arg clobbering),
               - `[STATE NO-OP]` (void mutator/rollback/delete methods leaving `self` unchanged).
  3. `diff`  : Differential & boundary-matrix oracle (`None`, `True` vs `1`, `False` vs `0`,
               nested containers, stateful cache sequences, strict type-tag parity, input purity).
"""

from __future__ import annotations

import argparse
import ast
import io
import json
import os
import re
import sys
import time
import traceback
import types
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path
from typing import Any

SKIP_DIRS = {
    ".git", ".hg", ".svn", "node_modules", "__pycache__", ".venv", "venv",
    "env", "dist", "build", "target", ".tox", ".mypy_cache", ".pytest_cache",
    ".ruff_cache", ".sieve", ".baton", ".recall", ".guard", ".probe",
    ".agents", ".claude", ".grok", ".cursor", ".codex", ".openclaw", ".hermes",
    "vendor", "coverage", ".idea", ".vscode",
}

READONLY_METHOD_PREFIXES = (
    "__", "get_", "is_", "has_", "can_", "should_", "to_", "as_",
    "format", "render", "dump", "print", "log", "record_metric", "check_",
    "validate", "verify", "count", "len", "peek", "match", "find", "search",
)

STDLIB_CONTAINER_METHODS = {
    "get", "pop", "items", "keys", "values", "setdefault", "update", "clear",
    "copy", "append", "extend", "insert", "remove", "add", "discard", "strip",
    "lstrip", "rstrip", "lower", "upper", "split", "rsplit", "join", "replace",
    "startswith", "endswith", "encode", "decode", "format", "sort",
}


# ---------------------------------------------------------------------------
# Strict Type-Tagged Structural Snapshots & Heap-Alias Inspection
# ---------------------------------------------------------------------------

def _short_repr(val: Any, max_len: int = 88) -> str:
    try:
        if isinstance(val, bool):
            return "True" if val else "False"
        if isinstance(val, (int, float, type(None))):
            return repr(val)
        if isinstance(val, str):
            s = repr(val)
            return s if len(s) <= max_len else s[: max_len - 3] + "...'"
        if isinstance(val, dict):
            items = []
            for idx, (k, v) in enumerate(val.items()):
                if idx >= 5:
                    items.append("...")
                    break
                items.append(f"{_short_repr(k, 24)}: {_short_repr(v, 32)}")
            return "{" + ", ".join(items) + "}"
        if isinstance(val, (list, tuple, set)):
            open_b, close_b = ("[", "]") if isinstance(val, list) else (("(", ")") if isinstance(val, tuple) else ("{", "}"))
            items = []
            for idx, item in enumerate(val):
                if idx >= 5:
                    items.append("...")
                    break
                items.append(_short_repr(item, 28))
            return open_b + ", ".join(items) + close_b
        d = getattr(val, "__dict__", None)
        if isinstance(d, dict):
            fields = []
            for idx, (k, v) in enumerate(d.items()):
                if idx >= 5:
                    fields.append("...")
                    break
                fields.append(f"{k}={_short_repr(v, 28)}")
            return f"{type(val).__name__}({', '.join(fields)})"
        s = repr(val)
        return s if len(s) <= max_len else s[: max_len - 3] + "..."
    except Exception:
        return f"<{type(val).__name__}>"


def _is_mutable_container_or_obj(val: Any) -> bool:
    if isinstance(val, (str, bytes, int, float, bool, tuple, frozenset, type(None), type, types.FunctionType, types.MethodType, types.ModuleType)):
        return False
    if isinstance(val, (dict, list, set, bytearray)):
        return True
    return hasattr(val, "__dict__") and not getattr(type(val), "__dataclass_params__", None) or (
        hasattr(val, "__dict__") and not getattr(getattr(type(val), "__dataclass_params__", None), "frozen", False)
    )


def _snapshot(val: Any, depth: int = 0, max_depth: int = 5, seen: set[int] | None = None) -> Any:
    """Side-effect-free structural snapshot preserving exact Python types (`bool` != `int`)."""
    if seen is None:
        seen = set()
    tname = type(val).__name__
    if val is None or isinstance(val, (bool, int, float, str, bytes)):
        return (tname, val)
    oid = id(val)
    if oid in seen:
        return ("cycle", tname)
    if depth >= max_depth:
        return (tname, _short_repr(val, 60))
    next_seen = seen | {oid}
    if isinstance(val, dict):
        items = []
        for k, v in list(val.items())[:40]:
            items.append((_snapshot(k, depth + 1, max_depth, next_seen), _snapshot(v, depth + 1, max_depth, next_seen)))
        items.sort(key=lambda pair: repr(pair[0]))
        return ("dict", tuple(items))
    if isinstance(val, (list, tuple)):
        return (tname, tuple(_snapshot(x, depth + 1, max_depth, next_seen) for x in list(val)[:40]))
    if isinstance(val, (set, frozenset)):
        elems = [_snapshot(x, depth + 1, max_depth, next_seen) for x in list(val)[:40]]
        elems.sort(key=repr)
        return (tname, tuple(elems))
    d = getattr(val, "__dict__", None)
    if isinstance(d, dict):
        attrs = []
        for k in sorted(d.keys())[:40]:
            if k.startswith("__"):
                continue
            attrs.append((k, _snapshot(d[k], depth + 1, max_depth, next_seen)))
        return ("obj:" + tname, tuple(attrs))
    return (tname, _short_repr(val, 60))


def _diff_snapshots(pre: Any, post: Any, path: str, out: list[tuple[str, str, str]], max_diffs: int = 6) -> None:
    if len(out) >= max_diffs or pre == post:
        return
    if not (isinstance(pre, tuple) and isinstance(post, tuple) and len(pre) == 2 and len(post) == 2):
        out.append((path, repr(pre), repr(post)))
        return
    kt1, v1 = pre
    kt2, v2 = post
    if kt1 != kt2:
        out.append((path, f"{v1!r} ({kt1})", f"{v2!r} ({kt2})"))
        return
    if kt1 == "dict":
        m1 = {k: v for k, v in v1}
        m2 = {k: v for k, v in v2}
        for k in list(m1.keys()):
            k_disp = repr(k[1]) if isinstance(k, tuple) and len(k) == 2 else repr(k)
            if k not in m2:
                out.append((f"{path}[{k_disp}]", _fmt_snap_leaf(m1[k]), "<deleted>"))
            elif m1[k] != m2[k]:
                _diff_snapshots(m1[k], m2[k], f"{path}[{k_disp}]", out, max_diffs)
            if len(out) >= max_diffs:
                return
        for k in list(m2.keys()):
            if k not in m1:
                k_disp = repr(k[1]) if isinstance(k, tuple) and len(k) == 2 else repr(k)
                out.append((f"{path}[{k_disp}]", "<absent>", _fmt_snap_leaf(m2[k])))
                if len(out) >= max_diffs:
                    return
        return
    if kt1.startswith("obj:"):
        m1 = dict(v1)
        m2 = dict(v2)
        for k in sorted(set(m1.keys()) | set(m2.keys())):
            if k not in m2:
                out.append((f"{path}.{k}", _fmt_snap_leaf(m1[k]), "<deleted>"))
            elif k not in m1:
                out.append((f"{path}.{k}", "<absent>", _fmt_snap_leaf(m2[k])))
            elif m1[k] != m2[k]:
                _diff_snapshots(m1[k], m2[k], f"{path}.{k}", out, max_diffs)
            if len(out) >= max_diffs:
                return
        return
    if kt1 in ("list", "tuple"):
        if len(v1) != len(v2):
            out.append((path, f"len={len(v1)} {_fmt_snap_leaf(pre)}", f"len={len(v2)} {_fmt_snap_leaf(post)}"))
            return
        for idx, (a, b) in enumerate(zip(v1, v2)):
            if a != b:
                _diff_snapshots(a, b, f"{path}[{idx}]", out, max_diffs)
                if len(out) >= max_diffs:
                    return
        return
    out.append((path, _fmt_snap_leaf(pre), _fmt_snap_leaf(post)))


def _fmt_snap_leaf(snap: Any) -> str:
    if isinstance(snap, tuple) and len(snap) == 2:
        kt, v = snap
        if kt in ("bool", "int", "float", "str", "NoneType"):
            return f"{v!r}"
        if kt == "dict":
            inner = ", ".join(f"{k[1]!r}: {_fmt_snap_leaf(val)}" for k, val in v[:4])
            if len(v) > 4:
                inner += ", ..."
            return "{" + inner + "}"
        if kt in ("list", "tuple", "set"):
            inner = ", ".join(_fmt_snap_leaf(x) for x in v[:4])
            if len(v) > 4:
                inner += ", ..."
            return f"[{inner}]" if kt == "list" else f"({inner})"
        if kt.startswith("obj:"):
            inner = ", ".join(f"{k}={_fmt_snap_leaf(val)}" for k, val in v[:4])
            return f"{kt[4:]}({inner})"
    return repr(snap)


def _collect_mutable_subobjects(val: Any, path: str, depth: int = 0, max_depth: int = 4, out: dict[int, tuple[str, str]] | None = None, seen: set[int] | None = None) -> dict[int, tuple[str, str]]:
    """Maps `id(mutable_obj) -> (access_path, type_name)` for nested mutable heap refs."""
    if out is None:
        out = {}
    if seen is None:
        seen = set()
    oid = id(val)
    if oid in seen or depth > max_depth:
        return out
    seen.add(oid)
    if _is_mutable_container_or_obj(val):
        out[oid] = (path, type(val).__name__)
    if isinstance(val, dict):
        for k, v in list(val.items())[:30]:
            _collect_mutable_subobjects(v, f"{path}[{k!r}]", depth + 1, max_depth, out, seen)
    elif isinstance(val, (list, tuple, set)):
        for idx, item in enumerate(list(val)[:30]):
            _collect_mutable_subobjects(item, f"{path}[{idx}]", depth + 1, max_depth, out, seen)
    else:
        d = getattr(val, "__dict__", None)
        if isinstance(d, dict):
            for k, v in list(d.items())[:30]:
                if not k.startswith("__"):
                    _collect_mutable_subobjects(v, f"{path}.{k}", depth + 1, max_depth, out, seen)
    return out


# ---------------------------------------------------------------------------
# 1. `slice` — Static AST + Import Reachability & Cross-File Contract Slice
# ---------------------------------------------------------------------------

def _fmt_func_sig(node: ast.FunctionDef | ast.AsyncFunctionDef) -> str:
    args_parts: list[str] = []
    pos_args = list(node.args.posonlyargs) + list(node.args.args)
    num_defaults = len(node.args.defaults)
    default_start = len(pos_args) - num_defaults
    for i, a in enumerate(pos_args):
        piece = a.arg
        if a.annotation is not None:
            try:
                piece += f": {ast.unparse(a.annotation)}"
            except Exception:
                pass
        if i >= default_start:
            try:
                piece += f" = {ast.unparse(node.args.defaults[i - default_start])}"
            except Exception:
                piece += "=..."
        args_parts.append(piece)
    if node.args.vararg:
        args_parts.append(f"*{node.args.vararg.arg}")
    for kwa, kwd in zip(node.args.kwonlyargs, node.args.kw_defaults):
        piece = kwa.arg
        if kwa.annotation is not None:
            try:
                piece += f": {ast.unparse(kwa.annotation)}"
            except Exception:
                pass
        if kwd is not None:
            try:
                piece += f" = {ast.unparse(kwd)}"
            except Exception:
                pass
        args_parts.append(piece)
    if node.args.kwarg:
        args_parts.append(f"**{node.args.kwarg.arg}")
    ret = ""
    if node.returns is not None:
        try:
            ret = f" -> {ast.unparse(node.returns)}"
        except Exception:
            pass
    end_ln = getattr(node, "end_lineno", node.lineno)
    return f"def {node.name}({', '.join(args_parts)}){ret} [L{node.lineno}-{end_ln}]"


def run_slice(root: Path, entries: list[str], as_json: bool = False) -> int:
    t0 = time.perf_counter()
    root = root.resolve()
    py_files: list[Path] = []
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS and not d.startswith(".git")]
        for fn in sorted(filenames):
            if fn.endswith(".py"):
                py_files.append((Path(dirpath) / fn).resolve())

    mod_to_file: dict[str, Path] = {}
    file_to_mod: dict[Path, str] = {}
    for pf in py_files:
        rel = pf.relative_to(root)
        parts = list(rel.parts)
        if parts and parts[0] == "src" and len(parts) > 1:
            mod_parts = parts[1:]
        else:
            mod_parts = parts[:]
        if mod_parts[-1] == "__init__.py":
            mod_name = ".".join(mod_parts[:-1])
        else:
            mod_name = ".".join(mod_parts[:-1] + [mod_parts[-1][:-3]])
        if mod_name:
            mod_to_file[mod_name] = pf
            file_to_mod[pf] = mod_name

    parsed: dict[Path, ast.Module] = {}
    imports_graph: dict[Path, list[Path]] = {pf: [] for pf in py_files}
    signatures: dict[Path, list[str]] = {pf: [] for pf in py_files}
    method_catalog: dict[str, list[tuple[Path, str, str]]] = {}  # name -> [(file, owner, sig)]
    import_symbol_map: dict[Path, dict[str, tuple[Path, str]]] = {pf: {} for pf in py_files}

    for pf in py_files:
        try:
            tree = ast.parse(pf.read_text(encoding="utf-8", errors="replace"))
        except SyntaxError:
            continue
        parsed[pf] = tree
        rel = pf.relative_to(root)
        parts = list(rel.parts)
        pkg_parts = parts[1:-1] if (parts and parts[0] == "src" and len(parts) > 1) else parts[:-1]

        for node in tree.body:
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                sig = _fmt_func_sig(node)
                signatures[pf].append(sig)
                method_catalog.setdefault(node.name, []).append((pf, "", sig))
            elif isinstance(node, ast.ClassDef):
                end_ln = getattr(node, "end_lineno", node.lineno)
                m_sigs: list[str] = []
                for sub in node.body:
                    if isinstance(sub, (ast.FunctionDef, ast.AsyncFunctionDef)):
                        ms = _fmt_func_sig(sub)
                        m_sigs.append(f"  {ms}")
                        if sub.name != "__init__":
                            method_catalog.setdefault(sub.name, []).append((pf, node.name, ms))
                header = f"class {node.name} [L{node.lineno}-{end_ln}]:"
                signatures[pf].append("\n".join([header, *m_sigs]) if m_sigs else header)

        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                for alias in node.names:
                    target = mod_to_file.get(alias.name)
                    if target and target not in imports_graph[pf]:
                        imports_graph[pf].append(target)
            elif isinstance(node, ast.ImportFrom):
                base_mod = node.module or ""
                if node.level > 0:
                    cut = len(pkg_parts) - (node.level - 1)
                    prefix = pkg_parts[: max(0, cut)]
                    full_mod = ".".join(prefix + ([base_mod] if base_mod else []))
                else:
                    full_mod = base_mod
                base_target = mod_to_file.get(full_mod)
                if base_target and base_target not in imports_graph[pf]:
                    imports_graph[pf].append(base_target)
                for alias in node.names:
                    sub_mod = f"{full_mod}.{alias.name}" if full_mod else alias.name
                    sub_target = mod_to_file.get(sub_mod)
                    local_alias = alias.asname or alias.name
                    if sub_target:
                        if sub_target not in imports_graph[pf]:
                            imports_graph[pf].append(sub_target)
                        import_symbol_map[pf][local_alias] = (sub_target, alias.name)
                    elif base_target:
                        import_symbol_map[pf][local_alias] = (base_target, alias.name)

    seed_files: list[Path] = []
    if entries:
        for e in entries:
            cand = (root / e).resolve()
            if e in mod_to_file:
                seed_files.append(mod_to_file[e])
            elif cand.is_file() and cand in parsed:
                seed_files.append(cand)
            elif cand.is_dir():
                pkg_init = (cand / "__init__.py").resolve()
                if pkg_init in parsed:
                    seed_files.append(pkg_init)
                else:
                    seed_files.extend(pf for pf in py_files if str(pf).startswith(str(cand)))
            else:
                for pf in py_files:
                    if e in pf.relative_to(root).as_posix():
                        seed_files.append(pf)
    if not seed_files:
        # Auto-seed from tests + top-level package __init__.py + main/cli/app
        for pf in py_files:
            rel_s = pf.relative_to(root).as_posix()
            if rel_s.startswith("tests/") or pf.name in ("main.py", "app.py", "cli.py", "__main__.py") or len(pf.relative_to(root).parts) == 2 and pf.name == "__init__.py":
                seed_files.append(pf)
        if not seed_files:
            seed_files = py_files[:1]

    reachable: list[Path] = []
    visited: set[Path] = set()
    queue = list(seed_files)
    while queue:
        cur = queue.pop(0)
        if cur in visited:
            continue
        visited.add(cur)
        reachable.append(cur)
        for nxt in imports_graph.get(cur, []):
            if nxt not in visited:
                queue.append(nxt)

    reachable_set = set(reachable)
    pruned = [pf for pf in py_files if pf not in reachable_set and pf.stat().st_size > 0]

    # Extract high-signal cross-file call contracts inside reachable non-test files
    call_contracts: list[str] = []
    seen_contracts: set[str] = set()
    for pf in reachable:
        rel_s = pf.relative_to(root).as_posix()
        if rel_s.startswith("tests/") or pf not in parsed:
            continue
        sym_map = import_symbol_map.get(pf, {})
        for node in ast.walk(parsed[pf]):
            if not isinstance(node, ast.Call):
                continue
            try:
                call_args_s = ", ".join(
                    [ast.unparse(a) for a in node.args] + [f"{kw.arg}={ast.unparse(kw.value)}" for kw in node.keywords if kw.arg]
                )
            except Exception:
                call_args_s = "..."
            if isinstance(node.func, ast.Name) and node.func.id in sym_map:
                tgt_file, orig_name = sym_map[node.func.id]
                if tgt_file != pf and tgt_file in reachable_set:
                    cands = [s for f, _, s in method_catalog.get(orig_name, []) if f == tgt_file]
                    sig_hint = f" -> {cands[0]}" if cands else f" -> {tgt_file.relative_to(root).as_posix()}:{orig_name}"
                    line = f"{rel_s}:L{node.lineno}: {node.func.id}({call_args_s}){sig_hint}"
                    if line not in seen_contracts:
                        seen_contracts.add(line)
                        call_contracts.append(line)
            elif isinstance(node.func, ast.Attribute) and node.func.attr not in STDLIB_CONTAINER_METHODS:
                mname = node.func.attr
                cands = [(f, cls_n, s) for f, cls_n, s in method_catalog.get(mname, []) if f in reachable_set and not f.relative_to(root).as_posix().startswith("tests/")]
                if len(cands) == 1:
                    tgt_f, cls_n, s = cands[0]
                    qual = f"{cls_n}.{mname}" if cls_n else mname
                    clean_sig = s.rsplit(" [L", 1)[0].strip()
                    line = f"{rel_s}:L{node.lineno}: .{mname}({call_args_s}) -> {tgt_f.relative_to(root).as_posix()}::{qual} [{clean_sig}]"
                    if line not in seen_contracts:
                        seen_contracts.add(line)
                        call_contracts.append(line)

    elapsed_ms = round((time.perf_counter() - t0) * 1000.0, 2)
    payload = {
        "root": root.as_posix(),
        "elapsed_ms": elapsed_ms,
        "total_py_files": len(py_files),
        "reachable_count": len(reachable),
        "pruned_unreferenced_count": len(pruned),
        "reachable_files": [
            {
                "path": pf.relative_to(root).as_posix(),
                "imports": [t.relative_to(root).as_posix() for t in imports_graph.get(pf, [])],
                "signatures": signatures.get(pf, []),
            }
            for pf in reachable
            if signatures.get(pf) or not pf.name.startswith("__init__")
        ],
        "pruned_files_sample": [pf.relative_to(root).as_posix() for pf in pruned[:15]],
        "cross_file_call_contracts": call_contracts[:30],
    }

    if as_json:
        sys.stdout.write(json.dumps(payload, indent=2, ensure_ascii=False) + "\n")
        return 0

    out_lines = [
        f"[probe slice] {len(reachable)}/{len(py_files)} Python files reachable ({len(pruned)} unreferenced distractor files pruned) in {elapsed_ms} ms",
    ]
    if pruned:
        sample = ", ".join(pf.relative_to(root).as_posix() for pf in pruned[:8])
        more = f" (+{len(pruned) - 8} more)" if len(pruned) > 8 else ""
        out_lines.append(f"[PRUNED DEAD/UNREFERENCED]: {sample}{more} — DO NOT waste turns reading these unless explicitly needed.")

    out_lines.append("\n## Active Reachable Modules & Signatures")
    for item in payload["reachable_files"]:
        if not item["signatures"]:
            continue
        out_lines.append(f"- `{item['path']}`")
        for sig in item["signatures"]:
            for subline in sig.splitlines():
                out_lines.append(f"    {subline}")

    if call_contracts:
        out_lines.append("\n## Cross-File Call Sites vs Target Parameter Contracts")
        for cc in call_contracts[:25]:
            out_lines.append(f"- {cc}")

    sys.stdout.write("\n".join(out_lines) + "\n")
    return 0


# ---------------------------------------------------------------------------
# 2. `run` — Causal Execution, Heap-Aliasing, Mutation & State No-Op Microscope
# ---------------------------------------------------------------------------

def run_trace(
    cwd: Path,
    focus: str,
    code: str | None,
    script: str | None,
    unittest_target: str | None,
    max_calls: int = 80,
    include_tests: bool = False,
    as_json: bool = False,
) -> int:
    t0 = time.perf_counter()
    cwd = cwd.resolve()
    cwd_norm = os.path.normcase(str(cwd))
    focus_norm = focus.strip().replace("\\", "/").lower()

    if str(cwd) not in sys.path:
        sys.path.insert(0, str(cwd))
    if (cwd / "src").is_dir() and str(cwd / "src") not in sys.path:
        sys.path.insert(0, str(cwd / "src"))

    call_log: list[str] = []
    alias_hazards: list[str] = []
    mutation_alerts: list[str] = []
    noop_alerts: list[str] = []
    exception_events: list[str] = []

    seen_alerts: set[str] = set()
    # Maps `id(mutable_subobj) -> "Class.method() return.path"`
    shared_alias_registry: dict[int, str] = {}
    frame_state: dict[int, dict[str, Any]] = {}
    depth_counter = [0]

    def _should_trace_file(filename: str) -> tuple[bool, str]:
        if not filename or filename.startswith("<"):
            return False, ""
        norm = os.path.normcase(os.path.abspath(filename))
        if not norm.startswith(cwd_norm):
            return False, ""
        rel = os.path.relpath(os.path.abspath(filename), str(cwd)).replace("\\", "/")
        parts = rel.split("/")
        if any(p in SKIP_DIRS for p in parts):
            return False, ""
        if not include_tests and (parts[0] == "tests" or parts[-1].startswith("test_")):
            return False, ""
        if focus_norm and focus_norm not in rel.lower():
            return False, ""
        return True, rel

    import inspect
    import linecache

    def tracer(frame: types.FrameType, event: str, arg: Any) -> Any:
        if event == "call":
            ok, rel = _should_trace_file(frame.f_code.co_filename)
            if not ok:
                return None
            fn_name = frame.f_code.co_name
            if fn_name.startswith("<"):
                return None
            first_line = linecache.getline(frame.f_code.co_filename, frame.f_code.co_firstlineno).lstrip()
            if first_line.startswith("class "):
                return None

            arg_info = inspect.getargvalues(frame)
            bound_args: list[tuple[str, Any]] = []
            for aname in arg_info.args:
                if aname in arg_info.locals:
                    bound_args.append((aname, arg_info.locals[aname]))
            if arg_info.varargs and arg_info.varargs in arg_info.locals:
                bound_args.append((f"*{arg_info.varargs}", arg_info.locals[arg_info.varargs]))
            if arg_info.keywords and arg_info.keywords in arg_info.locals:
                bound_args.append((f"**{arg_info.keywords}", arg_info.locals[arg_info.keywords]))

            qual_name = fn_name
            if bound_args and bound_args[0][0] == "self":
                qual_name = f"{type(bound_args[0][1]).__name__}.{fn_name}"

            pre_snaps: dict[str, Any] = {}
            pre_mut_ids: dict[str, dict[int, tuple[str, str]]] = {}
            arg_previews: list[str] = []
            for aname, aval in bound_args:
                pre_snaps[aname] = _snapshot(aval)
                pre_mut_ids[aname] = _collect_mutable_subobjects(aval, aname)
                if aname != "self":
                    arg_previews.append(f"{aname}={_short_repr(aval, 44)}")

            call_idx = len(call_log)
            indent = "  " * min(depth_counter[0], 6)
            if call_idx < max_calls:
                call_log.append(f"{indent}-> {rel}:L{frame.f_lineno} {qual_name}({', '.join(arg_previews)})")

            frame_state[id(frame)] = {
                "rel": rel,
                "lineno": frame.f_lineno,
                "fn_name": fn_name,
                "qual_name": qual_name,
                "bound_args": bound_args,
                "pre_snaps": pre_snaps,
                "pre_mut_ids": pre_mut_ids,
                "arg_previews": arg_previews,
                "call_idx": call_idx,
                "depth": depth_counter[0],
                "exc": None,
            }
            depth_counter[0] += 1
            return tracer

        st = frame_state.get(id(frame))
        if st is None:
            return tracer

        if event == "line":
            st["exc"] = None
            return tracer

        if event == "exception":
            exc_type, exc_val, _ = arg
            st["exc"] = f"{exc_type.__name__}: {exc_val}"
            msg = f"{st['rel']}:L{frame.f_lineno} in {st['qual_name']}: {exc_type.__name__}({exc_val})"
            if msg not in seen_alerts:
                seen_alerts.add(msg)
                exception_events.append(msg)
            return tracer

        if event == "return":
            depth_counter[0] = max(0, st["depth"])
            frame_state.pop(id(frame), None)
            qual = st["qual_name"]
            fn_name = st["fn_name"]
            rel = st["rel"]
            exc_desc = st["exc"]

            if st["call_idx"] < max_calls:
                if exc_desc and arg is None:
                    call_log[st["call_idx"]] += f"  !! RAISED {exc_desc}"
                else:
                    call_log[st["call_idx"]] += f" => {_short_repr(arg, 56)}"

            # A. Detect Shallow-Copy / Shared Mutable Nested Heap Aliasing
            if exc_desc is None and arg is not None and fn_name != "__init__":
                ret_muts = _collect_mutable_subobjects(arg, "return")
                for aname, aval in st["bound_args"]:
                    if arg is aval:
                        continue  # Normal fluent/mutating return `return env` or `return self`
                    in_muts = st["pre_mut_ids"].get(aname, {})
                    shared_paths: list[str] = []
                    for oid, (r_path, r_type) in ret_muts.items():
                        if oid == id(arg) or oid == id(aval):
                            continue
                        if oid in in_muts:
                            in_path, _ = in_muts[oid]
                            shared_paths.append(f"{in_path} <-> {r_path} ({r_type})")
                            shared_alias_registry[oid] = f"{qual}() {r_path}"
                    if shared_paths:
                        alert = (
                            f"{rel}:L{st['lineno']} `{qual}()` returned a distinct `{type(arg).__name__}`, "
                            f"but SHARES mutable nested heap object(s) with input `{aname}`: {', '.join(shared_paths[:4])}"
                        )
                        if alert not in seen_alerts:
                            seen_alerts.add(alert)
                            alias_hazards.append(alert)

            # B. Detect In-Place Argument Mutations (including mutations left behind on exception!)
            any_mutated = False
            for aname, aval in st["bound_args"]:
                post_snap = _snapshot(aval)
                pre_snap = st["pre_snaps"].get(aname)
                if post_snap != pre_snap:
                    any_mutated = True
                    if aname == "self" and fn_name == "__init__":
                        continue
                    if aname == "self" and exc_desc is None:
                        continue  # Normal state transition on `self`
                    diffs: list[tuple[str, str, str]] = []
                    _diff_snapshots(pre_snap, post_snap, aname, diffs)
                    if diffs:
                        # Check if any mutated nested container was registered in shared_alias_registry
                        alias_notes: list[str] = []
                        cur_muts = _collect_mutable_subobjects(aval, aname)
                        for oid in cur_muts:
                            if oid in shared_alias_registry:
                                alias_notes.append(shared_alias_registry[oid])
                        diff_str = "; ".join(f"{p}: {b} -> {a}" for p, b, a in diffs[:4])
                        prefix_tag = "[MUTATED ON EXCEPTION] " if exc_desc else ""
                        alias_suffix = f" [ALSO CORRUPTS ALIASED {alias_notes[0]}!]" if alias_notes else ""
                        alert = f"{prefix_tag}{rel}:L{st['lineno']} `{qual}()` mutated `{aname}` in-place: {diff_str}{alias_suffix}"
                        if alert not in seen_alerts:
                            seen_alerts.add(alert)
                            mutation_alerts.append(alert)

            # C. Detect Stateful Void-Method No-Ops (`def rollback/remove/delete/update(...) -> None` leaving `self` unchanged!)
            if (
                exc_desc is None
                and arg is None
                and not any_mutated
                and st["bound_args"]
                and st["bound_args"][0][0] == "self"
                and not fn_name.startswith(READONLY_METHOD_PREFIXES)
            ):
                self_obj = st["bound_args"][0][1]
                self_dict = getattr(self_obj, "__dict__", None)
                if isinstance(self_dict, dict) and self_dict:
                    alert = (
                        f"{rel}:L{st['lineno']} `{qual}({', '.join(st['arg_previews'])})` returned `None` "
                        f"with ZERO change to `self`! Current `self` state: {_short_repr(self_obj, 110)}"
                    )
                    if alert not in seen_alerts:
                        seen_alerts.add(alert)
                        noop_alerts.append(alert)

        return tracer

    out_buf = io.StringIO()
    err_buf = io.StringIO()
    exec_error: str | None = None

    old_trace = sys.gettrace()
    sys.settrace(tracer)
    try:
        with redirect_stdout(out_buf), redirect_stderr(err_buf):
            if code is not None:
                ns: dict[str, Any] = {"__name__": "__main__"}
                exec(compile(code, "<probe_snippet>", "exec"), ns, ns)
            elif script is not None:
                sp = (cwd / script).resolve() if not Path(script).is_absolute() else Path(script).resolve()
                ns = {"__name__": "__main__", "__file__": str(sp)}
                exec(compile(sp.read_text(encoding="utf-8"), str(sp), "exec"), ns, ns)
            elif unittest_target is not None:
                loader = unittest.TestLoader()
                if unittest_target == "discover" or Path(cwd / unittest_target).is_dir():
                    start_d = str(cwd / ("tests" if unittest_target == "discover" else unittest_target))
                    suite = loader.discover(start_dir=start_d, top_level_dir=str(cwd))
                else:
                    suite = loader.loadTestsFromName(unittest_target)
                runner = unittest.TextTestRunner(stream=out_buf, verbosity=2)
                res = runner.run(suite)
                if not res.wasSuccessful():
                    exec_error = f"Unittest failures={len(res.failures)} errors={len(res.errors)}"
    except BaseException as exc:
        exec_error = f"{type(exc).__name__}: {exc}\n" + "".join(traceback.format_exception_only(type(exc), exc)).strip()
    finally:
        sys.settrace(old_trace)

    elapsed_ms = round((time.perf_counter() - t0) * 1000.0, 2)
    payload = {
        "elapsed_ms": elapsed_ms,
        "exec_error": exec_error,
        "alias_hazards": alias_hazards,
        "mutation_alerts": mutation_alerts,
        "state_noop_alerts": noop_alerts,
        "exceptions_observed": exception_events,
        "call_tree": call_log,
        "stdout": out_buf.getvalue()[-1200:],
        "stderr": err_buf.getvalue()[-1200:],
    }

    if as_json:
        sys.stdout.write(json.dumps(payload, indent=2, ensure_ascii=False) + "\n")
        return 0 if exec_error is None else 1

    lines = [
        f"[probe run] Traced {len(call_log)} call(s) in {elapsed_ms} ms | "
        f"alias_hazards={len(alias_hazards)} | in_place_mutations={len(mutation_alerts)} | state_noops={len(noop_alerts)}"
    ]
    if exec_error:
        lines.append(f"[EXECUTION ERROR]: {exec_error}")
    if alias_hazards:
        lines.append("\n## !! [ALIAS HAZARD] Shared Mutable Nested Heap Objects")
        for a in alias_hazards:
            lines.append(f"- {a}")
    if mutation_alerts:
        lines.append("\n## !! [IN-PLACE MUTATION] Caller Argument / Shared State Mutations")
        for m in mutation_alerts:
            lines.append(f"- {m}")
    if noop_alerts:
        lines.append("\n## !! [STATE NO-OP] Void Mutator Methods That Left `self` Unchanged")
        for n in noop_alerts:
            lines.append(f"- {n}")
    if exception_events:
        lines.append("\n## [EXCEPTIONS RAISED / CAUGHT]")
        for e in exception_events[:10]:
            lines.append(f"- {e}")
    if call_log:
        lines.append("\n## Causal Call Tree")
        lines.extend(call_log[:max_calls])
    stdout_txt = out_buf.getvalue().strip()
    if stdout_txt:
        lines.append("\n## Program Output")
        lines.append(stdout_txt[-800:])

    sys.stdout.write("\n".join(lines) + "\n")
    return 0


# ---------------------------------------------------------------------------
# 3. `diff` — Differential Semantics, Strict Type Parity & Stateful Cache Probe
# ---------------------------------------------------------------------------

SCALAR_BOUNDARY_CORPUS: list[Any] = [
    None,
    True,
    1,
    False,
    0,
    -1,
    2,
    0.0,
    "",
    "a",
    "Alice",
    [1, False],
    [True, 0],
    {"flag": True, "nested": [1, False]},
    {"flag": 1, "nested": [True, 0]},
]


def _build_env_corpus(var_names: list[str]) -> list[dict[str, Any]]:
    atoms: list[Any] = [None, True, 1, False, 0, -1, 2, "", "Alice", [1, False], [True, 0]]
    envs: list[dict[str, Any]] = [{}]
    for v in atoms:
        envs.append({k: v for k in var_names})
    if len(var_names) >= 2:
        k0, k1 = var_names[0], var_names[1]
        for a, b in [
            (None, 0), (None, False), (None, True), (True, 1), (False, 0),
            (1, True), (0, False), ("a", "A"), (10, None),
        ]:
            envs.append({k0: a, k1: b})
    # Crucial sequential collision pairs (`bool` followed immediately by `int` and nested lists)
    k_main = var_names[0]
    envs.extend([
        {k_main: True, "nested": [1, False]},
        {k_main: 1, "nested": [True, 0]},
        {k_main: False, "nested": [0, True]},
        {k_main: 0, "nested": [False, 1]},
    ])
    return envs


def _deep_clone_for_probe(val: Any) -> Any:
    import copy
    return copy.deepcopy(val)


def _eval_one(fn: Any, inp: Any) -> tuple[str, Any, Any]:
    arg_copy = _deep_clone_for_probe(inp)
    before_snap = _snapshot(arg_copy)
    try:
        res = fn(arg_copy)
        after_snap = _snapshot(arg_copy)
        return ("ok", res, after_snap != before_snap)
    except Exception as exc:
        after_snap = _snapshot(arg_copy)
        return ("exc", f"{type(exc).__name__}: {exc}", after_snap != before_snap)


def run_diff(
    cwd: Path,
    setup_code: str,
    fn_a_code: str,
    fn_b_code: str,
    vars_csv: str,
    inputs_expr: str,
    as_json: bool = False,
) -> int:
    t0 = time.perf_counter()
    cwd = cwd.resolve()
    if str(cwd) not in sys.path:
        sys.path.insert(0, str(cwd))
    if (cwd / "src").is_dir() and str(cwd / "src") not in sys.path:
        sys.path.insert(0, str(cwd / "src"))

    def _make_pair() -> tuple[Any, Any]:
        ns: dict[str, Any] = {}
        if setup_code.strip():
            exec(compile(setup_code, "<probe_diff_setup>", "exec"), ns, ns)
        fa = eval(compile(fn_a_code, "<probe_fn_a>", "eval"), ns, ns)
        fb = eval(compile(fn_b_code, "<probe_fn_b>", "eval"), ns, ns)
        return fa, fb

    try:
        seq_fa, seq_fb = _make_pair()
    except Exception as exc:
        msg = f"[probe diff] Setup/compile failed: {type(exc).__name__}: {exc}\n"
        if as_json:
            sys.stdout.write(json.dumps({"setup_error": msg.strip()}, indent=2) + "\n")
        else:
            sys.stdout.write(msg)
        return 1

    if inputs_expr.strip():
        ns_in: dict[str, Any] = {}
        if setup_code.strip():
            exec(compile(setup_code, "<probe_diff_inputs_setup>", "exec"), ns_in, ns_in)
        corpus = list(eval(compile(inputs_expr, "<probe_inputs>", "eval"), ns_in, ns_in))
    else:
        vnames = [v.strip() for v in vars_csv.split(",") if v.strip()]
        if not vnames:
            # Auto-detect `Var("x")` or dict lookups `d["x"]` / `env.get("x")` in setup/fn strings
            combined = f"{setup_code}\n{fn_a_code}\n{fn_b_code}"
            found = re.findall(r"""(?:Var|get)\(\s*['"]([A-Za-z_][A-Za-z0-9_]*)['"]""", combined)
            found += re.findall(r"""\[\s*['"]([A-Za-z_][A-Za-z0-9_]*)['"]\s*\]""", combined)
            for cand in found:
                if cand not in vnames and not cand.startswith("__"):
                    vnames.append(cand)
        corpus = _build_env_corpus(vnames) if vnames else list(SCALAR_BOUNDARY_CORPUS)

    divergences: list[dict[str, Any]] = []
    stateful_drifts: list[dict[str, Any]] = []
    mutation_bugs: list[dict[str, Any]] = []

    for idx, inp in enumerate(corpus):
        # 1. Fresh-instance differential check
        fresh_fa, fresh_fb = _make_pair()
        ka, ra, mut_a = _eval_one(fresh_fa, inp)
        kb, rb, mut_b = _eval_one(fresh_fb, inp)

        if mut_a or mut_b:
            who = "both A & B" if (mut_a and mut_b) else ("fn_a" if mut_a else "fn_b")
            mutation_bugs.append({"index": idx, "input": _short_repr(inp, 90), "mutated_by": who})

        if ka != kb or _snapshot(ra) != _snapshot(rb):
            divergences.append({
                "index": idx,
                "input": _short_repr(inp, 90),
                "a": f"{ra!r} ({type(ra).__name__})" if ka == "ok" else f"RAISED {ra}",
                "b": f"{rb!r} ({type(rb).__name__})" if kb == "ok" else f"RAISED {rb}",
            })

        # 2. Persistent/sequential stateful check (catches cache collisions like True vs 1!)
        ska, sra, _ = _eval_one(seq_fa, inp)
        skb, srb, _ = _eval_one(seq_fb, inp)
        if (ska != ka or _snapshot(sra) != _snapshot(ra)):
            stateful_drifts.append({
                "index": idx,
                "target": "fn_a",
                "input": _short_repr(inp, 90),
                "fresh": f"{ra!r} ({type(ra).__name__})" if ka == "ok" else f"RAISED {ra}",
                "sequential": f"{sra!r} ({type(sra).__name__})" if ska == "ok" else f"RAISED {sra}",
            })
        if (skb != kb or _snapshot(srb) != _snapshot(rb)):
            stateful_drifts.append({
                "index": idx,
                "target": "fn_b",
                "input": _short_repr(inp, 90),
                "fresh": f"{rb!r} ({type(rb).__name__})" if kb == "ok" else f"RAISED {rb}",
                "sequential": f"{srb!r} ({type(srb).__name__})" if skb == "ok" else f"RAISED {srb}",
            })

    elapsed_ms = round((time.perf_counter() - t0) * 1000.0, 2)
    clean = not (divergences or stateful_drifts or mutation_bugs)
    payload = {
        "elapsed_ms": elapsed_ms,
        "cases_tested": len(corpus),
        "equivalent": clean,
        "divergences": divergences[:12],
        "stateful_cache_drifts": stateful_drifts[:12],
        "caller_input_mutations": mutation_bugs[:12],
    }

    if as_json:
        sys.stdout.write(json.dumps(payload, indent=2, ensure_ascii=False) + "\n")
        return 0 if clean else 1

    status = "EQUIVALENT (0 counterexamples)" if clean else "DIVERGENCE DETECTED"
    lines = [
        f"[probe diff] {status} across {len(corpus)} boundary/sequence inputs in {elapsed_ms} ms "
        f"(divergences={len(divergences)}, stateful_drifts={len(stateful_drifts)}, input_mutations={len(mutation_bugs)})"
    ]
    if divergences:
        lines.append("\n## !! Semantic / Type Divergences (fn_a vs fn_b)")
        for d in divergences[:10]:
            lines.append(f"- input={d['input']}  =>  fn_a: {d['a']}  !=  fn_b: {d['b']}")
    if stateful_drifts:
        lines.append("\n## !! Stateful / Cache Sequence Collisions (Fresh vs Sequential Run)")
        for s in stateful_drifts[:10]:
            lines.append(f"- [{s['target']}] input={s['input']}  =>  fresh: {s['fresh']}  !=  after prior calls: {s['sequential']}")
    if mutation_bugs:
        lines.append("\n## !! Caller Input Purity Violations (Input Mutated In-Place)")
        for m in mutation_bugs[:10]:
            lines.append(f"- input={m['input']} mutated in-place by {m['mutated_by']}")

    sys.stdout.write("\n".join(lines) + "\n")
    return 0 if clean else 1


# ---------------------------------------------------------------------------
# CLI Entrypoint
# ---------------------------------------------------------------------------

def main(argv: list[str] | None = None) -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    if hasattr(sys.stderr, "reconfigure"):
        sys.stderr.reconfigure(encoding="utf-8", errors="replace")

    parser = argparse.ArgumentParser(
        prog="probe.py",
        description="Causal Execution, Reachability & Differential Microscope for Coding Agents",
    )
    sub = parser.add_subparsers(dest="cmd", required=True)

    p_slice = sub.add_parser("slice", help="Static AST + import reachability slice & cross-file call contracts")
    p_slice.add_argument("root", nargs="?", default=".", help="Workspace root directory")
    p_slice.add_argument("--entry", action="append", default=[], help="Entrypoint file(s) or module(s) to slice from")
    p_slice.add_argument("--json", action="store_true", help="Emit JSON output")

    p_run = sub.add_parser("run", help="Trace execution & auto-detect heap aliasing, in-place mutation, and state no-ops")
    p_run.add_argument("--cwd", default=".", help="Target repository working directory")
    p_run.add_argument("--focus", default="", help="Substring filter on traced file paths (e.g. package name)")
    grp = p_run.add_mutually_exclusive_group(required=True)
    grp.add_argument("--code", help="Inline Python snippet to execute under causal microscope")
    grp.add_argument("--stdin", action="store_true", help="Read Python snippet from stdin (avoids shell quoting issues)")
    grp.add_argument("--script", help="Python script path to execute")
    grp.add_argument("--unittest", help="Unittest dotted target or 'discover'")
    p_run.add_argument("--max-calls", type=int, default=80, help="Max call-tree lines to print (default: 80)")
    p_run.add_argument("--include-tests", action="store_true", help="Also trace frames inside tests/ files")
    p_run.add_argument("--json", action="store_true", help="Emit JSON output")

    p_diff = sub.add_parser("diff", help="Differential & boundary-matrix oracle (optimize vs raw, cached vs uncached)")
    p_diff.add_argument("--cwd", default=".", help="Target repository working directory")
    p_diff.add_argument("--setup", default="", help="Python setup code executed before building fn_a and fn_b")
    p_diff.add_argument("--fn-a", required=True, help="Callable expression A (e.g. 'lambda d: opt.evaluate(\"r\", d)')")
    p_diff.add_argument("--fn-b", required=True, help="Callable expression B (e.g. 'lambda d: raw.evaluate(\"r\", d)')")
    p_diff.add_argument("--vars", default="", help="Comma-separated dict keys for boundary environment generation")
    p_diff.add_argument("--inputs", default="", help="Explicit Python list expression of probe inputs")
    p_diff.add_argument("--json", action="store_true", help="Emit JSON output")

    args = parser.parse_args(argv)
    if args.cmd == "slice":
        return run_slice(Path(args.root), args.entry, as_json=args.json)
    if args.cmd == "run":
        snippet = sys.stdin.read() if getattr(args, "stdin", False) else args.code
        return run_trace(
            cwd=Path(args.cwd),
            focus=args.focus,
            code=snippet,
            script=args.script,
            unittest_target=args.unittest,
            max_calls=args.max_calls,
            include_tests=args.include_tests,
            as_json=args.json,
        )
    if args.cmd == "diff":
        return run_diff(
            cwd=Path(args.cwd),
            setup_code=args.setup,
            fn_a_code=args.fn_a,
            fn_b_code=args.fn_b,
            vars_csv=args.vars,
            inputs_expr=args.inputs,
            as_json=args.json,
        )
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
