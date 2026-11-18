<div align="center">
  <img src="./assets/logo.svg" width="60" height="60" alt="ap logo" />
  <h1>ap · Agent Execution Accelerator</h1>
  <p><strong>Stop letting your AI coding agent waste 50%–80% of its tokens fighting Windows &amp; macOS shells, hung ports, noisy logs, and sloppy code.</strong></p>
  <p>
    <a href="https://github.com/xgbtxy/agent-primitives/actions/workflows/ci.yml"><img src="https://github.com/xgbtxy/agent-primitives/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
    <a href="https://github.com/xgbtxy/agent-primitives/releases/latest"><img src="https://img.shields.io/github/v/release/xgbtxy/agent-primitives?color=22c55e&label=download" alt="Latest Release" /></a>
    <img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-0ea5e9" alt="Platform" />
    <img src="https://img.shields.io/badge/dependencies-0%20(single%20Rust%20binary)-f59e0b" alt="Zero Dependencies" />
    <a href="./LICENSE"><img src="https://img.shields.io/badge/license-MIT-64748b" alt="MIT License" /></a>
  </p>
  <p>
    <strong>English</strong> · <a href="./README.zh-CN.md"><strong>简体中文</strong></a> · <a href="./docs/HANDBOOK.md">R&amp;D Philosophy</a> · <a href="https://github.com/xgbtxy/agent-primitives/releases/latest">Prebuilt Downloads</a>
  </p>
  <img src="./assets/hero-comparison.svg" width="100%" alt="Without ap vs With ap comparison" />
</div>

---

## Install in 10 Seconds, Then Forget About It

No `Python`, no `Node`, no `Playwright`, no `Docker`, and **zero new workflow to learn**. One command downloads the native `ap` binary and automatically wires all coding agents on your machine (`Claude Code`, `Codex`, `OpenCode`, `Gemini CLI`, `Qwen Code`, `Pi`, `Goose`, `Antigravity`, `Grok`):

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/install.ps1 | iex
```

**macOS / Linux:**
```bash
curl -fsSL https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/install.sh | sh
```

> **Prefer direct download?** Grab the standalone binary from Releases (**[Windows x64 `.zip`](https://github.com/xgbtxy/agent-primitives/releases/latest/download/ap-x86_64-pc-windows-msvc.zip)** · **[macOS Apple Silicon](https://github.com/xgbtxy/agent-primitives/releases/latest/download/ap-aarch64-apple-darwin.tar.gz)** · **[macOS Intel](https://github.com/xgbtxy/agent-primitives/releases/latest/download/ap-x86_64-apple-darwin.tar.gz)** · **[Linux x64](https://github.com/xgbtxy/agent-primitives/releases/latest/download/ap-x86_64-unknown-linux-musl.tar.gz)**) and run `ap init`. Clean 1-command removal anytime via `ap uninstall`.

---

## How Do I Use It? (You Barely Touch It)

1. **Chat with your coding agent exactly as you always do.** `ap` works silently underneath via native MCP + `<2ms` shell hooks.
2. **Whenever you want to see what `ap` saved you**, run **`ap stats`** (or `ap doctor` to check environment health):

```text
$ ap stats
[ap stats] Local Token & Windows Execution Ledger (100% offline)
----------------------------------------------------------------
  Accelerated Agent Actions : 28 calls
  Context & Log Diet        : 612.4 KB -> 19.2 KB (-96.9%)
  Windows Pitfalls Healed   : 11 auto-fixes (UTF-8/GBK, PS `&&`/`curl`, PATH, locks)
  AST Edits & Smoke Gates   : 14 verified checks (Tree-sitter + Edge CDP + 0-slop)
  Total Value Delivered     : ~186,800 tokens saved
```

---

## Why Does It Save 50%–83% of Tokens?

LLMs are trained 90%+ on Linux Bash, while most developers run coding agents on **Windows and macOS**. Auditing 160 real Windows coding sessions (`101,108` tool calls) uncovered **528 avoidable OS/shell crashes**—and every crash forces the model to resend tens of thousands of context tokens just to guess and retry.

`ap` upgrades all **5 stages of your agent's execution loop** automatically:

| Execution Stage | Without `ap` (Where Your Tokens Burn) | With `ap` (Automatic Under the Hood) |
| :--- | :--- | :--- |
| **1. Understand Repo** | Reads 15+ full files (including dead legacy code) & guesses your OS/shell wrong. | **<30 ms** PageRank code map + HTML `#id` ↔ API route radar + Windows shell fingerprint (`-99%` read tokens). |
| **2. Edit Files** | Breaks on Windows `CRLF`/`LF`, PowerShell `UTF-8 BOM` / `UTF-16LE` files, or shifted indentation. | Auto-cleans **UTF-8 BOM & UTF-16LE**, matches 3-tier indentation shifts & checks **6-language AST syntax** before saving. |
| **3. Run Commands** | **#1 Windows Token Sink**: `'gbk' codec` crashes, PS 5.1 `&&` / `curl` alias / `<<EOF` `ParserError`s, and 50KB build logs. | Locks BOM-free `UTF-8`, auto-heals PS `&&`/`curl.exe`/URL-`&`/POSIX pipes, prevents pipe hangs & compresses logs **-90%+**. |
| **4. Verify UI & Ports** | Spends **70%+ of turns** fighting hung background servers, `EADDRINUSE` port locks, or 400MB Playwright downloads. | Frees stuck ports, drives **built-in Windows Edge / Chrome** via pure-Rust CDP (`OPEN`/`FILL`/`CLICK`), and cleans the process tree in **<4.5s**. |
| **5. Final Quality Gate** | Stops early leaving `: any`, `@ts-ignore`, or silent `catch {}` AI slop. | **<2 ms `exit 2` Gate** blocks anti-slop violations and broken frontend↔backend routes before finishing. |

---

## Real Windows 11 Benchmarks (Zero Fabrication)

Tested head-to-head on Windows 11 across `$0` free agents and commercial CLIs on beginner 1-sentence prompts, full-stack upgrades (`WorkPulse`), and empty-directory random projects (`SplitMate`):

| Coding Agent & Task | Without `ap` → **With `ap`** (Time) | Token Consumption | Functional & Zero-Slop Check |
| :--- | :--- | :--- | :---: |
| **`OpenCode` Free (`$0` `big-pickle`)** · *1-Sentence Prompt* | `430.2s` → **`182.4s` (2.36x faster)** | `1,613,689` → **`401,930` (-75.1%)** | `7/8` *(slop)* → **`8/8` (`0` slop)** |
| **`Codex CLI` (`gpt-6-astra`)** · *From-Scratch Random App* | `345.3s` → **`244.6s` (-29.1%)** | `1,116,369` → **`186,379` (-83.3% / 6x less)** | **`6/6` → `6/6`** |
| **`Codex CLI` (`gpt-6-astra`)** · *Full-Stack Upgrade* | `333.6s` → **`294.3s` (-54% calls)** | `1,053,580` → **`364,682` (-65.4%)** | **`8/8` → `8/8`** |
| **`Pi CLI` (`deepseek-flash`)** · *Full-Stack Upgrade* | `483.0s` → **`271.5s` (-43.8%)** | `3,628,149` → **`2,295,801` (-36.7%)** | **`8/8` → `8/8`** |
| **`Pi CLI` (`deepseek-flash`)** · *From-Scratch Random App* | `1,821.1s` → **`1,499.3s` (-322s)** | `15.55M` → `19.94M` *(extra subset-sum fuzzer)* | `5/6` *(2x slop)* → **`6/6` (`0` slop)** |

<details>
<summary><strong>For Systems Engineers: Rust Architecture, Micro-Benchmarks &amp; Documentation</strong></summary>

<br/>

- **Zero-Coupling Pure-Rust Workspace**: 1 native `ap` binary backed by 7 isolated crates (`crates/ap-{atlas,splice,probe,sieve,guard,recall,baton}`) with `0` sibling crate dependencies, bundled SQLite FTS5, 6 compiled Tree-sitter C grammars (Py/TS/JS/Go/Rust/C), and `390` total idle frontmatter tokens.
- **Reproduce Deterministic Micro-Benchmarks**: Run `ap bench` (see [`bench/results.json`](./bench/results.json)).
- **Deep-Dive Docs**:
  - [Core R&D Philosophy & Evolution (`docs/HANDBOOK.md`)](./docs/HANDBOOK.md) | [中文研发手册 (`docs/HANDBOOK.zh-CN.md`)](./docs/HANDBOOK.zh-CN.md)
  - [System Architecture (`docs/DESIGN.md`)](./docs/DESIGN.md) | [架构设计 (`docs/DESIGN.zh-CN.md`)](./docs/DESIGN.zh-CN.md)
  - [Workflow Playbook (`docs/PLAYBOOK.md`)](./docs/PLAYBOOK.md) | [实战手册 (`docs/PLAYBOOK.zh-CN.md`)](./docs/PLAYBOOK.zh-CN.md)

</details>

## License

[MIT](./LICENSE)
