# `probe` — Causal Execution, Reachability & Differential Microscope

[English](./README.md) | [简体中文](./README.zh-CN.md)

**Standalone Mini-Project (`skills/probe/`)** · Zero third-party dependencies · `~78 tokens` idle skill cost.

## Why You Need It (Discovered via Live Frontier CLI Telemetry)

When we instrumented live multi-file bug-fixing sessions on frontier CLI coding agents (including `grok-4.7` via Grok Build CLI), two massive process bottlenecks dominated latency and cost:

1. **Reachability Blindness**: Text-only tools (`list_dir`, `grep`, `read_file`) cannot distinguish the 8–12 modules on the active execution call graph from 20+ unreferenced sibling modules, wasting turns and polluting KV-cache with dead files.
2. **Mental Heap-Execution Thrashing**: Without a structured runtime microscope, frontier models burn **25,000–55,000 reasoning tokens in prose** trying to mentally simulate shallow-copy nested dict aliasing, in-place argument clobbering, void state-rollback no-ops (`rollback(event_id)` vs `digest` keys), and Python `bool`/`int`/`None` (`True == 1`, `hash(True) == hash(1)`) edge cases—or hand-write brittle 55-line monkeypatch scripts.

`probe` upgrades an agent's foundational coding process with three `< 0.1s` deterministic stdlib primitives:

- **`probe slice`**: Static AST + relative/package import reachability from any entrypoint—prunes dead distractor files and prints cross-file call sites next to target parameter contracts.
- **`probe run`**: Scoped `sys.settrace` runtime microscope that automatically flags **`[ALIAS HAZARD]`** (shallow-copy shared nested mutable heap refs), **`[IN-PLACE MUTATION]` / `[MUTATED ON EXCEPTION]`**, and **`[STATE NO-OP]`** (void mutator methods leaving `self` unchanged, printing actual `self` keys).
- **`probe diff`**: Differential & adversarial boundary oracle (`None`, `True` vs `1`, `False` vs `0`, nested lists/dicts) checking strict type-tag parity (`type(a) is type(b)`), stateful/sequential cache collisions, and caller input purity.

## Copy Just This Folder

```bash
cp -r skills/probe ~/.agents/skills/
# or for Grok CLI / Claude Code:
cp -r skills/probe ~/.grok/skills/
cp -r skills/probe ~/.claude/skills/
```

## Real Commands

```bash
# 1. Static call-graph reachability slice (prunes unreferenced modules + shows call contracts)
python scripts/probe.py slice . --entry mypkg

# 2. Causal runtime trace with automatic Alias, Mutation & State No-Op detection
python scripts/probe.py run --cwd . --focus mypkg --code "from mypkg import Engine; ..."

# 3. Differential & stateful cache equivalence probe across boundary inputs
python scripts/probe.py diff --cwd . \
  --setup "from mypkg import Engine; opt=Engine(optimize=True); raw=Engine(optimize=False)" \
  --fn-a "lambda d: opt.eval(d)" \
  --fn-b "lambda d: raw.eval(d)"
```
