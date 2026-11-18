# Real-World Efficiency Playbook: 1-Command Pure-Rust Setup Across Every Harness

[English](./PLAYBOOK.md) | [简体中文](./PLAYBOOK.zh-CN.md) · [Handbook](./HANDBOOK.md) · [Anti-Obsolescence Architecture](./DESIGN.md)

Installing a tool does nothing if onboarding requires Python/Node version juggling, manual JSON editing, or fragile shell quoting. In **`v1.0`**, `agent-primitives` ships as **one self-contained native Rust binary (`ap` / `ap.exe`)** embedding 6 official **Tree-sitter C grammars**, `ripgrep`'s parallel walker, **bundled static SQLite FTS5/BM25**, and all 7 `SKILL.md` definitions—requiring **zero Python, Node, or Rust toolchain on user machines**.

---

## 0. Step Zero: Measure Your Own Local Agent Waste First (`ap audit`)

Scan your own local CLI session logs (`~/.codex`, `~/.pi`, `~/.claude`, `~/.grok`, `~/.gemini/antigravity-cli` — **100% local, zero network calls**) to see where *your* agents burn time and tokens:

```bash
ap audit --compare
```

In seconds, `ap audit` quantifies redundant same-file re-reads, manual shell line-slicing, inline `python -c`/`node -e` quoting & `ImportError` crashes, patch mismatches, UTF-8 decode failures, `>8 KB` raw output floods, and identical retry loops—comparing older vs. recent session trends.

---

## 1. Zero-Toolchain 1-Line Onboarding Across Every Harness

No `git clone`, no `python`, no `pip`, no `npm`, and no Rust compiler required:

```bash
# Universal 1-line prebuilt binary install + auto-onboarding (macOS / Linux):
curl -fsSL https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/install.sh | sh

# Universal 1-line prebuilt binary install + auto-onboarding (Windows PowerShell):
irm https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/install.ps1 | iex

# Rust developers can also install directly from source via Cargo:
cargo install --git https://github.com/xgbtxy/agent-primitives --locked && ap init

# Wire ONLY the current repository (.agents/bin/ap + .agents/skills + fenced AGENTS.md block):
ap init --here

# Target specific harness(es) + merge <2ms native Rust hooks (Claude Code, Codex, Antigravity):
ap init -a codex,pi,agy,claude --with-hooks

# Clean, reversible 1-command removal at any time (preserves all of your existing rules):
ap uninstall          # or: ap uninstall --here
```

### How Each Harness Is Wired Under the Hood
- **OpenAI Codex CLI (`codex`)**: Syncs `~/.codex/skills/` + `~/.agents/skills/`, injects a self-contained `<10-line` fenced routing block into `~/.codex/AGENTS.md`, and (`--with-hooks`) merges `<2ms` native `ap guard` `exit 2` hooks into `~/.codex/hooks.json`.
- **Pi (`pi` / `pi-coding-agent`)**: Syncs `~/.pi/agent/skills/` (or `.pi/skills/` with `--here -a pi`) + `~/.pi/agent/AGENTS.md`, plus root `package.json` `"pi"` manifest.
- **Google Antigravity (`agy` / IDE)**: Syncs `~/.gemini/config/skills/` + `~/.agents/skills/` + `~/.gemini/config/GEMINI.md`, and (`--with-hooks`) writes named group `"agent-primitives-guard"` (`"matcher": "run_command"`) into `~/.gemini/config/hooks.json` using Antigravity's camelCase `{"decision": "deny"|"allow"}` / `{"decision": "continue"|"stop"}` JSON contract.
- **Anthropic Claude Code (`claude`)**: Syncs `~/.claude/skills/` + `~/.claude/CLAUDE.md` and merges native `PreToolUse` (`Bash`), `PreCompact` (`ap baton`), and `Stop` (`exit 2`) into `~/.claude/settings.json` (with `.bak` backup).
- **xAI Grok Build (`grok`)**: Syncs `~/.grok/skills/` + `~/.grok/AGENTS.md`.

---

## 2. The 5-Stage Loop (`412 Tokens` Total Idle Cost Across All 7 Primitives)

```text
[Stage 1: Orient & Recall]   --> ap atlas  (16ms 6-language Tree-sitter map + >90d stale-TODO radar)
                             --> ap recall (3.3ms bundled SQLite FTS5 + CJK rule injection, ~47 tok)
[Stage 2: Slice & Diagnose]  --> ap probe  (7ms slice --bundle prunes Py/TS/JS/Go/Rust/C dead files + bundles code;
                                            run traces Py/TS/JS aliasing/mutation/-0; diff tests Py/TS/JS/Rust)
[Stage 3: Surgical Act]      --> ap splice (0.8ms Tree-sitter AST symbol view [-95.4% tok];
                                            3.1ms zero-escaping apply --stdin --test w/ 6-lang AST syntax rollback)
[Stage 4: Filter Output]     --> ap sieve  (compresses >1.2 KB logs/JSON/JSONL/HTML/CSV by 97-99% + FTS5 search)
[Stage 5: Gate & Persist]    --> ap guard  (<2ms native exit-2 secret/blast shield, 3x loop breaker & stop-gate)
                             --> ap baton  (<400 tok state + DO NOT RETRY dead-ends across compactions)
```

---

## 3. Workflow A: Coding Agents (`Claude Code`, `Codex`, `Grok`, `Antigravity`, `Pi`, `Cursor`)

1. **1-Call 6-Language Reachability & Active Bundle (`ap probe slice . --bundle`)**:
   - Prunes dead distractor files across **Python, TypeScript/JS, Go, Rust, and C** (`7.1 ms`) and emits manifest + numbered source of active reachable files in **1 single tool call**.
2. **Sub-200ms Causal & Differential Verification (`ap probe run` / `diff`)**:
   - Catch `[ALIAS HAZARD]`, `[IN-PLACE MUTATION]`, `[STATE NO-OP]`, `-0`/`NaN`, `True == 1` cache collisions, or negative Rust `i64` rounding drift (`ap probe diff --lang rust`) without burning 20,000 prose reasoning tokens.
3. **Zero-Escaping Atomic Patch + 6-Language Tree-sitter Syntax Gate + Auto-Test (`ap splice`)**:
   - Pipe `@@ FILE:`, `@@ SYMBOL:`, and `@@ WRITE:` blocks directly into `ap splice apply --cwd . --stdin --test --rollback-on-test-fail` (`@'...'@ |` on PowerShell; `<<'EOF'` on Bash). Every modified `.py`, `.ts`, `.tsx`, `.js`, `.go`, `.rs`, `.c`, `.h`, and `.json` file is syntax-checked via **Tree-sitter** *before* touching disk!

---

## 4. Workflow B: Personal Assistant Agents (`OpenClaw`, `Hermes`, Research/Ops Assistants)

1. **Knowledge Vault Outlines & Section Edits (`ap atlas` + `ap splice view --heading`)**: Map hundreds of Markdown notes in `<20ms` and read/edit exact sections atomically.
2. **Zero-Cloud Bilingual SOP Memory (`ap recall`)**: `3.3 ms` hybrid English + Chinese FTS5 recall (`100%` Top-1 accuracy on 200 rules).
3. **JSONL Streams, HTML Tables & CSV/TSV (`ap sieve`)**: Pipe noisy scrapes/APIs into `ap sieve pipe` and search across sessions via `ap sieve search <query>`.
4. **Compaction Continuity & `<2ms` Blast Shield (`ap baton` + `ap guard`)**: Resume multi-day work in `<400 tokens` while blocking leaked keys or `rm -rf` with `exit 2`.

---

## 5. Honest Anti-Bloat Rules: When NOT to Use Each Tool

- **Skip `sieve` for tiny outputs** (`pwd`, `git status -s`)—`sieve` passes `<1.2 KB` outputs through untouched (`0.0%`).
- **Skip `probe` for obvious 1-line typos**—apply directly and run `ap splice check . --test`.
- **Keep universal identity rules in `AGENTS.md`, conditional gotchas in `recall`**.
