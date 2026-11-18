# agent-primitives (`ap` v1.2)

**Pure-Rust, Single-Binary, Zero-Slop Cognitive & OS Infrastructure for Free and Frontier Coding Agents.**

[English (Default)](./README.md) | [简体中文](./README.zh-CN.md) · **[Engineering Handbook (EN)](./HANDBOOK.md)** | **[核心工程宪章 (中文)](./HANDBOOK.zh-CN.md)** · [Architecture](./DESIGN.md) · [Playbook](./PLAYBOOK.md)

---

## 1-Line Zero-Config Onboarding (Free & Commercial Agents on Windows / macOS / Linux)

**Zero `Python`, zero `Node`, zero `Docker`, zero `git clone`, zero flags to memorize.** One command downloads the SHA-256-verified native `ap` binary (statically embedding **6 Tree-sitter C grammars**, **bundled SQLite FTS5**, a **Pure-Rust Interactive Headless Edge/Chrome CDP engine**, and a **native `ap mcp` JSON-RPC server**) and auto-wires every coding agent on your machine—so even a **complete beginner using a `$0` free coding agent** gets immediate `2x–6x` efficiency gains:

```bash
# macOS / Linux (auto-detects OpenCode, Qwen, Gemini CLI, Goose, Pi, Codex, Antigravity, Claude, Grok):
curl -fsSL https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/install.sh | sh
```

```powershell
# Windows PowerShell:
irm https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/install.ps1 | iex
```

```bash
# Build from source via Cargo, or wire ONLY the current project:
cargo install --git https://github.com/xgbtxy/agent-primitives --locked && ap init
ap init --here -a opencode                 # Auto-wires opencode.json -> `ap mcp` + anti-slop rules
ap init --here --with-hooks --with-mcp     # Full repo hooks + MCP (.mcp.json, .qwen, .gemini, opencode.json)
ap uninstall                               # Clean 1-command removal (restores original settings)
```

### Supported Free (`$0`) & Commercial Agent Matrix

| Agent Harness | Free / Paid | How `ap init` Wires It Automatically | Transport & Safety Gate |
| :--- | :---: | :--- | :--- |
| **OpenCode** (`opencode`) | **`$0` Free Zen** + BYOK | `~/.config/opencode/opencode.json` + `AGENTS.md` | Native **`ap mcp`** (`ap_orient`, `ap_edit`, `ap_smoke`, `ap_verify`, `ap_probe`) |
| **Qwen Code** (`qwen`) | **Free Daily Quota** + OSS | `~/.qwen/settings.json` (`"trust": true`) + `QWEN.md` | Native **`ap mcp`** (`0` confirmation popups) |
| **Google Gemini CLI** (`gemini`) | **Free Tier** + Cloud | `~/.gemini/settings.json` (`"trust": true`) + `GEMINI.md` | Native **`ap mcp`** + `<140-tok` routing |
| **Pi** (`pi`) & **Goose** (`goose`) | OSS + Low-Cost APIs | `~/.pi/agent/skills/`, `~/.config/goose/` + `AGENTS.md` | `package.json` manifest + CLI / MCP |
| **OpenAI Codex CLI** (`codex`) | Subscription / API | `~/.codex/skills/` + `~/.codex/AGENTS.md` + `hooks.json` | `<2ms` `exit 2` hooks + Zero-Escaping CLI |
| **Claude Code** (`claude`) | Subscription / API | `~/.claude/skills/` + `CLAUDE.md` + `settings.json` | `<2ms` `exit 2` Stop-gate + **`rtk`-style `PreToolUse` auto-sieve** + `.mcp.json` |
| **Antigravity** (`agy`) & **Grok** | Built-in / API | `~/.gemini/config/`, `~/.grok/` | CamelCase JSON `deny`/`continue` hooks + CLI |

---

## Why `ap` v1.2 Works: Built Directly From LLM `<thinking>` Bottlenecks & Open-Source Best-of-Breed

Dissecting **`101,108` real local tool calls** and raw `<thinking>` streams across 6 harnesses revealed four places where LLMs burn 70%+ of their tokens and write AI slop:

1. **Cold-Start Runtime Guessing & Legacy Slop Contamination (`Turn 1–5`)** → **`ap-atlas` + `ap-probe slice`**
   - Synthesizes **Aider's Personalized PageRank** (`8` power iterations over Tree-sitter imports), a cold-start **Host Runtime Fingerprint** (`node:sqlite`, `--experimental-strip-types` `enum` hazard warnings, headless CDP readiness), and `6.35 ms` dead-file pruning so models never copy `: any` or `empty-catch` from dead legacy files.
2. **Cross-File Working-Memory Drift & Patch Indentation Failures (`Turn 6–15`)** → **`ap-atlas product` + `ap-splice` + `ap mcp`**
   - Statically cross-checks **HTML `id="..."` ↔ JS `getElementById`/`querySelector`** and **client `fetch()` ↔ backend routes** in `<17 ms`. Applies multi-file edits via **Aider's 3-tier exact → rstrip → indentation-delta** matcher (`2.33 ms`), validating **6-language Tree-sitter AST syntax *before* touching disk** (`0` quoting errors via `ap_edit` MCP).
3. **The 70% Windows Verification Token Tax (`Turn 16–60`)** → **`ap-probe smoke` (`ap_smoke`)**
   - In vanilla Windows benchmarks, agents spent **71%–75% of all turns** (`16–35` turns, `850k+` tokens) fighting hung background servers, orphaned ports, and raw 150-line Chrome DevTools scripts. `ap probe smoke` binds a free `PORT` with file-backed logs (`0` Windows pipe deadlocks), drives **Pure-Rust `TcpStream` Interactive Headless Edge/Chrome CDP** (`OPEN`, `FILL`, `SELECT`, `CLICK`, `EVAL`), asserts `0` `pageerror`/`console.error`/server-stderr crashes, and kills the full Windows process tree (`taskkill /T /F`) in **1 single `<4.5s` call**.
4. **Prose Rule Amnesia & AI Slop (`AGENTS.md` Is Not Enough)** → **`ap-guard` (`ap_verify`) + `ap-sieve`**
   - Enforces `<2 ms` **`exit 2`** code rules in `.agents/guard.json` (`0` `: any`, `0` `@ts-ignore`, `0` silent `catch {}`), delegates to existing `tsc`/`biome`/`cargo check`/tests, and performs **`rtk`-style transparent `PreToolUse` `updatedInput` auto-sieve compression** (`-98.8%` on CI logs).

---

## Live Controlled Windows Benchmark Matrix (Zero Fabrication)

Tested on real CLIs on Windows 11 across **1-sentence beginner wishes**, **ordinary user full-stack upgrades (`WorkPulse`)**, and **100% empty-directory from-scratch random projects (`SplitMate`)**, graded by independent headless-Edge + HTTP + anti-slop oracles:

| Agent Harness & Scenario | Mode | Wall Time & Tool Calls | Thinking / Verification Overhead | Total Tokens & Cost | Oracle & Code Hygiene |
| :--- | :--- | ---: | ---: | ---: | :---: |
| **`OpenCode 1.19` Free (`$0` `big-pickle`)**<br>*1-Sentence Beginner Prompt (`WorkPulse`)* | Vanilla → **`+ ap v1.2 (MCP)`** | `430.2s` (`60` calls) → **`182.4s` (`23` calls, -57.6% time / 2.36x faster)** | `15.4k` → **`12.6k` reasoning**; `1` `bash` call vs port-hunt loop | `1,613,689` → **`401,930 tok` (-75.1%, `$0.00`)** | `7/8` *(slop `empty-catch`)* → **`8/8` (`0` slop!)** |
| **`Codex CLI 0.167` (`gpt-6-astra`)**<br>*Empty-Dir Random Project (`SplitMate`)* | Vanilla → **`+ ap v1.2`** | `345.3s` (`23` calls) → **`244.6s` (`9` calls, -29.1% time)** | `16` `cua_repl` turns → **`0` `cua_repl`** (`1` CDP `probe smoke`!) | `1,116,369` → **`186,379 tok` (-83.3% / 6.0x less!)** | **`6/6` → `6/6`** |
| **`Pi CLI 1.5.2` (`deepseek-flash`)**<br>*Empty-Dir Random Project (`SplitMate`)* | Vanilla → **`+ ap v1.2`** | `1,821.1s` → **`1,499.3s` (-17.7% / -322s faster)** | Caught & prevented silent error-swallowing slop | `15.55M` → `19.94M` *(NP-hard subset-sum fuzzer — honest note)* | `5/6` *(2x `empty-catch` slop)* → **`6/6` (`0` slop!)** |
| **`Codex CLI 0.167` (`gpt-6-astra`)**<br>*Ordinary User Upgrade (`WorkPulse`)* | Vanilla → **`+ ap v1.2`** | `333.6s` (`24` calls) → **`294.3s` (`11` calls, -54.2% calls)** | `75%` verification turns → **1-shot `probe smoke` + `guard verify`** | `1,053,580` → **`364,682 tok` (-65.4%, `-689k`)** | **`8/8` → `8/8`** |
| **`Pi CLI 1.5.2` (`deepseek-flash`)**<br>*Ordinary User Upgrade (`WorkPulse`)* | Vanilla → **`+ ap v1.2`** | `483.0s` (`81` calls) → **`271.5s` (`70` calls, -43.8% time)** | `165,048` → **`94,584` thinking chars (-42.7%)** | `3,628,149` (`$0.117`) → **`2,295,801 tok` (-36.7%, `$0.075`)** | **`8/8` → `8/8`** |
| **`Grok 1.0.77` & `Antigravity 1.5.8`**<br>*24-File TS `ts-ledger` & 25-File Py `rulelang`* | Vanilla → **`+ ap init`** | *Grok*: **-26.5% to -49.1% time**; *Agy*: **-42.9% to -61.3% steps** | *Grok*: **-65.4% reasoning**; *Agy*: **-74.2% `read_file`**, **-85.7% edit errors** | *Grok*: **-38.7% tok**; *Agy*: **-49.8% net tok** | **`5/5` → `5/5`** |

---

## Deterministic Rust `v1.2.0` Micro-Benchmarks ([`bench/results.json`](./bench/results.json))

| Crate & Primitive | Baseline / Raw | With `ap` (`v1.2` Rust) | Reduction | Native Latency | Fidelity & Safety Guarantee |
| :--- | ---: | ---: | ---: | ---: | :--- |
| **`ap-splice`** (`view` / 3-tier `apply` / `check`) | `1,048 tok` module | `48 tok` exact AST fn | **-95.4% read tok** | **`0.71 ms`** (`apply`: **`2.33 ms`**) | Aider 3-tier indent match + **Tree-sitter AST check before disk write** |
| **`ap-recall`** (200-rule EN + CJK vault) | `6,880 tok` monolith | `47.0 tok` *(top-2)* | **-99.3%** | **`3.36 ms`** | **20/20 (100%)** Top-1 accuracy (bundled SQLite FTS5) |
| **`ap-probe`** (24-file TS `slice` + CDP `smoke`) | `24` `.ts` files | `9` active (`15` pruned) | **-62.5% dead files** | **`6.35 ms`** | Interactive Edge/Chrome CDP + Py/V8/`rustc` probes |
| **`ap-atlas`** (64-module Rust SDK PPR `map`) | `42,338 tok` full read | `438 tok` PPR map | **-99.0%** | **`17.2 ms`** | Personalized PageRank + DOM `#id`/API contract & slop radar |
| **`ap-sieve`** (2,500-line CI log / JSON / HTML) | `43.0k` / `27.0k` / `12.0k tok` | `518` / `105` / `198 tok` | **-98.8% / -99.6% / -98.4%** | `28.7` / `12.4` / `11.4 ms` | **`5.25 ms`** FTS5 `search` + `<1.2 KB` passthrough (`0.0%` neg. control) |

---

## Cargo Workspace Layout (`390 Tokens` Total Idle Frontmatter · `0` Sibling Crate Deps)

```text
agent-primitives/
├── Cargo.toml / src/{main,mcp,install,audit,bench}.rs  # Single native `ap` binary + `ap mcp` JSON-RPC server
├── crates/ap-{atlas,recall,probe,splice,sieve,guard,baton}/  # 7 isolated standalone crates (0 sibling ap-* deps)
├── install.sh / install.ps1                            # 1-line SHA-256 prebuilt binary installers
├── .github/workflows/{ci.yml,release.yml}              # 3-OS clippy+test gate & 5-target binary releases
└── tests/integration_primitives.rs                     # 14 isolation, MCP, CDP, indentation & hook test suites
```

## License

[MIT](./LICENSE)
