# agent-primitives

**One Folder = One Anti-Obsolescence, Zero-Dependency Micro-Infrastructure for Coding Agents & Personal Assistant Agents.**

[English (Default)](./README.md) | [简体中文](./README.zh-CN.md) · **[Handbook & Roadmap (EN)](./HANDBOOK.md)** | **[核心理念与演进总手册 (中文)](./HANDBOOK.zh-CN.md)** · [Architecture](./DESIGN.md) · [Playbook](./PLAYBOOK.md)

---

## 1-Line Onboarding for Every Harness (`Codex` · `Pi` · `Antigravity` · `Claude Code` · `Grok`)

Zero `git clone`, zero `pip install`, zero `npm` packages, zero background daemons. One command auto-detects every agent harness installed on your machine, syncs all 7 standalone primitives, installs the unified `ap` CLI, and idempotently wires `<10-line` routing instructions:

```bash
# macOS / Linux (auto-detects Codex, Pi, Antigravity, Claude Code, Grok, OpenClaw, Hermes):
curl -fsSL https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/ap.py | python3 - init
```

```powershell
# Windows PowerShell:
irm https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/ap.py | python - init
```

```bash
# Wire ONLY the current repository (.agents/skills + .agents/ap.py + fenced AGENTS.md block):
curl -fsSL https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/ap.py | python3 - init --here

# Target specific harness(es) + merge non-destructive PreToolUse/Stop hooks:
curl -fsSL https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/ap.py | python3 - init -a codex,pi,agy,claude --with-hooks

# Clean 1-command uninstall at any time (preserves all of your existing rules & settings):
ap uninstall          # or: python .agents/ap.py ... / python ap.py uninstall --here
```

### Ecosystem-Native 1-Liners & Exact Harness Discovery Matrix

Every harness discovers skills, global instructions, and hooks differently. `agent-primitives` natively supports both **`ap.py init`** and each harness's built-in package/plugin installer:

| Harness | Native 1-Line Install Command | Global Skills & Rule Paths Wired by `ap init` | Hook Protocol (`--with-hooks`) |
| :--- | :--- | :--- | :--- |
| **OpenAI Codex CLI** (`codex`) | `codex plugin marketplace add xgbtxy/agent-primitives && codex plugin add agent-primitives@agent-primitives`<br>*(or `ap.py init -a codex`)* | `~/.codex/skills/` + `~/.agents/skills/`<br>Rules: `~/.codex/AGENTS.md` | `~/.codex/hooks.json` (`exit 2` snake_case) |
| **Pi** (`pi` / `pi-coding-agent`) | `pi install git:github.com/xgbtxy/agent-primitives`<br>*(or `ap.py init -a pi`)* | `~/.pi/agent/skills/` (repo: `.pi/skills/`)<br>Rules: `~/.pi/agent/AGENTS.md` | Via `guard run` / `AGENTS.md` |
| **Google Antigravity** (`agy` / IDE) | `curl -fsSL .../ap.py \| python3 - init -a agy --with-hooks`<br>*(or `agy plugin install ./agent-primitives`)* | `~/.gemini/config/skills/` + `~/.agents/skills/`<br>Rules: `~/.gemini/config/GEMINI.md` | `~/.gemini/config/hooks.json` (camelCase JSON `deny`/`continue`) |
| **Claude Code** (`claude`) | `/plugin marketplace add xgbtxy/agent-primitives`<br>*(or `ap.py init -a claude --with-hooks`)* | `~/.claude/skills/` (repo: `.claude/skills/`)<br>Rules: `~/.claude/CLAUDE.md` | `~/.claude/settings.json` (`exit 2` + `PreCompact`) |
| **xAI Grok Build** (`grok`) | `curl -fsSL .../ap.py \| python3 - init -a grok` | `~/.grok/skills/` *(loaded before repo skills)*<br>Rules: `~/.grok/AGENTS.md` | Via `guard` CLI |
| **All 35+ Harnesses** (`skills.sh`) | `npx skills add xgbtxy/agent-primitives -g` | Auto-routed by `skills` CLI | Pair with `ap init --with-hooks` |

---

## Why This Repository Exists (And Why Model Upgrades Won't Replace It)

95% of agent repositories become obsolete within 6 months because they build **"thinking crutches"** (prompt roleplay, planning checklists, heavy orchestration wrappers, or paid vector RAG) that get absorbed into the next frontier model's post-training weights.

**`agent-primitives`** is built from empirical process telemetry across **`69,963` real local tool calls** (`Grok Build CLI`, `OpenAI Codex CLI`, `Google Antigravity CLI`, `Claude Code`). Regardless of how smart `GPT-6`, `Grok-5`, `Claude-5`, or `Gemini-4` becomes, an autoregressive token stream still collides with **four irreducible physical boundaries**:

1. **External OS/Web Entropy vs. `O(N)` KV-Cache Physics** — Dumping `170 KB` CI logs, `400`-item JSON payloads, or `15` dead unreferenced files into context physically inflates prefill latency and bills every subsequent turn.
2. **Prose Simulation vs. Concrete Runtime Heap & Types** — Mentally simulating shallow-copy dict aliasing, in-place state mutations, or `None`/`bool`/`int` (`True == 1`) collisions burns `35k–55k` reasoning tokens (`11–19 min`), while ad-hoc `python -c` one-liners hit shell-quoting and `ImportError` crashes (`2,716` inline probes & `177` crashes audited locally).
3. **BPE Token Generation vs. Byte/AST-Exact File Transactions** — Line shifts force `50.1%` redundant same-file re-reads, CRLF/whitespace patch mismatches (`528` audited), and Windows `cp936`/UTF-8 decode crashes (`160` audited).
4. **Compaction Amnesia & Self-Report Bias** — `/compact` erases falsified dead-ends, and prose rules cannot enforce a deterministic blocking truth gate after edits.

**Design Law**: **One subfolder = one standalone zero-dependency deterministic OS/AST/Heap/SQLite primitive** (`cp -r skills/probe ~/.agents/skills/`). Pure Python 3.9+ stdlib only (`0` pip packages, `0` daemons, `0` API keys, **`486 tokens`** total idle frontmatter cost across all 7 folders combined).

---

## Step 0: Measure Your Own Local Agent Waste in Seconds (`ap.py audit`)

Before or after installing, run the **100% local, zero-cloud telemetry auditor** against your own `~/.codex`, `~/.claude`, `~/.grok`, and `~/.gemini/antigravity-cli` session logs:

```bash
python ap.py audit --compare
```

Audited on our workstation (`178` recent sessions, `69,963` tool calls, `0` bytes uploaded):
- **`skills/probe` triggered**: `2,716` inline `python -c` probes, `106` `ImportError`/`sys.path` failures, `71` shell-quoting crashes
- **`skills/splice` triggered**: `679` redundant same-file re-reads, `3,918` manual shell line-slices, `528` patch mismatches, `160` UTF-8/CP936 crashes
- **`skills/sieve` triggered**: `17,110` tool outputs `>8 KB`, `171` truncation spills
- **`skills/atlas` triggered**: `2,994` broad shell search/listing calls
- **`skills/guard + baton` triggered**: `26` identical failing retry loops

---

## The 7 Standalone Mini-Projects Mapped to the 5-Stage Agent Loop

```text
[ap.py audit --compare] ──► Diagnose real local friction & before/after trends (0 cloud upload)
  ├─► Stage 1: Orient & Recall   ──► skills/atlas/  +  skills/recall/
  ├─► Stage 2: Slice & Diagnose  ──► skills/probe/   (slice --bundle · run · diff)
  ├─► Stage 3: Surgical Act      ──► skills/splice/  (view · apply · check --test)
  ├─► Stage 4: Filter Output     ──► skills/sieve/   (run · pipe · peek)
  └─► Stage 5: Gate & Persist    ──► skills/guard/  +  skills/baton/
```

| Mini-Project Folder | Physical Boundary Solved | For Coding Agents | For Personal Assistant Agents | Idle Cost |
| :--- | :--- | :--- | :--- | ---: |
| **[`skills/atlas/`](./skills/atlas/README.md)**<br>([中文](./skills/atlas/README.zh-CN.md)) | **Cold-start wandering** & `+77%` token waste chasing stale old TODOs | `~20ms` AST signatures, hub centrality, entrypoints & `>90-day` stale-TODO trap warnings | `<800-token` structural map & `H1/H2` outline of large Markdown vaults and note folders | `73 tok` |
| **[`skills/recall/`](./skills/recall/README.md)**<br>([中文](./skills/recall/README.zh-CN.md)) | **Static `AGENTS.md` / `MEMORY.md` bloat** burning thousands of tokens/turn | `<5ms` stdlib `sqlite3` FTS5/BM25 + path globs (`src/auth/*` -> inject `~40-token` gotcha) | `<5ms` bilingual (English + CJK) rule recall for contacts, invoicing SOPs & server ops | `71 tok` |
| **[`skills/probe/`](./skills/probe/README.md)**<br>([中文](./skills/probe/README.zh-CN.md)) | **Dead-module reading, `50k`-token prose heap guessing** & `python -c` crashes | Polyglot `slice --bundle` prunes Py/TS/JS/Go/Rust dead files & bundles active source in 1 call; loop-dedup `run` flags `[ALIAS HAZARD]`/`[IN-PLACE MUTATION]`/`[STATE NO-OP]`; `diff` checks boundaries | `diff` & `run` verify data-cleaning, financial rounding & schema transform scripts on edge inputs before batch runs | `62 tok` |
| **[`skills/splice/`](./skills/splice/README.md)**<br>([中文](./skills/splice/README.zh-CN.md)) | **`50.1%` same-file re-reads**, CRLF patch failures & test-runner guessing | Polyglot `view --symbol` (**-94.4%** tok); UTF-8/CRLF-safe atomic `apply` + `.pyc` purge; `check --test` auto-runs `pytest`/`unittest`/`node --test`/`go test`/`cargo test` | `view --heading` reads exact SOP/vault sections; `apply` updates long Markdown/JSON configs atomically without encoding corruption | `64 tok` |
| **[`skills/sieve/`](./skills/sieve/README.md)**<br>([中文](./skills/sieve/README.zh-CN.md)) | **Context Rot** from `>8 KB` logs/JSON/HTML + lost stack frames | Compress `pytest`/`cargo`/CI logs by **98.8%**; lossless `peek <id>` exact lines/regex in `<10ms` without re-running | Compress 400-item API JSON (**-99.6%**) & noisy HTML (**-98.4%**) with lossless `--json-path` / line `peek` | `83 tok` |
| **[`skills/guard/`](./skills/guard/README.md)**<br>([中文](./skills/guard/README.zh-CN.md)) | **Premature `"Done!"` claims**, retry loops & secret leaks across hook protocols | Dual-protocol Stop-gate & PreToolUse shield (`exit 2` for Claude/Codex + camelCase `deny`/`continue` JSON for Antigravity) + 3x loop breaker | Block exposed API keys (`sk-`, `ghp_`, `xai-`, `AKIA`), `rm -rf` wipes & unbounded outbound email/webhook blasts | `63 tok` |
| **[`skills/baton/`](./skills/baton/README.md)**<br>([中文](./skills/baton/README.zh-CN.md)) | **Compaction amnesia** & lost negative constraints across sessions/CLIs | Hand off verified `file:line` facts, git dirty state & **`DO NOT RETRY` dead-ends** across Claude ↔ Codex ↔ Pi ↔ Grok ↔ Antigravity | Resume multi-day research, procurement comparisons & personal checklists after compaction in `<400 tokens` | `70 tok` |

---

## Live Multi-Harness Controlled A/B Results (Zero Fabrication)

We ran live controlled CLI A/B experiments across **Grok Build CLI (`1.0.77`, `grok-4.7`)**, **Google Antigravity CLI (`1.5.8`)**, and **OpenAI Codex CLI (`0.167.0`, `gpt-6-astra`)** on multi-file Python & TypeScript repositories containing unreferenced distractor modules (`Task C: ts-ledger` 24-file TypeScript/Node 22 settlement ledger with 13 dead `.ts` files & 5 multi-file rounding/idempotency/aliasing/atomic-batch bugs; `Task B: rulelang` 25-file Python compiler/optimizer/cache with 5 semantic/aliasing/cache bugs), graded by independent hidden test oracles:

| CLI Harness & Task | Mode | Wall Time / Steps | `read_file` / Edit Errors | Reasoning / Thinking Tokens | Total / Net Tokens & Cost | Hidden Oracle |
| :--- | :--- | ---: | ---: | ---: | ---: | :---: |
| **Grok CLI `1.0.77`** (`grok-4.7`) · *Task C (TypeScript `ts-ledger`)* | Vanilla | `766.6s` / `36` calls (`14` turns) | `18` `read_file` | `37,426 tok` | `683,173 tok` (`$0.86`) | `5/5` |
| **Grok CLI `1.0.77`** (`grok-4.7`) · *Task C (TypeScript `ts-ledger`)* | **`+ ap init (v0.3)`** | **`563.2s` (-26.5%)** / **`23` calls (-36.1%)** | **`6` `read_file` (-66.7%)** | **`31,748 tok` (-15.2%)** | **`418,623 tok` (-38.7%, `$0.56` = -34.4%)** | **`5/5`** |
| **Antigravity `1.5.8`** (`agy`) · *Task B (Python `rulelang`)* | Vanilla | `1,039.1s` / `42` steps | `31` `read_file`, `1` crash | `50,680 tok` | `401,064 net tok` (`334.4k` in) | `5/5` |
| **Antigravity `1.5.8`** (`agy`) · *Task B (Python `rulelang`)* | **`+ ap init (v0.3)`** | **`789.5s` (-24.0%)** / **`24` steps (-42.9%)** | **`8` `read_file` (-74.2%)**, `0` quoting crashes | **`43,060 tok` (-15.0%)** | **`201,342 net tok` (-49.8%, `141.1k` in = -57.8%)** | **`5/5`** |
| **Antigravity `1.5.8`** (`agy`) · *Task C (TypeScript `ts-ledger`)* | Vanilla → **`+ ap init`** | `31` → **`12` steps (-61.3%)** | `15` → **`7` `read_file` (-53.3%)**; `7` → **`1` error (-85.7%)** | `22,210` → `24,063 tok` | `435,630` → **`369,397 gross` (-15.2%, `-66.2k`)** | **`5/5` → `5/5`** |
| **Grok CLI `1.0.77`** (`grok-4.7`) · *Task B (Python `rulelang`)* | Vanilla → **`+ probe + splice`** | `693.1s` → **`352.6s` (-49.1%)** | `8` → **`0` `search_replace`** | `36,013` → **`12,477 tok` (-65.4%)** | `$0.59` → **`$0.54` (-7.3%)** | **`5/5` → `5/5`** |
| **Codex CLI `0.167.0`** (`gpt-6-astra`) · *Task B (Python) & Task C (TS)* | Vanilla → **`+ ap init`** | *Task C*: `136.9s` → **`126.9s` (-7.3%)**; *Task B*: `7` → **`6` cmds (-14.3%)** | `0` errors (`probe slice --bundle` + `splice check --test`) | *Task C*: `31` → **`24` (-22.6%)**; *Task B*: `96` → **`53` (-44.8%)** | *Task B*: `180.8k` → **`174.7k` (-3.3%)** *(Task C: `140.7k` → `187.3k` — see note)* | **`5/5` → `5/5`** |

> **Honest Engineering Note**: Across multi-turn harnesses (`Grok CLI`, `Antigravity CLI`), `ap init` slashes `read_file` calls by **53%–74%**, edit errors by **85.7%**, net/total tokens by **15%–50%**, and wall-clock time by **24%–49%**. On ultra-short 5-command `Codex CLI` trajectories where the model has a native built-in `apply_patch` tool, `ap init` still finishes **7.3% faster (`126.9s` vs `136.9s`)** with **22.6% fewer reasoning tokens**, though serializing JSON specs vs. built-in `apply_patch` trades ~46k cached input tokens for atomic AST/test verification.

---

## Deterministic Micro-Benchmarks (Including Honest Negative Controls)

Run `python bench/run_bench.py` locally to reproduce [`bench/results.json`](./bench/results.json) (Python 3.12, seed `20261115`, zero external packages):

| Primitive & Workload | Baseline / Raw | With Primitive | Gain / Reduction | Latency | Fidelity / Safety Guarantee |
| :--- | ---: | ---: | ---: | ---: | :--- |
| **`probe`** · 25-file Python + 24-file TS (`slice` + `run`) | `25` Py / `24` TS files | `3` Py / `9` TS active (`20` / `15` pruned) | **-80.0% / -62.5% dead files** | `24.3 ms` / `14.4 ms` | Auto-flags `2` alias hazards, `1` mutation, `1` no-op |
| **`splice`** · 30-method CRLF module (`view` + `apply` + `check`) | `1,376 tok` full read | `77 tok` exact AST method | **-94.4% read tok** | `1.7 ms` / `5.5 ms` | Preserves CRLF + UTF-8; 100% rollback on syntax error |
| **`sieve`** · 2,500-line CI/pytest log | `172,075 B` (`~43,019 tok`) | `2,080 B` (`~520 tok`) | **-98.8%** | `31.0 ms` | 100% exceptions preserved + lossless `peek` |
| **`sieve`** · 400-record JSON API payload | `107,887 B` (`~26,972 tok`) | `423 B` (`~106 tok`) | **-99.6%** | `5.1 ms` | Schema + warnings preserved + `--json-path` peek |
| **`sieve`** · Noisy HTML page (JS/CSS bloat) | `48,160 B` (`~12,040 tok`) | `780 B` (`~195 tok`) | **-98.4%** | `12.0 ms` | Title, H1–H3 outline & lead body preserved |
| **`sieve`** · Short `<1.2 KB` output *(Negative Control)* | `69 B` (`~18 tok`) | `117 B` (`~30 tok`) | **0.0%** *(passthrough)* | `2.2 ms` | Short outputs pass through untouched |
| **`recall`** · 200-rule vault (EN + CJK hybrid) | `6,380 tok` *(monolith)* | `39.9 tok` *(top-2)* | **-99.4%** | `4.9 ms` | **20/20 (100%)** Top-1 accuracy |
| **`atlas`** · 64-module SDK corpus | `42,710 tok` *(full read)* | `446 tok` *(structural map)* | **-99.0%** | `36.7 ms` | AST symbols, hub modules & entrypoints |

---

## Repository Layout & Verification

```text
agent-primitives/
├── ap.py                            # 1-line zero-clone bootstrapper, multi-harness installer & unified CLI
├── README.md / README.zh-CN.md      # Default English + full Chinese overview, 1-line matrix & live A/B data
├── HANDBOOK.md / HANDBOOK.zh-CN.md  # Canonical philosophy, 6 constitutional laws, 5-step loop & evolution log
├── DESIGN.md / DESIGN.zh-CN.md      # Anti-obsolescence 4-boundary architecture deep dive
├── PLAYBOOK.md / PLAYBOOK.zh-CN.md  # Per-harness 1-line wiring & Coding / Personal Assistant workflows
├── package.json / .*-plugin/        # Zero-dep native manifests for Pi (`pi install`), Codex, Antigravity & Claude
├── audit/audit_sessions.py          # Local zero-cloud session pathology scanner + `--compare` trend delta
├── skills/                          # One folder = one standalone zero-dependency mini-project
│   ├── atlas/                       # Stage 1: Sub-second AST & vault map + >90d stale-TODO radar
│   ├── recall/                      # Stage 1: Sub-5ms SQLite FTS5/BM25 + CJK triggered memory
│   ├── probe/                       # Stage 2: Polyglot slice+bundle, heap/mutation tracer & diff oracle
│   ├── splice/                      # Stage 3: Polyglot symbol/MD surgeon, atomic patcher & auto-test runner
│   ├── sieve/                       # Stage 4: Lossless output compressor & FTS5 flight recorder
│   ├── guard/                       # Stage 5: Dual-protocol truth gate, 3x loop breaker & secret shield
│   └── baton/                       # Stage 5: Compaction-proof state & cross-agent handoff capsule
├── bench/run_bench.py               # Deterministic benchmark suite + negative controls
└── tests/test_primitives.py         # 14 isolation, zero-clone bootstrap, polyglot, hook & rollback test suites
```

```bash
python ap.py doctor
python -m unittest discover -s tests -v
```

## License

[MIT](./LICENSE)
