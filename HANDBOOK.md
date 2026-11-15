# `agent-primitives` Philosophy & Continuous Evolution Handbook

[English](./HANDBOOK.md) | [简体中文](./HANDBOOK.zh-CN.md) · [Home](./README.md) · [Architecture](./DESIGN.md) · [Playbook](./PLAYBOOK.md)

> **Purpose of This Document**: This is the **canonical engineering constitution and living R&D manual** for `agent-primitives`. It defines our core philosophy, our non-negotiable boundaries, and the exact closed-loop iteration protocol we follow on every future update—ensuring this repository stays ultra-lean, empirically proven, and indispensable across model generations.

---

## 1. Core Philosophy (Why We Exist & Why Models Won't Obsolete Us)

### 1.1 Never Build "Thinking Crutches"—Only Build "Physical Organs"
95% of open-source agent repositories become obsolete within 6 months because they try to **teach the LLM how to think** in prompts or heavy outer orchestration wrappers (roleplay personas, planning checklists, multi-agent state machines, or lossy vector RAG). Every frontier post-training cycle (`GPT-6`, `Grok-5`, `Claude-5`, `Gemini-4`) absorbs those crutches into model weights, turning yesterday's scaffolding into today's latency tax.

Our philosophy is the exact opposite: **the smarter the base model becomes, the less prompt hand-holding it wants—and the better it wields fast, deterministic OS/AST/Heap/Byte instruments.**

No matter how high model IQ climbs, an LLM remains an autoregressive token generator operating over an append-only KV-cache, separated from software and personal files by **four irreducible physical boundaries**:
1. **External Entropy vs. `O(N)` KV-Cache Physics**: Pouring `200 KB` CI logs, giant JSON API dumps, noisy HTML, or 20 unreferenced dead modules into context physically inflates prefill latency, dilutes attention, and bills every subsequent turn.
2. **Prose Simulation vs. Concrete Runtime Memory & Types**: Mentally simulating shallow-copy nested aliasing, in-place argument mutation, or `None` / `True == 1` type collisions in prose burns `35k–55k` reasoning tokens (`11–19 min`), while ad-hoc `python -c` one-liners crash on shell quoting and `ImportError`.
3. **BPE Token Streams vs. Byte/AST-Exact File Transactions**: Models emit BPE tokens, whereas disk files require strict UTF-8 encoding, preserved CRLF/LF line endings, valid AST syntax, and clean `.pyc` bytecode state.
4. **Compaction Amnesia & Self-Report Bias**: Context compaction (`/compact`) wipes negative constraints ("what already failed"), and probabilistic prose can never substitute for a deterministic **`exit 2`** OS gate.

**Core Law**: We build **Unix-style deterministic primitives** (`git` / `strace` / `sqlite` equivalents for agents) strictly at those four boundaries. Smarter engineers don't stop using `git` or `strace`—and smarter models wield `agent-primitives` with even higher precision.

### 1.2 Micro-Infrastructure Aesthetics: One Folder = One Standalone Mini-Project
We reject monolithic frameworks requiring `Docker`, `Node`/`Bun`, background daemons, vector databases, or paid cloud keys:
- **Single-Folder Autonomy**: Every folder under `skills/` is a self-contained mini-project. Want lossless output compression? Copy only `skills/sieve/`. Want sub-100ms causal debugging and atomic AST patching? Copy only `skills/probe/` and `skills/splice/`.
- **Zero External Dependencies**: 100% Python 3.9+ standard library (`0` pip packages, `0` daemons, `0` network uploads, `0` API keys) across Linux, macOS, and Windows.
- **Near-Zero Idle Context Tax**: Each folder's `SKILL.md` frontmatter costs **`63–83 tokens`** at rest (`< 500 tokens` for all 7 folders combined).

### 1.3 Unified Substrate for Coding Agents & Personal Assistant Agents
Coding Agents (`Claude Code`, `Codex`, `Grok Build`, `Antigravity`, `Cursor`) and Personal Assistant Agents (`OpenClaw`, `Hermes`, vault/research/ops assistants) share the **exact same underlying process bottlenecks**:
- Codebases need AST hub maps; 500-note Markdown vaults need directory + `H1/H2` outlines (`atlas`).
- Code subsystems have local gotchas; client invoicing, contracts, and server ops have conditional SOPs (`recall`).
- Code fixes need heap/boundary tracing; financial rounding and data-cleaning scripts need edge-case verification before batch runs (`probe`).
- Source files need AST symbol reads & atomic edits; long handbooks and JSON configs need section reads & UTF-8-safe atomic writes (`splice`).
- `pytest` floods need lossless compression; REST API JSON payloads and scraped web HTML need noise stripping (`sieve`).
- Force-pushes need pre-execution shields & test truth gates; autonomous assistants need secret-leak & mass-blast **`exit 2`** shields (`guard`).
- Multi-CLI coding handoffs need state capsules; multi-day personal research needs compaction-proof resume (`baton`).

---

## 2. Six Non-Negotiable Constitutional Rules

Any proposed feature or PR that violates even one of these six rules is rejected:

1. **Zero Pure-Prompt Fluff**: Never add prompt-only persona/checklist folders without deterministic runtime computation.
2. **Strict Physical Isolation**: Every `skills/<name>/` folder must run with **zero cross-folder imports** when copied alone into an empty directory (enforced in CI by `test_standalone_no_cross_imports`).
3. **Zero Third-Party Packages & Full Cross-OS Parity**: Pure Python 3.9+ stdlib only; must pass natively on Ubuntu, macOS, and Windows (including PowerShell quoting and `cp936` default codepages).
4. **Hard Context & Latency Budgets**: Idle `SKILL.md` frontmatter **`< 85 tokens`** per folder; core CLI latency **`< 100 ms`**; default stdout **`< 1,000 tokens`** with lossless on-demand `peek`.
5. **Deepen Before Multiplying**: Always prefer sharpening the existing 7 orthogonal primitives (subcommands, multi-language support, zero-friction ergonomics) over adding an 8th folder unless a genuinely new physical boundary is proven.
6. **Empirical Honesty & Negative Controls**: Every claim must trace to real session telemetry (`audit_sessions.py`), deterministic benchmarks (`bench/run_bench.py`), and live CLI A/B runs—including **honest negative controls** documenting when *not* to invoke a tool.

---

## 3. Current "1 + 7" Architecture & Responsibility Boundaries

```text
[Offline Diagnostics] audit/audit_sessions.py  ──► Scans local Codex / Claude / Grok / Antigravity logs (0 cloud)
   │
   ├── Stage 1: Orient & Recall  ──► skills/atlas/  (Sub-second AST/vault map + >90-day stale-TODO radar)
   │                             ──► skills/recall/ (<4ms SQLite FTS5 + EN/CJK triggered micro-memory)
   ├── Stage 2: Slice & Diagnose ──► skills/probe/  (Reachability dead-file pruning + heap/mutation tracer + diff)
   ├── Stage 3: Surgical Act     ──► skills/splice/ (AST/Markdown section view + UTF-8/CRLF atomic rollback patcher)
   ├── Stage 4: Filter Output    ──► skills/sieve/  (98%+ log/JSON/HTML compression + FTS5 lossless peek)
   └── Stage 5: Gate & Persist   ──► skills/guard/  (Native exit-2 secret/blast shield, 3x loop breaker & stop-gate)
                                 ──► skills/baton/  (<400-tok compaction capsule + DO NOT RETRY handoff)
```

---

## 4. The 5-Step Continuous Iteration Protocol

Every future iteration around this repository follows a strict five-step engineering loop:

### Step 1 — Mine Real Agent Trajectories (`Telemetry First`)
After real daily work in `Grok`, `Codex`, `Antigravity`, `Claude Code`, or `OpenClaw`, run:
```bash
python audit/audit_sessions.py
```
Or inspect concrete sessions where an agent stalled, burned `30k+` thinking tokens, hit shell/encoding crashes, or looped. **No observed trajectory friction = no speculative feature.**

### Step 2 — Pass the 4 Anti-Obsolescence Gate Questions
Before writing code, answer:
1. *If next year's model doubles in reasoning IQ, does this bottleneck disappear on its own, or is it a physical boundary?* (Build only the latter.)
2. *Does this belong inside one of the existing 7 primitives (e.g., adding TypeScript/Go symbol spans to `splice`), or is it truly orthogonal?*
3. *Can an agent invoke it in 1 shot via `--stdin` without shell-quoting traps or needing `--help`?*
4. *Where is the honest negative boundary where calling it adds overhead?*

### Step 3 — Build Zero-Dep & Enforce Isolation Tests
- Implement inside `skills/<folder>/scripts/<name>.py` and verify frontmatter stays `< 85 tokens` via `python bench/run_bench.py`.
- Add unit, rollback, CRLF/UTF-8 encoding, and standalone temp-dir isolation tests in `tests/test_primitives.py`.

### Step 4 — Validate via Live Controlled Multi-Harness CLI A/B Runs
- Test Vanilla vs. `+ Primitive` on live CLIs (`grok`, `codex`, `agy`, `claude`) with identical prompts and an independent hidden test oracle.
- Inspect real traces for UX friction (e.g., parameter naming, missing auto-detection) and polish until wall-clock time, thinking spikes, crash rates, or token throughput show clear, reproducible gains.

### Step 5 — Sync Bilingual Docs, Pass 6-Way Matrix CI & Log Evolution
- Keep English (`README.md`, `HANDBOOK.md`, `PLAYBOOK.md`, skill `README.md`s) and Simplified Chinese (`*.zh-CN.md`) 1-to-1 synchronized.
- Verify `python -m unittest discover -s tests -v`, `python bench/run_bench.py`, and GitHub Actions (`ubuntu`, `macos`, `windows` × Python `3.10`, `3.12`), then record the milestone in Section 6 below.

---

## 5. Active Iteration Backlog (Grounded in Observed Live Traces)

- [x] **P1 · Polyglot Structural & Symbol Support (`probe slice --bundle` + `splice`)**: Shipped in `v0.3` — zero-dependency lexical/brace-aware import reachability slicing, active code bundling, and symbol `view` + `replace_symbol` across **Python, TypeScript/JavaScript, Go, and Rust** (handling Rust `'a` lifetimes vs. char literals).
- [x] **P1 · Zero-Guess Test Runner Auto-Detection (`splice check --test`)**: Shipped in `v0.3` — auto-detects `pytest` vs. stdlib `unittest` vs. `node --experimental-strip-types --test` vs. `go test` vs. `cargo test` with bounded failure-preserving output.
- [x] **P2 · Before/After Efficiency Delta Mode in `audit_sessions.py` (`--compare`)**: Shipped in `v0.3` — compares older vs. newer local session cohorts per harness.
- [x] **P2 · 1-Line Zero-Clone Multi-Harness Onboarding (`ap.py init` + Native Manifests)**: Shipped in `v0.3` — 1-line `curl`/`irm | python - init` zero-clone bootstrapper across **`Codex`**, **`Pi`**, **`Antigravity`**, **`Claude Code`**, **`Grok`**, **`OpenClaw`**, and **`Hermes`**, dual-protocol `guard.py` hooks (Claude/Codex `exit 2` + Antigravity camelCase `deny`/`continue`), plus native `pi install`, `codex plugin`, `agy plugin`, and `Claude` plugin manifests.
- [ ] **P3 · Polyglot Structural Hub & Stale-TODO Map (`atlas`)**: Extend `atlas map` symbol signature & import centrality extraction to TypeScript/Go/Rust monorepos.
- [ ] **P3 · Personal Assistant Table, Email-Thread & JSONL Stream Extractors (`sieve` + `splice`)**: Higher-density extraction for wide HTML/Markdown tables, paginated APIs, and multi-message threads.

---

## 6. Evolution Log

| Date / Milestone | Shipped Capabilities | Real Bottleneck Addressed | Verified Empirical Proof |
| :--- | :--- | :--- | :--- |
| **2026-11-15 · `v0.1` Foundation** | Shipped `sieve`, `baton`, `recall`, `guard`, `atlas` + `install.py` + deterministic `bench/run_bench.py` w/ negative controls | Context rot, compaction amnesia, `AGENTS.md` bloat, false completions, cold-start wandering & stale TODO traps | CI logs **-98.8%**, JSON **-99.6%**, 200-rule EN+CJK recall **`4.0ms` (100% Top-1)**, `360 tok` idle |
| **2026-11-15 · `v0.2` Multi-CLI Empirical Upgrade** | Audited `69,963` real calls across 4 CLIs; shipped `audit/audit_sessions.py`, `skills/probe/` (`slice`/`run`/`diff`) & `skills/splice/` (`view`/`apply`/`check`) | `50.1%` same-file re-reads, `55k`-token prose heap simulation, `python -c` quoting crashes, Windows `cp936` errors & stale `.pyc` | `Grok 1.0.77` wall time **-49.1%** & reasoning **-65.4%**; `Antigravity 1.5.8` gross tokens **-53.4% (-506k tok)** & steps **-41.5%**; `10/10` tests & 6-job CI green |
| **2026-11-15 · `v0.2.1` Living Handbook & Repo Constitution** | Established bilingual `HANDBOOK.md` / `HANDBOOK.zh-CN.md` and repo-level `AGENTS.md` | Codifies the 4-boundary anti-obsolescence philosophy, 6 constitutional laws, 5-step iteration loop & backlog | Full bilingual parity & CI matrix verified |
| **2026-11-15 · `v0.3` Polyglot + 1-Line Multi-Harness Onboarding** | Shipped zero-clone `ap.py` (`curl`/`irm \| python - init`), native manifests (`pi`, `codex`, `agy`, `claude`), polyglot Py/TS/JS/Go/Rust `probe slice --bundle` & `splice` + `check --test`, `probe run` loop-dedup, dual-protocol `guard.py` hooks, and `audit --compare` | Eliminates 15-file sequential `read_file` waterfalls, 82KB loop trace spills, PowerShell quoting traps, Grok global-skill shadowing, and per-harness config friction | Live TS `ts-ledger` (`5/5` oracle): `Grok 1.0.77` **-26.5% time**, **-66.7% `read_file`**, **-38.7% tok (-34.4% cost)**; `Antigravity 1.5.8` **-61.3% steps**, **-85.7% edit errors** (Py `rulelang`: **-24.0% time**, **-74.2% `read_file`**, **-49.8% net tok**); `Codex 0.167` **-7.3% time**; `14/14` tests green |
