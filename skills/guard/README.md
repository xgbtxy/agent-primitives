# `guard` (`crates/ap-guard`) — Sub-2ms Exit-2 Truth Gate, Loop Breaker & Shield

[English](./README.md) | [简体中文](./README.zh-CN.md)

**Standalone Rust Crate (`crates/ap-guard`) & Skill (`skills/guard/`)** · `< 2 ms` native hook latency · `57 tokens` idle skill cost.

## Why You Need It

1. **Silent False Completion (`exit 1` Trap)**: In Claude Code, Codex, and modern harnesses, a `Stop` hook returning `exit 1` is non-blocking—only **`exit 2` + `stderr` diagnostics** deterministically halts premature completion.
2. **Repetition Loops**: Agents running the same failing command 5–8 times in a row (`237` loops measured in local audit).
3. **Secrets & Blast-Radius Accidents**: Leaking `sk-...` / `ghp_...` / `AKIA...` tokens or running `git push --force` / `rm -rf /`.

Compiled to native Rust machine code (`< 2 ms` startup—25x faster than Python hook scripts!), `ap guard` auto-detects both **Claude Code / Codex** snake_case hooks (`exit 2` + `stderr`) and **Google Antigravity** camelCase protojson hooks (`{"decision": "deny"|"allow"}` / `{"decision": "continue"|"stop"}`).

## Wire Automatically

```bash
ap init --with-hooks
```

## Real Commands

```bash
ap guard shield --text "git push --force origin main"
ap guard run --max-repeats 3 -- cargo test
ap guard stop-gate --check "cargo test --quiet"
```
