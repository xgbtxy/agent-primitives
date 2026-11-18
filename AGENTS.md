# `agent-primitives` Repository Operating Rules (`v1.0` Pure-Rust Edition)

Before proposing, adding, or modifying anything in this repository, read **[HANDBOOK.md](./HANDBOOK.md)** / **[HANDBOOK.zh-CN.md](./HANDBOOK.zh-CN.md)**.

## Six Non-Negotiable Invariants
1. **Physical Boundaries Only (Zero Prompt Crutches)**: Every primitive must execute deterministic OS / Tree-sitter AST / Heap / Byte / SQLite computation across one of the 4 irreducible physical boundaries—never pure-Markdown roleplay or prompt checklists.
2. **Crate-Level Physical Isolation + Single Binary (`ap`)**: Every `crates/ap-<name>/` crate must compile and run in isolation with **zero sibling `ap-*` crate dependencies** (enforced by `test_crate_physical_isolation_and_frontmatter_budget`), while compiling into a single self-contained native binary `ap` (`ap.exe`) with zero Python/Node/Docker runtime requirement.
3. **Zero External Runtime Dependencies & Prebuilt Cross-Platform Distribution**: Bundled static SQLite FTS5 (`rusqlite`) + compiled-in Tree-sitter C grammars (`Python`, `TS/TSX`, `JS`, `Go`, `Rust`, `C`) across Linux (`musl` static + `arm64`), macOS (`arm64` + `x64`), and Windows (`x64` MSVC), installable in 1 line (`install.sh` / `install.ps1` / `cargo install`).
4. **Hard Context & Latency Budgets**: `SKILL.md` YAML frontmatter **`< 85 tokens`** per folder (`57–61 tokens` measured); core CLI latency **`< 50 ms`** (`0.8–17 ms` measured); bounded stdout with lossless on-demand `peek` and BrokenPipe safety.
5. **Deepen Before Multiplying**: Prefer extending the existing 7 orthogonal primitives (`atlas`, `recall`, `probe`, `splice`, `sieve`, `guard`, `baton`) over creating new folders unless a new physical boundary is empirically proven.
6. **Empirical Truth & Bilingual Parity**: Every iteration must follow the 5-step loop in `HANDBOOK.md` (`ap audit` -> 4-gate check -> zero-warning `clippy -D warnings` + Rust tests -> live CLI A/B + negative controls -> 1:1 `*.md` & `*.zh-CN.md` sync + Evolution Log entry).

## Required Verification Before Every Commit
```bash
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run --release --bin ap -- doctor
cargo run --release --bin ap -- bench --out bench/results.json
```
