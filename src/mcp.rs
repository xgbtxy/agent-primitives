use serde_json::{json, Value};
use std::io::{self, BufRead, Read, Write};
use std::process::Command;

fn mcp_tools_schema() -> Value {
    json!([
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
            "name": "ap_verify",
            "description": "Run final quality gate before finishing: checks 0 broken HTML #id <-> JS selector or fetch() <-> route contracts, 0 anti-slop violations (:any, @ts-ignore, empty catch {}), and runs project tests.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "cwd": { "type": "string", "description": "Workspace directory (default: '.')" }
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
            if ops.is_empty() {
                return (
                    false,
                    "Provide at least one entry in `writes` or `edits`.".to_string(),
                );
            }
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
            let (ok, out) = run_self_captured(&cli, Some(&spec));
            if ok {
                crate::stats::record_event("edit", 0, 0, 0, n_ops, "mcp-edit");
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
            let (g_ok, g_txt) = run_self_captured(
                &["guard".into(), "verify".into(), "--cwd".into(), cwd.into()],
                None,
            );
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
