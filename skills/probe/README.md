# `probe` — Polyglot Reachability Bundler & Causal Execution Microscope

[English](./README.md) | [简体中文](./README.zh-CN.md)

**Standalone Mini-Project (`skills/probe/`)** · Zero third-party dependencies · `~62 tokens` idle skill cost.

## Why You Need It (Discovered via Live Frontier CLI Telemetry)

When we instrumented live multi-file bug-fixing sessions across **Grok Build CLI (`grok-4.7`)**, **Google Antigravity CLI (`agy`)**, and **OpenAI Codex CLI (`gpt-6-astra`)**, two massive process bottlenecks dominated latency and token burn:

1. **Reachability Blindness & Sequential Read Round-Trips**: Text-only tools (`list_dir`, `grep`, `read_file`) cannot distinguish the 8–12 modules on the active execution call graph from 15–20+ unreferenced sibling modules—and even after finding active files, agents burn 10–18 sequential `read_file` calls loading them one by one.
2. **Mental Heap-Execution Thrashing**: Without a structured runtime microscope, frontier models burn **25,000–55,000 reasoning tokens in prose** trying to mentally simulate shallow-copy nested dict aliasing, in-place argument clobbering, void state-rollback no-ops, and Python `bool`/`int`/`None` (`True == 1`, `hash(True) == hash(1)`) edge cases.

`probe` upgrades an agent's foundational coding process with three `< 0.1s` deterministic stdlib primitives:

- **`probe slice [--bundle]`**: Static import/AST reachability across **Python, TypeScript/JavaScript (`.ts/.tsx/.js/.mjs`), Go (`go.mod`), and Rust (`mod`/`crate::`)**—prunes dead distractor files, prints cross-file call contracts, and with `--bundle` emits numbered source of active reachable modules in **1 single call** (`--max-bundle-kb 28`).
- **`probe run`**: Scoped `sys.settrace` runtime microscope (call-site deduplicated so 200+ loop iterations stay `< 3.5 KB`) that automatically flags **`[ALIAS HAZARD]`**, **`[IN-PLACE MUTATION]` / `[MUTATED ON EXCEPTION]`**, and **`[STATE NO-OP]`**. Supports `--code`, `--stdin`, `--b64`, `--script`, and `--unittest`.
- **`probe diff`**: Differential & adversarial boundary oracle (`None`, `True` vs `1`, `False` vs `0`, nested lists/dicts) checking strict type-tag parity (`type(a) is type(b)`), stateful/sequential cache collisions, and caller input purity.

## 1-Command Onboarding (or Copy Just This Folder)

```bash
# Recommended: 1-command repo setup
python ap.py init --here

# Or copy just this single standalone folder:
cp -r skills/probe ~/.agents/skills/
```

## Real Commands (PowerShell & Bash Safe)

```bash
# 1. Polyglot static call-graph slice + 1-shot active source bundle (Py, TS/JS, Go, Rust)
python .agents/ap.py probe slice . --bundle

# 2. Causal runtime trace (call-site deduplicated; use `@'...'@ |` on PowerShell or `<<'EOF'` on Bash for multi-line)
python .agents/ap.py probe run --cwd . --focus mypkg --code "from mypkg import Engine; ..."

# 3. Differential & stateful cache equivalence probe across boundary inputs
python .agents/ap.py probe diff --cwd . \
  --setup "from mypkg import Engine; opt=Engine(optimize=True); raw=Engine(optimize=False)" \
  --fn-a "lambda d: opt.eval(d)" \
  --fn-b "lambda d: raw.eval(d)"
```
