# Anti-Obsolescence Design Architecture: Cognitive & Physical Boundaries (`ap` v1.2)

[English](./DESIGN.md) | [简体中文](./DESIGN.zh-CN.md) · [Handbook](./HANDBOOK.md) · [Playbook](./PLAYBOOK.md)

## 1. Why 95% of Agent Repositories Become Obsolete Within 6 Months

- **Prompt Roleplay & Checklists Die First**: Absorbed into frontier RL post-training weights and official slash commands, turning verbose `AGENTS.md` walls into latency and attention taxes.
- **Heavy Orchestration Wrappers Die Second**: Official CLIs (`codex`, `opencode`, `pi`, `claude`, `agy`, `qwen`, `gemini`, `grok`) natively own conversation loops and subagents.
- **Lossy Vector RAG Wrappers Die Third**: Replaced by exact structural maps and fast deterministic search without Docker/API-key overhead.

**Core Thesis**: Build **zero-slop, sub-20ms Pure-Rust physical instruments (`ap` / `ap.exe`)**—exposed via both **Unix CLI** and **Model Context Protocol (`ap mcp`)**—strictly at the **four boundaries where autoregressive token generation collides with OS/AST/Browser/Process reality**.

---

## 2. The Four Irreducible Cognitive & Physical Boundaries

| Boundary in `<thinking>` & OS Reality | Observed Waste Across `101k+` Local Calls & Live `<thinking>` Streams | Our Pure-Rust Organ & Open-Source Best-of-Breed Synthesis (`v1.2`) |
| :--- | :--- | :--- |
| **1. Cold-Start Entropy & Dead-Slop Contamination** | `30k+` outputs `>8 KB`; `10k–35k` thinking chars guessing host runtimes; copying `: any` & `empty-catch` from dead legacy files. | **`ap-atlas`** (Aider-style **Personalized PageRank** + Host Runtime Fingerprint + DOM `#id`/API Contract & Slop Radar) + **`ap-sieve`** (`-98.8%` logs, `5.2ms` FTS5) + **`ap-probe slice`** (`6.3ms` dead-code pruning) |
| **2. Cross-File Working-Memory Drift & Edit Indentation Errors** | Dropping HTML `#id` ↔ JS `getElementById` links across 6+ files; `735` patch mismatches on 2-space indentation drift; Windows PowerShell quoting crashes. | **`ap-splice`** (`0.7ms` Tree-sitter `view` + `2.3ms` Zero-Escaping & **Aider 3-tier exact → rstrip → indentation-delta** AST-verified `apply`) + **`ap mcp`** (`ap_edit` pure JSON-RPC = `0` shell quoting) |
| **3. The 70% Windows Verification Tax & Prose Heap Guessing** | **71%–75% of turns (`850k+` tokens)** burned on hung Windows servers, orphaned ports, and 150-line raw CDP scripts; `55k` thinking tokens simulating heap aliasing. | **`ap-probe smoke`** (Ephemeral `PORT` + file-backed logs + **Pure-Rust `TcpStream` Interactive Headless Edge/Chrome CDP** `OPEN`/`FILL`/`SELECT`/`CLICK`/`EVAL` + `taskkill /T /F`) + **`ap-probe run/diff`** (Py/Node-V8/`rustc`) |
| **4. Prose Rule Amnesia, AI Slop & Compaction Wipeout** | Models ignore `AGENTS.md` under load (emitting `empty-catch` / `: any`); `/compact` wipes falsified dead-ends. | **`ap-guard`** (`<2ms` `exit 2` `.agents/guard.json` anti-slop rules + `tsc`/`biome`/`cargo` delegation + **`rtk`-style `PreToolUse` `updatedInput` auto-sieve**) + **`ap-baton`** + **`ap-recall`** (`3.4ms` BM25) |

---

## 3. Why Pure Rust (`v1.2`) + Native `ap mcp`?

1. **Compile-Time Safety & Sub-2ms Startup (`Rust > C & Python`)**: Strict ownership, zero `Regex::new` inside inner loops (`OnceLock` DFAs), zero GC pauses, and zero Python `45ms` startup lag on hooks.
2. **Universal Zero-Brain Access for `$0` Free Models (`ap mcp`)**: Smaller/free models (`opencode/big-pickle`, `glm-5.3-flash-free`, `deepseek-flash`, `qwen`) struggle with Windows shell quoting; exposing `ap_orient`, `ap_edit`, `ap_smoke`, `ap_verify`, and `ap_probe` as native MCP tools removes 100% of shell syntax friction.
3. **7 Physically Isolated Crates (`0` Sibling Dependencies)**: Every primitive under `crates/ap-*` compiles standalone while linking into a single `~11 MB` stripped binary (`390 tokens` total idle frontmatter across all 7 skills).
