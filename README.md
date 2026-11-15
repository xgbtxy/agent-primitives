# agent-primitives

**One Folder = One Zero-Bloat Micro-Infrastructure for Coding Agents & Personal Assistant Agents.**

[English (Default)](./README.md) | [简体中文](./README.zh-CN.md) · [Efficiency Playbook (EN)](./PLAYBOOK.md) | [真实提效实战指南 (中文)](./PLAYBOOK.zh-CN.md)

---

## Why This Repository Exists

Across real 2026 agent workflows on GitHub and X, developers hit an uncomfortable split:
- **Pure-Markdown skill repos** are lightweight to copy, but have **zero deterministic runtime enforcement** (telling an LLM *"please verify before finishing"* in prose still lets it declare `"Done!"` on broken tests, and cannot losslessly compress a 200 KB log).
- **Full harness frameworks & memory plugins** have runtime tooling, but bloat into 50+ root files requiring `Node`/`Bun`, `Docker`, vector databases, paid cloud observers, or always-on MCP schemas that eat `8,000–15,000 tokens` before turn 1.

**`agent-primitives`** takes a strict micro-infrastructure approach: **one subfolder = one standalone mini-project**.
- **100% isolated & copyable**: Copy *just the single folder you want* (`cp -r skills/sieve ~/.agents/skills/`). Zero cross-folder imports.
- **Zero third-party dependencies**: Pure Python 3.9+ standard library (`sqlite3` FTS5, `ast`, `hashlib`, `html.parser`, `subprocess`) across Linux, macOS, and Windows.
- **360 tokens total idle cost**: Each folder's `SKILL.md` frontmatter costs only **`63–83 tokens`** at rest (`360 tokens` for all five combined).

---

## The 5 Mini-Projects (`skills/*`)

Click any folder for its standalone English & Chinese documentation:

| Mini-Project Folder | Core Bottleneck Solved | For Coding Agents | For Personal Assistant Agents | Idle Cost |
| :--- | :--- | :--- | :--- | ---: |
| **[`skills/sieve/`](./skills/sieve/README.md)**<br>([中文](./skills/sieve/README.zh-CN.md)) | **Context Rot** from huge logs/JSON/HTML + lost frames from lossy CLI proxies | Compress `pytest`/`cargo`/CI logs by **98.8%**; `peek <id>` exact lines/regex in `<10ms` without re-running | Compress 400-item API JSON (**-99.6%**) & noisy HTML pages (**-98.4%**) with lossless `--json-path` / line `peek` | `83 tok` |
| **[`skills/baton/`](./skills/baton/README.md)**<br>([中文](./skills/baton/README.zh-CN.md)) | **Compaction amnesia** & state loss across sessions, git worktrees, or switching agents | Hand off verified facts, git dirty state & **`DO NOT RETRY` dead-ends** across Claude Code ↔ Codex ↔ Grok | Resume multi-day research, checklists & personal workflows after compaction in `<400 tokens` (zero cloud/LLM tax) | `70 tok` |
| **[`skills/recall/`](./skills/recall/README.md)**<br>([中文](./skills/recall/README.zh-CN.md)) | **Retrieval bugs** & `AGENTS.md` / `MEMORY.md` bloat burning thousands of tokens every turn | `<5ms` stdlib `sqlite3` FTS5/BM25 + path globs (`src/auth/*` -> inject only the `~40-token` subsystem gotcha) | `<5ms` bilingual (English + CJK) rule recall for contacts, invoicing/finance SOPs & server ops | `71 tok` |
| **[`skills/guard/`](./skills/guard/README.md)**<br>([中文](./skills/guard/README.zh-CN.md)) | **False `"Done!"` claims** (`exit 1` is non-blocking!), 5x repetition loops & leaked secrets | Native **`exit 2`** Stop-gate, 3x identical-failure loop breaker & `git push --force` shield | Block exposed API keys/tokens, `rm -rf` wipes & unbounded outbound email/webhook blasts with `exit 2` | `63 tok` |
| **[`skills/atlas/`](./skills/atlas/README.md)**<br>([中文](./skills/atlas/README.zh-CN.md)) | **Cold-start wandering** (`50k–130k tok`) & `+77%` token waste chasing stale old TODOs | `0.08s` AST class/function signatures, hub centrality, entrypoints & `>90-day` stale-TODO trap warnings | `<800-token` structural map & `H1/H2` heading outline of large Markdown vaults and document folders | `73 tok` |

---

## How We Should Actually Use Them for Real Efficiency

> Full step-by-step client configurations, Hook JSON, and workflows: **[PLAYBOOK.md (English)](./PLAYBOOK.md)** · **[PLAYBOOK.zh-CN.md (简体中文)](./PLAYBOOK.zh-CN.md)**

Tools only save time and tokens when wired to the **5 real chokepoints** of daily agent work—without adding ceremony to trivial commands:

### 1. 60-Second Install & Wiring (`install.py`)

```bash
# Verify Python 3.9+ and built-in SQLite FTS5
python install.py --doctor

# Install all 5 folders—or pick a single folder with --only sieve
python install.py --agent agents --print-agents-md   # ~/.agents/skills (Antigravity, Codex, Cursor, OpenCode)
python install.py --agent claude --print-hooks       # ~/.claude/skills + ready-to-paste Hook JSON
python install.py --agent grok                       # ~/.grok/skills   (Grok Build CLI)
python install.py --agent openclaw                   # ~/.openclaw/skills (OpenClaw Personal Assistant)
python install.py --agent hermes                     # ~/.hermes/skills (Hermes Agent)

# Or via the open Agent Skills CLI:
npx skills add xgbtxy/agent-primitives --skill sieve
```

### 2. Daily Operating Loop (Coding Agents & Personal Assistants)

1. **Orient First (`atlas`)**: Start unfamiliar repo or document-vault sessions with `python skills/atlas/scripts/atlas.py map .` (`<800 tokens` in `<0.1s`) instead of burning 10+ `ls`/`grep`/`cat` turns, and ignore flagged `>90-day` stale TODO traps unless explicitly asked.
2. **Slim `AGENTS.md` + Triggered `recall`**: Keep global `AGENTS.md` / `SOUL.md` under **60 lines** (universal invariants only). Move all path-specific coding gotchas and client/topic personal SOPs into `recall` (`recall.py match` injects `~40 tokens` in `3.7 ms`; `recall.py add` captures new lessons so you never debug the same mistake twice).
3. **Never Pour `>2 KB` Raw Noise into Context (`sieve`)**: Wrap test suites, CI logs, verbose builds, large JSON APIs, and HTML scrapes in `sieve.py run -- <cmd>` or `<cmd> | python sieve.py pipe`. Inspect exact stack frames or JSON items on demand via `sieve.py peek <id> --lines / --grep / --json-path` with **zero re-execution**.
4. **Enforce Deterministic `exit 2` Guardrails (`guard`)**: Attach `guard.py shield` to `PreToolUse` (blocks secret leaks & destructive commands), `guard.py run --max-repeats 3` around flaky commands (halts repetition loops on the 3rd identical failure), and `guard.py stop-gate` on `Stop` hooks so the agent cannot claim completion while checks fail.
5. **Survive Compaction & Cross-Agent Switches (`baton`)**: Hook `baton.py checkpoint` to `PreCompact`, log falsified hypotheses via `baton.py deadend`, and run `baton.py resume` (`<400 tokens`) after `/compact` or when handing work between Claude Code, Codex, Antigravity, Grok Build, or OpenClaw.

### 3. Honest Anti-Bloat Rules (When NOT to Use)

- **Skip `sieve` for tiny outputs** (`git status -s`, `pwd`, 30-line files)—`sieve` passes `<1.2 KB` streams through untouched anyway; reserve it for commands/APIs that may emit `>2 KB`.
- **Keep universal rules in `AGENTS.md`, conditional rules in `recall`**—don't query a database for a rule needed on 100% of turns, and don't bloat `AGENTS.md` with 200 rules needed on 2% of turns.
- **Keep `baton` surgical (`<400 tokens`)**—record verified facts (`file:line`) and dead-ends (`DO NOT RETRY`), never raw chat transcripts.

---

## Reproducible Benchmarks (Including Honest Negative Controls)

Run `python bench/run_bench.py` locally to reproduce [`bench/results.json`](./bench/results.json) (Python 3.12, deterministic seed `20261115`, zero external packages):

| Primitive & Workload | Raw Input | Injected Context | Reduction | Latency | Fidelity / Accuracy |
| :--- | ---: | ---: | ---: | ---: | :--- |
| **`sieve`** · 2,500-line CI/pytest log | `172,075 B` (`~43,019 tok`) | `2,080 B` (`~520 tok`) | **-98.8%** | `23.9 ms` | 100% exceptions preserved + lossless `peek` |
| **`sieve`** · 400-record JSON API payload | `107,887 B` (`~26,972 tok`) | `423 B` (`~106 tok`) | **-99.6%** | `2.7 ms` | Schema + warnings preserved + `--json-path` peek |
| **`sieve`** · Noisy HTML page (JS/CSS bloat) | `48,160 B` (`~12,040 tok`) | `780 B` (`~195 tok`) | **-98.4%** | `13.5 ms` | Title, H1–H3 outline & lead body preserved |
| **`sieve`** · Short `<1.2 KB` output *(Negative Control)* | `69 B` (`~18 tok`) | `117 B` (`~30 tok`) | **0.0%** *(passthrough)* | `2.0 ms` | Short outputs pass through untouched |
| **`recall`** · 200-rule vault (EN + CJK hybrid) | `6,380 tok` *(monolith)* | `39.9 tok` *(top-2)* | **-99.4%** | `3.7 ms` | **20/20 (100%)** Top-1 accuracy |
| **`atlas`** · 64-module SDK corpus (`bench/run_bench.py`) | `42,710 tok` *(full read)* | `446 tok` *(structural map)* | **-99.0%** | `22.5 ms` | AST symbols, hub modules & entrypoints |

---

## Repository Layout & Verification

```text
agent-primitives/
├── README.md                  # Default English documentation & workflow guide
├── README.zh-CN.md            # Full Simplified Chinese documentation & workflow guide
├── PLAYBOOK.md                # Deep-dive efficiency playbook (Coding + Personal Agents)
├── PLAYBOOK.zh-CN.md          # 真实提效实战指南（完整中文版）
├── install.py                 # Zero-dependency selective installer, hook generator & doctor
├── skills/                    # One folder = one standalone zero-dependency mini-project
│   ├── sieve/                 # Lossless output compressor & flight recorder
│   ├── baton/                 # Compaction-proof state & cross-agent handoff capsule
│   ├── recall/                # Sub-5ms SQLite FTS5/BM25 triggered micro-memory
│   ├── guard/                 # Exit-2 truth gate, loop breaker & secret/blast shield
│   └── atlas/                 # Sub-second AST & vault structural map + stale-TODO radar
├── bench/run_bench.py         # Deterministic benchmark suite + negative controls
└── tests/test_primitives.py   # Single-folder isolation tests + end-to-end verification
```

```bash
python -m unittest discover -s tests -v
```

## License

[MIT](./LICENSE)
