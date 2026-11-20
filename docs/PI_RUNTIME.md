# Pi Coding Agent Runtime Maintenance (`ap pi` + `zz-ap-accelerator.ts`)

`ap` maintains both the zero-prompt-overhead Pi extension (`zz-ap-accelerator.ts`, installed via `ap init -a pi --with-hooks --no-rules`) and a pure-Rust, idempotent runtime patcher (`ap pi`) for `@earendil-works/pi-coding-agent` v1.5+.

## Commands

```bash
# Inspect detected @earendil-works/pi-coding-agent package and patch status
ap pi status

# Apply all 5 generic runtime patches idempotently (originals backed up to ~/.ap/pi-backups/<version>/)
ap pi patch

# Preview without modifying files
ap pi patch --dry-run

# Restore pristine originals from ~/.ap/pi-backups/<version>/
ap pi restore
```

Pass `--pi-dir <path>` (or set `PI_PACKAGE_DIR`) if Pi is installed in a non-standard location.

## Maintained Generic Pi v1.5+ Runtime Patches

1. **`before_agent_start_entries`**: Extends `emitBeforeAgentStart` & `AgentSession._prepareRun` (both bundled `dist/bundle/chunks/*.js` and unbundled `dist/core/*` + `types.d.ts`) so `before_agent_start` handlers can return `entries?: SessionBoundaryDraft[]` — enabling programmatic, zero-latency clean-slate compaction (`{ type: "compaction", firstKeptEntryId: null, summary }`) before a user turn without losing session logs or breaking KV-cache prefix reuse.
2. **`windows_python_utf8`**: Auto-injects `PYTHONUTF8=1` and `PYTHONIOENCODING=utf-8` into Pi's Windows subprocess environment to eliminate GBK codec crashes.
3. **`codemode_sleep_and_workers`**: Exposes abort-safe `await sleep(ms)` (capped at 30s) inside QuickJS `codemode` scripts and raises `DEFAULT_MAX_CODEMODE_WORKERS` from `2` to `4`.
4. **`http_keepalive_and_transient_retry`**: Adds resilient TCP/HTTP keepAlive & connect timeouts (`20s` connect, `25s` keepAlive) plus transient socket/gateway retry tokens (`ECONNRESET`, `ETIMEDOUT`, `EPIPE`, `UND_ERR_SOCKET`, `bad gateway`, `gateway timeout`) with jittered exponential backoff.
5. **`virtual_model_cache_warmer`**: Allows virtual models (`api === "pi-virtual"`, such as `deepseek/flash-auto`) to match `CacheWarmer` physical sessions cleanly.

## Pairing with `ap atlas surface` & `ap baton frontier`

Long-horizon reasoning sessions degrade when an agent relies solely on in-context memory to judge coverage. Combine clean-slate rotation (`/ap fresh [goal]` or `before_agent_start` boundary drafts) with external deterministic coverage tracking:

```bash
# 1. Extract 9 orthogonal entrypoint/sink axes across code, manifests & .asar/.vsix/.zip/.jar archives
ap atlas surface . --json | ap baton frontier seed --stdin

# 2. Pull Top-K unvisited items prioritizing strictly unexplored axes first
ap baton frontier next -k 5

# 3. Record verified / stopped items so fresh sessions never repeat dead ends
ap baton frontier mark <id-or-target> --status stopped --reason "verified safe"
```
