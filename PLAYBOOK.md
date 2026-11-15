# Real-World Efficiency Playbook: Coding Agents & Personal Assistant Agents

[English](./PLAYBOOK.md) | [简体中文](./PLAYBOOK.zh-CN.md) · [Handbook](./HANDBOOK.md) · [Anti-Obsolescence Architecture](./DESIGN.md)

Installing a tool does nothing if onboarding takes 15 minutes of manual JSON editing, if your agent trips over OS shell quoting, or if the tool itself adds more prompt bloat than it saves. This playbook documents **how to wire `agent-primitives` in 1 command**—backed by **69,963 real audited tool calls** and live controlled CLI experiments across **Grok Build CLI (`1.0.77`)**, **Google Antigravity CLI (`1.5.8`)**, **OpenAI Codex CLI (`0.167.0`)**, and **Claude Code**.

---

## 0. Step Zero: Measure Your Own Local Agent Waste First (`ap.py audit`)

Before installing anything, scan your own local CLI session logs (`~/.codex`, `~/.claude`, `~/.grok`, `~/.gemini/antigravity-cli` — **100% local, zero network calls**) to see where *your* agents burn time and tokens:

```bash
python ap.py audit --compare
```

In seconds, `audit` quantifies your exact local pathology counts (redundant same-file re-reads, manual shell line-slicing, inline `python -c` quoting/`ImportError` crashes, patch mismatches, Windows `cp936`/UTF-8 decode failures, `>8 KB` raw output floods) and compares older vs. recent session trends.

---

## 1. Zero-Clone 1-Line Onboarding Across Every Harness (`ap.py init`)

You do **not** need `git clone`, `pip install`, `npm`, or manual JSON/path editing:

```bash
# Universal 1-line auto-onboarding (macOS / Linux):
curl -fsSL https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/ap.py | python3 - init

# Universal 1-line auto-onboarding (Windows PowerShell):
irm https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/ap.py | python - init

# Wire ONLY the current repository (.agents/skills + .agents/ap.py + fenced AGENTS.md block):
curl -fsSL https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/ap.py | python3 - init --here

# Target specific harness(es) + merge non-destructive hooks (Claude Code, Codex, Antigravity):
curl -fsSL https://raw.githubusercontent.com/xgbtxy/agent-primitives/main/ap.py | python3 - init -a codex,pi,agy,claude --with-hooks

# Ecosystem-native 1-liners also work out of the box:
pi install git:github.com/xgbtxy/agent-primitives
codex plugin marketplace add xgbtxy/agent-primitives && codex plugin add agent-primitives@agent-primitives
npx skills add xgbtxy/agent-primitives -g

# Clean, reversible 1-command removal at any time (preserves all of your existing rules):
ap uninstall          # or: python ap.py uninstall --here
```

### How Each Harness Is Wired Under the Hood
- **OpenAI Codex CLI (`codex`)**: Syncs `~/.codex/skills/` + `~/.agents/skills/`, injects `<10-line` fenced routing block into `~/.codex/AGENTS.md`, and (`--with-hooks`) merges `exit 2` shield/stop-gate into `~/.codex/hooks.json`. Also ships `.agents/plugins/marketplace.json` + `.codex-plugin/plugin.json`.
- **Pi (`pi` / `pi-coding-agent`)**: Syncs `~/.pi/agent/skills/` (or `.pi/skills/` with `--here -a pi`) + `~/.pi/agent/AGENTS.md`, and ships a zero-dependency `package.json` `"pi"` manifest for `pi install git:github.com/xgbtxy/agent-primitives`.
- **Google Antigravity (`agy` / IDE)**: Syncs `~/.gemini/config/skills/` + `~/.agents/skills/` + `~/.gemini/config/GEMINI.md`, and (`--with-hooks`) writes named group `"agent-primitives-guard"` (`"matcher": "run_command"`) into `~/.gemini/config/hooks.json` using Antigravity's native camelCase `{"decision": "deny"|"allow"}` / `{"decision": "continue"|"stop"}` JSON contract. Also ships `.antigravity-plugin/plugin.json` + `rules/AGENTS.md`.
- **Anthropic Claude Code (`claude`)**: Syncs `~/.claude/skills/` + `~/.claude/CLAUDE.md`, merges `PreToolUse` (`Bash`), `PreCompact` (`baton`), and `Stop` (`exit 2`) into `~/.claude/settings.json` (with `.bak` backup), and ships `.claude-plugin/marketplace.json`.
- **xAI Grok Build (`grok`)**: Syncs `~/.grok/skills/` (which Grok loads ahead of repo-local skills!) + `~/.grok/AGENTS.md`.

---

## 2. The 5-Stage Loop: Replace Prompt Hope with Deterministic Boundaries

Wire the **7 standalone folders** (`486 tokens` combined idle frontmatter cost) to the **5 physical stages** of every agent turn:

```text
[Stage 1: Orient & Recall]   --> atlas  (<0.1s AST/vault map + >90d stale-TODO radar)
                             --> recall (<6ms SQLite FTS5 + CJK rule injection, ~40 tok)
[Stage 2: Slice & Diagnose]  --> probe  (slice --bundle prunes Py/TS/JS/Go/Rust dead files + bundles active code in 1 call;
                                         run traces Python heap aliasing, mutations & state no-ops; diff checks boundaries)
[Stage 3: Surgical Act]      --> splice (view exact Py/TS/JS/Go/Rust symbol or Markdown heading [-94% tok];
                                         apply UTF-8/CRLF-safe atomic edits; check --test auto-runs repo test suite)
[Stage 4: Filter Output]     --> sieve  (compresses >1.2 KB logs/JSON/HTML by 98%+; lossless FTS5 peek)
[Stage 5: Gate & Persist]    --> guard  (exit-2 secret/blast shield, 3x loop breaker & stop-gate)
                             --> baton  (<400 tok state + DO NOT RETRY dead-ends across compactions)
```

---

## 3. Workflow A: Coding Agents (`Claude Code`, `Codex`, `Grok Build`, `Antigravity`, `Cursor`)

1. **1-Call Polyglot Reachability & Active Bundle (`probe slice --bundle`)**:
   - Start unfamiliar multi-file tasks across **Python, TypeScript/JS, Go, or Rust** with `python .agents/ap.py probe slice . --bundle` (`~25–60 ms`).
   - Prunes dead distractor files (**80%** in micro-bench; **13/24** in TypeScript `ts-ledger`; **15/34** in Python `fluxpipe`) AND prints line-numbered source of active reachable files in **1 single tool call**, eliminating 10–16 sequential `read_file` round-trips.
2. **Sub-200ms Causal Diagnosis Instead of 20-Minute Prose Guessing (`probe run` / `diff`)**:
   - Pipe a 5-line reproducer into `python .agents/ap.py probe run --cwd . --focus <pkg> --stdin` (`@'...'@ |` on PowerShell; `<<'EOF'` on Bash). Call-site deduplication guarantees `< 3.5 KB` output even across 200-iteration loops while automatically flagging **`[ALIAS HAZARD]`**, **`[IN-PLACE MUTATION]`**, and **`[STATE NO-OP]`**.
   - For compilers, serializers, caches, or validators, run `python .agents/ap.py probe diff` to catch `None`/`bool`/`int` (`True` vs `1`) boundary & stateful cache collisions in `<50 ms`.
3. **Polyglot Symbol Reads, Atomic Edits & Auto-Test Verification (`splice`)**:
   - Inspect specific symbols across 1+ files via `python .agents/ap.py splice view src/a.ts src/b.ts --symbol MyClass.run` (**-94.4%** tokens in `~1.6 ms`).
   - Batch multi-file edits atomically via `python .agents/ap.py splice apply --cwd . --stdin` (or `--spec _edits.json --unlink-spec`), then verify UTF-8, syntax, relative imports AND run the auto-detected test suite (`pytest` / `unittest` / `node --experimental-strip-types --test` / `go test` / `cargo test`) in 1 shot via `python .agents/ap.py splice check . --test`.
4. **Test Floods, Loop Breaking & Cross-Agent Handoffs (`sieve` + `guard` + `baton`)**:
   - Wrap noisy commands in `python .agents/ap.py sieve run -- <cmd>` (**-98.8%**) and inspect frames via `sieve peek <id> --lines 905:930`.
   - Record falsified hypotheses with `baton deadend` and persist newly discovered directory gotchas via `recall add`.

---

## 4. Workflow B: Personal Assistant Agents (`OpenClaw`, `Hermes`, Research/Ops Assistants)

1. **Markdown Vault & Document Tree Navigation (`atlas` + `splice view --heading`)**:
   - Run `python .agents/ap.py atlas map ~/Notes` for a `<800-token` structural outline of hundreds of notes in `<0.1s`.
   - Read a specific section of a 2,000-line handbook via `splice view ~/Notes/ops.md --heading "Incident Escalation"`, and update structured JSON/Markdown files atomically via `splice apply` without Windows UTF-8 corruption.
2. **Zero-Cloud Bilingual Personal SOP & Contact Memory (`recall`)**:
   - Store recurring personal/ops rules (`invoice/报销`, `contract/合同`, `server/服务器`) in `recall` (**`5.5 ms`** median query, `100%` Top-1 accuracy across 200 rules, zero cloud API keys).
3. **Web Scrapes, REST JSON APIs & Server Logs (`sieve`)**:
   - Pipe `curl` JSON payloads or scraped HTML pages through `sieve pipe` (**-99.6%** on 400-record JSON; **-98.4%** on JS/CSS-bloated HTML) and drill into exact records via `sieve peek <id> --json-path "items.0"`.
4. **Multi-Day Continuity & Outbound Blast Protection (`baton` + `guard`)**:
   - Resume multi-day ops checklists after compaction via `baton resume` (`<400 tokens`), while `guard shield` blocks secret leaks or destructive commands with `exit 2`.

---

## 5. Honest Anti-Bloat Rules: When NOT to Use Each Tool

- **Skip `sieve` for tiny outputs** (`pwd`, `git status -s`, 30-line files)—`sieve` passes `<1.2 KB` outputs through untouched anyway.
- **Skip `probe` for trivial 1-line typos**—if the stack trace already names the exact line, edit directly and run `splice check . --test`.
- **Keep universal invariants in `AGENTS.md`, conditional rules in `recall`**—never bloat `AGENTS.md` with 200 low-frequency rules.
