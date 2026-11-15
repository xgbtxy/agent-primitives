#!/usr/bin/env python3
"""install.py - Zero-dependency installer, hook generator & doctor for agent-primitives.

Lets you install a single folder (`--only sieve`) or all 5 folders into any
Coding Agent or Personal Assistant Agent skills directory on Linux, macOS, or
Windows, and generates ready-to-paste Hook JSON + minimal AGENTS.md wiring.
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
SKILLS_DIR = ROOT / "skills"
ALL_PRIMITIVES = ("sieve", "baton", "recall", "guard", "atlas", "probe", "splice")

AGENT_TARGETS: dict[str, str] = {
    "agents": "~/.agents/skills",              # Universal Agent Skills (Antigravity, Codex, OpenCode, Cursor)
    "antigravity": "~/.gemini/config/skills",  # Google Antigravity IDE & CLI
    "claude": "~/.claude/skills",              # Anthropic Claude Code
    "codex": "~/.codex/skills",                # OpenAI Codex CLI
    "pi": "~/.pi/agent/skills",                # Pi (pi-coding-agent)
    "grok": "~/.grok/skills",                  # xAI Grok Build CLI
    "openclaw": "~/.openclaw/skills",          # OpenClaw Personal Assistant
    "hermes": "~/.hermes/skills",              # Hermes Personal/Research Agent
}


def _posix_cmd_path(p: Path) -> str:
    return str(p.resolve()).replace("\\", "/")


def render_claude_hooks(dest_skills_dir: Path) -> str:
    guard_py = _posix_cmd_path(dest_skills_dir / "guard" / "scripts" / "guard.py")
    baton_py = _posix_cmd_path(dest_skills_dir / "baton" / "scripts" / "baton.py")
    cfg = {
        "hooks": {
            "PreToolUse": [
                {
                    "matcher": "Bash",
                    "hooks": [
                        {
                            "type": "command",
                            "command": f'python "{guard_py}" shield',
                        }
                    ],
                }
            ],
            "PreCompact": [
                {
                    "hooks": [
                        {
                            "type": "command",
                            "command": f'python "{baton_py}" checkpoint --note "auto pre-compact snapshot"',
                        }
                    ]
                }
            ],
            "Stop": [
                {
                    "hooks": [
                        {
                            "type": "command",
                            "command": f'python "{guard_py}" stop-gate',
                        }
                    ]
                }
            ],
        }
    }
    return json.dumps(cfg, indent=2)


def render_agents_md_snippet(dest_skills_dir: Path) -> str:
    base = _posix_cmd_path(dest_skills_dir)
    return f"""## Runtime Efficiency Primitives (Zero-Bloat Protocol)
1. **Cold Start (`atlas`)**: Before running broad `find`/`grep` in an unfamiliar repo or vault, run `python "{base}/atlas/scripts/atlas.py" map .` (<800 tok). Ignore stale TODO traps unless asked.
2. **Causal Reachability & Runtime Microscope (`probe`)**: Before hunting across multi-file codebases or mentally simulating state/aliasing in prose, run `python "{base}/probe/scripts/probe.py" slice . --entry <pkg>` to prune dead modules, `probe.py run --cwd . --focus <pkg> --code "<repro>"` (or `--stdin`) to surface `[ALIAS HAZARD]`, `[IN-PLACE MUTATION]`, and `[STATE NO-OP]`, and `probe.py diff` to verify boundary/cache equivalence.
3. **Surgical AST Read & Atomic Patch (`splice`)**: Inspect individual symbols/sections via `python "{base}/splice/scripts/splice.py" view <file> --symbol <Class.method>` instead of re-reading whole files; apply multi-file edits atomically with UTF-8/CRLF safety, syntax rollback, and `.pyc` cleanup via `splice.py apply --cwd . --stdin` and verify with `splice.py check .`.
4. **Targeted Memory (`recall`)**: Before editing tricky subsystems or executing recurring personal SOPs, query `python "{base}/recall/scripts/recall.py" match --query "<task>" --files "<paths>"`. Store new reusable lessons with `recall.py add`.
5. **Noisy Outputs (`sieve`)**: Wrap commands/scrapes/APIs that may emit >2 KB via `python "{base}/sieve/scripts/sieve.py" run -- <cmd>` (or pipe to `sieve.py pipe`). Inspect details losslessly via `sieve.py peek <id> --lines/--grep`.
6. **Anti-Loop & Truth Gate (`guard`)**: Never repeat an identical failing command >2 times (`guard.py run --max-repeats 3 -- <cmd>`). Before declaring completion, verify via `python "{base}/guard/scripts/guard.py" stop-gate`.
7. **Compaction & Handoff (`baton`)**: Record verified facts (`baton.py fact`) and falsified hypotheses (`baton.py deadend`) during multi-step work; run `baton.py resume` immediately after context compaction or session handoff.
"""


def install_to(dest_dir: Path, selected: list[str]) -> list[Path]:
    dest_dir.mkdir(parents=True, exist_ok=True)
    installed: list[Path] = []
    for name in selected:
        src = SKILLS_DIR / name
        if not src.exists():
            raise FileNotFoundError(f"Unknown primitive folder: {name}")
        target = dest_dir / name
        if target.exists():
            shutil.rmtree(target)
        shutil.copytree(src, target)
        installed.append(target)
    return installed


def run_doctor() -> int:
    ok = sys.version_info >= (3, 9)
    sys.stdout.write(f"[doctor] Python {sys.version.split()[0]} ({sys.platform}) -> {'OK' if ok else 'NEED >=3.9'}\n")
    try:
        import sqlite3
        conn = sqlite3.connect(":memory:")
        conn.execute("CREATE VIRTUAL TABLE t USING fts5(x);")
        sys.stdout.write("[doctor] stdlib sqlite3 FTS5/BM25 -> OK\n")
    except Exception as exc:
        sys.stdout.write(f"[doctor] stdlib sqlite3 FTS5 -> FALLBACK ({exc})\n")
    for name in ALL_PRIMITIVES:
        script = SKILLS_DIR / name / "scripts" / f"{name}.py"
        sys.stdout.write(f"[doctor] primitive '{name}' ({script.relative_to(ROOT)}) -> {'OK' if script.exists() else 'MISSING'}\n")
    return 0 if ok else 1


def main(argv: list[str] | None = None) -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")

    parser = argparse.ArgumentParser(prog="install.py", description="Install 1 or all agent-primitives folders")
    parser.add_argument(
        "--only",
        default=",".join(ALL_PRIMITIVES),
        help="Comma-separated folders to install (e.g. sieve,baton,guard)",
    )
    parser.add_argument(
        "--agent",
        choices=sorted(AGENT_TARGETS.keys()),
        help="Named agent target (~/.agents/skills, ~/.claude/skills, ~/.openclaw/skills, etc.)",
    )
    parser.add_argument("--dest", help="Explicit destination skills directory path")
    parser.add_argument("--print-hooks", action="store_true", help="Print ready-to-paste Hook JSON config")
    parser.add_argument("--print-agents-md", action="store_true", help="Print 10-line AGENTS.md / CLAUDE.md snippet")
    parser.add_argument("--doctor", action="store_true", help="Verify local Python stdlib & primitive health")

    args = parser.parse_args(argv)

    if args.doctor:
        return run_doctor()

    selected = [s.strip() for s in args.only.split(",") if s.strip()]
    for s in selected:
        if s not in ALL_PRIMITIVES:
            sys.stderr.write(f"[install] unknown folder '{s}'. Choose from: {', '.join(ALL_PRIMITIVES)}\n")
            return 2

    target_dir: Path | None = None
    if args.dest:
        target_dir = Path(args.dest).expanduser()
    elif args.agent:
        target_dir = Path(AGENT_TARGETS[args.agent]).expanduser()

    ref_dir = target_dir if target_dir else SKILLS_DIR

    if target_dir:
        installed = install_to(target_dir, selected)
        sys.stdout.write(f"[install] installed {len(installed)} folder(s) into {target_dir}:\n")
        for p in installed:
            sys.stdout.write(f"  - {p.name}/ ({_posix_cmd_path(p)})\n")

    if args.print_hooks:
        sys.stdout.write("\n# Ready-to-paste Hook Config (.claude/settings.json or compatible harness):\n")
        sys.stdout.write(render_claude_hooks(ref_dir) + "\n")

    if args.print_agents_md:
        sys.stdout.write("\n# Ready-to-paste AGENTS.md / CLAUDE.md / SOUL.md Protocol:\n")
        sys.stdout.write(render_agents_md_snippet(ref_dir))

    if not (target_dir or args.print_hooks or args.print_agents_md):
        parser.print_help()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
