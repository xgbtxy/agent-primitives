#!/usr/bin/env python3
"""ap.py - 1-Line Zero-Clone Bootstrapper, Multi-Harness Installer & Unified CLI for agent-primitives.

1-Line Zero-Clone Onboarding (Auto-detects Codex, Pi, Antigravity, Claude Code, Grok, OpenClaw, Hermes):
  macOS / Linux:
    curl -fsSL https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/ap.py | python3 - init
  Windows PowerShell:
    irm https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/ap.py | python - init

Target Specific Harnesses or Current Repo:
  python ap.py init -a codex,pi,agy,claude --with-hooks   # Global setup for chosen harnesses + hooks
  python ap.py init --here                                # Repo-local (.agents/skills + .agents/ap.py + AGENTS.md)
  python ap.py uninstall [--here]                         # Cleanly remove skills, shims, hooks & fenced blocks
  python ap.py audit --compare                            # Local-only telemetry audit + before/after trend
  python ap.py doctor                                     # Verify Python >=3.9, SQLite FTS5/BM25 & all 7 primitives

Agent Runtime Invocation:
  python .agents/ap.py probe slice . --bundle
  python .agents/ap.py splice view src/a.ts --symbol Ledger.post
  python .agents/ap.py splice apply --cwd . --stdin
  python .agents/ap.py splice check . --test
"""

from __future__ import annotations

import argparse
import io
import json
import os
import re
import shutil
import sqlite3
import subprocess
import sys
import tarfile
import urllib.request
from pathlib import Path
from typing import Any

_RAW_FILE = globals().get("__file__", "")
IS_STDIN_BOOTSTRAP = (not _RAW_FILE) or (_RAW_FILE in ("<stdin>", "-")) or not Path(_RAW_FILE).exists()
HERE = Path.cwd().resolve() if IS_STDIN_BOOTSTRAP else Path(_RAW_FILE).resolve().parent

DEFAULT_TARBALL_URL = "https://codeload.github.com/xgbtxy/agent-primitives/tar.gz/refs/heads/main"
ALL_PRIMITIVES = ("sieve", "baton", "recall", "guard", "atlas", "probe", "splice")

FENCE_START = "<!-- agent-primitives:start -->"
FENCE_END = "<!-- agent-primitives:end -->"

HARNESS_ALIASES: dict[str, str] = {
    "agy": "antigravity",
    "antigravity": "antigravity",
    "gemini": "antigravity",
    "codex": "codex",
    "pi": "pi",
    "pi-coding-agent": "pi",
    "claude": "claude",
    "claude-code": "claude",
    "grok": "grok",
    "openclaw": "openclaw",
    "hermes": "hermes",
    "agents": "agents",
}

# (canonical_id, cli_binary, home_probe_dirs, global_skill_dirs, global_rule_file)
HARNESS_SPECS: tuple[tuple[str, str, tuple[str, ...], tuple[str, ...], str], ...] = (
    ("codex", "codex", (".codex",), (".codex/skills", ".agents/skills"), ".codex/AGENTS.md"),
    ("pi", "pi", (".pi",), (".pi/agent/skills", ".agents/skills"), ".pi/agent/AGENTS.md"),
    (
        "antigravity",
        "agy",
        (".gemini/antigravity-cli", ".gemini/antigravity", ".gemini/config"),
        (".gemini/config/skills", ".agents/skills"),
        ".gemini/config/GEMINI.md",
    ),
    ("claude", "claude", (".claude",), (".claude/skills",), ".claude/CLAUDE.md"),
    ("grok", "grok", (".grok",), (".grok/skills",), ".grok/AGENTS.md"),
    ("openclaw", "openclaw", (".openclaw",), (".openclaw/skills",), ".openclaw/AGENTS.md"),
    ("hermes", "hermes", (".hermes",), (".hermes/skills",), ".hermes/AGENTS.md"),
)


def _posix(p: Path) -> str:
    return str(p.resolve()).replace("\\", "/")


def _safe_extract_tarball(raw_bytes: bytes, dest_root: Path) -> Path:
    stage = dest_root.parent / (dest_root.name + ".staging")
    if stage.exists():
        shutil.rmtree(stage)
    stage.mkdir(parents=True, exist_ok=True)

    with tarfile.open(fileobj=io.BytesIO(raw_bytes), mode="r:gz") as tf:
        members = tf.getmembers()
        common_prefix = ""
        names = [m.name.lstrip("./") for m in members if m.name.lstrip("./")]
        if names and all("/" in n or m.isdir() for n, m in zip(names, members)):
            first_seg = names[0].split("/", 1)[0]
            if first_seg and all(n == first_seg or n.startswith(first_seg + "/") for n in names):
                common_prefix = first_seg + "/"

        for member in members:
            rel_name = member.name.lstrip("./")
            if common_prefix and rel_name.startswith(common_prefix):
                rel_name = rel_name[len(common_prefix) :]
            elif common_prefix and rel_name == common_prefix.rstrip("/"):
                continue
            if not rel_name:
                continue
            parts = Path(rel_name).parts
            if any(pt in ("..", "") or pt.startswith("/") or ":" in pt for pt in parts):
                raise ValueError(f"Unsafe path in archive: {member.name}")
            out_path = (stage / Path(*parts)).resolve()
            if not str(out_path).startswith(str(stage.resolve())):
                raise ValueError(f"Archive path escaped staging root: {member.name}")
            if member.isdir():
                out_path.mkdir(parents=True, exist_ok=True)
            elif member.isfile():
                out_path.parent.mkdir(parents=True, exist_ok=True)
                extracted = tf.extractfile(member)
                if extracted is not None:
                    out_path.write_bytes(extracted.read())

    if not (stage / "skills" / "probe" / "scripts" / "probe.py").is_file():
        shutil.rmtree(stage, ignore_errors=True)
        raise RuntimeError("Bootstrap archive did not contain `skills/probe/scripts/probe.py`.")

    if dest_root.exists():
        shutil.rmtree(dest_root)
    stage.rename(dest_root)
    return dest_root


def _ensure_repo_root(home: Path) -> Path:
    """Locate local repo/installed bundle, or bootstrap zero-clone tarball when piped via `curl | python3 -`."""
    override_archive = os.environ.get("AP_SOURCE_ARCHIVE", "").strip()
    if not IS_STDIN_BOOTSTRAP and not override_archive:
        if (HERE / "skills" / "probe" / "scripts" / "probe.py").is_file():
            return HERE
        if (HERE / "probe" / "scripts" / "probe.py").is_file():
            return HERE.parent

    cache_dir = home / ".agent-primitives"
    if override_archive:
        raw = Path(override_archive).expanduser().resolve().read_bytes()
        return _safe_extract_tarball(raw, cache_dir)

    if not IS_STDIN_BOOTSTRAP and (cache_dir / "skills" / "probe" / "scripts" / "probe.py").is_file():
        return cache_dir

    url = os.environ.get("AP_TARBALL_URL", DEFAULT_TARBALL_URL)
    sys.stdout.write(f"[ap bootstrap] Downloading zero-dependency bundle from {url} ...\n")
    req = urllib.request.Request(url, headers={"User-Agent": "agent-primitives-bootstrap/0.3"})
    with urllib.request.urlopen(req, timeout=30) as resp:
        raw_bytes = resp.read()
    return _safe_extract_tarball(raw_bytes, cache_dir)


def render_fenced_agents_block(ap_cmd: str, selected: list[str] | None = None) -> str:
    sel = set(selected or ALL_PRIMITIVES)
    bullets: list[str] = []
    if "probe" in sel:
        bullets.append(
            f"1. **Orient & Bundle Active Slice in 1 Call (`probe`)**: Start unfamiliar multi-file tasks with "
            f"`{ap_cmd} probe slice . --bundle` (prunes dead distractor files + emits numbered source of active "
            f"reachable Py/TS/JS/Go/Rust modules in 1 shot — skip redundant `read_file` loops)."
        )
        bullets.append(
            f"2. **Python Runtime & Equivalence Microscope (`probe`)**: Never simulate heap aliasing, mutation, or "
            f"cache collisions in prose. Run `{ap_cmd} probe run --cwd . --focus <pkg> --code \"<repro>\"` "
            f"(multi-line stdin: PowerShell `@' ... '@ | {ap_cmd} probe run --cwd . --stdin`; bash `<<'EOF'`) "
            f"and `{ap_cmd} probe diff`."
        )
    if "splice" in sel:
        bullets.append(
            f"3. **Surgical Symbol Read, Atomic Patch & Auto-Test (`splice`)**: Inspect symbols across Py/TS/JS/Go/Rust "
            f"via `{ap_cmd} splice view <files...> --symbol <Sym>`. Batch multi-file edits & new test files (`\"op\": \"replace\"|\"replace_symbol\"|\"write\"`) "
            f"atomically via `{ap_cmd} splice apply --cwd . --stdin` (or `--spec _edits.json --unlink-spec`), then verify syntax, "
            f"imports & tests in 1 call via `{ap_cmd} splice check . --test`."
        )
    extras = [x for x in ("atlas", "sieve", "recall", "guard", "baton") if x in sel]
    if extras:
        bullets.append(
            f"4. **Vault Map, Sieve, Memory, Guard & Handoff**: Map large vaults via `{ap_cmd} atlas map .`; "
            f"compress noisy >2KB logs via `{ap_cmd} sieve run -- <cmd>`; query/save gotchas via `{ap_cmd} recall`; "
            f"break retry loops via `{ap_cmd} guard`; preserve verified facts across compaction via `{ap_cmd} baton`."
        )
    body = "\n".join(bullets)
    return f"{FENCE_START}\n## Agent Primitives (`{ap_cmd} <tool>`)\n{body}\n{FENCE_END}\n"


def _upsert_fenced_block(md_path: Path, block: str) -> str:
    md_path.parent.mkdir(parents=True, exist_ok=True)
    if md_path.exists():
        orig = md_path.read_text(encoding="utf-8", errors="replace")
        pattern = re.compile(re.escape(FENCE_START) + r".*?" + re.escape(FENCE_END) + r"\n?", re.DOTALL)
        if pattern.search(orig):
            updated = pattern.sub(block, orig, count=1)
            md_path.write_text(updated, encoding="utf-8")
            return "updated"
        sep = "" if (not orig or orig.endswith("\n\n")) else ("\n" if orig.endswith("\n") else "\n\n")
        md_path.write_text(orig + sep + block, encoding="utf-8")
        return "appended"
    md_path.write_text(block, encoding="utf-8")
    return "created"


def _remove_fenced_block(md_path: Path) -> bool:
    if not md_path.exists():
        return False
    orig = md_path.read_text(encoding="utf-8", errors="replace")
    pattern = re.compile(r"\n*" + re.escape(FENCE_START) + r".*?" + re.escape(FENCE_END) + r"\n*", re.DOTALL)
    if not pattern.search(orig):
        return False
    cleaned = pattern.sub("\n", orig).strip()
    if not cleaned:
        md_path.unlink()
    else:
        md_path.write_text(cleaned + "\n", encoding="utf-8")
    return True


def _write_repo_dispatcher(dest_ap_py: Path) -> None:
    """Writes a tiny 25-line standalone `.agents/ap.py` dispatcher inside a repository."""
    code = '''#!/usr/bin/env python3
"""Local zero-dependency agent-primitives dispatcher."""
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
SKILLS = HERE / "skills"
TOOLS = ("sieve", "baton", "recall", "guard", "atlas", "probe", "splice")

def main() -> int:
    if len(sys.argv) < 2 or sys.argv[1] in ("-h", "--help"):
        avail = [t for t in TOOLS if (SKILLS / t / "scripts" / f"{t}.py").is_file()]
        sys.stdout.write(f"usage: python {HERE.name}/ap.py <{'|'.join(avail)}> [args...]\\n")
        return 0
    tool = sys.argv[1]
    script = SKILLS / tool / "scripts" / f"{tool}.py"
    if not script.is_file():
        sys.stderr.write(f"[ap] unknown or uninstalled primitive '{tool}'. Available: {', '.join(TOOLS)}\\n")
        return 2
    return subprocess.run([sys.executable, str(script), *sys.argv[2:]]).returncode

if __name__ == "__main__":
    raise SystemExit(main())
'''
    dest_ap_py.parent.mkdir(parents=True, exist_ok=True)
    dest_ap_py.write_text(code, encoding="utf-8")


def _install_global_ap_runtime(src_root: Path, home: Path) -> tuple[Path, Path]:
    """Installs full `ap.py` + `audit/audit_sessions.py` under `~/.agents/` + `~/.local/bin/ap` shims."""
    agents_dir = home / ".agents"
    agents_dir.mkdir(parents=True, exist_ok=True)
    global_ap = agents_dir / "ap.py"
    src_ap = src_root / "ap.py"
    if src_ap.is_file() and src_ap.resolve() != global_ap.resolve():
        shutil.copy2(src_ap, global_ap)
    elif not src_ap.is_file():
        _write_repo_dispatcher(global_ap)

    src_audit = src_root / "audit" / "audit_sessions.py"
    if src_audit.is_file():
        dst_audit = agents_dir / "audit" / "audit_sessions.py"
        dst_audit.parent.mkdir(parents=True, exist_ok=True)
        if src_audit.resolve() != dst_audit.resolve():
            shutil.copy2(src_audit, dst_audit)

    bin_dir = home / ".local" / "bin"
    bin_dir.mkdir(parents=True, exist_ok=True)
    ap_posix = _posix(global_ap)
    sh_shim = bin_dir / "ap"
    sh_shim.write_text(f'#!/usr/bin/env sh\nexec "{_posix(Path(sys.executable))}" "{ap_posix}" "$@"\n', encoding="utf-8")
    try:
        sh_shim.chmod(0o755)
    except OSError:
        pass
    cmd_shim = bin_dir / "ap.cmd"
    cmd_shim.write_text(f'@echo off\r\n"{sys.executable}" "{global_ap}" %*\r\n', encoding="utf-8")
    return global_ap, bin_dir


def _merge_claude_or_codex_hooks(settings_path: Path, skills_dir: Path, include_precompact: bool = True) -> str:
    guard_py = _posix(skills_dir / "guard" / "scripts" / "guard.py")
    baton_py = _posix(skills_dir / "baton" / "scripts" / "baton.py")
    data: dict[str, Any] = {}
    if settings_path.exists():
        shutil.copy2(settings_path, settings_path.with_suffix(".json.bak"))
        try:
            data = json.loads(settings_path.read_text(encoding="utf-8"))
        except Exception:
            data = {}
    hooks = data.setdefault("hooks", {})

    def _ensure_hook(event: str, cmd_str: str, matcher: str | None = None) -> None:
        arr = hooks.setdefault(event, [])
        if any(cmd_str in json.dumps(item) for item in arr):
            return
        entry: dict[str, Any] = {"hooks": [{"type": "command", "command": cmd_str}]}
        if matcher:
            entry["matcher"] = matcher
        arr.append(entry)

    if (skills_dir / "guard" / "scripts" / "guard.py").is_file():
        _ensure_hook("PreToolUse", f'python "{guard_py}" shield', matcher="Bash")
        _ensure_hook("Stop", f'python "{guard_py}" stop-gate')
    if include_precompact and (skills_dir / "baton" / "scripts" / "baton.py").is_file():
        _ensure_hook("PreCompact", f'python "{baton_py}" checkpoint --note "auto pre-compact snapshot"')

    settings_path.parent.mkdir(parents=True, exist_ok=True)
    settings_path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    return _posix(settings_path)


def _remove_claude_or_codex_hooks(settings_path: Path) -> bool:
    if not settings_path.exists():
        return False
    try:
        data = json.loads(settings_path.read_text(encoding="utf-8"))
    except Exception:
        return False
    hooks = data.get("hooks")
    if not isinstance(hooks, dict):
        return False
    changed = False
    for ev in list(hooks.keys()):
        arr = hooks.get(ev)
        if isinstance(arr, list):
            filtered = [
                item
                for item in arr
                if "guard/scripts/guard.py" not in json.dumps(item)
                and "baton/scripts/baton.py" not in json.dumps(item)
            ]
            if len(filtered) != len(arr):
                changed = True
            if filtered:
                hooks[ev] = filtered
            else:
                hooks.pop(ev, None)
    if not hooks:
        data.pop("hooks", None)
    if not changed:
        return False
    if not data:
        settings_path.unlink()
    else:
        settings_path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    return True


def _merge_antigravity_hooks(hooks_json_path: Path, skills_dir: Path) -> str:
    """Writes Antigravity's native named-group `hooks.json` schema (`run_command` matcher + flat `Stop`)."""
    guard_py = _posix(skills_dir / "guard" / "scripts" / "guard.py")
    if not (skills_dir / "guard" / "scripts" / "guard.py").is_file():
        return ""
    data: dict[str, Any] = {}
    if hooks_json_path.exists():
        shutil.copy2(hooks_json_path, hooks_json_path.with_suffix(".json.bak"))
        try:
            data = json.loads(hooks_json_path.read_text(encoding="utf-8"))
        except Exception:
            data = {}
    data["agent-primitives-guard"] = {
        "enabled": True,
        "PreToolUse": [
            {
                "matcher": "run_command",
                "hooks": [
                    {
                        "type": "command",
                        "command": f'python "{guard_py}" shield',
                        "timeout": 10,
                    }
                ],
            }
        ],
        "Stop": [
            {
                "type": "command",
                "command": f'python "{guard_py}" stop-gate',
                "timeout": 15,
            }
        ],
    }
    hooks_json_path.parent.mkdir(parents=True, exist_ok=True)
    hooks_json_path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    return _posix(hooks_json_path)


def _remove_antigravity_hooks(hooks_json_path: Path) -> bool:
    if not hooks_json_path.exists():
        return False
    try:
        data = json.loads(hooks_json_path.read_text(encoding="utf-8"))
    except Exception:
        return False
    if not isinstance(data, dict) or "agent-primitives-guard" not in data:
        return False
    data.pop("agent-primitives-guard", None)
    if not data:
        hooks_json_path.unlink()
        bak = hooks_json_path.with_suffix(".json.bak")
        if bak.exists():
            try:
                bak_data = json.loads(bak.read_text(encoding="utf-8"))
                if isinstance(bak_data, dict) and set(bak_data.keys()) <= {"agent-primitives-guard"}:
                    bak.unlink()
            except Exception:
                pass
    else:
        hooks_json_path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    return True


def _copy_selected_skills(src_skills: Path, dest_skills: Path, selected: list[str]) -> list[str]:
    dest_skills.mkdir(parents=True, exist_ok=True)
    copied: list[str] = []
    for name in selected:
        src = src_skills / name
        if not src.is_dir():
            raise FileNotFoundError(f"Unknown primitive folder: {name}")
        dst = dest_skills / name
        if src.resolve() == dst.resolve():
            copied.append(name)
            continue
        if dst.exists():
            shutil.rmtree(dst)
        shutil.copytree(src, dst)
        copied.append(name)
    return copied


def _resolve_requested_harnesses(agent_arg: str, home: Path, check_path_bins: bool) -> list[str]:
    raw = [a.strip().lower() for a in agent_arg.split(",") if a.strip()]
    if not raw or raw == ["auto"]:
        found: list[str] = []
        for cid, bin_name, probe_dirs, _, _ in HARNESS_SPECS:
            if any((home / d).exists() for d in probe_dirs) or (check_path_bins and shutil.which(bin_name)):
                found.append(cid)
        return found
    if raw == ["all"]:
        return [cid for cid, _, _, _, _ in HARNESS_SPECS]
    resolved: list[str] = []
    for item in raw:
        if item not in HARNESS_ALIASES:
            valid = sorted(set(HARNESS_ALIASES.keys()) | {"auto", "all"})
            raise ValueError(f"Unknown harness '{item}'. Choose from: {', '.join(valid)}")
        canon = HARNESS_ALIASES[item]
        if canon not in resolved:
            resolved.append(canon)
    return resolved


def cmd_init(args: argparse.Namespace) -> int:
    home = Path(args.home).expanduser().resolve()
    check_bins = home == Path.home().resolve()
    src_root = _ensure_repo_root(home)
    src_skills = src_root / "skills"

    selected = [s.strip() for s in args.only.split(",") if s.strip()]
    for s in selected:
        if s not in ALL_PRIMITIVES:
            sys.stderr.write(f"[ap init] unknown primitive '{s}'. Choose from: {', '.join(ALL_PRIMITIVES)}\n")
            return 2

    try:
        harnesses = _resolve_requested_harnesses(args.agent, home, check_path_bins=check_bins)
    except ValueError as verr:
        sys.stderr.write(f"[ap init] {verr}\n")
        return 2

    explicit_agents = args.agent.strip().lower() not in ("", "auto")

    if args.here or args.dir:
        repo_dir = Path(args.dir or ".").resolve()
        agents_root = repo_dir / ".agents"
        dest_skills = agents_root / "skills"
        copied = _copy_selected_skills(src_skills, dest_skills, selected)
        local_ap = agents_root / "ap.py"
        _write_repo_dispatcher(local_ap)

        extra_skill_dirs: list[str] = []
        if ("pi" in harnesses and explicit_agents) or (repo_dir / ".pi").is_dir():
            _copy_selected_skills(src_skills, repo_dir / ".pi" / "skills", selected)
            extra_skill_dirs.append(".pi/skills")
        if ("claude" in harnesses and explicit_agents) or (repo_dir / ".claude").is_dir():
            _copy_selected_skills(src_skills, repo_dir / ".claude" / "skills", selected)
            extra_skill_dirs.append(".claude/skills")
        if ("grok" in harnesses and explicit_agents) or (repo_dir / ".grok").is_dir():
            _copy_selected_skills(src_skills, repo_dir / ".grok" / "skills", selected)
            extra_skill_dirs.append(".grok/skills")

        rule_notes: list[str] = []
        if not args.no_rules:
            block = render_fenced_agents_block("python .agents/ap.py", selected)
            st_agents = _upsert_fenced_block(repo_dir / "AGENTS.md", block)
            rule_notes.append(f"AGENTS.md ({st_agents})")
            if ("claude" in harnesses and explicit_agents) or (repo_dir / "CLAUDE.md").exists():
                st_cl = _upsert_fenced_block(repo_dir / "CLAUDE.md", block)
                rule_notes.append(f"CLAUDE.md ({st_cl})")

        hook_paths: list[str] = []
        if args.with_hooks:
            claude_sk = (repo_dir / ".claude" / "skills") if (repo_dir / ".claude" / "skills").is_dir() else dest_skills
            if not explicit_agents or "claude" in harnesses:
                hook_paths.append(_merge_claude_or_codex_hooks(repo_dir / ".claude" / "settings.json", claude_sk, True))
            if explicit_agents and "codex" in harnesses:
                hook_paths.append(_merge_claude_or_codex_hooks(repo_dir / ".codex" / "hooks.json", dest_skills, False))
            if not explicit_agents or "antigravity" in harnesses:
                hp_agy = _merge_antigravity_hooks(repo_dir / ".agents" / "hooks.json", dest_skills)
                if hp_agy:
                    hook_paths.append(hp_agy)

        extras_str = f" (+ {', '.join(extra_skill_dirs)})" if extra_skill_dirs else ""
        rules_str = f"\n  - Routing rules wired -> {', '.join(rule_notes)}" if rule_notes else ""
        hooks_str = f"\n  - Hooks merged (with .bak backup) -> {', '.join(hook_paths)}" if hook_paths else ""
        sys.stdout.write(
            f"[ap init --here] Ready in {repo_dir.as_posix()}!\n"
            f"  - Installed {len(copied)} standalone primitives -> .agents/skills/{{{','.join(copied)}}}{extras_str}\n"
            f"  - Local CLI shim -> python .agents/ap.py <{'|'.join(copied)}>{rules_str}{hooks_str}\n"
        )
        return 0

    # Global multi-harness onboarding
    spec_map = {cid: (sdirs, rfile) for cid, _, _, sdirs, rfile in HARNESS_SPECS}
    skill_targets: list[Path] = [home / ".agents" / "skills"]
    rule_targets: list[Path] = []

    for cid in harnesses:
        if cid == "agents":
            continue
        sdirs, rfile = spec_map[cid]
        for sd in sdirs:
            tp = home / sd
            if tp not in skill_targets:
                skill_targets.append(tp)
        rp = home / rfile
        if rp not in rule_targets:
            rule_targets.append(rp)

    for st in skill_targets:
        _copy_selected_skills(src_skills, st, selected)

    global_ap, bin_dir = _install_global_ap_runtime(src_root, home)
    ap_cmd_global = f'python "{_posix(global_ap)}"'

    wired_rules: list[str] = []
    if not args.no_rules:
        global_block = render_fenced_agents_block(ap_cmd_global, selected)
        if not rule_targets:
            rule_targets.append(home / ".agents" / "AGENTS.md")
        for rt in rule_targets:
            st_r = _upsert_fenced_block(rt, global_block)
            wired_rules.append(f"{_posix(rt)} ({st_r})")

    hook_paths_global: list[str] = []
    if args.with_hooks:
        active_set = set(harnesses)
        if not active_set or "claude" in active_set:
            sk_cl = (home / ".claude" / "skills") if (home / ".claude" / "skills").is_dir() else (home / ".agents" / "skills")
            hook_paths_global.append(_merge_claude_or_codex_hooks(home / ".claude" / "settings.json", sk_cl, True))
        if "codex" in active_set:
            hook_paths_global.append(_merge_claude_or_codex_hooks(home / ".codex" / "hooks.json", home / ".agents" / "skills", False))
        if "antigravity" in active_set:
            hp_agy = _merge_antigravity_hooks(home / ".gemini" / "config" / "hooks.json", home / ".agents" / "skills")
            if hp_agy:
                hook_paths_global.append(hp_agy)

    rules_line = f"\n  - Global routing blocks wired: {', '.join(wired_rules)}" if wired_rules else ""
    hooks_line = f"\n  - Hooks merged (with .bak backup): {', '.join(hook_paths_global)}" if hook_paths_global else ""
    sys.stdout.write(
        f"[ap init] Global onboarding complete!\n"
        f"  - Harnesses configured: {', '.join(harnesses) if harnesses else 'agents (~/.agents)'}\n"
        f"  - Synced {len(selected)} primitives into {len(skill_targets)} skill directory(s): {', '.join(_posix(t) for t in skill_targets)}\n"
        f"  - Installed CLI launcher: {_posix(bin_dir / 'ap')} (+ ap.cmd) & {_posix(global_ap)}{rules_line}{hooks_line}\n"
    )
    return 0


def cmd_uninstall(args: argparse.Namespace) -> int:
    if args.here or args.dir:
        repo_dir = Path(args.dir or ".").resolve()
        removed: list[str] = []
        for sub_root in (repo_dir / ".agents" / "skills", repo_dir / ".pi" / "skills", repo_dir / ".claude" / "skills", repo_dir / ".grok" / "skills"):
            for name in ALL_PRIMITIVES:
                p = sub_root / name
                if p.exists():
                    shutil.rmtree(p)
                    if name not in removed:
                        removed.append(name)
            if sub_root.is_dir() and not any(sub_root.iterdir()):
                sub_root.rmdir()

        local_ap = repo_dir / ".agents" / "ap.py"
        if local_ap.exists():
            local_ap.unlink()
        shutil.rmtree(repo_dir / ".agents" / "__pycache__", ignore_errors=True)
        _remove_antigravity_hooks(repo_dir / ".agents" / "hooks.json")
        _remove_claude_or_codex_hooks(repo_dir / ".claude" / "settings.json")
        _remove_claude_or_codex_hooks(repo_dir / ".codex" / "hooks.json")

        for dot_dir in (repo_dir / ".agents", repo_dir / ".pi", repo_dir / ".claude", repo_dir / ".codex", repo_dir / ".grok"):
            if dot_dir.is_dir() and not any(dot_dir.iterdir()):
                dot_dir.rmdir()

        md_removed = _remove_fenced_block(repo_dir / "AGENTS.md")
        _remove_fenced_block(repo_dir / "CLAUDE.md")
        sys.stdout.write(
            f"[ap uninstall --here] Removed {len(removed)} primitives from {repo_dir.as_posix()} "
            f"(AGENTS.md block removed={md_removed}).\n"
        )
        return 0

    home = Path(args.home).expanduser().resolve()
    count = 0
    all_skill_dirs = {home / ".agents" / "skills"}
    all_rule_files = {home / ".agents" / "AGENTS.md"}
    for _, _, _, sdirs, rfile in HARNESS_SPECS:
        for sd in sdirs:
            all_skill_dirs.add(home / sd)
        all_rule_files.add(home / rfile)

    for st in sorted(all_skill_dirs):
        for name in ALL_PRIMITIVES:
            p = st / name
            if p.exists():
                shutil.rmtree(p)
                count += 1

    rules_cleaned = 0
    for rf in sorted(all_rule_files):
        if _remove_fenced_block(rf):
            rules_cleaned += 1

    _remove_claude_or_codex_hooks(home / ".claude" / "settings.json")
    _remove_claude_or_codex_hooks(home / ".codex" / "hooks.json")
    _remove_antigravity_hooks(home / ".gemini" / "config" / "hooks.json")

    for f in (
        home / ".agents" / "ap.py",
        home / ".agents" / "audit" / "audit_sessions.py",
        home / ".local" / "bin" / "ap",
        home / ".local" / "bin" / "ap.cmd",
    ):
        if f.exists():
            f.unlink()
    if (home / ".agents" / "audit").is_dir() and not any((home / ".agents" / "audit").iterdir()):
        (home / ".agents" / "audit").rmdir()
    if (home / ".agent-primitives").is_dir():
        shutil.rmtree(home / ".agent-primitives", ignore_errors=True)

    sys.stdout.write(
        f"[ap uninstall] Removed {count} installed skill folder(s), {rules_cleaned} global rule block(s), and `ap` shims.\n"
    )
    return 0


def cmd_doctor(home: Path) -> int:
    src_skills = _ensure_repo_root(home) / "skills"
    ok = sys.version_info >= (3, 9)
    sys.stdout.write(f"[ap doctor] Python {sys.version.split()[0]} ({sys.platform}) -> {'OK' if ok else 'NEED >=3.9'}\n")
    try:
        conn = sqlite3.connect(":memory:")
        conn.execute("CREATE VIRTUAL TABLE t USING fts5(x);")
        sys.stdout.write("[ap doctor] stdlib sqlite3 FTS5/BM25 -> OK\n")
    except Exception as exc:
        sys.stdout.write(f"[ap doctor] stdlib sqlite3 FTS5 -> FALLBACK ({exc})\n")
    for name in ALL_PRIMITIVES:
        script = src_skills / name / "scripts" / f"{name}.py"
        exists = script.is_file()
        ok = ok and exists
        sys.stdout.write(f"[ap doctor] primitive '{name}' -> {'OK' if exists else 'MISSING'}\n")
    return 0 if ok else 1


def cmd_status(repo_dir: Path, home: Path) -> int:
    local_skills = [n for n in ALL_PRIMITIVES if (repo_dir / ".agents" / "skills" / n / "scripts" / f"{n}.py").is_file()]
    has_fence = (repo_dir / "AGENTS.md").is_file() and FENCE_START in (repo_dir / "AGENTS.md").read_text(encoding="utf-8", errors="replace")
    sys.stdout.write(
        f"[ap status]\n"
        f"  - Repo ({repo_dir.as_posix()}): {len(local_skills)}/7 primitives ({', '.join(local_skills) or 'none'}), AGENTS.md wired={has_fence}\n"
    )
    for cid, _, _, sdirs, rfile in HARNESS_SPECS:
        st_dir = home / sdirs[0]
        installed = [n for n in ALL_PRIMITIVES if (st_dir / n / "scripts" / f"{n}.py").is_file()]
        rf = home / rfile
        rf_wired = rf.is_file() and FENCE_START in rf.read_text(encoding="utf-8", errors="replace")
        if installed or rf_wired:
            sys.stdout.write(f"  - Global [{cid}] ({_posix(st_dir)}): {len(installed)}/7 primitives, rules wired={rf_wired}\n")
    return 0


def main(argv: list[str] | None = None) -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    if hasattr(sys.stderr, "reconfigure"):
        sys.stderr.reconfigure(encoding="utf-8", errors="replace")

    raw_args = list(sys.argv[1:] if argv is None else argv)

    # Fast pass-through when invoked as `ap <primitive> [args...]` or `ap audit [args...]`
    if raw_args and raw_args[0] in ALL_PRIMITIVES:
        tool = raw_args[0]
        script = _ensure_repo_root(Path.home()) / "skills" / tool / "scripts" / f"{tool}.py"
        return subprocess.run([sys.executable, str(script), *raw_args[1:]]).returncode

    if raw_args and raw_args[0] == "audit":
        root = _ensure_repo_root(Path.home())
        audit_py = root / "audit" / "audit_sessions.py"
        if not audit_py.is_file():
            sys.stderr.write("[ap audit] audit_sessions.py not found.\n")
            return 2
        return subprocess.run([sys.executable, str(audit_py), *raw_args[1:]]).returncode

    parser = argparse.ArgumentParser(
        prog="ap",
        description="1-Line Zero-Clone Bootstrapper, Multi-Harness Installer & Unified CLI for agent-primitives",
    )
    sub = parser.add_subparsers(dest="cmd", required=True)

    p_init = sub.add_parser("init", help="Auto-detect harnesses globally or wire current repo (--here) in 1 command")
    p_init.add_argument("--here", action="store_true", help="Install into current repo (.agents/skills + .agents/ap.py + AGENTS.md)")
    p_init.add_argument("--dir", default="", help="Explicit repository directory for --here setup")
    p_init.add_argument("--home", default=str(Path.home()), help="Override user home directory for global init")
    p_init.add_argument(
        "-a",
        "--agent",
        default="auto",
        help="Target harness(es): auto (default), all, or comma-separated codex,pi,agy,claude,grok,openclaw,hermes",
    )
    p_init.add_argument("--only", default=",".join(ALL_PRIMITIVES), help="Comma-separated subset of primitives to install")
    p_init.add_argument("--no-rules", action="store_true", help="Install skills & CLI only without modifying AGENTS.md/CLAUDE.md/GEMINI.md")
    p_init.add_argument("--with-hooks", action="store_true", help="Non-destructively merge guard/baton hooks (Claude, Codex, Antigravity)")

    p_un = sub.add_parser("uninstall", help="Cleanly remove installed skills, shims, hooks, and fenced instruction blocks")
    p_un.add_argument("--here", action="store_true", help="Remove from current repo")
    p_un.add_argument("--dir", default="", help="Explicit repository directory for --here uninstall")
    p_un.add_argument("--home", default=str(Path.home()), help="Override user home directory for global uninstall")

    p_doc = sub.add_parser("doctor", help="Verify Python >=3.9, SQLite FTS5/BM25, and all 7 standalone primitives")
    p_doc.add_argument("--home", default=str(Path.home()), help="User home directory")

    p_st = sub.add_parser("status", help="Show local repo and per-harness global onboarding status")
    p_st.add_argument("--dir", default=".", help="Repository directory to inspect")
    p_st.add_argument("--home", default=str(Path.home()), help="User home directory")

    args = parser.parse_args(raw_args)
    if args.cmd == "init":
        return cmd_init(args)
    if args.cmd == "uninstall":
        return cmd_uninstall(args)
    if args.cmd == "doctor":
        return cmd_doctor(Path(args.home).expanduser().resolve())
    if args.cmd == "status":
        return cmd_status(Path(args.dir).resolve(), Path(args.home).expanduser().resolve())
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
