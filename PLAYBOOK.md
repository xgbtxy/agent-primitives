# Real-World Efficiency Playbook: Coding Agents & Personal Assistant Agents

[English](./PLAYBOOK.md) | [简体中文](./PLAYBOOK.zh-CN.md) · [Anti-Obsolescence Architecture](./DESIGN.md)

Installing a tool does nothing if your agent never calls it, or if the tool itself adds more prompt bloat than it saves. This playbook documents **how to wire `agent-primitives` into real daily sessions**—backed by telemetry from **185 historical sessions** and live controlled CLI experiments across **Grok Build CLI (`1.0.77`)**, **OpenAI Codex CLI (`0.167.0`)**, **Google Antigravity CLI (`1.5.8`)**, and **Claude Code**.

---

## 0. Step Zero: Measure Your Own Local Agent Waste First (`audit/`)

Before installing anything, scan your own local CLI session logs (`~/.codex`, `~/.claude`, `~/.gemini/antigravity-cli` — **100% local, zero network calls**) to see where *your* agents are burning time and tokens:

```bash
python audit/audit_sessions.py
```

In `<5 seconds`, `audit_sessions.py` quantifies your exact local pathology counts (redundant same-file re-reads, manual shell line-slicing, inline `python -c` quoting/`ImportError` crashes, patch mismatches, Windows `cp936`/UTF-8 decode failures, `>8 KB` raw output floods) and prints a **tailored `--only ...` install command** for your highest-friction bottlenecks.

---

## 1. The 5-Stage Loop: Replace Prompt Hope with Deterministic Boundaries

Most agent setups degrade over time for four physical reasons:
1. **Blind Repo Hunting & Static Prompt Bloat**: Reading unreferenced dead modules + dumping 500 lines of conditional rules into `AGENTS.md` / `MEMORY.md` every turn.
2. **Prose Heap Simulation & Inline `python -c` Crashes**: Burning `35k–55k` reasoning tokens trying to mentally simulate shallow-copy aliasing, in-place dict mutation, or `None`/`bool`/`int` edge cases—or crashing PowerShell/bash quoting with multi-line `python -c`.
3. **Full-File Re-Reads & Fragile Edits**: Re-reading the same file 3–5x after line numbers shift (**50.1%** redundant `read_file` rate in Antigravity), hitting CRLF mismatches, bare `Path.read_text()` `cp936` crashes on Windows, or leaving stale `.pyc` caches.
4. **Raw Output Floods & Unverified Completions**: Pouring `50–300 KB` logs/JSON/HTML into context (**Context Rot**) and trusting prose instructions instead of native **`exit 2`** hooks.

Wire the **7 standalone folders** (`492 tokens` combined idle cost) to the **5 physical stages** of every agent turn:

```text
[Stage 1: Orient & Recall]   --> atlas (<0.1s AST/vault map + >90d stale-TODO radar)
                             --> recall (<4ms SQLite FTS5 + CJK rule injection, ~40 tok)
[Stage 2: Slice & Diagnose]  --> probe (slice prunes dead files; run --stdin traces heap aliasing,
                                        in-place mutations & state no-ops; diff checks boundaries)
[Stage 3: Surgical Act]      --> splice (view exact AST symbol or Markdown heading [-94% tok];
                                         apply UTF-8/CRLF-safe syntax-verified atomic edits + .pyc purge)
[Stage 4: Filter Output]     --> sieve (compresses >1.2 KB logs/JSON/HTML by 98%+; lossless peek)
[Stage 5: Gate & Persist]    --> guard (exit-2 secret/blast shield, 3x loop breaker & stop-gate)
                             --> baton (<400 tok state + DO NOT RETRY dead-ends across compactions)
```

---

## 2. 60-Second Setup (Zero Third-Party Dependencies)

### Step A — Install All 7 Folders (or Only What `audit` Recommended)

```bash
# Verify Python 3.9+ & built-in SQLite FTS5
python install.py --doctor

# Install all 7 folders—or pass --only probe,splice,sieve for a minimal slice:
python install.py --agent agents --print-agents-md   # ~/.agents/skills (Antigravity, Codex, Cursor, OpenCode)
python install.py --agent claude --print-hooks       # ~/.claude/skills (Claude Code + Hook JSON)
python install.py --agent grok                       # ~/.grok/skills   (Grok Build CLI)
python install.py --agent openclaw                   # ~/.openclaw/skills (OpenClaw Personal Assistant)
python install.py --agent hermes                     # ~/.hermes/skills (Hermes Agent)
```

### Step B — Attach Deterministic Hooks (Zero LLM Willpower Needed)

Generate exact OS-resolved hook JSON via `python install.py --agent claude --print-hooks`:
1. **`PreToolUse` -> `guard.py shield`**: Blocks API-key leaks (`sk-...`, `ghp_...`, `xai-...`, `AKIA...`), `git push --force`, `rm -rf`, and unbounded outbound blasts with **`exit 2`** *before* execution.
2. **`PreCompact` -> `baton.py checkpoint`**: Snapshots objective, git dirty state, verified facts, and dead-ends right before context compaction.
3. **`Stop` -> `guard.py stop-gate`**: Blocks premature `"Done!"` claims with **`exit 2`** if merge conflict markers, exposed secrets, or failing test gates remain.

### Step C — Keep `AGENTS.md` / `SOUL.md` Under 60 Lines

Keep **only** universal invariants (5–10 bullets) plus the compact routing protocol (`python install.py --print-agents-md`) in `AGENTS.md` / `SOUL.md`. Move conditional subsystem/client rules into `recall`.

---

## 3. Workflow A: Coding Agents (`Claude Code`, `Codex`, `Grok Build`, `Antigravity`, `Cursor`)

1. **Cold Repo Orientation + Reachability Pruning (`atlas` + `probe slice`)**:
   - Start unfamiliar repos with `atlas.py map .` (`<800 tok` in `~20 ms`, flags `>90-day` stale TODO traps).
   - When debugging a package or entrypoint, run `probe.py slice . --entry <pkg>` (`~20–50 ms`) to prune unreferenced distractor modules (**80%** pruned in micro-bench; **15/34** dead files pruned on `fluxpipe`) and inspect cross-file call contracts before reading files.
2. **Sub-100ms Causal Diagnosis Instead of 20-Minute Prose Guessing (`probe run` / `diff`)**:
   - Instead of writing fragile `python -c` one-liners or burning `50,000` thinking tokens simulating object references, pipe a 5-line scenario into `probe.py run --cwd . --focus <pkg> --stdin`. `probe` automatically flags **`[ALIAS HAZARD]`** (shared mutable nested dicts/lists), **`[IN-PLACE MUTATION]`** (caller input mutated or not rolled back on exception), and **`[STATE NO-OP]`** (mutator called whose `self.__dict__` stayed byte-identical).
   - For compilers, serializers, caches, or validators, run `probe.py diff` to test `None`/`bool`/`int` (`True` vs `1`) boundary & cache-key collisions in `<50 ms`.
3. **Exact Symbol Reads & Atomic Multi-File Edits (`splice`)**:
   - Need to inspect one method in a 500-line file? Run `splice.py view path/to/mod.py --symbol MyClass.method` (**-94.4%** tokens in `1.5 ms`) instead of dumping the whole file.
   - Apply multi-file fixes and new tests in **one ACID transaction** via `splice.py apply --cwd . --stdin`: enforces UTF-8 on Windows, preserves CRLF/LF per file, verifies `ast.parse` / `json.loads` *before* touching disk (rolling back 100% of files on any error), and purges stale `.pyc` bytecode automatically. Finish with `splice.py check .` (`~12 ms`).
4. **Test Floods, Loop Breaking & Cross-Agent Handoffs (`sieve` + `guard` + `baton`)**:
   - Run noisy suites through `sieve.py run -- pytest` (**-98.8%**) and inspect frames via `sieve.py peek <id> --lines 905:930`.
   - Wrap flaky commands in `guard.py run --max-repeats 3`, record falsified hypotheses with `baton.py deadend`, and persist newly discovered directory gotchas via `recall.py add`.

---

## 4. Workflow B: Personal Assistant Agents (`OpenClaw`, `Hermes`, Research/Ops Assistants)

1. **Markdown Vault & Document Tree Navigation (`atlas` + `splice view --heading`)**:
   - Run `atlas.py map ~/Notes` to get a `<800-token` structural outline of hundreds of notes in `<0.1s`.
   - Read a specific section of a 2,000-line handbook or SOP via `splice.py view ~/Notes/ops.md --heading "Incident Escalation"` without loading the rest of the file, and update structured JSON/Markdown files atomically via `splice.py apply` without UTF-8 corruption on Windows.
2. **Zero-Cloud Bilingual Personal SOP & Contact Memory (`recall`)**:
   - Store recurring personal/ops rules (`invoice/报销`, `contract/合同`, `server/服务器`, client preferences) in `recall`. Hybrid English + CJK indexing recalls the exact rule in **`4.0 ms`** (`100%` Top-1 accuracy across 200 rules) with **zero cloud API keys and zero privacy leak**.
3. **Web Scrapes, REST JSON APIs & Server Logs (`sieve`)**:
   - Pipe `curl` JSON payloads or scraped HTML pages through `sieve.py pipe` (**-99.6%** on 400-record JSON; **-98.4%** on JS/CSS-bloated HTML) and drill into exact records via `sieve.py peek <id> --json-path "items.0"`.
4. **Multi-Day Continuity & Outbound Blast Protection (`baton` + `guard`)**:
   - Resume multi-day research, procurement, or ops checklists after daily compaction via `baton.py resume` (`<400 tokens`), while `guard.py shield` blocks accidental secret exposure or destructive/mass-outbound commands with `exit 2`.

---

## 5. Honest Anti-Bloat Rules: When NOT to Use Each Tool

- **Skip `sieve` for tiny outputs** (`pwd`, `git status -s`, 30-line files)—`sieve` passes `<1.2 KB` outputs through untouched anyway; use it when output may exceed `~2 KB`.
- **Skip `probe` for trivial 1-file edits**—if the stack trace already points to a 1-line typo, jump straight to `splice.py apply`. Use `probe` when multiple files, state/aliasing, or subtle edge cases interact.
- **Keep universal rules in `AGENTS.md`, conditional rules in `recall`**—never query SQLite for an identity rule needed on 100% of turns, and never bloat `AGENTS.md` with 200 rules needed on `<5%` of turns.
