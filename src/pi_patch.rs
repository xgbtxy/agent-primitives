use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(
    name = "pi",
    about = "Inspect, apply, or restore generic idempotent runtime patches for @earendil-works/pi-coding-agent (v1.5+)"
)]
struct PiCli {
    #[command(subcommand)]
    subcmd: PiSubCmd,
}

#[derive(Subcommand, Debug)]
enum PiSubCmd {
    /// Inspect detected @earendil-works/pi-coding-agent installation and patch status
    Status {
        #[arg(long)]
        pi_dir: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Idempotently apply all 5 generic Pi v1.5+ runtime patches (saves pristine originals to ~/.ap/pi-backups/)
    Patch {
        #[arg(long)]
        pi_dir: Option<String>,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        json: bool,
    },
    /// Restore original unpatched Pi runtime files from ~/.ap/pi-backups/
    Restore {
        #[arg(long)]
        pi_dir: Option<String>,
        #[arg(long)]
        json: bool,
    },
}

struct PatchHunk {
    id: &'static str,
    rel_hint: Option<&'static str>,
    old: &'static str,
    new: &'static str,
}

struct PatchSpec {
    id: &'static str,
    summary: &'static str,
    hunks: &'static [PatchHunk],
}

const PATCH_SPECS: &[PatchSpec] = &[
    PatchSpec {
        id: "before_agent_start_entries",
        summary: "Allow before_agent_start handlers to return boundary entries (e.g. firstKeptEntryId: null clean-slate compaction)",
        hunks: &[
            PatchHunk {
                id: "bundle_emit_before_agent_start",
                rel_hint: None,
                old: r#"let messages=[];for(let{ext:ext2,handlers}of snapshotEventHandlers(this.extensions,"before_agent_start"))for(let handler of handlers)try{let event={...trigger,type:"before_agent_start",get systemPrompt(){return renderCurrentSystemPrompt()},systemPromptOptions:currentOptions},handlerResult=await handler(event,ctx);if(handlerResult){let result=handlerResult;result.message&&messages.push(result.message),result.systemPrompt!==void 0&&(currentOptions.forceSystemPrompt=result.systemPrompt)}}catch(err){let message=err instanceof Error?err.message:String(err),stack=err instanceof Error?err.stack:void 0;this.emitError({extensionPath:ext2.path,event:"before_agent_start",error:message,stack})}return{messages,systemPromptOptions:currentOptions}"#,
                new: r#"let messages=[],entries=[];for(let{ext:ext2,handlers}of snapshotEventHandlers(this.extensions,"before_agent_start"))for(let handler of handlers)try{let event={...trigger,type:"before_agent_start",get systemPrompt(){return renderCurrentSystemPrompt()},systemPromptOptions:currentOptions},handlerResult=await handler(event,ctx);if(handlerResult){let result=handlerResult;result.message&&messages.push(result.message),Array.isArray(result.entries)&&entries.push(...result.entries),result.systemPrompt!==void 0&&(currentOptions.forceSystemPrompt=result.systemPrompt)}}catch(err){let message=err instanceof Error?err.message:String(err),stack=err instanceof Error?err.stack:void 0;this.emitError({extensionPath:ext2.path,event:"before_agent_start",error:message,stack})}return{messages,entries,systemPromptOptions:currentOptions}"#,
            },
            PatchHunk {
                id: "bundle_prepare_run_commit",
                rel_hint: None,
                old: r#"let selectedToolsBefore=this._baseSystemPromptOptions.selectedTools,result=await this._extensionRunner.emitBeforeAgentStart(trigger,this._baseSystemPromptOptions);result.systemPromptOptions.selectedTools.length!==selectedToolsBefore.length"#,
                new: r#"let selectedToolsBefore=this._baseSystemPromptOptions.selectedTools,result=await this._extensionRunner.emitBeforeAgentStart(trigger,this._baseSystemPromptOptions);Array.isArray(result.entries)&&result.entries.length>0&&this._commitBoundaryDrafts(result.entries);result.systemPromptOptions.selectedTools.length!==selectedToolsBefore.length"#,
            },
            PatchHunk {
                id: "core_runner_emit_entries",
                rel_hint: Some("dist/core/extensions/runner.js"),
                old: "        const messages = [];\n        for (const { ext, handlers } of snapshotEventHandlers(this.extensions, \"before_agent_start\")) {\n            for (const handler of handlers) {\n                try {\n                    const event = {\n                        ...trigger,\n                        type: \"before_agent_start\",\n                        get systemPrompt() {\n                            return renderCurrentSystemPrompt();\n                        },\n                        systemPromptOptions: currentOptions,\n                    };\n                    const handlerResult = await handler(event, ctx);\n                    if (handlerResult) {\n                        const result = handlerResult;\n                        if (result.message)\n                            messages.push(result.message);\n                        if (result.systemPrompt !== undefined) {\n                            currentOptions.forceSystemPrompt = result.systemPrompt;\n                        }\n                    }\n                }\n                catch (err) {\n                    const message = err instanceof Error ? err.message : String(err);\n                    const stack = err instanceof Error ? err.stack : undefined;\n                    this.emitError({\n                        extensionPath: ext.path,\n                        event: \"before_agent_start\",\n                        error: message,\n                        stack,\n                    });\n                }\n            }\n        }\n        return { messages, systemPromptOptions: currentOptions };",
                new: "        const messages = [];\n        const entries = [];\n        for (const { ext, handlers } of snapshotEventHandlers(this.extensions, \"before_agent_start\")) {\n            for (const handler of handlers) {\n                try {\n                    const event = {\n                        ...trigger,\n                        type: \"before_agent_start\",\n                        get systemPrompt() {\n                            return renderCurrentSystemPrompt();\n                        },\n                        systemPromptOptions: currentOptions,\n                    };\n                    const handlerResult = await handler(event, ctx);\n                    if (handlerResult) {\n                        const result = handlerResult;\n                        if (result.message)\n                            messages.push(result.message);\n                        if (Array.isArray(result.entries))\n                            entries.push(...result.entries);\n                        if (result.systemPrompt !== undefined) {\n                            currentOptions.forceSystemPrompt = result.systemPrompt;\n                        }\n                    }\n                }\n                catch (err) {\n                    const message = err instanceof Error ? err.message : String(err);\n                    const stack = err instanceof Error ? err.stack : undefined;\n                    this.emitError({\n                        extensionPath: ext.path,\n                        event: \"before_agent_start\",\n                        error: message,\n                        stack,\n                    });\n                }\n            }\n        }\n        return { messages, entries, systemPromptOptions: currentOptions };",
            },
            PatchHunk {
                id: "core_agent_session_prepare_run",
                rel_hint: Some("dist/core/agent-session.js"),
                old: "        const selectedToolsBefore = this._baseSystemPromptOptions.selectedTools;\n        const result = await this._extensionRunner.emitBeforeAgentStart(trigger, this._baseSystemPromptOptions);",
                new: "        const selectedToolsBefore = this._baseSystemPromptOptions.selectedTools;\n        const result = await this._extensionRunner.emitBeforeAgentStart(trigger, this._baseSystemPromptOptions);\n        if (Array.isArray(result.entries) && result.entries.length > 0) {\n            this._commitBoundaryDrafts(result.entries);\n        }",
            },
            PatchHunk {
                id: "core_types_before_agent_start",
                rel_hint: Some("dist/core/extensions/types.d.ts"),
                old: "export interface BeforeAgentStartEventResult {\n    message?: Pick<CustomMessage, \"customType\" | \"content\" | \"display\" | \"details\">;\n    /** Replace the complete system prompt for this turn. Later handlers observe this exact override. */\n    systemPrompt?: string;\n}",
                new: "export interface BeforeAgentStartEventResult {\n    message?: Pick<CustomMessage, \"customType\" | \"content\" | \"display\" | \"details\">;\n    /** Optional boundary drafts (e.g. self-retaining clean-slate compaction) committed before this run's prompt. */\n    entries?: SessionBoundaryDraft[];\n    /** Replace the complete system prompt for this turn. Later handlers observe this exact override. */\n    systemPrompt?: string;\n}",
            },
        ],
    },
    PatchSpec {
        id: "windows_python_utf8",
        summary: "Inject PYTHONUTF8=1 and PYTHONIOENCODING=utf-8 into Windows shell subprocess env",
        hunks: &[PatchHunk {
            id: "bundle_win_python_utf8",
            rel_hint: None,
            old: r#").join(delimiter);return{...process.env,[pathKey]:updatedPath}}function sanitize"#,
            new: r#").join(delimiter);return{...process.env,...process.platform==="win32"?{PYTHONUTF8:process.env.PYTHONUTF8??"1",PYTHONIOENCODING:process.env.PYTHONIOENCODING??"utf-8"}:{},[pathKey]:updatedPath}}function sanitize"#,
        }],
    },
    PatchSpec {
        id: "codemode_sleep_and_workers",
        summary: "Expose abort-safe await sleep(ms) (max 30s) in codemode QuickJS and raise max workers 2 -> 4",
        hunks: &[
            PatchHunk {
                id: "bundle_codemode_workers",
                rel_hint: None,
                old: r#"*1024*1024,DEFAULT_MAX_CODEMODE_WORKERS=2;function maxCodemodeWorkers()"#,
                new: r#"*1024*1024,DEFAULT_MAX_CODEMODE_WORKERS=4;function maxCodemodeWorkers()"#,
            },
            PatchHunk {
                id: "bundle_codemode_sleep_binding",
                rel_hint: None,
                old: r#":samples.get(name)??""});return[{name:"searchTools",spread:!0,execute:args=>{"#,
                new: r#":samples.get(name)??""});return[{name:"sleep",spread:!0,execute:([ms],{signal})=>{let c=Math.min(3e4,Math.max(0,Number(ms)||0));return new Promise((res,rej)=>{if(signal?.aborted){rej(new Error("aborted"));return}let onA=()=>{clearTimeout(t);rej(new Error("aborted"))},t=setTimeout(()=>{signal?.removeEventListener("abort",onA);res(c)},c);signal?.addEventListener("abort",onA,{once:!0})})}},{name:"searchTools",spread:!0,execute:args=>{"#,
            },
            PatchHunk {
                id: "bundle_codemode_prompt_hint",
                rel_hint: None,
                old: r#"No Node, file system, network, or timers."#,
                new: r#"No Node, file system, or network; await sleep(ms) is available (max 30s)."#,
            },
        ],
    },
    PatchSpec {
        id: "http_keepalive_and_transient_retry",
        summary: "Add resilient HTTP keepAlive/connect timeouts and transient socket/gateway retry with jittered backoff",
        hunks: &[
            PatchHunk {
                id: "bundle_http_keepalive",
                rel_hint: None,
                old: r#",bodyTimeout:normalizedTimeoutMs,connect:{autoSelectFamilyAttemptTimeout:DEFAULT_AUTO_SELECT_FAMILY_ATTEMPT_TIMEOUT_MS},headersTimeout:normalizedTimeoutMs,cli"#,
                new: r#",bodyTimeout:normalizedTimeoutMs,connectTimeout:20000,keepAliveTimeout:25000,keepAliveMaxTimeout:55000,connect:{autoSelectFamilyAttemptTimeout:DEFAULT_AUTO_SELECT_FAMILY_ATTEMPT_TIMEOUT_MS,timeout:20000,keepAlive:!0,keepAliveInitialDelay:15000},headersTimeout:normalizedTimeoutMs,cli"#,
            },
            PatchHunk {
                id: "bundle_transient_retry_jitter",
                rel_hint: None,
                old: r#","subscription_sharing_user_unavailable"]),DEFAULT_MAX_AGENT_RETRY_DELAY_MS=6e4;function retryDelayMs(policy,attempt){let delay=policy.baseDelayMs*2**Math.max(0,attempt-1),safeDelay=Number.isSafeInteger(delay)?delay:Number.MAX_SAFE_INTEGER;return Math.min(safeDelay,policy.maxAgentDelayMs??DEFAULT_MAX_AGENT_RETRY_DELAY_MS)}"#,
                new: r#","subscription_sharing_user_unavailable","ECONNRESET","ETIMEDOUT","EPIPE","ECONNABORTED","UND_ERR_SOCKET","UND_ERR_BODY_TIMEOUT","UND_ERR_HEADERS_TIMEOUT","premature close","bad gateway","gateway timeout"]),DEFAULT_MAX_AGENT_RETRY_DELAY_MS=6e4;function retryDelayMs(policy,attempt){let delay=policy.baseDelayMs*2**Math.max(0,attempt-1),safeDelay=Number.isSafeInteger(delay)?delay:Number.MAX_SAFE_INTEGER;let capped=Math.min(safeDelay,policy.maxAgentDelayMs??DEFAULT_MAX_AGENT_RETRY_DELAY_MS);return capped<=0?0:Math.max(100,Math.round(capped*(.85+Math.random()*.3)))}"#,
            },
        ],
    },
    PatchSpec {
        id: "virtual_model_cache_warmer",
        summary: "Allow virtual models (e.g. deepseek/flash-auto) to match CacheWarmer physical sessions",
        hunks: &[PatchHunk {
            id: "bundle_virtual_cache_warmer",
            rel_hint: None,
            old: r#"Model.provider===requestModel.provider&&currentModel.id===requestModel.id&&messages.length<=currentMessages.lengt"#,
            new: r#"Model.provider===requestModel.provider&&(currentModel.id===requestModel.id||currentModel.api==="pi-virtual")&&messages.length<=currentMessages.lengt"#,
        }],
    },
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchReportItem {
    pub id: String,
    pub summary: String,
    pub status: String,
    pub applied_hunks: usize,
    pub total_hunks: usize,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiPatchReport {
    pub detected: bool,
    pub pi_dir: String,
    pub pi_version: String,
    pub all_applied: bool,
    pub applied_count: usize,
    pub total_count: usize,
    pub patches: Vec<PatchReportItem>,
}

fn read_pi_version(pi_dir: &Path) -> String {
    let pkg = pi_dir.join("package.json");
    fs::read_to_string(&pkg)
        .ok()
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        .and_then(|v| v.get("version").and_then(Value::as_str).map(String::from))
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn find_pi_package_dir(explicit: Option<&str>) -> Option<PathBuf> {
    if let Some(p) = explicit.map(str::trim).filter(|s| !s.is_empty()) {
        let cand = PathBuf::from(p);
        if cand.join("package.json").is_file() {
            return Some(cand);
        }
        return None;
    }
    if let Ok(env_p) = std::env::var("PI_PACKAGE_DIR") {
        let cand = PathBuf::from(env_p.trim());
        if cand.join("package.json").is_file() {
            return Some(cand);
        }
    }
    let mut cands: Vec<PathBuf> = Vec::new();
    if let Ok(appdata) = std::env::var("APPDATA") {
        cands.push(
            PathBuf::from(appdata)
                .join("npm")
                .join("node_modules")
                .join("@earendil-works")
                .join("pi-coding-agent"),
        );
    }
    if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
        let h = PathBuf::from(home);
        cands.push(
            h.join(".npm-global")
                .join("lib")
                .join("node_modules")
                .join("@earendil-works")
                .join("pi-coding-agent"),
        );
        cands.push(
            h.join("AppData")
                .join("Roaming")
                .join("npm")
                .join("node_modules")
                .join("@earendil-works")
                .join("pi-coding-agent"),
        );
    }
    cands.push(PathBuf::from(
        "/usr/local/lib/node_modules/@earendil-works/pi-coding-agent",
    ));
    cands.push(PathBuf::from(
        "/opt/homebrew/lib/node_modules/@earendil-works/pi-coding-agent",
    ));

    // Also probe relative to `pi` / `pi.cmd` on PATH
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            if dir.join("pi").is_file()
                || dir.join("pi.cmd").is_file()
                || dir.join("pi.ps1").is_file()
            {
                cands.push(
                    dir.join("node_modules")
                        .join("@earendil-works")
                        .join("pi-coding-agent"),
                );
                if let Some(parent) = dir.parent() {
                    cands.push(
                        parent
                            .join("lib")
                            .join("node_modules")
                            .join("@earendil-works")
                            .join("pi-coding-agent"),
                    );
                }
            }
        }
    }

    cands.into_iter().find(|c| c.join("package.json").is_file())
}

fn collect_bundle_chunks(pi_dir: &Path) -> Vec<PathBuf> {
    let chunks_dir = pi_dir.join("dist").join("bundle").join("chunks");
    let mut out = Vec::new();
    let Ok(rd) = fs::read_dir(&chunks_dir) else {
        return out;
    };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("js") {
            out.push(p);
        }
    }
    out.sort();
    out
}

fn normalize_lf(s: &str) -> String {
    s.replace("\r\n", "\n")
}

fn backup_root(pi_version: &str) -> PathBuf {
    let ap_home = std::env::var("AP_HOME")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let h = std::env::var_os("HOME")
                .or_else(|| std::env::var_os("USERPROFILE"))
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."));
            h.join(".ap")
        });
    ap_home.join("pi-backups").join(pi_version)
}

fn ensure_backup(pi_dir: &Path, pi_version: &str, file_path: &Path) -> Result<(), String> {
    let rel = file_path.strip_prefix(pi_dir).unwrap_or(file_path);
    let dst = backup_root(pi_version).join(rel);
    if dst.exists() {
        return Ok(());
    }
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::copy(file_path, &dst).map_err(|e| e.to_string())?;
    Ok(())
}

fn resolve_hunk_file(
    pi_dir: &Path,
    chunks: &[PathBuf],
    hunk: &PatchHunk,
) -> Option<(PathBuf, bool, bool)> {
    // Returns (file_path, has_new, has_old)
    if let Some(rel) = hunk.rel_hint {
        let p = pi_dir.join(rel);
        if !p.is_file() {
            return None;
        }
        let raw = fs::read_to_string(&p).ok()?;
        let norm = normalize_lf(&raw);
        let has_new = norm.contains(hunk.new);
        let has_old = !has_new && norm.contains(hunk.old);
        return Some((p, has_new, has_old));
    }
    for ch in chunks {
        let Ok(raw) = fs::read_to_string(ch) else {
            continue;
        };
        if raw.contains(hunk.new) {
            return Some((ch.clone(), true, false));
        }
        if raw.contains(hunk.old) {
            return Some((ch.clone(), false, true));
        }
    }
    None
}

pub fn inspect_pi_patches(pi_dir_opt: Option<&str>) -> PiPatchReport {
    let Some(pi_dir) = find_pi_package_dir(pi_dir_opt) else {
        return PiPatchReport {
            detected: false,
            pi_dir: String::new(),
            pi_version: String::new(),
            all_applied: false,
            applied_count: 0,
            total_count: PATCH_SPECS.len(),
            patches: Vec::new(),
        };
    };
    let pi_version = read_pi_version(&pi_dir);
    let chunks = collect_bundle_chunks(&pi_dir);
    let mut patches = Vec::new();
    let mut applied_count = 0usize;

    for spec in PATCH_SPECS {
        let mut applied_hunks = 0usize;
        let mut patchable_hunks = 0usize;
        let mut files = Vec::new();
        for h in spec.hunks {
            if let Some((fp, has_new, has_old)) = resolve_hunk_file(&pi_dir, &chunks, h) {
                let rel = fp
                    .strip_prefix(&pi_dir)
                    .unwrap_or(&fp)
                    .to_string_lossy()
                    .replace('\\', "/");
                if !files.contains(&rel) {
                    files.push(rel);
                }
                if has_new {
                    applied_hunks += 1;
                } else if has_old {
                    patchable_hunks += 1;
                }
            }
        }
        let total = spec.hunks.len();
        let status = if applied_hunks == total {
            applied_count += 1;
            "applied"
        } else if applied_hunks > 0 && patchable_hunks > 0 {
            "partial"
        } else if patchable_hunks == total {
            "unpatched"
        } else if applied_hunks > 0 {
            // Optional unbundled files may be absent in minimal installs
            applied_count += 1;
            "applied"
        } else if patchable_hunks > 0 {
            "unpatched"
        } else {
            "incompatible"
        };
        patches.push(PatchReportItem {
            id: spec.id.to_string(),
            summary: spec.summary.to_string(),
            status: status.to_string(),
            applied_hunks,
            total_hunks: total,
            files,
        });
    }

    PiPatchReport {
        detected: true,
        pi_dir: pi_dir.to_string_lossy().to_string(),
        pi_version,
        all_applied: applied_count == PATCH_SPECS.len(),
        applied_count,
        total_count: PATCH_SPECS.len(),
        patches,
    }
}

pub fn apply_pi_patches(pi_dir_opt: Option<&str>, dry_run: bool) -> Result<Value, String> {
    let Some(pi_dir) = find_pi_package_dir(pi_dir_opt) else {
        return Err(
            "Could not locate @earendil-works/pi-coding-agent installation (pass --pi-dir <path>)."
                .to_string(),
        );
    };
    let pi_version = read_pi_version(&pi_dir);
    let chunks = collect_bundle_chunks(&pi_dir);
    let mut newly_applied = Vec::new();
    let mut already_applied = Vec::new();
    let mut skipped = Vec::new();

    for spec in PATCH_SPECS {
        for h in spec.hunks {
            let Some((fp, has_new, has_old)) = resolve_hunk_file(&pi_dir, &chunks, h) else {
                skipped.push(format!("{}:{} (target not found)", spec.id, h.id));
                continue;
            };
            if has_new {
                already_applied.push(format!("{}:{}", spec.id, h.id));
                continue;
            }
            if !has_old {
                skipped.push(format!("{}:{} (pattern mismatch)", spec.id, h.id));
                continue;
            }
            if !dry_run {
                ensure_backup(&pi_dir, &pi_version, &fp)?;
                let raw = fs::read_to_string(&fp).map_err(|e| e.to_string())?;
                let uses_crlf = raw.contains("\r\n");
                let norm = normalize_lf(&raw);
                let count = norm.matches(h.old).count();
                if count != 1 {
                    return Err(format!(
                        "Refusing ambiguous patch {}:{} in {} (expected 1 match, found {})",
                        spec.id,
                        h.id,
                        fp.display(),
                        count
                    ));
                }
                let updated_lf = norm.replacen(h.old, h.new, 1);
                let final_txt = if uses_crlf {
                    updated_lf.replace('\n', "\r\n")
                } else {
                    updated_lf
                };
                fs::write(&fp, final_txt).map_err(|e| e.to_string())?;
            }
            newly_applied.push(format!("{}:{}", spec.id, h.id));
        }
    }

    let report = inspect_pi_patches(Some(&pi_dir.to_string_lossy()));
    Ok(json!({
        "ok": true,
        "dry_run": dry_run,
        "pi_dir": report.pi_dir,
        "pi_version": report.pi_version,
        "backup_dir": backup_root(&pi_version).to_string_lossy(),
        "newly_applied": newly_applied,
        "already_applied": already_applied,
        "skipped": skipped,
        "report": report,
    }))
}

pub fn restore_pi_patches(pi_dir_opt: Option<&str>) -> Result<Value, String> {
    let Some(pi_dir) = find_pi_package_dir(pi_dir_opt) else {
        return Err(
            "Could not locate @earendil-works/pi-coding-agent installation (pass --pi-dir <path>)."
                .to_string(),
        );
    };
    let pi_version = read_pi_version(&pi_dir);
    let b_root = backup_root(&pi_version);
    if !b_root.is_dir() {
        return Err(format!(
            "No Pi backup directory found at {} (nothing to restore).",
            b_root.display()
        ));
    }
    let mut restored = Vec::new();
    let mut stack = vec![b_root.clone()];
    while let Some(dir) = stack.pop() {
        let Ok(rd) = fs::read_dir(&dir) else { continue };
        for entry in rd.flatten() {
            let p = entry.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.is_file() {
                let rel = p.strip_prefix(&b_root).unwrap_or(&p);
                let dst = pi_dir.join(rel);
                if let Some(parent) = dst.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                fs::copy(&p, &dst).map_err(|e| e.to_string())?;
                restored.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    Ok(json!({
        "ok": true,
        "pi_dir": pi_dir.to_string_lossy(),
        "pi_version": pi_version,
        "restored_count": restored.len(),
        "restored_files": restored,
    }))
}

pub fn run_cli(args: &[String]) -> i32 {
    let cli = match PiCli::try_parse_from(args) {
        Ok(c) => c,
        Err(e) => {
            let code = if e.use_stderr() { 2 } else { 0 };
            let _ = e.print();
            return code;
        }
    };
    match cli.subcmd {
        PiSubCmd::Status { pi_dir, json } => {
            let rep = inspect_pi_patches(pi_dir.as_deref());
            if json {
                println!("{}", serde_json::to_string_pretty(&rep).unwrap_or_default());
            } else if !rep.detected {
                println!(
                    "[ap pi] @earendil-works/pi-coding-agent not detected (pass --pi-dir <path>)."
                );
            } else {
                println!(
                    "[ap pi] Pi {} ({}) — {}/{} runtime patches applied",
                    rep.pi_version, rep.pi_dir, rep.applied_count, rep.total_count
                );
                for p in &rep.patches {
                    println!(
                        "  - {:<36} [{}] ({}/{} hunks) — {}",
                        p.id, p.status, p.applied_hunks, p.total_hunks, p.summary
                    );
                }
            }
            if rep.detected {
                0
            } else {
                1
            }
        }
        PiSubCmd::Patch {
            pi_dir,
            dry_run,
            json,
        } => match apply_pi_patches(pi_dir.as_deref(), dry_run) {
            Ok(v) => {
                if json {
                    println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
                } else {
                    let newly = v["newly_applied"].as_array().map(|a| a.len()).unwrap_or(0);
                    let already = v["already_applied"]
                        .as_array()
                        .map(|a| a.len())
                        .unwrap_or(0);
                    let mode = if dry_run { " (dry-run)" } else { "" };
                    println!(
                        "[ap pi]{mode} Pi {} — newly applied {newly} hunk(s), already applied {already} hunk(s) (backups: {})",
                        v["pi_version"].as_str().unwrap_or("?"),
                        v["backup_dir"].as_str().unwrap_or("?")
                    );
                }
                0
            }
            Err(e) => {
                if json {
                    println!("{}", json!({"ok": false, "error": e}));
                } else {
                    eprintln!("[ap pi] error: {e}");
                }
                1
            }
        },
        PiSubCmd::Restore { pi_dir, json } => match restore_pi_patches(pi_dir.as_deref()) {
            Ok(v) => {
                if json {
                    println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
                } else {
                    println!(
                        "[ap pi] restored {} file(s) for Pi {}.",
                        v["restored_count"].as_u64().unwrap_or(0),
                        v["pi_version"].as_str().unwrap_or("?")
                    );
                }
                0
            }
            Err(e) => {
                if json {
                    println!("{}", json!({"ok": false, "error": e}));
                } else {
                    eprintln!("[ap pi] error: {e}");
                }
                1
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patches_and_restores_mock_pi_installation_idempotently() {
        let base = std::env::temp_dir().join(format!("ap_pi_patch_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        let pi_dir = base.join("pi-coding-agent");
        let chunks = pi_dir.join("dist").join("bundle").join("chunks");
        let core_ext = pi_dir.join("dist").join("core").join("extensions");
        let ap_home = base.join("ap_home");
        fs::create_dir_all(&chunks).unwrap();
        fs::create_dir_all(&core_ext).unwrap();
        fs::write(
            pi_dir.join("package.json"),
            r#"{"name":"@earendil-works/pi-coding-agent","version":"1.5.2-test"}"#,
        )
        .unwrap();

        // Populate minimal mock bundle + unbundled core files matching all 5 patch specs
        fs::write(
            chunks.join("chunk-main.js"),
            format!(
                "{}\n{}\n{}\n{}\n",
                PATCH_SPECS[0].hunks[0].old,
                PATCH_SPECS[0].hunks[1].old,
                PATCH_SPECS[3].hunks[0].old,
                PATCH_SPECS[4].hunks[0].old,
            ),
        )
        .unwrap();
        fs::write(chunks.join("chunk-env.js"), PATCH_SPECS[1].hunks[0].old).unwrap();
        fs::write(
            chunks.join("chunk-cm.js"),
            format!(
                "{}\n{}\n{}\n",
                PATCH_SPECS[2].hunks[0].old,
                PATCH_SPECS[2].hunks[1].old,
                PATCH_SPECS[2].hunks[2].old,
            ),
        )
        .unwrap();
        fs::write(chunks.join("chunk-retry.js"), PATCH_SPECS[3].hunks[1].old).unwrap();
        fs::write(core_ext.join("runner.js"), PATCH_SPECS[0].hunks[2].old).unwrap();
        fs::write(
            pi_dir.join("dist").join("core").join("agent-session.js"),
            PATCH_SPECS[0].hunks[3].old,
        )
        .unwrap();
        fs::write(core_ext.join("types.d.ts"), PATCH_SPECS[0].hunks[4].old).unwrap();

        std::env::set_var("AP_HOME", &ap_home);
        let pi_str = pi_dir.to_string_lossy().to_string();

        let before = inspect_pi_patches(Some(&pi_str));
        assert!(before.detected);
        assert_eq!(before.applied_count, 0);

        let applied = apply_pi_patches(Some(&pi_str), false).unwrap();
        assert_eq!(applied["report"]["all_applied"], true);
        assert_eq!(applied["report"]["applied_count"], 5);

        // Second run must be 100% idempotent (0 newly applied)
        let second = apply_pi_patches(Some(&pi_str), false).unwrap();
        assert_eq!(second["newly_applied"].as_array().unwrap().len(), 0);

        // Restore must bring everything back to 0 applied
        let restored = restore_pi_patches(Some(&pi_str)).unwrap();
        assert!(restored["restored_count"].as_u64().unwrap() >= 6);
        let after_restore = inspect_pi_patches(Some(&pi_str));
        assert_eq!(after_restore.applied_count, 0);

        let _ = fs::remove_dir_all(&base);
    }
}
