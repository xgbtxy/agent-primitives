---
name: guard
description: Enforce sub-2ms `exit 2` anti-slop code rules (`rule`/`verify`), completion Stop-gates (`stop-gate`), 3x loop breakers (`run`), and secret/blast-radius shields (`shield`).
---

# Guard — Sub-2ms Exit-2 Anti-Slop Rules, Verification Gate & Blast Shield

Prose rules in `AGENTS.md` get ignored; deterministic **`exit 2`** code gates cannot be bypassed. `guard` enforces anti-slop code hygiene (`0` `any`/`@ts-ignore`/empty `catch {}`), project tests, `.agents/smoke.ap` live smoke gates, 3x loop breaking, and secret shields.

## Usage

```bash
# 1. Run full anti-slop rules + auto-detected tests + .agents/smoke.ap (exit 2 on any violation)
ap guard verify

# 2. Turn any past agent mistake into a permanent deterministic rule in .agents/guard.json
ap guard rule init
ap guard rule add --id no-legacy-import --glob "src/*.ts" --forbid "old_csv_dump" --msg "Use RFC4180 src/csv.ts"

# 3. Pre-execution shield, 3x loop breaker & Stop-hook gate
ap guard shield --text "git push --force origin main"
ap guard run --max-repeats 3 -- npm test
ap guard stop-gate
```
