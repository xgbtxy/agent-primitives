# Real-World Efficiency Playbook: Coding Agents & Personal Assistant Agents

[English](./PLAYBOOK.md) | [简体中文](./PLAYBOOK.zh-CN.md)

Installing a tool does nothing if your agent never calls it, or if the tool itself adds more prompt bloat than it saves. This playbook documents **what actually works in daily production** to cut token waste, eliminate compaction amnesia, and stop false completions across **Coding Agents** and **Personal Assistant Agents**.

---

## 1. The Core Principle: Replace Prompt Hope with Deterministic Boundaries

Most agent setups fail for three structural reasons:
1. **Static Prompt Bloat**: Dumping 500 lines of every historical rule into `CLAUDE.md` / `AGENTS.md` / `MEMORY.md` burns `5,000–15,000 tokens` on *every* turn and causes middle-context instruction dilution.
2. **Unfiltered Raw Streams**: Allowing raw `pytest`, `cargo`, `curl`, or HTML outputs (`50–300 KB`) into conversation history triggers **Context Rot** and forces early compaction.
3. **Non-Blocking Advice**: Writing *"Please run tests before saying done"* in Markdown fails silently unless backed by a deterministic **`exit 2`** hook.

Instead, wire the 5 standalone folders to the **5 physical chokepoints** of an agent session:

```text
[Session Start]      --> atlas   (<0.1s structural map, <800 tok; flags >90d stale TODO traps)
[Before Action]      --> recall  (<5ms SQLite FTS5 match; injects ONLY the 1-3 relevant rules)
[Noisy Tool / API]   --> sieve   (compresses >1.2 KB logs/JSON/HTML by 98%+; lossless `peek` by id)
[PreTool / Stop]     --> guard   (exit-2 secret/blast shield, 3x loop breaker & completion gate)
[PreCompact / Switch]--> baton   (<400 tok state + `DO NOT RETRY` dead-ends across sessions/agents)
```

---

## 2. Step-by-Step Setup (60 Seconds, Zero Dependencies)

### Step A — Install the Folders You Want

```bash
# Check Python 3.9+ & stdlib SQLite FTS5 health
python install.py --doctor

# Install all 5 folders (or pass --only sieve,baton,guard for a subset):
python install.py --agent agents     # -> ~/.agents/skills (Antigravity, Codex, Cursor, OpenCode)
python install.py --agent claude     # -> ~/.claude/skills (Claude Code)
python install.py --agent grok       # -> ~/.grok/skills   (Grok Build CLI)
python install.py --agent openclaw   # -> ~/.openclaw/skills (OpenClaw Personal Assistant)
python install.py --agent hermes     # -> ~/.hermes/skills (Hermes Agent)
```

### Step B — Wire Deterministic Hooks (Zero LLM Willpower Required)

Generate exact OS-resolved hook JSON with `python install.py --agent claude --print-hooks`:

1. **`PreToolUse` -> `guard.py shield`**: Automatically blocks accidental API-key/token leaks (`sk-...`, `ghp_...`, `xai-...`, `AKIA...`), `git push --force`, `rm -rf`, and unbounded outbound blasts with **`exit 2`** *before* the command executes.
2. **`PreCompact` -> `baton.py checkpoint`**: Automatically snapshots active objective, git dirty files, verified facts, and dead-ends right before context compaction runs.
3. **`Stop` -> `guard.py stop-gate`**: Blocks premature `"Done!"` claims with **`exit 2`** if merge conflict markers, exposed secrets, or failing test checks remain.

### Step C — Keep `AGENTS.md` / `SOUL.md` Under 60 Lines

Keep **only** universal project/identity invariants (5–10 bullets) in `AGENTS.md` or `SOUL.md`, add the 5-line routing protocol (`python install.py --print-agents-md`), and move all conditional rules (*"when touching `src/auth/*`..."*, *"when invoicing client X..."*) into `recall`.

---

## 3. Workflow A: How Coding Agents Genuinely Gain 3x–5x Efficiency

*(Applies to Claude Code, OpenAI Codex CLI, Cursor, Google Antigravity, Grok Build, OpenCode)*

1. **Cold Repo Triage (`0s–5s`)**:
   - **Bad**: Agent runs 14 `ls`, `find`, and full-file `cat` calls (`~90,000 tokens`), then chases a 2-year-old `# TODO` comment (+77% wasted tokens).
   - **Good**: Run `atlas.py map .` once (`788 tokens` in `77 ms` on `opentelemetry-sdk/src`). Jump straight to the top hub module and ignore flagged stale TODOs.
2. **Subsystem Gotchas Without Bloat**:
   - **Good**: Run `recall.py match --query "<bug>" --files "src/exporter/http.py"`. In `3.7 ms`, get the exact `~40-token` invariant for that directory. Whenever you solve a subtle bug, persist the 1-line lesson via `recall.py add` so no future session ever repeats the mistake.
3. **Test & CI Loops Without Context Rot**:
   - **Good**: Run `sieve.py run -- pytest` (`172 KB -> 2.0 KB`, **-98.8%**). Read the extracted traceback lines (`L915-L920`), then `sieve.py peek <id> --lines 905:930` if surrounding context is needed—without re-running a 2-minute test suite!
4. **Anti-Loop + Multi-Agent Handoff**:
   - Wrap flaky debugging commands in `guard.py run --max-repeats 3 -- <cmd>`. If an approach fails, log `baton.py deadend "<what failed>" --reason "<why>"`. If you switch from Claude Code to Codex/Grok Build (or hit `/compact`), run `baton.py resume` (`< 400 tokens`) and continue without re-trying dead ends.

---

## 4. Workflow B: How Personal Assistant Agents Genuinely Gain Efficiency

*(Applies to OpenClaw, Hermes, desktop/research/ops assistants)*

1. **Knowledge Vault & Document Folder Navigation (`atlas`)**:
   - Point `atlas.py map ~/Documents/Notes` at an Obsidian/Markdown vault or project folder to get an instant `< 800-token` directory + `H1/H2` outline of active notes instead of reading dozens of files.
2. **Bilingual Personal SOP & Contact Memory (`recall`)**:
   - Store recurring life/work rules (`invoice/报销`, `server/服务器`, `weekly-report/周报`, client preferences) in `recall`. Native English + CJK indexing surfaces the exact SOP in `< 5 ms` with **zero cloud subscription and zero privacy leak**.
3. **Web Scrapes, JSON APIs & Log Digestion (`sieve`)**:
   - Pipe verbose REST API responses, scraped HTML pages, RSS feeds, or server logs through `sieve.py pipe`. Strips `<script>`/`<style>` noise (-98.4%) and summarizes 400-item JSON payloads (-99.6%) while allowing `sieve.py peek <id> --json-path "items.0"` on demand.
4. **Multi-Day Task Continuity (`baton`) & Outbound Safety (`guard`)**:
   - Multi-day research, travel/procurement comparisons, or ops checklists survive daily compactions via `baton.py resume`. `guard.py shield` prevents autonomous assistants from leaking keys or firing destructive commands / mass outbound messages without human approval.

---

## 5. Honest Boundaries: When NOT to Use These Tools

Real efficiency requires knowing when **not** to add an extra step:
- **Do NOT wrap known-tiny commands in `sieve`** (`pwd`, `git branch`, reading a 40-line config file). Use `sieve` only when output may exceed `~1.5–2 KB` (`pytest`, `cargo`, `gh run view --log`, `curl`, HTML, big logs).
- **Do NOT put global identity rules into `recall`**. Rules needed on *100% of turns* belong in the top 30 lines of `AGENTS.md` / `SOUL.md`; `recall` is for the 50–1,000 *conditional* rules needed on `< 10%` of turns.
- **Do NOT dump raw chat history into `baton`**. `baton` stays fast and sharp because each entry is a 1-line verified fact or falsified dead-end (`< 400 tokens` total).
