# `agent-primitives` (`ap` v1.2) — Engineering Constitution & Evolution Handbook

[English](./HANDBOOK.md) | [简体中文](./HANDBOOK.zh-CN.md) · [Home](./README.md) · [Architecture](./DESIGN.md) · [Playbook](./PLAYBOOK.md)

> **Mandate**: Build zero-slop, sub-20ms, Pure-Rust deterministic instruments grounded directly in **how LLMs actually think, drift, and verify**—so even a complete beginner using a **$0 free coding agent on Windows** gets senior-engineer reliability and massive token/time savings out of the box.

---

## 1. Core Philosophy: Upgrade the LLM's Cognitive Loop, Never Write Prompt Fluff

### 1.1 Map Every Primitive to an Observed `<thinking>` Bottleneck
Dissecting over **`101,108` real local tool calls** and hundreds of thousands of raw `<thinking>` tokens across `Codex`, `OpenCode`, `Pi`, `Antigravity`, `Claude Code`, and `Grok` reveals **four universal cognitive bottlenecks**:

1. **Cold-Start Runtime & Graph Blindness (`Turn 1–5`)**: Models burn `10k–35k` thinking chars guessing local runtimes (`node:sqlite`? `--experimental-strip-types` `enum` traps? headless browser?) or wandering unreferenced legacy slop files.
   - **Organ (`ap-atlas` + `ap-probe slice`)**: Personalized PageRank (`PPR`, 8 power iterations over Tree-sitter imports) + cold-start Host Runtime Fingerprinting + dead-file pruning (`6.35 ms`).
2. **Cross-File Working-Memory Drift & Patch Friction (`Turn 6–15`)**: Across 6+ files, models drop HTML `#id` ↔ JS `getElementById` contracts, mismatch `fetch('/api/...')` ↔ backend routes, or fail edits on 2-space indentation drift and Windows PowerShell quoting.
   - **Organ (`ap-atlas product` + `ap-splice` + `ap mcp`)**: Static DOM `#id` & HTTP route contract verifier (`<17 ms`), Aider-inspired **3-tier exact → rstrip → indentation-delta** AST-gated patcher (`2.33 ms`), and structured JSON-RPC **`ap mcp`** (`0` shell quoting).
3. **The 70% Verification Token Tax & Windows Server Hangs (`Turn 16–60`)**: In live Windows runs, vanilla agents spent **71%–75% of all turns** (`16–35` turns, `850k+` tokens) fighting hung background servers, orphaned ports, and raw Chrome DevTools WebSockets.
   - **Organ (`ap-probe smoke`)**: Pure-Rust ephemeral server spawner (file-backed logs = `0` Windows pipe deadlocks) + native `TcpStream` **Interactive Headless Edge/Chrome CDP** (`OPEN`, `FILL`, `SELECT`, `CLICK`, `EVAL`, automatic `pageerror`/`console.error` capture) + `taskkill /T /F` tree cleanup in **1 single `<4.5s` turn**.
4. **Prose Rule Amnesia & AI Slop (`AGENTS.md` Is Not Enough)**: Models ignore prose reminders under load, writing `: any`, `@ts-ignore`, or silent `catch (e) {}` blocks that swallow failures.
   - **Organ (`ap-guard` + `ap-sieve` + `ap-baton` + `ap-recall`)**: Unbypassable `<2 ms` **`exit 2`** code rules (`.agents/guard.json`), ecosystem checker delegation (`tsc`/`biome`/`cargo check`/tests), `rtk`-style transparent `PreToolUse` `updatedInput` auto-sieve compression, and `<400-tok` `DO NOT RETRY` capsules.

### 1.2 Zero-Brain Beginner UX + Best-of-Breed Ecosystem Leverage (No 0-to-1 Reinvention)
- **1 Line for Beginners (`ap init`)**: Auto-detects **free & commercial agents** (`OpenCode`, `Qwen Code`, `Gemini CLI`, `Goose`, `Pi`, `Codex`, `Antigravity`, `Claude Code`, `Grok`, `OpenClaw`, `Hermes`), auto-wires native **`ap mcp`** (`opencode.json`, `.qwen/settings.json`, `.gemini/settings.json`) + concise `<140-tok` rules + hooks. A beginner never learns `ap` flags—their free AI automatically calls `ap_orient`, `ap_edit`, `ap_smoke`, and `ap_verify`.
- **Leverage Mature Standards**: Speaks standard **MCP JSON-RPC 2.0**, embeds official **Tree-sitter** C grammars & **SQLite FTS5**, drives system **Edge/Chrome CDP**, and delegates to installed `tsc`/`biome`/`cargo`/`pytest` rather than reinventing compilers.

---

## 2. Six Non-Negotiable Senior-Engineer Laws

1. **Zero AI Slop**: No preachy prompt walls, no `Regex::new` inside inner loops, no swallowed errors, no `: any` shortcuts, `0` `clippy -D warnings` across all crates.
2. **Strict Physical Crate Isolation**: All 7 crates (`crates/ap-*`) maintain **`0` sibling `ap-*` dependencies** (`test_crate_physical_isolation_and_frontmatter_budget`).
3. **Zero External Runtime Prerequisite**: Single prebuilt Rust binary (`ap` / `ap.exe`) across Linux (`musl` + `arm64`), macOS (`arm64` + `x64`), and Windows (`x64 MSVC`).
4. **Hard Token & Latency Budgets**: Idle `SKILL.md` frontmatter **`< 65 tokens`** per folder (`48–61` actual = **`390 tokens` total**); core AST/SQLite/Hook paths **`0.7–18 ms`**.
5. **Tools Enforce Rules, Not Prose**: Every recurring agent mistake must become an executable rule (`ap guard rule add`), contract check (`atlas product`), or smoke step (`probe smoke`).
6. **Empirical Honesty**: Test on ordinary user & beginner prompts (both legacy bugfixes and empty-dir random projects) across paid & `$0` free agents, reporting all metrics transparently.

---

## 3. Architecture Overview (`v1.2.0`)

```text
[ap init] ──► Auto-wires Native `ap mcp` + Concise Rules + Hooks across OpenCode / Qwen / Gemini / Pi / Codex / Claude / Agy
   │
   ├── ap_orient  ──► ap-atlas (PPR RepoMap + Host Runtime + DOM #id / API Contract & Slop Radar) + ap-probe slice
   ├── ap_edit    ──► ap-splice (Aider 3-tier indentation-resilient + Zero-Escaping AST-gated multi-file patcher)
   ├── ap_smoke   ──► ap-probe smoke (Windows-safe server + Interactive Headless Edge/Chrome CDP + taskkill /T /F)
   ├── ap_verify  ──► ap-guard verify/stop-gate (exit-2 anti-slop rules + tsc/biome/cargo/tests + RTK auto-sieve)
   └── ap_probe   ──► ap-probe run/diff + ap-sieve (-98.8% logs + FTS5) + ap-recall (3.4ms BM25) + ap-baton
```

---

## 4. Evolution Log

| Milestone | Key Engineering Upgrades | Verified Real-World Impact |
| :--- | :--- | :--- |
| **`v0.1`–`v0.3` (2026-11-15)** | Initial 7 primitives, 4-CLI session audit (`69,963` calls), polyglot `slice --bundle` & 1-line onboarding | `Grok 1.0.77` **-26.5%–49.1% time**; `Antigravity 1.5.8` **-49.8% net tok**, **-74.2% `read_file`** |
| **`v1.0.0` (2026-11-18)** | Complete **Pure-Rust single-binary rewrite (`0` `.py` files)**: 7 isolated crates, 6-lang Tree-sitter, bundled SQLite FTS5, Zero-Escaping `@@ FILE:`, 5-target GitHub Releases | `splice view` **`0.71ms`**, `apply` **`2.33ms`**, `recall` **`3.36ms`**; `Codex` solved 24-file `ts-ledger` (`5/5`) in **`4` commands** |
| **`v1.2.0` (2026-11-18)** | **Thinking-Process & Zero-Config Free-Agent Engine**: Pure-Rust Interactive CDP (`OPEN`/`FILL`/`SELECT`/`CLICK`/`EVAL`), DOM `#id`↔JS & route contract radar, Personalized PageRank, Aider 3-tier indentation patcher, `rtk`-style PreToolUse auto-sieve, and native **`ap mcp`** for `OpenCode`/`Qwen`/`Gemini` | **Ordinary User `WorkPulse`**: `Codex` tok **-65.4% (`1.05M → 365k`)**; `Pi` time **-43.8% (`483s → 272s`)** & thinking **-42.7%**. **Empty-Dir Random Project `SplitMate`**: `Codex` tok **-83.3% (`1.12M → 186k`, `6.0x` less!)** & time **-29.1%**; `Pi` lifted **`5/6` (slop `empty-catch`) → `6/6` (0 slop, -322s)** |
