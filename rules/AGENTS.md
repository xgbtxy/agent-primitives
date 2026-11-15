# Agent Primitives Runtime Routing

1. **Orient & Bundle Active Slice (`probe`)**: Start unfamiliar multi-file Py/TS/JS/Go/Rust tasks with `probe slice . --bundle` to prune dead distractor files and inspect active reachable modules in 1 call.
2. **Runtime & Equivalence Microscope (`probe`)**: Use `probe run --cwd . --focus <pkg> --stdin` and `probe diff` instead of simulating Python heap aliasing, in-place mutations, or cache collisions in prose.
3. **Surgical Read, Atomic Patch & Auto-Test (`splice`)**: Use `splice view <files...> --symbol <Sym>`, batch atomic edits via `splice apply --cwd . --stdin`, and verify syntax/imports/tests via `splice check . --test`.
4. **Sieve, Memory, Guard & Baton**: Compress >2KB outputs via `sieve run -- <cmd>`, query/save gotchas via `recall`, enforce `exit 2` shields/loop-breakers via `guard`, and preserve verified facts across compaction via `baton`.
