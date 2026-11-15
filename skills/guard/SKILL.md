---
name: guard
description: Enforce deterministic `exit 2` completion gates, break repetitive failure loops (3x identical error/command), and block secret leaks or high-blast-radius destructive actions across Coding Agents and Personal Assistant Agents.
---

# Guard — Exit-2 Truth Gate, Loop Breaker & Blast-Radius Shield

In Claude Code, Codex, Grok Build, and modern agent harnesses, **`exit 1` is non-blocking** on Stop/PreToolUse hooks—only **`exit 2` + `stderr` feedback** deterministically stops premature completion, halts repetition loops, or blocks unsafe external actions.

## Usage

```bash
# 1. Pre-execution shield: blocks leaked secrets & destructive blast-radius commands (exit 2)
python scripts/guard.py shield --text "git push --force origin main"

# 2. Loop breaker: tracks command + failure fingerprints; trips exit 2 on 3rd repeat
python scripts/guard.py run --max-repeats 3 -- pytest tests/test_auth.py

# 3. Stop-hook truth gate: blocks false "Done!" claims until checks + conflict markers pass (exit 2)
python scripts/guard.py stop-gate --check "python -m unittest discover -s tests"
```

## Hook Wiring (`ap.py init --with-hooks`)

`guard.py` auto-detects both **Claude Code / Codex** snake_case hooks (`exit 2` + `stderr`) and **Google Antigravity** camelCase protojson hooks (`{"decision": "deny"|"allow"}` and `{"decision": "continue"|"stop"}` with `continuationActive` loop protection). Run `python ap.py init --with-hooks` to wire automatically.
