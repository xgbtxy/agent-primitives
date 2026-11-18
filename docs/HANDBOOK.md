# `agent-primitives` (`ap` v1.2) — Core R&D Philosophy & Continuous Evolution Handbook

[English](./HANDBOOK.md) | [简体中文](./HANDBOOK.zh-CN.md) · [Home](../README.md) · [Architecture](./DESIGN.md) · [Playbook](./PLAYBOOK.md)

> **Mission**: We do not build heavy multi-agent wrappers or preachy prompt checklists (AI Slop). Our engineering method is direct and empirical: **observe how real coding agents actually execute and think in the wild, deeply survey and synthesize battle-tested mechanisms from mature open-source projects, and ship ultra-lean Pure-Rust primitives that elevate core coding capability**—so even a complete beginner using a `$0` free coding agent gets measurable speed, token, and code-quality gains.

---

## 1. Our Three Core Engineering Principles

### 1.1 Observe Real Coding-Agent Execution & `<thinking>` Processes First
Speculative features are rejected. Every upgrade starts by inspecting **`101,108+` real local tool calls and raw `<thinking>` streams** across `OpenCode`, `Codex`, `Pi`, `Claude Code`, `Antigravity`, `Qwen Code`, `Gemini CLI`, and `Grok` on authentic user prompts (1-sentence beginner wishes, legacy bugfixes, and empty-directory random projects):
- **Turns 1–5 (Cold-Start Guessing & Slop Contamination)**: Agents burn `10k–35k` thinking chars guessing local runtimes (`node:sqlite`? `--experimental-strip-types` `enum` traps? headless browser?) or reading unreferenced legacy files and copying `: any` and silent `catch {}` patterns.
- **Turns 6–15 (Cross-File Working-Memory Drift & Patch Retries)**: Across 6+ files, models mismatch HTML `id="..."` vs. JS `getElementById`, mismatch `fetch('/api/...')` vs. backend routes, or fail edits due to 2-space indentation shifts and Windows PowerShell quoting.
- **The Last 70%+ of Turns & Tokens (Verification Bottlenecks)**: In live Windows runs, vanilla agents spent **`71%–75%` of all turns** fighting hung background servers, orphaned ports, and hand-rolled 150-line Chrome DevTools scripts.
- **Why Prose Rules Fail**: Long `AGENTS.md` reminders get ignored under context load; only deterministic tool checks and **`exit 2` physical gates** force self-discovery and self-repair before completion.

### 1.2 Survey & Synthesize Mature Open-Source Engineering (Never Reinvent 0-to-1 Slop)
Instead of reinventing fragile wheels from scratch, we study proven open-source systems and distill their sharpest mechanisms into one zero-dependency Rust binary:
- **From `Aider`**: Tree-sitter import-graph **Personalized PageRank (`PPR`, 8 power iterations)** for high-signal repo maps, plus **3-tier edit matching (exact → trailing-whitespace normalized → relative indentation-delta shift)** to eliminate whitespace/indentation patch retries.
- **From `RTK (rtk-ai/rtk)`**: Transparent `PreToolUse` `updatedInput` command rewriting to pipe noisy test runners through lossless log compression automatically.
- **From `SWE-agent` (ACI)**: Pre-write 6-language Tree-sitter syntax verification, closest-anchor line hints (`closest anchor at L42`) on edit misses, and cold-start host runtime fingerprinting.
- **From Official `MCP` & Local Ecosystem Standards**: Built-in **`ap mcp`** JSON-RPC 2.0 stdio server so free models call structured tools with zero shell-quoting errors, native `TcpStream` **Headless Edge/Chrome CDP** (`OPEN`, `FILL`, `SELECT`, `CLICK`, `EVAL`), and automatic delegation to installed `tsc`, `biome`, `cargo check`, and `pytest`.

### 1.3 Measure Everything by Core Coding Elevation & Zero-Slop Code Quality
- **Effortless for Beginners**: One command installs prebuilt native `ap` (no Python, Node, or Rust required) and auto-wires free & commercial agents. Beginners never learn CLI flags—they type plain everyday requests.
- **Senior-Engineer Rigor**: Sub-20ms core paths (`0.7–18 ms`), `OnceLock` single-compiled regexes (zero hot-loop recompilation), 7 isolated crates (`0` sibling `ap-*` dependencies), `0` `clippy -D warnings`, `390 tokens` total idle frontmatter, and hard anti-slop enforcement (`0` `: any`, `0` `@ts-ignore`, `0` silent `catch {}`).

---

## 2. The 4-Step Continuous Iteration Loop

Every future iteration on this repository follows four strict steps:

1. **Step 1 — Observe**: Run `ap audit --compare` and execute real user tasks across free and frontier coding agents, dissecting tool traces and `<thinking>` spikes.
2. **Step 2 — Survey**: Research how mature open-source projects solve that exact bottleneck; extract proven algorithms and standards rather than piling on prompts.
3. **Step 3 — Craft in Pure Rust**: Implement cleanly inside the appropriate isolated `crates/ap-*` crate (exposing both CLI and `ap mcp`), passing `rustfmt`, `clippy -D warnings`, and cross-OS tests.
4. **Step 4 — Verify**: Run controlled Vanilla vs. `+ ap` A/B benchmarks on real tasks, verifying speed, token, reliability, and code-hygiene gains transparently.

---

## 3. Architecture & Evolution Log

```text
[1-Line ap init] ──► Auto-wires OpenCode / Qwen / Gemini CLI / Goose / Pi / Codex / Claude / Antigravity / Grok
   ├── Orient   (ap_orient) ──► ap-atlas (PPR map + host fingerprint + DOM #id/route contract & slop radar) + ap-probe slice
   ├── Edit     (ap_edit)   ──► ap-splice (Aider 3-tier indentation-resilient + zero-escaping AST-gated multi-file patcher)
   ├── Smoke    (ap_smoke)  ──► ap-probe smoke (ephemeral PORT + Pure-Rust headless Edge/Chrome interactive CDP + tree kill)
   ├── Verify   (ap_verify) ──► ap-guard (exit-2 anti-slop rules + tsc/biome/cargo/test delegation + RTK auto-sieve)
   └── Diagnose (ap_probe)  ──► ap-probe run/diff + ap-sieve (-98.8% log compression) + ap-recall (3.4ms BM25) + ap-baton
```

| Milestone | Observed Process Bottleneck & Open-Source Synthesis | Verified Live Impact |
| :--- | :--- | :--- |
| **`v0.1`–`v0.3` (2026-11-15)** | Audited `69k` calls for re-reads & log floods; shipped 7 primitives & 1-line multi-harness setup | `Grok 1.0.77` **-26.5% to -49.1% time**; `Antigravity 1.5.8` **-49.8% net tok**, **-74.2% `read_file`** |
| **`v1.0.0` (2026-11-18)** | Eliminated Python startup lag & runtime deps via **Pure-Rust single binary + 7 isolated crates** (6-lang Tree-sitter + bundled SQLite FTS5) | `splice view` **`0.71ms`**, `apply` **`2.33ms`**, `recall` **`3.36ms`**; `Codex` solved 24-file `ts-ledger` (`5/5`) in **`4` commands** |
| **`v1.2.0` (2026-11-18)** | Dissected `<thinking>` verification tax & cross-file drift; synthesized **Aider PPR & 3-tier patcher**, **RTK auto-sieve**, **interactive CDP**, and **`ap mcp`** | **`OpenCode Free ($0)` beginner prompt**: **`2.36x` faster (`430s→182s`), `-75.1%` tok, `7/8→8/8` (`0` slop)**; **Empty-dir `SplitMate`**: `Codex` **`-83.3%` tok (`6.0x` less)**, `Pi` **`5/6→6/6` (`0` slop, `-322s`)** |
| **`v1.3.0` (2026-11-18)** | Mined 160 real Windows sessions exposing **`528` OS/shell crashes** (`235` GBK/BOM, `138` PS `&&`/`curl`/heredoc/PATH, `82` port/file locks, `73` pipe hangs); built **Windows Transparent Execution Shield (`ap exec` + Base64 `PreToolUse` Hook)**, **UTF-8 BOM / UTF-16LE auto-clean**, **`ap stats` (`ap gain`) offline value ledger**, and simplified user UX to 3 commands (`init`/`doctor`/`stats`) | **5-Trap Windows 11 Torture Test**: stock PS 5.1 crashed on all 5 traps (`UnicodeEncodeError`, `&&` & heredoc `ParserError`s, `EADDRINUSE`); `ap exec` **100% auto-healed first try**, compressed 2,000-line build logs by **`-99.1%` (`110.3KB → 0.9KB`)**, with instant `ap stats` accounting |
| **`v1.4.0` (2026-11-18)** | Targeted multi-file **2-of-7 Partial Refactoring** ceiling & OS orphan resilience: built **`<15ms` Tree-sitter AST Impact Engine (`atlas impact` / `ap_impact`)**, **Proactive Post-Edit `[BLAST RADIUS]` caller warnings**, **Win32 NT Kernel `Job Object` (`KILL_ON_JOB_CLOSE`)**, and **`Cursor`/`Windsurf`/`VS Code` MCP auto-wiring** | **14-File Multi-Module Refactor (`PayLedger`)**: `Codex` **`151.6s → 96.6s` (`1.57x` faster), `-57.4%` tok (`296k → 126k`), `7/7`**; `Pi (deepseek-flash)` **`102.6s → 90.5s`, `-24.0%` tok (`256k → 195k`), `7/7` (`0` slop)** |
| **`v1.5.0` (2026-11-18)** | Expanded from coding-only to **Universal General-Agent Substrate**: built **Zero-Python Office/Data/Web Reader (`ap read` / `ap_read` for `.docx`, `.xlsx`, `.pptx`, `.pdf`, `GBK`/`UTF-16` & URLs)**, **Non-Git Safety Checkpoints (`ap undo` + `>75%` truncation guard)**, and **`Claude Desktop` auto-wiring** | **Non-Coding Office Task on No-Python Windows PC (`.docx` + `.xlsx` + `GBK .csv`)**: Vanilla dumped binary ZIP garbage & failed `import zipfile` (`70.5s`, `126.8k` tok) → **`+ ap v1.5.0`**: **`46.5s` (`1.52x` faster), `58.0k` tok (`-54.3%`), `5/5`** |
