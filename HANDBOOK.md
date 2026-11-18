# `agent-primitives` Philosophy & Continuous Evolution Handbook

[English](./HANDBOOK.md) | [简体中文](./HANDBOOK.zh-CN.md) · [Home](./README.md) · [Architecture](./DESIGN.md) · [Playbook](./PLAYBOOK.md)

> **Purpose of This Document**: This is the **canonical engineering constitution and living R&D manual** for `agent-primitives`. It defines our core philosophy, our non-negotiable boundaries, and the exact closed-loop iteration protocol we follow on every future update—ensuring this repository stays ultra-lean, empirically proven, and indispensable across model generations.

---

## 1. Core Philosophy (Why We Exist & Why Models Won't Obsolete Us)

### 1.1 Never Build "Thinking Crutches"—Only Build "Physical Organs"
95% of open-source agent repositories become obsolete within 6 months because they try to **teach the LLM how to think** in prompts or heavy outer orchestration wrappers (roleplay personas, planning checklists, multi-agent state machines, or lossy vector RAG). Every frontier post-training cycle (`GPT-6`, `Grok-5`, `Claude-5`, `Gemini-4`, `DeepSeek`) absorbs those crutches into model weights, turning yesterday's scaffolding into today's latency tax.

Our philosophy is the exact opposite: **the smarter the base model becomes, the less prompt hand-holding it wants—and the better it wields sub-10ms, memory-safe, deterministic OS/AST/Heap/Byte instruments.**

No matter how high model IQ climbs, an LLM remains an autoregressive token generator operating over an append-only KV-cache, separated from software and personal files by **four irreducible physical boundaries**:
1. **External Entropy vs. `O(N)` KV-Cache Physics**: Pouring `200 KB` CI logs, giant JSON/JSONL dumps, noisy HTML, or 15 unreferenced dead modules into context physically inflates prefill latency, dilutes attention, and bills every subsequent turn.
2. **Prose Simulation vs. Concrete Runtime Memory & Types**: Mentally simulating shallow-copy nested aliasing, in-place argument mutation, `-0.0` signed-zero drift, or `None` / `True == 1` type collisions in prose burns `35k–55k` reasoning tokens (`11–19 min`), while ad-hoc inline one-liners crash on shell quoting and module path mismatches.
3. **BPE Token Streams vs. Byte/AST-Exact File Transactions**: Models emit BPE tokens, whereas disk files require strict UTF-8 encoding, preserved CRLF/LF line endings, valid concrete Tree-sitter AST syntax across every target language *before* hitting disk, and zero JSON-escaping corruption.
4. **Compaction Amnesia & Self-Report Bias**: Context compaction (`/compact`) wipes negative constraints ("what already failed"), and probabilistic prose can never substitute for a `<2 ms` deterministic **`exit 2`** OS gate.

**Core Law**: We build **Unix-style deterministic primitives in Pure Rust** (`ripgrep` / `git` / `strace` / `sqlite` equivalents for agents) strictly at those four boundaries. Smarter engineers don't stop using `ripgrep` or `git`—and smarter models wield `ap` with even higher precision.

### 1.2 Micro-Infrastructure Aesthetics: Pure-Rust Single Binary + 7 Standalone Crates
We reject monolithic frameworks requiring `Python`, `Node`/`Bun`, `Docker`, background daemons, vector databases, or paid cloud keys:
- **Zero Runtime Prerequisite (`ap` / `ap.exe`)**: One native prebuilt Rust binary statically embeds **6-language Tree-sitter grammars** (`Python`, `TypeScript/TSX`, `JavaScript`, `Go`, `Rust`, `C`), **bundled SQLite 3 (`FTS5` + `BM25`)**, `.gitignore`-aware parallel traversal, and all 7 `SKILL.md` manifests. Users need neither Python nor Rust installed (`curl ... | sh` or `irm ... | iex`).
- **Physical Crate Autonomy**: Every primitive lives in its own isolated Cargo crate (`crates/ap-{atlas,recall,probe,splice,sieve,guard,baton}`) with **`0` sibling `ap-*` dependencies** and its own standalone `main.rs` (`cargo install --path crates/ap-splice` works independently).
- **Near-Zero Idle Context Tax**: Each folder's `SKILL.md` frontmatter costs **`57–61 tokens`** at rest (**`412 tokens`** total across all 7 folders combined).

### 1.3 Unified Substrate for Coding Agents & Personal Assistant Agents
Coding Agents (`Codex`, `Pi`, `Antigravity`, `Claude Code`, `Grok Build`, `Cursor`) and Personal Assistant Agents (`OpenClaw`, `Hermes`, vault/research/ops assistants) share the **exact same underlying process bottlenecks**:
- Codebases need 6-language Tree-sitter hub maps; 500-note Markdown vaults need directory + `H1/H2` outlines (`atlas`).
- Code subsystems have local gotchas; client invoicing, contracts, and server ops have conditional bilingual SOPs (`recall`).
- Code fixes need reachability slicing & heap/boundary tracing; financial rounding and data scripts need differential verification before batch runs (`probe`).
- Source files need `<1 ms` Tree-sitter AST symbol reads & Zero-Escaping `@@ FILE:` atomic edits; long handbooks and JSON configs need section reads & UTF-8-safe atomic writes (`splice`).
- `pytest`/`cargo` floods need lossless compression; REST API JSON/JSONL streams, CSV/TSV exports, and HTML `<table>` pages need noise stripping + `<6 ms` FTS5 search (`sieve`).
- Force-pushes need `<2 ms` pre-execution shields & test truth gates; autonomous assistants need secret-leak & mass-blast **`exit 2`** shields (`guard`).
- Multi-CLI coding handoffs need state capsules; multi-day personal research needs compaction-proof resume (`baton`).

---

## 2. Six Non-Negotiable Constitutional Rules

Any proposed feature or PR that violates even one of these six rules is rejected:

1. **Zero Pure-Prompt Fluff**: Never add prompt-only persona/checklist folders without deterministic native computation.
2. **Strict Physical Crate Isolation**: Every `crates/ap-<name>/` crate must compile and run standalone with **zero dependencies on sibling `ap-*` crates** (enforced in CI by `test_crate_physical_isolation`).
3. **Zero External Runtime Dependencies & Full Cross-OS Parity**: Prebuilt single-file Rust binary (`musl` static on Linux x64, native Mach-O on macOS, native `.exe` on Windows MSVC) with bundled SQLite FTS5 + Tree-sitter; must pass `clippy -D warnings` and integration suites across Linux, macOS, and Windows (including PowerShell pipes, `\\?\` canonical paths, and CRLF preservation).
4. **Hard Context & Latency Budgets**: Idle `SKILL.md` frontmatter **`< 85 tokens`** per folder (`57–61` actual); core CLI latency **`< 50 ms`** (`0.68 ms`–`16.3 ms` actual on AST/SQLite paths); default stdout **`< 1,000 tokens`** with lossless on-demand `search` / `peek`.
5. **Deepen Before Multiplying**: Always prefer sharpening the existing 7 orthogonal primitives over adding an 8th crate unless a genuinely new physical boundary is proven.
6. **Empirical Honesty & Negative Controls**: Every claim must trace to real session telemetry (`ap audit`), deterministic benchmarks (`ap bench`), and live CLI runs—including **honest negative controls and harness-specific limitations**.

---

## 3. Current "1 + 7" Pure-Rust Architecture & Responsibility Boundaries

```text
[Offline Diagnostics] ap audit [--compare]  ──► Scans 5 local harnesses: Codex / Pi / Antigravity / Claude / Grok (0 cloud)
   │
   ├── Stage 1: Orient & Recall  ──► crates/ap-atlas/  (16ms 6-lang Tree-sitter AST/vault map + >90d stale-TODO radar)
   │                             ──► crates/ap-recall/ (3.4ms bundled SQLite FTS5/BM25 + EN/CJK triggered micro-memory)
   ├── Stage 2: Slice & Diagnose ──► crates/ap-probe/  (6.3ms 6-lang Tree-sitter slice --bundle + Py/TS/JS/Rust run/diff)
   ├── Stage 3: Surgical Act     ──► crates/ap-splice/ (0.7ms AST view + 2.5ms Zero-Escaping AST-verified atomic patcher)
   ├── Stage 4: Filter Output    ──► crates/ap-sieve/  (98%+ log/JSON/JSONL/HTML-table/CSV compressor + 5.7ms FTS5 search)
   └── Stage 5: Gate & Persist   ──► crates/ap-guard/  (<2ms dual-protocol exit-2 / JSON hook shield & 3x loop breaker)
                                 ──► crates/ap-baton/  (<400-tok compaction capsule + DO NOT RETRY handoff)
```

---

## 4. The 5-Step Continuous Iteration Protocol

1. **Step 1 — Mine Real Agent Trajectories (`ap audit --compare`)**: Inspect real local trajectories across `Codex`, `Pi`, `Antigravity`, `Claude Code`, and `Grok` (`101,108` tool calls across `427` sessions audited locally). **No observed trajectory friction = no speculative feature.**
2. **Step 2 — Pass the 4 Anti-Obsolescence Gate Questions**: (a) Does higher model IQ erase this or is it a physical boundary? (b) Does it belong inside one of the 7 existing crates? (c) Can an agent invoke it in 1 shot via `--stdin` with zero escaping traps? (d) Where is the honest negative boundary?
3. **Step 3 — Implement in Isolated Rust Crates**: Implement inside `crates/ap-<name>/`, keep `SKILL.md` frontmatter `< 85 tokens`, pass `cargo clippy --workspace --all-targets -- -D warnings`, and add integration/rollback/encoding tests in `tests/integration_primitives.rs`.
4. **Step 4 — Validate via Live Controlled Multi-Harness CLI Runs**: Run real CLIs (`codex`, `pi`, `agy`, `grok`, `claude`) against hidden test oracles and inspect exact tool traces for friction.
5. **Step 5 — Sync Bilingual Docs, Pass 3-OS CI & Publish Cross-Platform Binaries**: Keep English (`*.md`) and Simplified Chinese (`*.zh-CN.md`) 1:1 synchronized, verify Linux/macOS/Windows CI (`ci.yml`), and ship SHA-256 verified prebuilt binaries via `release.yml`.

---

## 5. Shipped Roadmap & Next Frontier Backlog

- [x] **P1 · Polyglot Structural Slicing, Bundling & Auto-Test Runner (`probe` + `splice`)**: Shipped (`v0.3` → upgraded to real 6-language **Tree-sitter** AST in `v1.0.0`).
- [x] **P2 · 1-Line Multi-Harness Onboarding & Cohort Trend Auditor (`ap init` + `ap audit --compare`)**: Shipped (`v0.3` → zero-Python native prebuilt binaries + 5-harness `Pi` support in `v1.0.0`).
- [x] **P3 · 6-Language Tree-Sitter Hub & Stale-TODO Map (`ap-atlas`)**: Shipped in `v1.0.0` — native Tree-sitter signatures, `.gitignore`-aware hub scoring, `[dirty]` git markers & `>90d` stale-TODO warnings across Python, TS/TSX, JS, Go, Rust, and C (`16.3 ms` on 64 Rust modules).
- [x] **P3 · Personal Assistant HTML `<table>`, CSV/TSV, JSONL Stream & FTS5 Search (`ap-sieve` + `ap-splice`)**: Shipped in `v1.0.0` — lossless HTML `<table>` → Markdown pipe-table extraction, CSV/TSV summaries, JSONL schemas, `<6 ms` bundled SQLite FTS5 `sieve search`, and Zero-Escaping `@@ FILE:` / `@@ SYMBOL:` / `@@ WRITE:` patching.
- [ ] **P4 · Cross-Language Call-Graph Symbol Impact Cone (`ap-probe impact <symbol>`)**: Given a modified function/type signature, list exact downstream call sites across Py/TS/Go/Rust/C in `<15 ms` before patching.
- [ ] **P4 · Native Windows Git-Bash PATH Bridge for `Pi` (`pi-coding-agent`)**: Auto-emit POSIX forward-slash `.agents/bin/ap` wrapper hints so `Pi`'s Git-Bash shell on Windows invokes `probe slice --bundle` on Turn 1 without falling back to `for f in src/*` loops.

---

## 6. Evolution Log

| Date / Milestone | Shipped Capabilities | Real Bottleneck Addressed | Verified Empirical Proof |
| :--- | :--- | :--- | :--- |
| **2026-11-15 · `v0.1` Foundation** | Initial `sieve`, `baton`, `recall`, `guard`, `atlas` + deterministic benchmarks w/ negative controls | Context rot, compaction amnesia, `AGENTS.md` bloat, false completions & stale TODO traps | CI logs **-98.8%**, JSON **-99.6%**, 200-rule EN+CJK recall **100% Top-1** |
| **2026-11-15 · `v0.2`–`v0.2.1` Multi-CLI Upgrade & Handbook** | Audited `69,963` calls; shipped `probe` (`slice`/`run`/`diff`), `splice` (`view`/`apply`/`check`) & bilingual `HANDBOOK` | `50.1%` same-file re-reads, `55k`-tok prose heap simulation, quoting crashes & Windows `cp936` errors | `Grok 1.0.77` wall time **-49.1%** & reasoning **-65.4%**; `Antigravity 1.5.8` net tok **-49.8%** & `read_file` **-74.2%** |
| **2026-11-15 · `v0.3` Polyglot + 1-Line Multi-Harness** | 1-line `init` across `Codex`/`Pi`/`Antigravity`/`Claude`/`Grok`, `slice --bundle`, `check --test`, dual-protocol hooks & `audit --compare` | 15-file sequential `read_file` waterfalls, 82KB loop trace spills & per-harness config friction | TS `ts-ledger` (`5/5` oracle): `Grok 1.0.77` **-26.5% time**, **-38.7% tok**; `Antigravity 1.5.8` **-61.3% steps**, **-85.7% edit errors** |
| **2026-11-18 · `v1.0.0` Pure-Rust Single-Binary Engine** | Complete **Pure-Rust rewrite (`0` `.py` files)**: single native `ap` binary + 7 isolated Cargo crates (`ap-*`), 6-lang **Tree-sitter** AST (`Py/TS/JS/Go/Rust/C`), **bundled SQLite FTS5**, Zero-Escaping `@@ FILE:` AST-verified patcher, Node V8 `Proxy` + `rustc` probes, `sieve search` + HTML-table/JSONL/CSV, `101,108`-call 5-harness `ap audit`, and `install.sh`/`install.ps1` + 5-target GitHub Releases | Eliminates Python/interpreter startup overhead (`40–90ms` → `0.7–16ms`), brace-heuristic edge cases, JSON patch escaping errors, BrokenPipe crashes, Windows `\\?\` Node path bugs & Python runtime prerequisites | `splice view` **`0.68ms` (-95.4% tok)**, `apply` **`2.46ms`**, `recall` **`3.37ms` (20/20)**, `probe slice` **`6.32ms`**, `atlas` **`16.3ms`**, `412 tok` total idle; live `Codex 0.167 + Rust ap v1.0` solved 24-file TS `ts-ledger` (`5/5` oracle) in **`4` total commands** (`161.8k tok`, `0` manual file reads, 13 edits + 15 tests in 1 atomic `ap splice apply --stdin --test` call); `11/11` Rust integration suites & `0` clippy warnings |
