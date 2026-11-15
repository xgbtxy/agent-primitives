# `guard` — Exit-2 Truth Gate, Loop Breaker & Blast-Radius Shield

[English](./README.md) | [简体中文](./README.zh-CN.md)

**Standalone Mini-Project (`skills/guard/`)** · Zero third-party dependencies · `63 tokens` idle skill cost.

## Why You Need It

Three expensive runtime failures hit autonomous agents every day:
1. **Silent False Completion (`exit 1` Trap)**: In Claude Code, Codex, and modern harnesses, a `Stop` hook returning `exit 1` is treated as a *non-blocking warning*—the agent politely finishes anyway even when tests fail! Only **`exit 2` + `stderr` diagnostics** forces the agent back to fix the failure.
2. **Repetition Loops**: An agent runs the exact same broken command 5–8 times in a row, burning dollars with zero progress.
3. **Secrets & Blast-Radius Accidents**: Leaking `sk-...` / `ghp_...` / `AKIA...` tokens into shell commands or commits, running `git push --force` / `rm -rf`, or firing unbounded outbound emails/webhooks.

`guard` enforces all three deterministically via the native **`exit 2`** protocol.

## Copy Just This Folder

```bash
cp -r skills/guard ~/.agents/skills/
# or: npx skills add xgbtxy/agent-primitives --skill guard
```

## Real Commands

```bash
# 1. PreToolUse Shield: blocks secrets & destructive commands with exit 2 (accepts --text or stdin JSON)
python scripts/guard.py shield --text "git push --force origin main"

# 2. Loop Breaker: trips exit 2 automatically on the 3rd identical failure signature
python scripts/guard.py run --max-repeats 3 -- pytest tests/test_payment.py

# 3. Stop-Hook Truth Gate: verifies clean conflict markers, no exposed secrets & passing checks
python scripts/guard.py stop-gate --check "python -m unittest discover -s tests"
```
