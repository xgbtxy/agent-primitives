# agent-primitives

**One Folder = One Anti-Obsolescence, Zero-Dependency Micro-Infrastructure for Coding Agents & Personal Assistant Agents.**

[English (Default)](./README.md) | [简体中文](./README.zh-CN.md) · [Anti-Obsolescence Architecture (EN)](./DESIGN.md) | [抗淘汰底层设计 (中文)](./DESIGN.zh-CN.md) · [Efficiency Playbook (EN)](./PLAYBOOK.md) | [实战指南 (中文)](./PLAYBOOK.zh-CN.md)

---

## Why This Repository Exists (And Why Model Upgrades Won't Replace It)

95% of agent repositories become obsolete within 6 months because they build **"thinking crutches"** (prompt roleplay, planning checklists, heavy orchestration wrappers, or paid vector RAG) that get absorbed into the next frontier model's post-training weights or official CLI runtime.

**`agent-primitives`** is built from empirical process telemetry across **`69,953` real local tool calls** (`Grok Build CLI`, `OpenAI Codex CLI`, `Google Antigravity CLI`, `Claude Code`). Regardless of how smart `GPT-6`, `Grok-5`, `Claude-5`, or `Gemini-4` becomes, an autoregressive token stream still collides with **four irreducible physical boundaries**:

1. **External OS/Web Entropy vs. `O(N)` KV-Cache Physics** — Dumping `170 KB` CI logs, `400`-item JSON payloads, or `20` dead unreferenced files into context physically inflates prefill latency and bills every subsequent turn.
2. **Prose Simulation vs. Concrete Runtime Heap & Types** — Mentally simulating shallow-copy dict aliasing, in-place state mutations, or `None`/`bool`/`int` (`True == 1`) collisions burns `35k–55k` reasoning tokens (`11–19 min`), while ad-hoc `python -c` one-liners hit shell-quoting and `ImportError` crashes (`2,716` inline probes & `177` crashes audited locally).
3. **BPE Token Generation vs. Byte/AST-Exact File Transactions** — Line shifts force `50.1%` redundant same-file re-reads, CRLF/whitespace patch mismatches (`528` audited), and Windows `cp936`/UTF-8 decode crashes (`160` audited).
4. **Compaction Amnesia & Self-Report Bias** — `/compact` erases falsified dead-ends, and prose rules cannot enforce a blocking **`exit 2`** truth gate after edits.

**Design Law**: **One subfolder = one standalone zero-dependency deterministic OS/AST/Heap/SQLite primitive** (`cp -r skills/probe ~/.agents/skills/`). Pure Python 3.9+ stdlib only (`0` pip packages, `0` daemons, `0` API keys, **`492 tokens`** total idle frontmatter cost across all 7 folders combined). Smarter models don't replace deterministic instruments—they wield them faster.

---

## Step 0: Measure Your Own Local Agent Waste in Seconds (`audit/`)

Before installing anything, run the **100% local, zero-cloud telemetry auditor** against your own `~/.codex`, `~/.claude`, `~/.grok`, and `~/.gemini/antigravity-cli` session logs:

```bash
python audit/audit_sessions.py
```

Example output from our local workstation (`178` recent sessions, `69,953` tool calls, `0` bytes uploaded):
- **`skills/probe` triggered**: `2,716` inline `python -c` probes, `106` `ImportError`/`sys.path` failures, `71` shell-quoting crashes
- **`skills/splice` triggered**: `679` redundant same-file re-reads, `3,918` manual shell line-slices, `528` patch mismatches, `160` UTF-8/CP936 crashes
- **`skills/sieve` triggered**: `17,110` tool outputs `>8 KB`, `171` truncation spills
- **`skills/atlas` triggered**: `2,994` broad shell search/listing calls
- **`skills/guard + baton` triggered**: `26` identical failing retry loops

---

## The 7 Standalone Mini-Projects Mapped to the 5-Stage Agent Loop

```text
[audit/audit_sessions.py] ──► Diagnose real local friction (0 cloud upload)
  ├─► Stage 1: Orient & Recall   ──► skills/atlas/  +  skills/recall/
  ├─► Stage 2: Slice & Diagnose  ──► skills/probe/   (slice · run · diff)
  ├─► Stage 3: Surgical Act      ──► skills/splice/  (view · apply · check)
  ├─► Stage 4: Filter Output     ──► skills/sieve/   (run · pipe · peek)
  └─► Stage 5: Gate & Persist    ──► skills/guard/  +  skills/baton/
```

| Mini-Project Folder | Physical Boundary Solved | For Coding Agents | For Personal Assistant Agents | Idle Cost |
| :--- | :--- | :--- | :--- | ---: |
| **[`skills/atlas/`](./skills/atlas/README.md)**<br>([中文](./skills/atlas/README.zh-CN.md)) | **Cold-start wandering** & `+77%` token waste chasing stale old TODOs | `~20ms` AST signatures, hub centrality, entrypoints & `>90-day` stale-TODO trap warnings | `<800-token` structural map & `H1/H2` outline of large Markdown vaults and note folders | `73 tok` |
| **[`skills/recall/`](./skills/recall/README.md)**<br>([中文](./skills/recall/README.zh-CN.md)) | **Static `AGENTS.md` / `MEMORY.md` bloat** burning thousands of tokens/turn | `<4ms` stdlib `sqlite3` FTS5/BM25 + path globs (`src/auth/*` -> inject `~40-token` gotcha) | `<4ms` bilingual (English + CJK) rule recall for contacts, invoicing SOPs & server ops | `71 tok` |
| **[`skills/probe/`](./skills/probe/README.md)**<br>([中文](./skills/probe/README.zh-CN.md)) | **Dead-module reading, `50k`-token prose heap guessing** & `python -c` crashes | `slice` prunes dead modules (`80%` pruned); `run --stdin` auto-flags `[ALIAS HAZARD]`, `[IN-PLACE MUTATION]` & `[STATE NO-OP]`; `diff` checks boundaries | `diff` & `run` verify data-cleaning, financial rounding & schema transform scripts on edge inputs before batch runs | `64 tok` |
| **[`skills/splice/`](./skills/splice/README.md)**<br>([中文](./skills/splice/README.zh-CN.md)) | **`50.1%` same-file re-reads**, CRLF patch failures & Windows UTF-8 crashes | `view --symbol Cls.fn` (**-94.4%** tok); `apply` UTF-8/CRLF-safe syntax-verified atomic multi-file rollback patcher + auto `.pyc` purge; `<15ms` `check` | `view --heading` reads exact SOP/vault sections; `apply` updates long Markdown/JSON configs atomically without encoding corruption | `68 tok` |
| **[`skills/sieve/`](./skills/sieve/README.md)**<br>([中文](./skills/sieve/README.zh-CN.md)) | **Context Rot** from `>8 KB` logs/JSON/HTML + lost stack frames | Compress `pytest`/`cargo`/CI logs by **98.8%**; lossless `peek <id>` exact lines/regex in `<10ms` without re-running | Compress 400-item API JSON (**-99.6%**) & noisy HTML (**-98.4%**) with lossless `--json-path` / line `peek` | `83 tok` |
| **[`skills/guard/`](./skills/guard/README.md)**<br>([中文](./skills/guard/README.zh-CN.md)) | **Premature `"Done!"` claims** (`exit 1` is non-blocking!), retry loops & secret leaks | Native **`exit 2`** Stop-gate, 3x identical-failure loop breaker & `git push --force` shield | Block exposed API keys (`sk-`, `ghp_`, `xai-`, `AKIA`), `rm -rf` wipes & unbounded outbound email/webhook blasts | `63 tok` |
| **[`skills/baton/`](./skills/baton/README.md)**<br>([中文](./skills/baton/README.zh-CN.md)) | **Compaction amnesia** & lost negative constraints across sessions/CLIs | Hand off verified `file:line` facts, git dirty state & **`DO NOT RETRY` dead-ends** across Claude ↔ Codex ↔ Grok ↔ Antigravity | Resume multi-day research, procurement comparisons & personal checklists after compaction in `<400 tokens` | `70 tok` |

---

## Live Multi-Harness Controlled A/B Results (Zero Fabrication)

We ran live controlled CLI A/B experiments across **Grok Build CLI (`1.0.77`, `grok-4.7`)**, **Google Antigravity CLI (`1.5.8`)**, and **OpenAI Codex CLI (`0.167.0`, `gpt-6-astra`)** on multi-file repositories with hidden unreferenced distractor modules (`Task B: rulelang` 25-file compiler/optimizer/cache with 5 semantic/aliasing/cache bugs; `Task A: fluxpipe` 34-file webhook delivery engine with 6 aliasing/unit/TTL bugs), graded by independent hidden test oracles:

| CLI Harness & Task | Mode | Wall Time / Steps | Reasoning / Thinking Tokens | Peak 1-Turn Thinking Spike | Gross / Total Tokens | Hidden Oracle |
| :--- | :--- | ---: | ---: | ---: | ---: | :---: |
| **Grok CLI `1.0.77`** (`grok-4.7`) · *Task B (`rulelang`)* | Vanilla | `693.1s` (`23` calls, `8` `search_replace`) | `36,013 tok` | `14,285 tok` | `418,542 tok` (`$0.59`) | `5/5` |
| **Grok CLI `1.0.77`** (`grok-4.7`) · *Task B (`rulelang`)* | **`+ probe + splice`** | **`352.6s` (-49.1%)** (`0` `search_replace`) | **`12,477 tok` (-65.4%)** | **`4,302 tok` (-69.9%)** | `522,141 tok` (**`$0.54`, -7.3%**) | **`5/5`** |
| **Antigravity CLI `1.5.8`** (`agy`) · *Task B (`rulelang`)* | Vanilla | `41` tool steps *(1 live `python -c` quote crash)* | `50,680 tok` | — | `948,334 gross` (`401,064` net) | `5/5` |
| **Antigravity CLI `1.5.8`** (`agy`) · *Task B (`rulelang`)* | **`+ probe + splice`** | **`24` tool steps (-41.5%)** *(0 crashes)* | **`29,449 tok` (-41.9%)** | — | **`442,327 gross` (-53.4%)** (`215,318` net) | **`5/5`** |
| **Codex CLI `0.167.0`** (`gpt-6-astra`) · *Task B (`rulelang`)* | Vanilla | `7` shell cmds *(manual `__pycache__` cleanup)* | `96 tok` | — | `180,757 tok` | `5/5` |
| **Codex CLI `0.167.0`** (`gpt-6-astra`) · *Task B (`rulelang`)* | **`+ probe + splice`** | **`6` shell cmds (-14.3%)** *(auto `.pyc` purge)* | **`53 tok` (-44.8%)** | — | **`174,709 tok` (-3.3%)** | **`5/5`** |
| **Grok CLI `1.0.77`** (`grok-4.7`) · *Task A (`fluxpipe`)* | Vanilla → **`+ probe + splice`** | `40` → **`31` calls (-22.5%)** (`13` → `0` `search_replace`) | `55,398` → **`41,499` (-25.1%)** | `24,700` → **`10,039` (-59.4%)** | `1.033M` → `1.050M` *(honest parity: first-time skill discovery)* | **`6/6` → `6/6`** |

> **Honest Engineering Note**: On ultra-terse trajectories where a model already solves a repo in `8` shell calls (`Codex` on `fluxpipe`), reading `SKILL.md` for the first time adds `1` extra command (`8 → 9` cmds) even as `probe` catches all `3` alias hazards in `64.8 ms` and `splice` patches `6` files atomically in `24.7 ms`. Pre-installing skills globally (`install.py`) amortizes discovery to zero and maximizes net savings on multi-file debugging.

---

## Deterministic Micro-Benchmarks (Including Honest Negative Controls)

Run `python bench/run_bench.py` locally to reproduce [`bench/results.json`](./bench/results.json) (Python 3.12, seed `20261115`, zero external packages):

| Primitive & Workload | Baseline / Raw | With Primitive | Gain / Reduction | Latency | Fidelity / Safety Guarantee |
| :--- | ---: | ---: | ---: | ---: | :--- |
| **`probe`** · 25-file package (`slice` + `run`) | `25` files + manual debug | `3` active files (`20` dead pruned) | **-80.0% dead files** | `18.8 ms` / `12.0 ms` | Auto-flags `2` alias hazards, `1` mutation, `1` no-op |
| **`splice`** · 30-method CRLF module (`view` + `apply` + `check`) | `1,376 tok` full read | `77 tok` exact AST method | **-94.4% read tok** | `1.5 ms` / `4.0 ms` | Preserves CRLF + UTF-8; 100% rollback on syntax error |
| **`sieve`** · 2,500-line CI/pytest log | `172,075 B` (`~43,019 tok`) | `2,080 B` (`~520 tok`) | **-98.8%** | `23.7 ms` | 100% exceptions preserved + lossless `peek` |
| **`sieve`** · 400-record JSON API payload | `107,887 B` (`~26,972 tok`) | `423 B` (`~106 tok`) | **-99.6%** | `3.0 ms` | Schema + warnings preserved + `--json-path` peek |
| **`sieve`** · Noisy HTML page (JS/CSS bloat) | `48,160 B` (`~12,040 tok`) | `780 B` (`~195 tok`) | **-98.4%** | `8.5 ms` | Title, H1–H3 outline & lead body preserved |
| **`sieve`** · Short `<1.2 KB` output *(Negative Control)* | `69 B` (`~18 tok`) | `117 B` (`~30 tok`) | **0.0%** *(passthrough)* | `2.7 ms` | Short outputs pass through untouched |
| **`recall`** · 200-rule vault (EN + CJK hybrid) | `6,380 tok` *(monolith)* | `39.9 tok` *(top-2)* | **-99.4%** | `4.0 ms` | **20/20 (100%)** Top-1 accuracy |
| **`atlas`** · 64-module SDK corpus | `42,710 tok` *(full read)* | `446 tok` *(structural map)* | **-99.0%** | `19.3 ms` | AST symbols, hub modules & entrypoints |

---

## 60-Second Quick Start

```bash
# 0. Audit your own local CLI sessions first (100% offline)
python audit/audit_sessions.py

# 1. Verify Python 3.9+ & built-in SQLite FTS5
python install.py --doctor

# 2. Install all 7 folders—or pick a single folder with --only probe,splice
python install.py --agent agents --print-agents-md   # ~/.agents/skills (Antigravity, Codex, Cursor, OpenCode)
python install.py --agent claude --print-hooks       # ~/.claude/skills + ready-to-paste Hook JSON
python install.py --agent grok                       # ~/.grok/skills   (Grok Build CLI)
python install.py --agent openclaw                   # ~/.openclaw/skills (OpenClaw Personal Assistant)
python install.py --agent hermes                     # ~/.hermes/skills (Hermes Agent)
```

---

## Repository Layout & Verification

```text
agent-primitives/
├── README.md / README.zh-CN.md      # Default English + full Chinese overview & live A/B data
├── DESIGN.md / DESIGN.zh-CN.md      # Anti-obsolescence 4-boundary architecture & admission laws
├── PLAYBOOK.md / PLAYBOOK.zh-CN.md  # Step-by-step wiring for Coding & Personal Assistant Agents
├── audit/audit_sessions.py          # Local zero-cloud session pathology scanner (Codex/Claude/Grok/AGY)
├── install.py                       # Zero-dependency selective installer, hook generator & doctor
├── skills/                          # One folder = one standalone zero-dependency mini-project
│   ├── atlas/                       # Stage 1: Sub-second AST & vault map + >90d stale-TODO radar
│   ├── recall/                      # Stage 1: Sub-4ms SQLite FTS5/BM25 + CJK triggered memory
│   ├── probe/                       # Stage 2: Reachability slicer, heap/mutation tracer & diff oracle
│   ├── splice/                      # Stage 3: AST/Markdown surgeon & UTF-8/CRLF atomic patcher
│   ├── sieve/                       # Stage 4: Lossless output compressor & FTS5 flight recorder
│   ├── guard/                       # Stage 5: Exit-2 truth gate, 3x loop breaker & secret shield
│   └── baton/                       # Stage 5: Compaction-proof state & cross-agent handoff capsule
├── bench/run_bench.py               # Deterministic benchmark suite + negative controls
└── tests/test_primitives.py         # 10 isolation, rollback, encoding & end-to-end test suites
```

```bash
python -m unittest discover -s tests -v
```

## License

[MIT](./LICENSE)
