# agent-primitives (`ap` v1.0)

**Pure-Rust, Single-Binary, Anti-Obsolescence Micro-Infrastructure for Coding Agents & Personal Assistant Agents.**

[English (Default)](./README.md) | [简体中文](./README.zh-CN.md) · **[Handbook & Constitution (EN)](./HANDBOOK.md)** | **[核心理念与演进总手册 (中文)](./HANDBOOK.zh-CN.md)** · [Architecture](./DESIGN.md) · [Playbook](./PLAYBOOK.md)

---

## 1-Line Onboarding for Every Harness (`Codex` · `Pi` · `Antigravity` · `Claude Code` · `Grok`)

**Zero `Python`, zero `Node`, zero `Docker`, zero `git clone`, zero background daemons.** One command downloads the SHA-256-verified native `ap` binary (with statically embedded **6-language Tree-sitter** + **bundled SQLite FTS5**), auto-detects every agent harness installed on your machine, installs all 7 standalone primitives, and idempotently wires self-contained `<10-line` routing instructions:

```bash
# macOS / Linux (prebuilt x86_64 musl & aarch64 binaries; falls back to cargo if needed):
curl -fsSL https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/install.sh | sh
```

```powershell
# Windows PowerShell (prebuilt x86_64-pc-windows-msvc ap.exe + SHA-256 check):
irm https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/install.ps1 | iex
```

```bash
# Or install from source with Cargo (unified `ap` binary, or any individual `crates/ap-*` crate):
cargo install --git https://github.com/xgbtxy/agent-primitives --locked && ap init

# Wire ONLY the current repository (.agents/bin/ap + .agents/skills + fenced AGENTS.md block):
ap init --here

# Target specific harness(es) + merge non-destructive <2ms PreToolUse/Stop hooks:
ap init -a codex,pi,agy,claude --with-hooks

# Clean 1-command uninstall at any time (preserves all of your existing rules & settings):
ap uninstall          # or: ap uninstall --here
```

### Ecosystem-Native 1-Liners & Exact Harness Discovery Matrix

Every harness discovers skills, global instructions, and hooks differently. `agent-primitives` natively supports both **`ap init`** and each harness's built-in plugin/package manager:

| Harness | Native 1-Line Install Command | Global Skills & Rule Paths Wired by `ap init` | Hook Protocol (`--with-hooks`) |
| :--- | :--- | :--- | :--- |
| **OpenAI Codex CLI** (`codex`) | `codex plugin marketplace add xgbtxy/agent-primitives && codex plugin add agent-primitives@agent-primitives`<br>*(or `ap init -a codex`)* | `~/.codex/skills/` + `~/.agents/skills/`<br>Rules: `~/.codex/AGENTS.md` | `~/.codex/hooks.json` (`<2ms` `exit 2` snake_case) |
| **Pi** (`pi` / `pi-coding-agent`) | `pi install git:github.com/xgbtxy/agent-primitives`<br>*(or `ap init -a pi`)* | `~/.pi/agent/skills/` (repo: `.pi/skills/`)<br>Rules: `~/.pi/agent/AGENTS.md` | Via `ap guard` / `AGENTS.md` |
| **Google Antigravity** (`agy` / IDE) | `ap init -a agy --with-hooks`<br>*(or `agy plugin install ./agent-primitives`)* | `~/.gemini/config/skills/` + `~/.agents/skills/`<br>Rules: `~/.gemini/config/GEMINI.md` | `~/.gemini/config/hooks.json` (camelCase JSON `deny`/`continue`) |
| **Claude Code** (`claude`) | `/plugin marketplace add xgbtxy/agent-primitives`<br>*(or `ap init -a claude --with-hooks`)* | `~/.claude/skills/` (repo: `.claude/skills/`)<br>Rules: `~/.claude/CLAUDE.md` | `~/.claude/settings.json` (`<2ms` `exit 2` + `PreCompact`) |
| **xAI Grok Build** (`grok`) | `ap init -a grok` | `~/.grok/skills/` *(loaded before repo skills)*<br>Rules: `~/.grok/AGENTS.md` | Via `ap guard` CLI |
| **OpenClaw / Hermes / 35+ Harnesses** | `npx skills add xgbtxy/agent-primitives -g` + `ap init` | `~/.openclaw/skills/`, `~/.hermes/skills/`, `~/.agents/skills/` | Pair with `ap init --with-hooks` |

---

## Why Pure Rust (`v1.0`) — And Why Model Upgrades Won't Obsolete It

95% of agent repositories become obsolete within 6 months because they build **"thinking crutches"** (prompt personas, rigid planning checklists, heavy orchestration wrappers, or paid vector RAG) that get absorbed into the next frontier model's post-training weights.

**`agent-primitives`** is grounded in empirical process telemetry across **`101,108` real local tool calls** (`427` sessions across `Codex`, `Pi`, `Antigravity`, `Claude Code`, and `Grok`). No matter how smart `GPT-6`, `Grok-5`, `Claude-5`, or `Gemini-4` becomes, an autoregressive token stream still collides with **four irreducible physical boundaries**:

1. **External OS/Web Entropy vs. `O(N)` KV-Cache Physics** — Dumping `170 KB` CI logs, `400`-item JSON/JSONL streams, or `15` unreferenced dead modules into context physically inflates prefill latency and bills every subsequent turn.
2. **Prose Simulation vs. Concrete Runtime Memory & Types** — Mentally simulating shallow-copy dict aliasing, in-place argument mutations, `-0.0` signed-zero drift, or `True == 1` collisions burns `35k–55k` reasoning tokens (`11–19 min`), while ad-hoc inline one-liners crash on shell quoting and import paths (`3,908` inline probes audited locally).
3. **BPE Token Generation vs. Concrete AST & Byte-Exact File Transactions** — Line shifts trigger `50%+` redundant same-file re-reads, JSON patch-escaping corruption, CRLF mismatches (`728` audited), and Windows encoding crashes (`214` audited).
4. **Compaction Amnesia & Self-Report Bias** — `/compact` erases falsified dead-ends, and prose rules cannot enforce a `<2 ms` deterministic **`exit 2`** OS gate on every tool call without Python/Node interpreter startup lag (`40–90 ms`).

**Why Rust v1.0 Over Python/C**: Compiled Rust cuts AST/SQLite/Hook latency from `40–90 ms` down to **`0.68–16.3 ms`**, eliminates `Python`/`pip`/`venv` breakages across target machines, embeds real **6-language Tree-sitter C grammars** + **static SQLite FTS5** with compile-time memory safety, and enforces **`0` sibling dependencies** across 7 standalone crates (**`412 tokens`** total idle frontmatter across all 7 skills).

---

## Step 0: Measure Your Own Local Agent Waste in Seconds (`ap audit`)

Run the **100% local, zero-cloud streaming JSONL telemetry auditor** against your own `~/.codex`, `~/.pi`, `~/.gemini/antigravity-cli`, `~/.claude`, and `~/.grok` logs:

```bash
ap audit --compare
```

Audited on our workstation (`427` sessions, `101,108` tool calls scanned in `8.2s`, `0` bytes uploaded):
- **`ap probe` triggered**: `3,908` ad-hoc inline `python -c` / `node -e` probes, `249` module/quoting crashes
- **`ap splice` triggered**: `1,104` redundant same-file re-reads, `5,412` shell line-slices, `728` patch mismatches, `214` encoding crashes
- **`ap sieve` triggered**: `24,619` oversized tool outputs `>8 KB`, `238` truncation spills
- **`ap atlas` triggered**: `4,380` broad directory/grep exploration calls
- **`ap guard + baton` triggered**: `39` identical failing `3x` retry loops

---

## The 7 Standalone Rust Crates Mapped to the 5-Stage Agent Loop

| Crate & Skill | Physical Boundary Solved | For Coding Agents (`Py/TS/JS/Go/Rust/C`) | For Personal Assistant Agents | Latency / Idle |
| :--- | :--- | :--- | :--- | ---: |
| **[`ap-atlas`](./skills/atlas/README.md)**<br>([中文](./skills/atlas/README.zh-CN.md)) | **Cold-start wandering** & `+77%` token waste chasing stale TODOs | `16.3ms` 6-lang **Tree-sitter** signatures, `.gitignore` hub centrality, `[dirty]` git status & `>90d` stale-TODO radar | `<800-tok` structural map & `H1/H2` outline of 500-note Markdown vaults | `16.3ms` / `58 tok` |
| **[`ap-recall`](./skills/recall/README.md)**<br>([中文](./skills/recall/README.zh-CN.md)) | **Static `AGENTS.md` / `MEMORY.md` bloat** burning thousands of tokens/turn | `3.37ms` **bundled SQLite FTS5/BM25** + path globs (`src/auth/*` → `~47-tok` gotcha) | `3.37ms` bilingual (EN + CJK bigram) SOP recall for contacts, invoicing & ops | `3.37ms` / `58 tok` |
| **[`ap-probe`](./skills/probe/README.md)**<br>([中文](./skills/probe/README.zh-CN.md)) | **Dead-module reading, `50k`-tok prose heap guessing** & quoting crashes | `6.32ms` 6-lang Tree-sitter `slice --bundle`; Py (`sys.settrace`), TS/JS (Node V8 `Proxy`), & native `rustc` `run`/`diff` (`[ALIAS HAZARD]`/`[MUTATION]`/`-0.0`) | `diff` & `run` verify financial rounding, tax & CSV/JSON cleaning scripts on boundary inputs before batch runs | `6.32ms` / `59 tok` |
| **[`ap-splice`](./skills/splice/README.md)**<br>([中文](./skills/splice/README.zh-CN.md)) | **`50%+` same-file re-reads**, JSON patch escaping & broken syntax writes | `0.68ms` 6-lang Tree-sitter `view --symbol` (**-95.4%** tok); `2.46ms` Zero-Escaping `@@ FILE:` / `@@ SYMBOL:` atomic `apply` (**verifies Tree-sitter AST before disk write** + rollback) + `--test` | `view --heading` reads exact Markdown sections; `@@ WRITE:` / `@@ FILE:` updates handbooks & JSON configs with CRLF/UTF-8 safety | `0.68ms` / `61 tok` |
| **[`ap-sieve`](./skills/sieve/README.md)**<br>([中文](./skills/sieve/README.zh-CN.md)) | **Context Rot** from `>8 KB` logs/JSON/JSONL/HTML/CSV | Compress CI/test logs (**-98.8%**) & JSONL (**-97.0%**); `5.65ms` bundled SQLite FTS5 `search` + lossless `peek` | Compress 400-item JSON (**-99.6%**), HTML + `<table>` → pipe-tables (**-98.4%**) & CSV/TSV with `--json-path` peek | `5.65ms` / `59 tok` |
| **[`ap-guard`](./skills/guard/README.md)**<br>([中文](./skills/guard/README.zh-CN.md)) | **Premature `"Done!"` claims**, retry loops & secret leaks | `<2ms` native dual-protocol PreToolUse/Stop gate (`exit 2` for Claude/Codex + camelCase JSON for Antigravity) + 3x loop breaker | Block exposed keys (`sk-`, `ghp_`, `xai-`, `AKIA`), `rm -rf /` wipes & unbounded email/webhook blasts | `<2ms` / `57 tok` |
| **[`ap-baton`](./skills/baton/README.md)**<br>([中文](./skills/baton/README.zh-CN.md)) | **Compaction amnesia** & lost negative constraints across CLIs | Hand off verified `file:line` facts, git dirty state & **`DO NOT RETRY` dead-ends** across Codex ↔ Pi ↔ Claude ↔ Grok ↔ Antigravity | Resume multi-day research, procurement comparisons & personal checklists after `/compact` in `<400 tokens` | `<2ms` / `60 tok` |

---

## Live Multi-Harness Controlled Results (Zero Fabrication)

Evaluated on live CLIs against hidden external test oracles (`Task C: ts-ledger`, 24-file TypeScript/Node 22 settlement ledger with 13 unreferenced distractor `.ts` files & 5 multi-file rounding/idempotency/aliasing/atomic-batch bugs; `Task B: rulelang`, 25-file Python compiler/optimizer/cache):

| CLI Harness & Task | Mode | Commands / Steps / Time | File Reads & Patch Friction | Total / Net Tokens & Cost | Hidden Oracle |
| :--- | :--- | ---: | ---: | ---: | :---: |
| **Codex CLI `0.167.0`** (`gpt-6-astra`) · *Task C (`ts-ledger`)* | **`+ Rust ap v1.0 (--here)`** | **`4` total shell cmds** *(Run 1: `6` cmds in `133.7s`)* | **`0` manual file reads**; **13 edits + 15 tests in 1 `ap splice apply --stdin --test`** (`0` errors) | **`161,797 tok`** (`144.4k` cached = **91.8% hit**, **`12.9k` uncached in**) | **`5/5`** |
| **Grok CLI `1.0.77`** (`grok-4.7`) · *Task C (`ts-ledger`)* | Vanilla → **`+ ap init`** | `766.6s` (`36` calls) → **`563.2s` (`23` calls, -26.5% time)** | `18` → **`6` `read_file` (-66.7%)** | `683,173` → **`418,623 tok` (-38.7%, `$0.86`→`$0.56`)** | **`5/5` → `5/5`** |
| **Antigravity `1.5.8`** (`agy`) · *Task B (`rulelang`) & Task C (`ts-ledger`)* | Vanilla → **`+ ap init`** | *B*: `1,039s` → **`789.5s` (-24.0%)**; *C*: `31` → **`12` steps (-61.3%)** | *B*: `31` → **`8` `read_file` (-74.2%)**; *C*: `7` → **`1` edit error (-85.7%)** | *B*: `401.1k` → **`201.3k net tok` (-49.8%)**; *C*: **-15.2% gross tok** | **`5/5` → `5/5`** |
| **Pi CLI `1.5.2`** (`deepseek-flash`) · *Task C (`ts-ledger`)* | Vanilla → **`+ Rust ap v1.0`** | `37` → **`35` tool calls** (`301.3s` → `352.4s`) | `11` → **`0` `read` calls (-100%)** *(used bash loop on Windows Git-Bash — see note)* | `538.9k` (`$0.059`) → `858.1k` (`$0.086`) | **`5/5` → `5/5`** |

> **Honest Engineering Note**: With the Pure-Rust `v1.0` Zero-Escaping `@@ FILE:` patcher + self-contained `AGENTS.md` syntax block, **`Codex CLI 0.167`** solved the entire 24-file `ts-ledger` benchmark in just **`4` shell commands** (`ap probe slice . --bundle` on Cmd 1, and patching all files + creating `tests/regressions.test.ts` + passing 15 tests in a single `ap splice apply --cwd . --stdin --test` on Cmd 3, burning only `12.9k` uncached input tokens). Meanwhile, on **`Pi CLI 1.5.2` (`deepseek-flash`)** on Windows, `ap v1.0` eliminated `100%` of `read` tool calls (`11 → 0`), but because `Pi`'s system prompt encourages raw Git-Bash loops (`for f in src/*.ts`), `deepseek-flash` read dead files via `bash` before calling `ap.exe` (`P4` roadmap item). We report every run transparently without cherry-picking.

---

## Deterministic Rust `v1.0.0` Micro-Benchmarks (`bench/results.json`)

Reproduce locally in `<2s` via `ap bench` (Rust `1.100`, Tree-sitter, bundled SQLite FTS5):

| Primitive & Workload | Baseline / Raw | With `ap` (`v1.0` Rust) | Reduction | Native Latency | Guarantee |
| :--- | ---: | ---: | ---: | ---: | :--- |
| **`ap-splice`** · Tree-sitter `view` + Zero-Escaping `apply` + `check` | `1,048 tok` module | `48 tok` exact AST fn | **-95.4% read tok** | **`0.68 ms`** (`apply`: **`2.46 ms`**) | Verifies Tree-sitter AST **before** disk write; preserves CRLF |
| **`ap-recall`** · 200-rule vault (EN + CJK hybrid) | `6,880 tok` monolith | `47.0 tok` *(top-2)* | **-99.3%** | **`3.37 ms`** | **20/20 (100%)** Top-1 accuracy (bundled SQLite FTS5) |
| **`ap-probe`** · 24-file TypeScript repo (`slice`) | `24` `.ts` files | `9` active (`15` pruned) | **-62.5% dead files** | **`6.32 ms`** | 6-lang Tree-sitter import graph (`Py/TS/JS/Go/Rust/C`) |
| **`ap-atlas`** · 64-module Rust SDK corpus (`map`) | `42,338 tok` full read | `438 tok` structural map | **-99.0%** | **`16.3 ms`** | `.gitignore`-aware Tree-sitter symbols, hubs & `>90d` TODOs |
| **`ap-sieve`** · 2,500-line CI log / 400-item JSON / 300-item JSONL | `43.0k` / `27.0k` / `4.8k tok` | `519` / `105` / `146 tok` | **-98.8% / -99.6% / -97.0%** | `61.4` / `13.5` / `10.3 ms` | **`5.65 ms`** FTS5 `search` + lossless `peek` |
| **`ap-sieve`** · Short `<1.2 KB` output *(Negative Control)* | `70 B` (`18 tok`) | `118 B` (`30 tok`) | **0.0%** *(passthrough)* | `9.37 ms` | Short outputs pass through untouched |

---

## Cargo Workspace Layout & Verification

```text
agent-primitives/
├── Cargo.toml / src/{main,install,audit,bench}.rs  # Unified `ap` single binary (embeds all 7 SKILL.md manifests)
├── crates/                                         # 7 physically isolated standalone crates (0 sibling ap-* deps)
│   ├── ap-atlas/   ap-recall/   ap-probe/   ap-splice/   ap-sieve/   ap-guard/   ap-baton/
├── install.sh / install.ps1                        # 1-line SHA-256 verified prebuilt binary installers
├── .github/workflows/{ci.yml,release.yml}          # 3-OS clippy+test gate & 5-target cross-platform binary releases
├── skills/{atlas,recall,probe,splice,sieve,guard,baton}/  # Concise <65-tok SKILL.md + bilingual READMEs
├── HANDBOOK.md / DESIGN.md / PLAYBOOK.md (+ .zh-CN.md)    # 1:1 English & Simplified Chinese engineering docs
└── tests/integration_primitives.rs                 # 11 end-to-end isolation, AST rollback, encoding & hook suites
```

```bash
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
ap doctor
```

## License

[MIT](./LICENSE)
