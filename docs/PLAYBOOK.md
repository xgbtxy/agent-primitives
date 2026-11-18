# Real-World Efficiency Playbook: 1-Command Pure-Rust Setup for Free & Frontier Agents

[English](./PLAYBOOK.md) | [简体中文](./PLAYBOOK.zh-CN.md) · [Handbook](./HANDBOOK.md) · [Architecture](./DESIGN.md)

`agent-primitives` (`v1.2`) ships as **one native Rust binary (`ap` / `ap.exe`)** embedding **6 Tree-sitter C grammars**, **bundled SQLite FTS5/BM25**, a **Pure-Rust Interactive Headless Edge/Chrome CDP runner**, and a **built-in Model Context Protocol (`ap mcp`) stdio server**—requiring **zero Python, Node, or Rust toolchain on user machines**.

---

## 1. Zero-Config 1-Line Onboarding (Even for Complete Beginners)

```bash
# macOS / Linux (auto-detects OpenCode, Qwen, Gemini CLI, Goose, Pi, Codex, Antigravity, Claude, Grok):
curl -fsSL https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/install.sh | sh
```

```powershell
# Windows PowerShell:
irm https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/install.ps1 | iex
```

```bash
# Onboard ONLY the current directory (auto-wires opencode.json / .qwen / .gemini MCP when targeted):
ap init --here -a opencode
ap init --here --with-hooks --with-mcp

# Clean 1-command uninstall (restores your original configs untouched):
ap uninstall          # or: ap uninstall --here
```

### Free & Open-Source Coding Agents Supported Out-of-the-Box (`$0` Cost Ready)
- **OpenCode (`opencode`)**: Auto-wires `"agent-primitives": { "type": "local", "command": [".../ap", "mcp"], "enabled": true }` into `~/.config/opencode/opencode.json` (or repo `opencode.json`) + `AGENTS.md`. Even free models (`opencode/big-pickle`, `glm-5.3-flash-free`, `deepseek-v4.1-flash-free`) call `ap_orient`, `ap_edit`, `ap_smoke`, and `ap_verify` natively with zero shell quoting!
- **Qwen Code (`qwen`) & Google Gemini CLI (`gemini`)**: Auto-wires `"mcpServers": { "agent-primitives": { "command": ".../ap", "args": ["mcp"], "trust": true } }` into `~/.qwen/settings.json` and `~/.gemini/settings.json` (`"trust": true` eliminates confirmation popups).
- **Pi (`pi` / `pi-coding-agent`) & Goose (`goose`)**: Syncs `~/.pi/agent/skills/` + `~/.pi/agent/AGENTS.md` + `package.json` manifest.
- **Claude Code (`claude`), OpenAI Codex (`codex`), Antigravity (`agy`) & Grok (`grok`)**: Syncs skills, `<140-token` self-contained routing rules, and (`--with-hooks`) `<2ms` native `exit 2` / JSON hooks + `rtk`-style transparent `PreToolUse` auto-sieve compression for Claude `Bash` test runs.

---

## 2. How the 5 Native MCP Tools (`ap mcp`) & CLI Primitives Work Together

| MCP Tool / CLI Command | Cognitive / OS Bottleneck Eliminated | Latency |
| :--- | :--- | ---: |
| **`ap_orient`** / `ap atlas map` + `atlas product` + `probe slice --bundle` | Cold-start runtime guessing, dead slop file reading, and missing HTML `#id` ↔ JS selector or `fetch()` ↔ backend route links | `6–18 ms` |
| **`ap_edit`** / `ap splice apply --cwd . --stdin --test` | Multi-file edit waterfalls, 2-space indentation drift (Aider 3-tier fallback), PowerShell quoting traps, and broken AST writes | `2.3 ms` |
| **`ap_smoke`** / `ap probe smoke --cwd . --stdin` | **The 70% Windows verification tax**: auto-binds free `PORT`, drives interactive headless Edge/Chrome CDP (`OPEN`, `FILL`, `SELECT`, `CLICK`, `EVAL`), checks `0` `pageerror`/stderr crashes, and kills Windows process trees (`taskkill /T /F`) | `2.5–4.5 s` |
| **`ap_verify`** / `ap guard verify` + `guard rule add` | Prose rule amnesia & AI slop (`: any`, `@ts-ignore`, empty `catch {}`), delegating automatically to `tsc`/`biome`/`cargo`/tests (`exit 2`) | `<2 ms` (+ tests) |
| **`ap_probe`** / `ap probe run` · `ap sieve` · `ap recall` · `ap baton` | Heap-aliasing prose guessing, `200 KB` CI log floods (`-98.8%`), `AGENTS.md` rule bloat (`3.4ms` SQLite FTS5), and `/compact` amnesia | `3–29 ms` |

---

## 3. Step Zero Diagnostics (`ap audit`)

Measure your own local agent waste across `~/.codex`, `~/.pi`, `~/.claude`, `~/.grok`, and `~/.gemini/antigravity-cli` (`0` bytes uploaded):

```bash
ap audit --compare
```
