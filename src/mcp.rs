use serde_json::{json, Value};
use std::io::{self, BufRead, Read, Write};
use std::process::Command;

fn mcp_tools_schema() -> Value {
    json!([
        {
            "name": "ap_read",
            "description": "Universal zero-dependency reader: parses .docx/.xlsx/.pptx/.pdf/.sqlite/.gz, Electron .asar & .zip/.vsix/.jar/.whl virtual archives (--entry without extracting), PE/ELF/Mach-O headers + ASCII & Windows UTF-16LE strings, minified JS bundles (--offset/--chars/--pretty), GBK files, and JS-rendered URLs.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "target": { "type": "string", "description": "Local file/folder path or http(s):// URL" },
                    "lines": { "type": "string", "description": "Optional line window or comma-separated multi-ranges, e.g. '1-30,120-160'" },
                    "query": { "type": "string", "description": "Optional keyword/regex filter for rows, minified slices, archive entries, or binary strings" },
                    "sheet": { "type": "string", "description": "Optional worksheet filter for .xlsx" },
                    "entry": { "type": "string", "description": "Optional archive entry path inside .asar/.zip/.vsix/.jar/.apk/.whl" },
                    "offset": { "type": "string", "description": "Optional byte offset or comma-separated multi-offsets (e.g. '18000,95000') for slicing minified bundles in 1 call" },
                    "chars": { "type": "integer", "description": "Optional slice window length with offset (default: 2400)" },
                    "pretty": { "type": "boolean", "description": "Pretty-format minified JS/JSON snippets" },
                    "json_path": { "type": "string", "description": "Zero-jq JSON/JSONL path query (e.g. 'items[*].name' or '..token')" },
                    "map_pos": { "type": "string", "description": "SourceMap v3 generated 'line:col' to resolve back to original source via VLQ" },
                    "decode": { "type": "boolean", "description": "Auto-decode embedded JWT tokens, Base64 blobs, and URL-encoded strings" },
                    "render": { "type": "boolean", "description": "Render JS-driven URLs in real Headless Edge/Chrome" },
                    "hex": { "type": "boolean", "description": "Emit canonical hex+ASCII dump with SHA-256 & entropy (supports hex/decimal offset)" }
                },
                "required": ["target"]
            }
        },
        {
            "name": "ap_query",
            "description": "Execute exact SQLite 3.46+ SQL (`JOIN`, `GROUP BY`, `SUM`, `ROUND`, `CTE`, Window Functions) directly across `.xlsx` Excel workbooks, `GBK`/`UTF-8` `.csv`/`.tsv` tables, and `.json`/`.jsonl` files in <20ms with zero Python/DuckDB and zero numerical hallucination.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "files": {
                        "type": "array",
                        "description": "Table bindings like ['orders=sales.xlsx', 'bank=statement.csv']",
                        "items": { "type": "string" }
                    },
                    "sql": { "type": "string", "description": "SQL query to execute (omit or set schema=true to inspect table columns first)" },
                    "schema": { "type": "boolean", "description": "If true, list loaded table names, row counts & columns" },
                    "out": { "type": "string", "description": "Optional output CSV file path" }
                },
                "required": ["files"]
            }
        },
        {
            "name": "ap_search",
            "description": "Fast single-pass .gitignore-aware regex search (supports multi-pattern batching via `patterns`) across codebases (with AST spans/blocks), minified JS bundles, SourceMaps (`.map`), Electron `.asar`/`.vsix`/`.zip`, Office/PDF docs, and binary ASCII/UTF-16LE strings.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "pattern": { "type": "string", "description": "Primary regex or literal search pattern" },
                    "patterns": {
                        "type": "array",
                        "description": "Optional batch of extra regex patterns evaluated in the same single filesystem walk",
                        "items": { "type": "string" }
                    },
                    "path": { "type": "string", "description": "Directory or file to search (default: '.')" },
                    "ext": { "type": "string", "description": "Optional comma-separated extensions, e.g. 'ts,js' or 'asar,vsix,map'" },
                    "pretty": { "type": "boolean", "description": "Pretty-format matches inside minified single-line bundles" },
                    "multiline": { "type": "boolean", "description": "Enable cross-line multiline regex matching" },
                    "block": { "type": "boolean", "description": "Emit full enclosing Tree-sitter AST function/class block around each hit" }
                }
            }
        },
        {
            "name": "ap_undo",
            "description": "Safety net for non-Git folders (Desktop, Documents, notes, configs): atomically restores files modified by the most recent `ap_edit` checkpoint, or lists checkpoints.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "list": { "type": "boolean", "description": "If true, list available checkpoints without restoring" }
                }
            }
        },
        {
            "name": "ap_orient",
            "description": "Call FIRST on any empty or existing project. Returns verified host runtimes (Node/SQLite/Python/Rust/Browser), Tree-sitter code map, reachable files (pruning dead slop), UI controls, API routes, and broken DOM #id or route mismatches in <30ms.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Workspace root directory (default: '.')" }
                }
            }
        },
        {
            "name": "ap_impact",
            "description": "Call BEFORE changing any shared function, class, struct, interface, or route across multiple files. Maps exact Tree-sitter AST definitions, downstream callers/references (ignoring comments & strings), and related test files in <15ms so you never ship a partial refactor.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Workspace root directory (default: '.')" },
                    "symbol": { "type": "string", "description": "Symbol name or comma-separated symbols (e.g. 'computeTotal,OrderItem')" }
                },
                "required": ["symbol"]
            }
        },
        {
            "name": "ap_edit",
            "description": "Atomically create or edit multiple files in 1 call with zero shell-quoting issues. Cleans UTF-8 BOM/UTF-16LE, validates Tree-sitter AST syntax BEFORE writing to disk, and proactively warns if a modified function/type signature has untouched downstream callers (`[BLAST RADIUS]`).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "cwd": { "type": "string", "description": "Workspace directory (default: '.')" },
                    "writes": {
                        "type": "array",
                        "description": "Full file writes [{path, content}]",
                        "items": {
                            "type": "object",
                            "properties": {
                                "path": { "type": "string" },
                                "content": { "type": "string" }
                            },
                            "required": ["path", "content"]
                        }
                    },
                    "edits": {
                        "type": "array",
                        "description": "Exact substring replacements [{path, old, new}]",
                        "items": {
                            "type": "object",
                            "properties": {
                                "path": { "type": "string" },
                                "old": { "type": "string" },
                                "new": { "type": "string" }
                            },
                            "required": ["path", "old", "new"]
                        }
                    },
                    "appends": {
                        "type": "array",
                        "description": "Zero-truncation appends to end of files [{path, content}]",
                        "items": {
                            "type": "object",
                            "properties": {
                                "path": { "type": "string" },
                                "content": { "type": "string" }
                            },
                            "required": ["path", "content"]
                        }
                    },
                    "diff": { "type": "string", "description": "Optional standard Git unified diff (`--- a/...`, `+++ b/...`, `@@ ... @@`)" },
                    "run_tests": { "type": "boolean", "description": "Run auto-detected test suite after applying" }
                }
            }
        },
        {
            "name": "ap_smoke",
            "description": "Verify web/API apps on Windows/macOS/Linux without hanging terminals or leaking zombie processes. Auto-starts server on a free PORT, drives real headless Edge/Chrome CDP (OPEN /, FILL #id text, SELECT #id val, CLICK #id or text=Btn, EVAL <js>) + UTF-8 HTTP requests, checks 0 JS pageerror/console.error/server-stderr crashes, and kills all process trees.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "cwd": { "type": "string", "description": "Workspace directory (default: '.')" },
                    "start": { "type": "string", "description": "Optional start command (auto-detected from package.json if omitted)" },
                    "steps": {
                        "type": "array",
                        "description": "Smoke DSL lines, e.g. ['OPEN /', 'FILL #name Alice', 'CLICK #add-btn', 'EXPECT CONTAINS Alice', 'GET /api/items', 'EXPECT STATUS 200']",
                        "items": { "type": "string" }
                    }
                },
                "required": ["steps"]
            }
        },
        {
            "name": "ap_fuzz",
            "description": "Torture-test any created web/API server or CLI against malformed JSON, negative/zero/NaN boundary inputs, CJK/XSS/SQLi payloads, 404 paths, and simultaneous multi-threaded concurrency bursts in <1s.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "cwd": { "type": "string", "description": "Workspace directory (default: '.')" },
                    "start": { "type": "string", "description": "Optional server start command (auto-detected if omitted)" },
                    "cli": { "type": "string", "description": "Optional CLI command to fuzz instead of an HTTP server" },
                    "concurrency": { "type": "integer", "description": "Parallel race-condition threads (default: 8)" }
                }
            }
        },
        {
            "name": "ap_mutate",
            "description": "Test the quality of your unit tests in <2s via Tree-sitter AST fault-seeding (`>=`->`>`, `===`->`!==`, `+`->`-`, `&&`->`||`). Reports exact surviving mutants where your tests missed boundary or logic cases.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "cwd": { "type": "string", "description": "Workspace directory (default: '.')" },
                    "target": { "type": "string", "description": "Source file to mutate (e.g. 'src/wallet.ts')" },
                    "test_cmd": { "type": "string", "description": "Optional test command (auto-detected if omitted)" },
                    "max_mutants": { "type": "integer", "description": "Max mutants to evaluate (default: 8)" }
                },
            "required": ["target"]
        }
    },
    {
        "name": "ap_ui",
        "description": "Audit any web UI in real Headless Edge/Chrome across Desktop (1280x800) + Mobile (390x844): catches horizontal overflow, overlapped/blocked buttons via elementFromPoint, broken <img>, JS pageerror/console.error, and silent 4xx/5xx network leaks (0 MB Playwright).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "cwd": { "type": "string", "description": "Workspace directory (default: '.')" },
                "start": { "type": "string", "description": "Optional server start command (auto-detected if omitted)" },
                "path": { "type": "string", "description": "Page path (default: '/')" },
                "screenshot": { "type": "string", "description": "Optional PNG screenshot output path" }
            }
        }
    },
    {
        "name": "ap_locate",
        "description": "Hierarchical Agentless/AutoCodeRover fault localizer (<25ms): ranks Top-K suspect functions across the codebase via Tree-sitter + BM25 + camelCase/CJK splitting and returns exact numbered source lines so you find bugs in 1 call.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "Bug description, error message, or symptom keywords" },
                "path": { "type": "string", "description": "Workspace root directory (default: '.')" },
                "top_k": { "type": "integer", "description": "Number of suspect functions to return (default: 5)" }
            },
            "required": ["query"]
        }
    },
    {
        "name": "ap_trace",
        "description": "Multi-hop Tree-sitter call-chain tracer (<30ms): maps upstream callers (1..depth hops), downstream callees & notable I/O/exec/DOM/SQL sinks for any function across Py/TS/JS/Go/Rust/C.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "symbol": { "type": "string", "description": "Function or method symbol name to trace" },
                "path": { "type": "string", "description": "Workspace root directory (default: '.')" },
                "depth": { "type": "integer", "description": "Max upstream caller hops (default: 3)" }
            },
            "required": ["symbol"]
        }
    },
    {
        "name": "ap_cdp",
        "description": "Attach directly to an ALREADY-RUNNING Chrome/Edge/Electron/WebView CDP debugging port (--list targets, --target filter, --eval async JS, --watch-ms non-blocking console/network capture, --audit layout, --screenshot PNG) without killing the host app.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "host": { "type": "string", "description": "CDP host (default: '127.0.0.1')" },
                "port": { "type": "integer", "description": "CDP debugging port (default: 9222)" },
                "list": { "type": "boolean", "description": "List all live debuggable targets" },
                "target": { "type": "string", "description": "Substring matching target title, url, or id" },
                "eval": { "type": "string", "description": "JavaScript expression to evaluate (awaits Promises)" },
                "watch_ms": { "type": "integer", "description": "Optional non-blocking bounded event watch window in ms (captures async console logs & network responses)" },
                "audit": { "type": "boolean", "description": "Run live DOM layout/overflow/overlap check" },
                "screenshot": { "type": "string", "description": "Optional PNG screenshot output path" }
            }
        }
    },
    {
        "name": "ap_http",
        "description": "Zero-dependency HTTP/API diagnostic probe: execute single or batch (`batch_requests`) HTTP requests with custom headers, body, timeout, and automatic loopback proxy bypass.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "url": { "type": "string", "description": "Target HTTP(S) URL for single request" },
                "method": { "type": "string", "description": "HTTP method (default: 'GET')" },
                "headers": {
                    "type": "array",
                    "description": "Optional headers like ['Authorization: Bearer ...']",
                    "items": { "type": "string" }
                },
                "body": { "type": "string", "description": "Optional raw request body" },
                "batch_requests": {
                    "type": "array",
                    "description": "Optional list of 'METHOD URL [BODY]' lines to run in one batch call",
                    "items": { "type": "string" }
                },
                "timeout": { "type": "integer", "description": "Per-request timeout in seconds (default: 15)" },
                "insecure": { "type": "boolean", "description": "Skip TLS verification (-k)" },
                "follow": { "type": "boolean", "description": "Follow HTTP redirects (-L)" }
            }
        }
    },
    {
        "name": "ap_ports",
        "description": "Cross-platform listening TCP port, process PID & live CDP DevTools radar: lists local listening ports and auto-badges `[CDP READY]` Chrome/Electron debugging ports.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "port": { "type": "integer", "description": "Optional specific TCP port to inspect" }
            }
        }
    },
    {
        "name": "ap_ps",
        "description": "Cross-platform running process tree, exact command-line arguments & listening TCP ports inspector (replaces wmic/tasklist/ps).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "filter": { "type": "string", "description": "Optional case-insensitive substring matching process name, exe path, or command line" },
                "pid": { "type": "integer", "description": "Optional PID (matches process & its direct children)" },
                "limit": { "type": "integer", "description": "Max processes to return (default: 40)" }
            }
        }
    },
    {
        "name": "ap_fs_diff",
        "description": "Atomic before/after directory snapshot diff (`run`, `snapshot`, `diff`) and recent-files radar (`recent` seconds).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "dir": { "type": "string", "description": "Directory to inspect (default: '.')" },
                "recent": { "type": "integer", "description": "Filter files modified in the last N seconds" },
                "snapshot": { "type": "string", "description": "Save named snapshot for later diff" },
                "diff": { "type": "string", "description": "Diff current directory against saved snapshot name" },
                "run": { "type": "string", "description": "Execute shell command and atomically report added/modified/deleted files" }
            }
        }
    },
    {
        "name": "ap_tcp",
        "description": "Deadline-bounded raw TCP banner & text/hex send-recv probe (zero hanging Python sockets).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "addr": { "type": "string", "description": "Target host:port (e.g. '127.0.0.1:6379')" },
                "send": { "type": "string", "description": "Optional text payload (supports \\r\\n)" },
                "send_hex": { "type": "string", "description": "Optional hex bytes payload (e.g. '050100')" },
                "wait_ms": { "type": "integer", "description": "Max read wait window in ms (default: 800)" }
            },
            "required": ["addr"]
        }
    },
    {
        "name": "ap_listen",
        "description": "Bounded ephemeral HTTP webhook/callback capture server (auto-exits after max_reqs or wait_ms; substitutes {{PORT}} in `run`).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "port": { "type": "integer", "description": "Local TCP port to bind (0 = free ephemeral port)" },
                "wait_ms": { "type": "integer", "description": "Max listen duration in ms (default: 5000)" },
                "max_reqs": { "type": "integer", "description": "Stop cleanly after N requests (default: 1)" },
                "run": { "type": "string", "description": "Optional command to execute while listening (replaces {{PORT}})" }
            }
        }
    },
    {
        "name": "ap_ws",
        "description": "Deadline-bounded RFC 6455 non-CDP WebSocket (`ws://`) send/recv probe (never hangs on recv).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "url": { "type": "string", "description": "Target ws://host:port/path URL" },
                "send": { "type": "array", "items": { "type": "string" }, "description": "Text frames to send after handshake" },
                "headers": { "type": "array", "items": { "type": "string" }, "description": "Optional extra HTTP upgrade headers" },
                "wait_ms": { "type": "integer", "description": "Max receive window in ms (default: 1000)" }
            },
            "required": ["url"]
        }
    },
    {
        "name": "ap_tls",
        "description": "Cross-platform TLS protocol & X.509 certificate Subject/Issuer/Expiry/SHA-256 inspector.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "addr": { "type": "string", "description": "Target host:port (e.g. 'example.com:443')" },
                "sni": { "type": "string", "description": "Optional SNI hostname override" },
                "timeout": { "type": "integer", "description": "Timeout in seconds (default: 6)" }
            },
            "required": ["addr"]
        }
    },
    {
        "name": "ap_baton_claim",
        "description": "Atomic multi-agent resource lease manager (claim/release/list shared VMs, servers, ports with TTL expiry; exit 2 if actively held by another agent).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "action": { "type": "string", "description": "'claim', 'release', or 'list'" },
                "resource": { "type": "string", "description": "Resource key (e.g. 'win10-vm', 'cdp-9222')" },
                "owner": { "type": "string", "description": "Agent/workspace owner identifier" },
                "ttl": { "type": "integer", "description": "Lease TTL in seconds (default: 900)" },
                "note": { "type": "string", "description": "Optional short lease note" }
            },
            "required": ["action"]
        }
    },
    {
        "name": "ap_baton_dedup",
        "description": "Sub-15ms hybrid BM25 + CJK bigram + camelCase semantic near-duplicate finder across Markdown ledgers, notes & JSON/JSONL indexes.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "Candidate title, component, or effect summary to check" },
                "files": { "type": "array", "items": { "type": "string" }, "description": "Paths to ledger/index/markdown files" },
                "top_k": { "type": "integer", "description": "Max near-duplicate hits (default: 8)" }
            },
            "required": ["query", "files"]
        }
    },
    {
        "name": "ap_guard_pack",
        "description": "Deterministic deliverable/evidence bundle verifier: validates full 64-hex SHA-256 artifact manifests, scans secret/absolute-path leaks, and checks required headers.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "dir": { "type": "string", "description": "Bundle directory to verify" },
                "max_lines": { "type": "integer", "description": "Optional max non-empty narrative lines budget" },
                "require_headers": { "type": "array", "items": { "type": "string" }, "description": "Optional required header prefixes" }
            },
            "required": ["dir"]
        }
    },
    {
        "name": "ap_diff_audit",
        "description": "Semantic git diff self-audit: catches test-weakening cheating (deleted assertions), newly added debug leftovers (debugger/breakpoint/dbg!), and silent mass deletions.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "cwd": { "type": "string", "description": "Workspace directory (default: '.')" }
            }
        }
    },
    {
        "name": "ap_verify",
            "description": "Run final quality gate before finishing: checks 0 broken HTML #id <-> JS selector or fetch() <-> route contracts, 0 anti-slop violations (:any, @ts-ignore, empty catch {}), and runs project tests.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "cwd": { "type": "string", "description": "Workspace directory (default: '.')" },
                    "deep": { "type": "boolean", "description": "Also run cross-file export/arity linker, API chaos/concurrency fuzzer, and AST mutation audit in 1 pass" }
                }
            }
        },
        {
            "name": "ap_exec",
            "description": "Run shell, build, git, package-manager, Python or Node commands on Windows/macOS/Linux WITHOUT PowerShell `&&`/`curl`/heredoc syntax crashes, GBK/BOM encoding mojibake, pipe deadlocks, or noisy log floods. Automatically heals Windows commands, frees locked ports (`free_port`), kills hung process trees on timeout, and compresses >1.8KB logs.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "cwd": { "type": "string", "description": "Workspace directory (default: '.')" },
                    "command": { "type": "string", "description": "Shell command string to execute safely" },
                    "timeout": { "type": "integer", "description": "Timeout in seconds (default: 120)" },
                    "free_port": { "type": "integer", "description": "Optional local TCP port to free before running" }
                },
                "required": ["command"]
            }
        },
        {
            "name": "ap_probe",
            "description": "Execute a concrete Python/TS/JS/Rust snippet in the workspace (detecting heap aliasing, mutation & rounding bugs) OR run a noisy command through lossless log compression.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "cwd": { "type": "string", "description": "Workspace directory (default: '.')" },
                    "code": { "type": "string", "description": "Code snippet to trace via `ap probe run`" },
                    "sieve_cmd": { "type": "string", "description": "Shell command to run through `ap sieve` compressor" }
                }
            }
        }
    ])
}

fn run_self_captured(args: &[String], stdin_data: Option<&str>) -> (bool, String) {
    let Ok(exe) = std::env::current_exe() else {
        return (false, "Failed to locate ap binary".to_string());
    };
    let mut cmd = Command::new(exe);
    cmd.args(args).env("AP_MCP_CHILD", "1");
    cmd.stdin(if stdin_data.is_some() {
        std::process::Stdio::piped()
    } else {
        std::process::Stdio::null()
    });
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => return (false, format!("spawn error: {e}")),
    };
    if let Some(inp) = stdin_data {
        if let Some(mut sin) = child.stdin.take() {
            let _ = sin.write_all(inp.as_bytes());
        }
    }
    match child.wait_with_output() {
        Ok(o) => {
            let mut txt = String::from_utf8_lossy(&o.stdout).to_string();
            let err = String::from_utf8_lossy(&o.stderr);
            if !err.trim().is_empty() {
                if !txt.is_empty() && !txt.ends_with('\n') {
                    txt.push('\n');
                }
                txt.push_str(&err);
            }
            (o.status.success(), txt)
        }
        Err(e) => (false, format!("wait error: {e}")),
    }
}

fn dispatch_tool_call(name: &str, args: &Value) -> (bool, String) {
    match name {
        "ap_read" => {
            let target = args
                .get("target")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim();
            if target.is_empty() {
                return (
                    false,
                    "Pass non-empty `target` path or URL to `ap_read`.".to_string(),
                );
            }
            let mut cli = vec!["read".to_string(), target.to_string()];
            if let Some(l) = args
                .get("lines")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
            {
                cli.push("--lines".to_string());
                cli.push(l.to_string());
            }
            if let Some(q) = args
                .get("query")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
            {
                cli.push("--query".to_string());
                cli.push(q.to_string());
            }
            if let Some(sh) = args
                .get("sheet")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
            {
                cli.push("--sheet".to_string());
                cli.push(sh.to_string());
            }
            if let Some(ent) = args
                .get("entry")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
            {
                cli.push("--entry".to_string());
                cli.push(ent.to_string());
            }
            if let Some(off_val) = args.get("offset") {
                if let Some(s) = off_val.as_str().filter(|x| !x.trim().is_empty()) {
                    cli.push("--offset".to_string());
                    cli.push(s.to_string());
                } else if let Some(n) = off_val.as_u64() {
                    cli.push("--offset".to_string());
                    cli.push(n.to_string());
                }
            }
            if let Some(ch) = args.get("chars").and_then(Value::as_u64) {
                cli.push("--chars".to_string());
                cli.push(ch.to_string());
            }
            if args.get("pretty").and_then(Value::as_bool).unwrap_or(false) {
                cli.push("--pretty".to_string());
            }
            if let Some(jp) = args
                .get("json_path")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
            {
                cli.push("--json-path".to_string());
                cli.push(jp.to_string());
            }
            if let Some(mp) = args
                .get("map_pos")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
            {
                cli.push("--map-pos".to_string());
                cli.push(mp.to_string());
            }
            if args.get("decode").and_then(Value::as_bool).unwrap_or(false) {
                cli.push("--decode".to_string());
            }
            if args.get("render").and_then(Value::as_bool).unwrap_or(false) {
                cli.push("--render".to_string());
            }
            if args.get("hex").and_then(Value::as_bool).unwrap_or(false) {
                cli.push("--hex".to_string());
            }
            run_self_captured(&cli, None)
        }
        "ap_query" => {
            let files: Vec<&str> = args
                .get("files")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            if files.is_empty() {
                return (
                    false,
                    "Pass non-empty `files` array (e.g. ['t=data.xlsx']) to `ap_query`."
                        .to_string(),
                );
            }
            let mut cli = vec!["query".to_string()];
            for f in files {
                cli.push("-f".to_string());
                cli.push(f.to_string());
            }
            if args.get("schema").and_then(Value::as_bool).unwrap_or(false) {
                cli.push("--schema".to_string());
            }
            if let Some(out_p) = args
                .get("out")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
            {
                cli.push("--out".to_string());
                cli.push(out_p.to_string());
            }
            if let Some(sql) = args
                .get("sql")
                .and_then(Value::as_str)
                .filter(|s| !s.trim().is_empty())
            {
                cli.push(sql.to_string());
            }
            run_self_captured(&cli, None)
        }
        "ap_search" => {
            let mut all_pats: Vec<String> = Vec::new();
            if let Some(p) = args
                .get("pattern")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                all_pats.push(p.to_string());
            }
            if let Some(arr) = args.get("patterns").and_then(Value::as_array) {
                for item in arr {
                    if let Some(s) = item.as_str().map(str::trim).filter(|s| !s.is_empty()) {
                        all_pats.push(s.to_string());
                    }
                }
            }
            if all_pats.is_empty() {
                return (
                    false,
                    "Pass non-empty `pattern` or `patterns` to `ap_search`.".to_string(),
                );
            }
            let path = args.get("path").and_then(Value::as_str).unwrap_or(".");
            let mut cli = vec!["search".to_string(), all_pats[0].clone(), path.to_string()];
            for extra in all_pats.iter().skip(1) {
                cli.push("--pat".to_string());
                cli.push(extra.clone());
            }
            if let Some(ext) = args
                .get("ext")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
            {
                cli.push("--ext".to_string());
                cli.push(ext.to_string());
            }
            if args.get("pretty").and_then(Value::as_bool).unwrap_or(false) {
                cli.push("--pretty".to_string());
            }
            if args
                .get("multiline")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                cli.push("--multiline".to_string());
            }
            if args.get("block").and_then(Value::as_bool).unwrap_or(false) {
                cli.push("--block".to_string());
            }
            run_self_captured(&cli, None)
        }
        "ap_undo" => {
            let mut cli = vec!["undo".to_string()];
            if args.get("list").and_then(Value::as_bool).unwrap_or(false) {
                cli.push("--list".to_string());
            }
            run_self_captured(&cli, None)
        }
        "ap_orient" => {
            let root = args.get("path").and_then(Value::as_str).unwrap_or(".");
            let (_, map_json) = run_self_captured(
                &["atlas".into(), "map".into(), root.into(), "--json".into()],
                None,
            );
            let raw_b = serde_json::from_str::<Value>(&map_json)
                .ok()
                .and_then(|v| v.get("raw_bytes").and_then(Value::as_u64))
                .unwrap_or(0);
            let (_, map_txt) =
                run_self_captured(&["atlas".into(), "map".into(), root.into()], None);
            let (_, prod_txt) =
                run_self_captured(&["atlas".into(), "product".into(), root.into()], None);
            let out = format!("{map_txt}\n{prod_txt}");
            crate::stats::record_event(
                "orient",
                raw_b.max(out.len() as u64),
                out.len() as u64,
                0,
                1,
                "mcp-orient",
            );
            (true, out)
        }
        "ap_edit" => {
            let cwd = args.get("cwd").and_then(Value::as_str).unwrap_or(".");
            let run_tests = args
                .get("run_tests")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let mut ops = Vec::new();
            if let Some(writes) = args.get("writes").and_then(Value::as_array) {
                for w in writes {
                    if let (Some(p), Some(c)) = (
                        w.get("path").and_then(Value::as_str),
                        w.get("content").and_then(Value::as_str),
                    ) {
                        ops.push(json!({ "op": "write", "path": p, "content": c }));
                    }
                }
            }
            if let Some(edits) = args.get("edits").and_then(Value::as_array) {
                for e in edits {
                    if let (Some(p), Some(o), Some(n)) = (
                        e.get("path").and_then(Value::as_str),
                        e.get("old").and_then(Value::as_str),
                        e.get("new").and_then(Value::as_str),
                    ) {
                        ops.push(json!({ "op": "replace", "path": p, "old": o, "new": n }));
                    }
                }
            }
            if let Some(appends) = args.get("appends").and_then(Value::as_array) {
                for a in appends {
                    if let (Some(p), Some(c)) = (
                        a.get("path").and_then(Value::as_str),
                        a.get("content").and_then(Value::as_str),
                    ) {
                        ops.push(json!({ "op": "append", "path": p, "content": c }));
                    }
                }
            }
            if let Some(diff_raw) = args
                .get("diff")
                .and_then(Value::as_str)
                .filter(|s| !s.trim().is_empty())
            {
                if let Ok(parsed) = ap_splice::patch::parse_edit_spec(diff_raw) {
                    for p_op in parsed {
                        if let Ok(v) = serde_json::to_value(p_op) {
                            ops.push(v);
                        }
                    }
                }
            }
            if ops.is_empty() {
                return (
                    false,
                    "Provide at least one entry in `writes`, `edits`, `appends`, or `diff`."
                        .to_string(),
                );
            }
            let mut touched_for_undo = Vec::new();
            for item in &ops {
                if let Some(p) = item.get("path").and_then(Value::as_str) {
                    touched_for_undo.push(p.to_string());
                }
            }
            let undo_snap =
                crate::undo::record_pre_edit_snapshot(std::path::Path::new(cwd), &touched_for_undo);
            let n_ops = ops.len() as u32;
            let spec = json!({ "edits": ops }).to_string();
            let mut cli = vec![
                "splice".to_string(),
                "apply".to_string(),
                "--cwd".to_string(),
                cwd.to_string(),
                "--stdin".to_string(),
            ];
            if run_tests {
                cli.push("--test".to_string());
            }
            let (ok, mut out) = run_self_captured(&cli, Some(&spec));
            if ok {
                crate::stats::record_event("edit", 0, 0, 0, n_ops, "mcp-edit");
                if let Some(ref snap) = undo_snap {
                    let warns = crate::undo::check_truncation_warnings(snap);
                    if !warns.is_empty() {
                        out.push('\n');
                        out.push_str(&warns.join("\n"));
                    }
                }
            }
            (ok, out)
        }
        "ap_impact" => {
            let root = args.get("path").and_then(Value::as_str).unwrap_or(".");
            let sym = args
                .get("symbol")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim();
            if sym.is_empty() {
                return (false, "Pass non-empty `symbol` to `ap_impact`.".to_string());
            }
            let (ok, out) = run_self_captured(
                &["atlas".into(), "impact".into(), sym.into(), root.into()],
                None,
            );
            crate::stats::record_event("impact", 0, out.len() as u64, 0, 1, "mcp-impact");
            (ok, out)
        }
        "ap_fuzz" => {
            let cwd = args.get("cwd").and_then(Value::as_str).unwrap_or(".");
            let mut cli = vec![
                "probe".to_string(),
                "fuzz".to_string(),
                "--cwd".to_string(),
                cwd.to_string(),
            ];
            if let Some(st) = args
                .get("start")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
            {
                cli.push("--start".to_string());
                cli.push(st.to_string());
            }
            if let Some(cc) = args
                .get("cli")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
            {
                cli.push("--cli".to_string());
                cli.push(cc.to_string());
            }
            if let Some(conc) = args.get("concurrency").and_then(Value::as_u64) {
                cli.push("--concurrency".to_string());
                cli.push(conc.to_string());
            }
            let (ok, out) = run_self_captured(&cli, None);
            crate::stats::record_event("fuzz", 0, out.len() as u64, 0, 1, "mcp-fuzz");
            (ok, out)
        }
        "ap_mutate" => {
            let cwd = args.get("cwd").and_then(Value::as_str).unwrap_or(".");
            let target = args
                .get("target")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim();
            if target.is_empty() {
                return (false, "Pass `target` file path to `ap_mutate`.".to_string());
            }
            let mut cli = vec![
                "probe".to_string(),
                "mutate".to_string(),
                target.to_string(),
                "--cwd".to_string(),
                cwd.to_string(),
            ];
            if let Some(tc) = args
                .get("test_cmd")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
            {
                cli.push("--test-cmd".to_string());
                cli.push(tc.to_string());
            }
            if let Some(mm) = args.get("max_mutants").and_then(Value::as_u64) {
                cli.push("--max-mutants".to_string());
                cli.push(mm.to_string());
            }
            let (ok, out) = run_self_captured(&cli, None);
            crate::stats::record_event("mutate", 0, out.len() as u64, 0, 1, "mcp-mutate");
            (ok, out)
        }
        "ap_ui" => {
            let cwd = args.get("cwd").and_then(Value::as_str).unwrap_or(".");
            let mut cli = vec![
                "probe".to_string(),
                "ui".to_string(),
                "--cwd".to_string(),
                cwd.to_string(),
            ];
            if let Some(st) = args
                .get("start")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
            {
                cli.push("--start".to_string());
                cli.push(st.to_string());
            }
            if let Some(p) = args
                .get("path")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
            {
                cli.push("--path".to_string());
                cli.push(p.to_string());
            }
            if let Some(sc) = args
                .get("screenshot")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
            {
                cli.push("--screenshot".to_string());
                cli.push(sc.to_string());
            }
            let (ok, out) = run_self_captured(&cli, None);
            crate::stats::record_event("ui", 0, out.len() as u64, 0, 1, "mcp-ui");
            (ok, out)
        }
        "ap_locate" => {
            let q = args
                .get("query")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim();
            if q.is_empty() {
                return (false, "Pass non-empty `query` to `ap_locate`.".to_string());
            }
            let root = args.get("path").and_then(Value::as_str).unwrap_or(".");
            let top_k = args.get("top_k").and_then(Value::as_u64).unwrap_or(5);
            let (ok, out) = run_self_captured(
                &[
                    "atlas".into(),
                    "locate".into(),
                    q.into(),
                    root.into(),
                    "-k".into(),
                    top_k.to_string(),
                ],
                None,
            );
            crate::stats::record_event("locate", 0, out.len() as u64, 0, 1, "mcp-locate");
            (ok, out)
        }
        "ap_exec" => {
            let cwd = args.get("cwd").and_then(Value::as_str).unwrap_or(".");
            let cmd = args
                .get("command")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim();
            if cmd.is_empty() {
                return (false, "Pass non-empty `command` to `ap_exec`.".to_string());
            }
            let mut cli = vec![
                "exec".to_string(),
                "--cwd".to_string(),
                cwd.to_string(),
                "-c".to_string(),
                cmd.to_string(),
            ];
            if let Some(t) = args.get("timeout").and_then(Value::as_u64) {
                cli.push("--timeout".to_string());
                cli.push(t.to_string());
            }
            if let Some(fp) = args.get("free_port").and_then(Value::as_u64) {
                cli.push("--free-port".to_string());
                cli.push(fp.to_string());
            }
            run_self_captured(&cli, None)
        }
        "ap_smoke" => {
            let cwd = args.get("cwd").and_then(Value::as_str).unwrap_or(".");
            let steps: Vec<&str> = args
                .get("steps")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            let spec_txt = steps.join("\n");
            let mut cli = vec![
                "probe".to_string(),
                "smoke".to_string(),
                "--cwd".to_string(),
                cwd.to_string(),
                "--stdin".to_string(),
            ];
            if let Some(st) = args
                .get("start")
                .and_then(Value::as_str)
                .filter(|s| !s.trim().is_empty())
            {
                cli.push("--start".to_string());
                cli.push(st.to_string());
            }
            let (ok, out) = run_self_captured(&cli, Some(&spec_txt));
            crate::stats::record_event("smoke", 0, 0, 1, 1, "mcp-smoke");
            (ok, out)
        }
        "ap_trace" => {
            let sym = args
                .get("symbol")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim();
            if sym.is_empty() {
                return (false, "Pass non-empty `symbol` to `ap_trace`.".to_string());
            }
            let path = args.get("path").and_then(Value::as_str).unwrap_or(".");
            let depth = args.get("depth").and_then(Value::as_u64).unwrap_or(3);
            run_self_captured(
                &[
                    "atlas".into(),
                    "trace".into(),
                    sym.into(),
                    path.into(),
                    "--depth".into(),
                    depth.to_string(),
                ],
                None,
            )
        }
        "ap_cdp" => {
            let host = args
                .get("host")
                .and_then(Value::as_str)
                .unwrap_or("127.0.0.1");
            let port = args.get("port").and_then(Value::as_u64).unwrap_or(9222);
            let mut cli = vec![
                "probe".into(),
                "cdp".into(),
                "--host".into(),
                host.into(),
                "--port".into(),
                port.to_string(),
            ];
            if args.get("list").and_then(Value::as_bool).unwrap_or(false) {
                cli.push("--list".into());
            }
            if let Some(t) = args
                .get("target")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
            {
                cli.push("--target".into());
                cli.push(t.into());
            }
            if let Some(ev) = args
                .get("eval")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
            {
                cli.push("--eval".into());
                cli.push(ev.into());
            }
            if args.get("audit").and_then(Value::as_bool).unwrap_or(false) {
                cli.push("--audit".into());
            }
            if let Some(sp) = args
                .get("screenshot")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
            {
                cli.push("--screenshot".into());
                cli.push(sp.into());
            }
            if let Some(wm) = args.get("watch_ms").and_then(Value::as_u64) {
                if wm > 0 {
                    cli.push("--watch-ms".into());
                    cli.push(wm.to_string());
                }
            }
            run_self_captured(&cli, None)
        }
        "ap_http" => {
            let mut cli = vec!["probe".into(), "http".into()];
            if let Some(u) = args
                .get("url")
                .and_then(Value::as_str)
                .filter(|s| !s.trim().is_empty())
            {
                cli.push(u.trim().to_string());
            }
            if let Some(m) = args
                .get("method")
                .and_then(Value::as_str)
                .filter(|s| !s.trim().is_empty())
            {
                cli.push("-X".into());
                cli.push(m.trim().to_string());
            }
            if let Some(hdrs) = args.get("headers").and_then(Value::as_array) {
                for h in hdrs.iter().filter_map(Value::as_str) {
                    cli.push("-H".into());
                    cli.push(h.to_string());
                }
            }
            if let Some(b) = args
                .get("body")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
            {
                cli.push("-d".into());
                cli.push(b.to_string());
            }
            if let Some(t) = args.get("timeout").and_then(Value::as_u64) {
                cli.push("--timeout".into());
                cli.push(t.to_string());
            }
            if args
                .get("insecure")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                cli.push("--insecure".into());
            }
            if args.get("follow").and_then(Value::as_bool).unwrap_or(false) {
                cli.push("--follow".into());
            }
            let batch_lines: Vec<&str> = args
                .get("batch_requests")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            if !batch_lines.is_empty() {
                cli.push("--batch-stdin".into());
                let payload = batch_lines.join("\n");
                run_self_captured(&cli, Some(&payload))
            } else {
                run_self_captured(&cli, None)
            }
        }
        "ap_ports" => {
            let mut cli = vec!["probe".into(), "ports".into()];
            if let Some(p) = args.get("port").and_then(Value::as_u64) {
                cli.push("--port".into());
                cli.push(p.to_string());
            }
            run_self_captured(&cli, None)
        }
        "ap_ps" => {
            let mut cli = vec!["probe".into(), "ps".into()];
            if let Some(f) = args
                .get("filter")
                .and_then(Value::as_str)
                .filter(|s| !s.trim().is_empty())
            {
                cli.push("--filter".into());
                cli.push(f.to_string());
            }
            if let Some(pid) = args.get("pid").and_then(Value::as_u64) {
                cli.push("--pid".into());
                cli.push(pid.to_string());
            }
            if let Some(lim) = args.get("limit").and_then(Value::as_u64) {
                cli.push("--limit".into());
                cli.push(lim.to_string());
            }
            run_self_captured(&cli, None)
        }
        "ap_fs_diff" => {
            let dir = args.get("dir").and_then(Value::as_str).unwrap_or(".");
            let mut cli = vec!["probe".into(), "fs".into(), dir.into()];
            if let Some(r) = args.get("recent").and_then(Value::as_u64) {
                if r > 0 {
                    cli.push("--recent".into());
                    cli.push(r.to_string());
                }
            }
            if let Some(sn) = args
                .get("snapshot")
                .and_then(Value::as_str)
                .filter(|s| !s.trim().is_empty())
            {
                cli.push("--snapshot".into());
                cli.push(sn.to_string());
            }
            if let Some(df) = args
                .get("diff")
                .and_then(Value::as_str)
                .filter(|s| !s.trim().is_empty())
            {
                cli.push("--diff".into());
                cli.push(df.to_string());
            }
            if let Some(rc) = args
                .get("run")
                .and_then(Value::as_str)
                .filter(|s| !s.trim().is_empty())
            {
                cli.push("--run".into());
                cli.push(rc.to_string());
            }
            run_self_captured(&cli, None)
        }
        "ap_tcp" => {
            let addr = args
                .get("addr")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim();
            if addr.is_empty() {
                return (
                    false,
                    "Pass non-empty `addr` (host:port) to `ap_tcp`.".to_string(),
                );
            }
            let mut cli = vec!["probe".into(), "tcp".into(), addr.to_string()];
            if let Some(s) = args.get("send").and_then(Value::as_str) {
                cli.push("--send".into());
                cli.push(s.to_string());
            }
            if let Some(hx) = args
                .get("send_hex")
                .and_then(Value::as_str)
                .filter(|s| !s.trim().is_empty())
            {
                cli.push("--send-hex".into());
                cli.push(hx.to_string());
            }
            if let Some(wm) = args.get("wait_ms").and_then(Value::as_u64) {
                cli.push("--wait-ms".into());
                cli.push(wm.to_string());
            }
            run_self_captured(&cli, None)
        }
        "ap_listen" => {
            let mut cli = vec!["probe".into(), "listen".into()];
            if let Some(p) = args.get("port").and_then(Value::as_u64) {
                cli.push("--port".into());
                cli.push(p.to_string());
            }
            if let Some(wm) = args.get("wait_ms").and_then(Value::as_u64) {
                cli.push("--wait-ms".into());
                cli.push(wm.to_string());
            }
            if let Some(mr) = args.get("max_reqs").and_then(Value::as_u64) {
                cli.push("--max-reqs".into());
                cli.push(mr.to_string());
            }
            if let Some(rc) = args
                .get("run")
                .and_then(Value::as_str)
                .filter(|s| !s.trim().is_empty())
            {
                cli.push("--run".into());
                cli.push(rc.to_string());
            }
            run_self_captured(&cli, None)
        }
        "ap_ws" => {
            let url = args.get("url").and_then(Value::as_str).unwrap_or("").trim();
            if url.is_empty() {
                return (
                    false,
                    "Pass non-empty `url` (ws://...) to `ap_ws`.".to_string(),
                );
            }
            let mut cli = vec!["probe".into(), "ws".into(), url.to_string()];
            if let Some(arr) = args.get("send").and_then(Value::as_array) {
                for m in arr.iter().filter_map(Value::as_str) {
                    cli.push("--send".into());
                    cli.push(m.to_string());
                }
            }
            if let Some(hdrs) = args.get("headers").and_then(Value::as_array) {
                for h in hdrs.iter().filter_map(Value::as_str) {
                    cli.push("-H".into());
                    cli.push(h.to_string());
                }
            }
            if let Some(wm) = args.get("wait_ms").and_then(Value::as_u64) {
                cli.push("--wait-ms".into());
                cli.push(wm.to_string());
            }
            run_self_captured(&cli, None)
        }
        "ap_tls" => {
            let addr = args
                .get("addr")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim();
            if addr.is_empty() {
                return (
                    false,
                    "Pass non-empty `addr` (host:port) to `ap_tls`.".to_string(),
                );
            }
            let mut cli = vec!["probe".into(), "tls".into(), addr.to_string()];
            if let Some(s) = args
                .get("sni")
                .and_then(Value::as_str)
                .filter(|x| !x.is_empty())
            {
                cli.push("--sni".into());
                cli.push(s.to_string());
            }
            if let Some(t) = args.get("timeout").and_then(Value::as_u64) {
                cli.push("--timeout".into());
                cli.push(t.to_string());
            }
            run_self_captured(&cli, None)
        }
        "ap_baton_claim" => {
            let act = args
                .get("action")
                .and_then(Value::as_str)
                .unwrap_or("list")
                .to_ascii_lowercase();
            let mut cli = vec!["baton".into()];
            match act.as_str() {
                "claim" => {
                    let res = args.get("resource").and_then(Value::as_str).unwrap_or("");
                    let own = args.get("owner").and_then(Value::as_str).unwrap_or("");
                    if res.is_empty() || own.is_empty() {
                        return (
                            false,
                            "Pass `resource` and `owner` for action='claim'.".into(),
                        );
                    }
                    cli.extend(["claim".into(), res.into(), "--owner".into(), own.into()]);
                    if let Some(ttl) = args.get("ttl").and_then(Value::as_u64) {
                        cli.extend(["--ttl".into(), ttl.to_string()]);
                    }
                    if let Some(note) = args.get("note").and_then(Value::as_str) {
                        cli.extend(["--note".into(), note.into()]);
                    }
                }
                "release" => {
                    let res = args.get("resource").and_then(Value::as_str).unwrap_or("");
                    if res.is_empty() {
                        return (false, "Pass `resource` for action='release'.".into());
                    }
                    cli.extend(["release".into(), res.into()]);
                    if let Some(own) = args.get("owner").and_then(Value::as_str) {
                        cli.extend(["--owner".into(), own.into()]);
                    }
                }
                _ => cli.push("claims".into()),
            }
            run_self_captured(&cli, None)
        }
        "ap_baton_dedup" => {
            let q = args
                .get("query")
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim();
            let files: Vec<&str> = args
                .get("files")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            if q.is_empty() || files.is_empty() {
                return (false, "Pass non-empty `query` and `files` array.".into());
            }
            let mut cli = vec!["baton".into(), "dedup".into(), q.to_string()];
            for f in files {
                cli.push(f.to_string());
            }
            if let Some(k) = args.get("top_k").and_then(Value::as_u64) {
                cli.extend(["-k".into(), k.to_string()]);
            }
            run_self_captured(&cli, None)
        }
        "ap_guard_pack" => {
            let dir = args.get("dir").and_then(Value::as_str).unwrap_or(".");
            let mut cli = vec!["guard".into(), "pack".into(), dir.to_string()];
            if let Some(ml) = args.get("max_lines").and_then(Value::as_u64) {
                cli.extend(["--max-lines".into(), ml.to_string()]);
            }
            if let Some(hdrs) = args.get("require_headers").and_then(Value::as_array) {
                for h in hdrs.iter().filter_map(Value::as_str) {
                    cli.extend(["--require-header".into(), h.to_string()]);
                }
            }
            run_self_captured(&cli, None)
        }
        "ap_diff_audit" => {
            let cwd = args.get("cwd").and_then(Value::as_str).unwrap_or(".");
            run_self_captured(&["guard".into(), "diff".into(), cwd.into()], None)
        }
        "ap_verify" => {
            let cwd = args.get("cwd").and_then(Value::as_str).unwrap_or(".");
            let (_, prod_json) = run_self_captured(
                &[
                    "atlas".into(),
                    "product".into(),
                    cwd.into(),
                    "--json".into(),
                ],
                None,
            );
            let mut mismatch_lines = Vec::new();
            if let Ok(pj) = serde_json::from_str::<Value>(&prod_json) {
                if let Some(arr) = pj.get("contract_mismatches").and_then(Value::as_array) {
                    for m in arr {
                        if let Some(s) = m.as_str() {
                            mismatch_lines.push(format!("- [CONTRACT MISMATCH] {s}"));
                        }
                    }
                }
            }
            let mut v_args = vec!["guard".into(), "verify".into(), "--cwd".into(), cwd.into()];
            if args.get("deep").and_then(Value::as_bool).unwrap_or(false) {
                v_args.push("--deep".into());
            }
            let (g_ok, g_txt) = run_self_captured(&v_args, None);
            let ok = g_ok && mismatch_lines.is_empty();
            crate::stats::record_event("verify", 0, 0, 0, 1, "mcp-verify");
            let combined = if mismatch_lines.is_empty() {
                g_txt
            } else {
                format!(
                    "Contract mismatches detected:\n{}\n{g_txt}",
                    mismatch_lines.join("\n")
                )
            };
            (ok, combined)
        }
        "ap_probe" => {
            let cwd = args.get("cwd").and_then(Value::as_str).unwrap_or(".");
            if let Some(code) = args
                .get("code")
                .and_then(Value::as_str)
                .filter(|s| !s.trim().is_empty())
            {
                return run_self_captured(
                    &[
                        "probe".into(),
                        "run".into(),
                        "--cwd".into(),
                        cwd.into(),
                        "--stdin".into(),
                    ],
                    Some(code),
                );
            }
            if let Some(scmd) = args
                .get("sieve_cmd")
                .and_then(Value::as_str)
                .filter(|s| !s.trim().is_empty())
            {
                return run_self_captured(
                    &[
                        "exec".into(),
                        "--cwd".into(),
                        cwd.into(),
                        "-c".into(),
                        scmd.into(),
                    ],
                    None,
                );
            }
            (
                false,
                "Pass `code` or `sieve_cmd` to `ap_probe`.".to_string(),
            )
        }
        other => (false, format!("Unknown tool: {other}")),
    }
}

fn handle_rpc_request(req: &Value) -> Option<Value> {
    let method = req.get("method").and_then(Value::as_str).unwrap_or("");
    let id_opt = req.get("id").cloned();

    // Notifications have no `id` and expect no response
    id_opt.as_ref()?;
    let id = id_opt.unwrap();

    match method {
        "initialize" => {
            let client_proto = req
                .pointer("/params/protocolVersion")
                .and_then(Value::as_str)
                .unwrap_or("2024-11-05");
            Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": client_proto,
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "agent-primitives",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                }
            }))
        }
        "ping" => Some(json!({ "jsonrpc": "2.0", "id": id, "result": {} })),
        "tools/list" => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "tools": mcp_tools_schema()
            }
        })),
        "resources/list" => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": { "resources": [] }
        })),
        "prompts/list" => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": { "prompts": [] }
        })),
        "tools/call" => {
            let tname = req
                .pointer("/params/name")
                .and_then(Value::as_str)
                .unwrap_or("");
            let targs = req
                .pointer("/params/arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            let (ok, text) = dispatch_tool_call(tname, &targs);
            Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [{ "type": "text", "text": text }],
                    "isError": !ok
                }
            }))
        }
        _ => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32601,
                "message": format!("Method not found: {method}")
            }
        })),
    }
}

pub fn run_mcp_stdio() -> i32 {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = stdin.lock();
    let mut writer = stdout.lock();

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(_) => break,
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Mode A: LSP-style `Content-Length: N` framing
        if trimmed.to_ascii_lowercase().starts_with("content-length:") {
            let len: usize = trimmed
                .split_once(':')
                .and_then(|(_, v)| v.trim().parse().ok())
                .unwrap_or(0);
            loop {
                let mut hdr = String::new();
                if reader.read_line(&mut hdr).unwrap_or(0) == 0 || hdr.trim().is_empty() {
                    break;
                }
            }
            if len == 0 {
                continue;
            }
            let mut body = vec![0u8; len];
            if reader.read_exact(&mut body).is_err() {
                break;
            }
            if let Ok(req) = serde_json::from_slice::<Value>(&body) {
                if let Some(resp) = handle_rpc_request(&req) {
                    let s = resp.to_string();
                    let _ = write!(writer, "Content-Length: {}\r\n\r\n{}", s.len(), s);
                    let _ = writer.flush();
                }
            }
            continue;
        }

        // Mode B: Standard MCP 2024-11-05 / 2025-03-26 newline-delimited JSON-RPC
        if let Ok(req) = serde_json::from_str::<Value>(trimmed) {
            if let Some(resp) = handle_rpc_request(&req) {
                let _ = writeln!(writer, "{resp}");
                let _ = writer.flush();
            }
        }
    }
    0
}
