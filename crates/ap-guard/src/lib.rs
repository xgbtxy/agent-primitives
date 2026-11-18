use clap::{Parser, Subcommand};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{IsTerminal, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

pub const EXIT_BLOCK: i32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub sig: String,
    pub cmd: String,
    pub exit: i32,
    pub at: u64,
}

#[derive(Parser, Debug)]
#[command(name = "guard", about = "Sub-2ms exit-2 truth gate, loop breaker & blast-radius shield")]
struct Cli {
    #[command(subcommand)]
    subcmd: SubCmd,
}

#[derive(Subcommand, Debug)]
enum SubCmd {
    /// Inspect command/payload or Hook stdin JSON for unsafe actions & secrets
    Shield {
        #[arg(long, default_value = "")]
        text: String,
    },
    /// Execute command with shield + repetition-loop breaker
    Run {
        #[arg(long, default_value_t = 3)]
        max_repeats: usize,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        cmd: Vec<String>,
    },
    /// Verify completion invariants; exit 2 on stderr if unverified
    StopGate {
        #[arg(long = "check")]
        checks: Vec<String>,
    },
    /// Clear loop ledger
    Reset,
}

struct SafetyPatterns {
    destructive: Vec<(&'static str, Regex)>,
    secrets: Vec<(&'static str, Regex)>,
    conflict_re: Regex,
    hex_ts_re: Regex,
}

fn safety_patterns() -> &'static SafetyPatterns {
    static PATS: OnceLock<SafetyPatterns> = OnceLock::new();
    PATS.get_or_init(|| {
        let destructive = vec![
            (
                "recursive root/home delete",
                Regex::new(r#"rm\s+-[a-zA-Z]*r[a-zA-Z]*f[a-zA-Z]*\s+(?:/|~|\$HOME|\.\.)(?:[\s"';&|]|$)"#).unwrap(),
            ),
            (
                "forced git history overwrite",
                Regex::new(r"\bgit\s+push\b.*(?:\s--force\b|\s-f\b)").unwrap(),
            ),
            (
                "hard working-tree wipe",
                Regex::new(r"\bgit\s+(?:reset\s+--hard|clean\s+-[a-zA-Z]*f)").unwrap(),
            ),
            (
                "destructive SQL DDL",
                Regex::new(r"(?i)\b(?:DROP\s+TABLE|DROP\s+DATABASE|TRUNCATE\s+TABLE)\b").unwrap(),
            ),
            (
                "unbounded outbound mail/webhook blast",
                Regex::new(r"(?i)\b(?:sendmail\s+-t|mass[_-]?send|blast[_-]?all)\b").unwrap(),
            ),
        ];
        let secrets = vec![
            (
                "OpenAI/Anthropic/xAI API key",
                Regex::new(r"\b(?:sk-(?:proj-|ant-)?[A-Za-z0-9_-]{20,}|xai-[A-Za-z0-9_-]{20,})\b").unwrap(),
            ),
            (
                "GitHub Personal/App token",
                Regex::new(r"\b(?:ghp|gho|ghu|ghs|github_pat)_[A-Za-z0-9_]{24,}\b").unwrap(),
            ),
            (
                "AWS Access Key ID",
                Regex::new(r"\bAKIA[0-9A-Z]{16}\b").unwrap(),
            ),
            (
                "PEM Private Key block",
                Regex::new(r"-----BEGIN (?:RSA |EC |OPENSSH |)?PRIVATE KEY-----").unwrap(),
            ),
        ];
        let conflict_re = Regex::new(r"(?m)^(?:<{7}\s|={7}$|>{7}\s)").unwrap();
        let hex_ts_re = Regex::new(r"\b0x[0-9a-fA-F]+\b|\b\d{4}-\d{2}-\d{2}[T\s]\S+").unwrap();
        SafetyPatterns {
            destructive,
            secrets,
            conflict_re,
            hex_ts_re,
        }
    })
}

pub fn guard_dir(cwd: &Path) -> PathBuf {
    let base = match std::env::var("GUARD_HOME") {
        Ok(v) if !v.trim().is_empty() => PathBuf::from(v),
        _ => cwd.join(".guard"),
    };
    let _ = fs::create_dir_all(&base);
    let gi = base.join(".gitignore");
    if !gi.exists() {
        let _ = fs::write(&gi, "*\n");
    }
    base
}

pub fn inspect_text_safety(text: &str) -> Vec<String> {
    let pats = safety_patterns();
    let mut findings = Vec::new();
    for (label, pat) in &pats.destructive {
        if pat.is_match(text) {
            findings.push(format!("Destructive action blocked: {label}"));
        }
    }
    for (label, pat) in &pats.secrets {
        if pat.is_match(text) {
            findings.push(format!("Credential/secret leak blocked: {label}"));
        }
    }
    findings
}

fn normalize_error_sig(cmd_str: &str, output: &str, exit_code: i32) -> String {
    let pats = safety_patterns();
    let norm_cmd = cmd_str.split_whitespace().collect::<Vec<_>>().join(" ");
    let err_lines: Vec<String> = output
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(|l| pats.hex_ts_re.replace_all(l, "<HEX>").to_string())
        .collect();
    let start = err_lines.len().saturating_sub(6);
    let tail_joined = err_lines[start..].join("\n");
    let tail_sig: String = tail_joined.chars().take(400).collect();
    let raw = format!("{exit_code}::{norm_cmd}::{tail_sig}");
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    let hex = format!("{:x}", hasher.finalize());
    hex[..12].to_string()
}

fn load_ledger(base: &Path) -> Vec<LedgerEntry> {
    let p = base.join("loop_ledger.json");
    fs::read_to_string(&p)
        .ok()
        .and_then(|s| serde_json::from_str::<Vec<LedgerEntry>>(&s).ok())
        .unwrap_or_default()
}

fn save_ledger(base: &Path, entries: &[LedgerEntry]) {
    let start = entries.len().saturating_sub(40);
    if let Ok(js) = serde_json::to_string_pretty(&entries[start..]) {
        let _ = fs::write(base.join("loop_ledger.json"), js);
    }
}

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn record_attempt(base: &Path, cmd_str: &str, exit_code: i32, output: &str, max_repeats: usize) -> (bool, usize, String) {
    let mut entries = load_ledger(base);
    let cmd_trunc: String = cmd_str.chars().take(160).collect();
    if exit_code == 0 {
        entries.push(LedgerEntry {
            sig: "ok".to_string(),
            cmd: cmd_trunc,
            exit: 0,
            at: now_epoch(),
        });
        save_ledger(base, &entries);
        return (false, 0, "ok".to_string());
    }
    let sig = normalize_error_sig(cmd_str, output, exit_code);
    entries.push(LedgerEntry {
        sig: sig.clone(),
        cmd: cmd_trunc,
        exit: exit_code,
        at: now_epoch(),
    });
    save_ledger(base, &entries);
    let win_start = entries.len().saturating_sub(8);
    let repeat_count = entries[win_start..].iter().filter(|e| e.sig == sig).count();
    (repeat_count >= max_repeats, repeat_count, sig)
}

fn scan_conflict_and_secret_markers(root: &Path) -> Vec<String> {
    let pats = safety_patterns();
    let secrets = &pats.secrets;
    let conflict_re = &pats.conflict_re;
    let mut bad_files = Vec::new();
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(out) = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(root)
        .output()
    {
        if out.status.success() {
            for ln in String::from_utf8_lossy(&out.stdout).lines() {
                if ln.len() > 3 {
                    let rel = ln[3..].trim().trim_matches('"');
                    let rel = rel.rsplit(" -> ").next().unwrap_or(rel);
                    candidates.push(root.join(rel));
                }
            }
        }
    }
    if candidates.is_empty() {
        fn walk_shallow(dir: &Path, out: &mut Vec<PathBuf>) {
            if out.len() >= 150 {
                return;
            }
            if let Ok(rd) = fs::read_dir(dir) {
                for entry in rd.flatten() {
                    let p = entry.path();
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with('.') || matches!(name.as_str(), "target" | "node_modules" | "dist" | "build" | "__pycache__") {
                        continue;
                    }
                    if p.is_dir() {
                        walk_shallow(&p, out);
                    } else if p.is_file() {
                        if let Ok(m) = entry.metadata() {
                            if m.len() < 512_000 {
                                out.push(p);
                            }
                        }
                    }
                }
            }
        }
        walk_shallow(root, &mut candidates);
    }

    for fp in candidates {
        if let Ok(meta) = fp.metadata() {
            if meta.is_file() && meta.len() < 1_000_000 {
                if let Ok(bytes) = fs::read(&fp) {
                    let txt = String::from_utf8_lossy(&bytes);
                    let rel_disp = fp
                        .strip_prefix(root)
                        .unwrap_or(&fp)
                        .to_string_lossy()
                        .replace('\\', "/");
                    if conflict_re.is_match(&txt) {
                        bad_files.push(rel_disp.clone());
                    }
                    for (_, spat) in secrets {
                        if spat.is_match(&txt) {
                            bad_files.push(format!("{rel_disp} (contains exposed secret pattern)"));
                            break;
                        }
                    }
                }
            }
        }
    }
    bad_files
}

fn spawn_command_list(cmd: &[String], cwd: &Path) -> Result<std::process::Output, std::io::Error> {
    match Command::new(&cmd[0]).args(&cmd[1..]).current_dir(cwd).output() {
        Ok(o) => Ok(o),
        #[cfg(windows)]
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Command::new("cmd")
            .arg("/C")
            .args(cmd)
            .current_dir(cwd)
            .output(),
        Err(e) => Err(e),
    }
}

fn spawn_shell_string(shell_cmd: &str, cwd: &Path) -> Result<std::process::Output, std::io::Error> {
    #[cfg(windows)]
    {
        Command::new("cmd").args(["/C", shell_cmd]).current_dir(cwd).output()
    }
    #[cfg(not(windows))]
    {
        Command::new("sh").args(["-c", shell_cmd]).current_dir(cwd).output()
    }
}

pub fn run_cli(args: &[String]) -> i32 {
    let cli = match Cli::try_parse_from(args) {
        Ok(c) => c,
        Err(e) => {
            let code = if e.use_stderr() { EXIT_BLOCK } else { 0 };
            let _ = e.print();
            return code;
        }
    };
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    match cli.subcmd {
        SubCmd::Reset => {
            let base = guard_dir(&cwd);
            save_ledger(&base, &[]);
            println!("[guard] loop ledger reset.");
            0
        }
        SubCmd::Shield { text } => {
            if !text.is_empty() {
                let findings = inspect_text_safety(&text);
                if !findings.is_empty() {
                    for f in findings {
                        eprintln!("[GUARD BLOCKED | exit 2] {f}");
                    }
                    eprintln!("Action halted. Request explicit human confirmation or sanitize secrets first.");
                    return EXIT_BLOCK;
                }
                println!("[guard] shield passed.");
                return 0;
            }

            let mut payload = String::new();
            if !std::io::stdin().is_terminal() {
                let _ = std::io::stdin().read_to_string(&mut payload);
            }
            let trimmed = payload.trim_start();
            if trimmed.starts_with('{') {
                if let Ok(Value::Object(map)) = serde_json::from_str::<Value>(trimmed) {
                    // Antigravity camelCase hook contract (`hookEventName` / `toolCall`)
                    if map.contains_key("hookEventName") || map.contains_key("toolCall") {
                        let target_val = map.get("toolCall").cloned().unwrap_or(Value::Object(map));
                        let target_text = serde_json::to_string(&target_val).unwrap_or_default();
                        let findings = inspect_text_safety(&target_text);
                        if !findings.is_empty() {
                            println!(
                                "{}",
                                json!({
                                    "decision": "deny",
                                    "reason": format!("[GUARD BLOCKED] {}", findings.join("; "))
                                })
                            );
                        } else {
                            println!("{}", json!({ "decision": "allow" }));
                        }
                        return 0;
                    }
                    // Claude Code / Codex snake_case hook contract (`hook_event_name` / `tool_input`)
                    if map.contains_key("hook_event_name") || map.contains_key("tool_input") {
                        let target_val = map.get("tool_input").cloned().unwrap_or(Value::Object(map));
                        let target_text = serde_json::to_string(&target_val).unwrap_or_default();
                        let findings = inspect_text_safety(&target_text);
                        if !findings.is_empty() {
                            for f in findings {
                                eprintln!("[GUARD BLOCKED | exit 2] {f}");
                            }
                            eprintln!("Action halted. Request explicit human confirmation or sanitize secrets first.");
                            return EXIT_BLOCK;
                        }
                        return 0;
                    }
                }
            }

            let findings = inspect_text_safety(&payload);
            if !findings.is_empty() {
                for f in findings {
                    eprintln!("[GUARD BLOCKED | exit 2] {f}");
                }
                eprintln!("Action halted. Request explicit human confirmation or sanitize secrets first.");
                return EXIT_BLOCK;
            }
            println!("[guard] shield passed.");
            0
        }
        SubCmd::Run { max_repeats, cmd } => {
            let clean_cmd: Vec<String> = if cmd.first().map(|s| s.as_str()) == Some("--") {
                cmd[1..].to_vec()
            } else {
                cmd
            };
            if clean_cmd.is_empty() {
                eprintln!("[guard] usage: ap guard run -- <command...>");
                return EXIT_BLOCK;
            }
            let cmd_str = clean_cmd.join(" ");
            let findings = inspect_text_safety(&cmd_str);
            if !findings.is_empty() {
                for f in findings {
                    eprintln!("[GUARD BLOCKED | exit 2] {f}");
                }
                return EXIT_BLOCK;
            }

            let base = guard_dir(&cwd);
            let prior = load_ledger(&base);
            let win_start = prior.len().saturating_sub(6);
            let cmd_key: String = cmd_str.chars().take(160).collect();
            let prior_fails = prior[win_start..]
                .iter()
                .filter(|e| e.cmd == cmd_key && e.exit != 0)
                .count();
            if prior_fails >= max_repeats {
                eprintln!(
                    "[GUARD LOOP BREAKER | exit 2] Command '{cmd_str}' already failed {prior_fails}x in a row.\n\
                     STOP retrying the identical command. Diagnose root cause or record a dead-end in `baton` first."
                );
                return EXIT_BLOCK;
            }

            let out = match spawn_command_list(&clean_cmd, &cwd) {
                Ok(o) => o,
                Err(err) => {
                    let msg = format!("Failed to spawn `{cmd_str}`: {err}\n");
                    eprint!("{msg}");
                    let (tripped, count, sig) = record_attempt(&base, &cmd_str, 127, &msg, max_repeats);
                    if tripped {
                        eprintln!(
                            "\n[GUARD LOOP BREAKER | exit 2] Failure signature {sig} repeated {count}x.\n\
                             Do NOT re-run the same command unchanged. Formulate a new hypothesis first."
                        );
                        return EXIT_BLOCK;
                    }
                    return 127;
                }
            };

            let mut combined = String::from_utf8_lossy(&out.stdout).to_string();
            let err_s = String::from_utf8_lossy(&out.stderr);
            if !err_s.is_empty() {
                if !combined.is_empty() && !combined.ends_with('\n') {
                    combined.push('\n');
                }
                combined.push_str(&err_s);
            }
            if !combined.is_empty() {
                print!("{combined}");
            }
            let code = out.status.code().unwrap_or(1);
            let (tripped, count, sig) = record_attempt(&base, &cmd_str, code, &combined, max_repeats);
            if tripped {
                eprintln!(
                    "\n[GUARD LOOP BREAKER | exit 2] Failure signature {sig} repeated {count}x.\n\
                     Do NOT re-run the same command unchanged. Formulate a new hypothesis first."
                );
                return EXIT_BLOCK;
            }
            code
        }
        SubCmd::StopGate { checks } => {
            let mut raw_in = String::new();
            if !std::io::stdin().is_terminal() {
                let _ = std::io::stdin().read_to_string(&mut raw_in);
            }
            let hook_obj = if raw_in.trim_start().starts_with('{') {
                serde_json::from_str::<Value>(raw_in.trim_start())
                    .ok()
                    .and_then(|v| v.as_object().cloned())
            } else {
                None
            };

            let is_agy = hook_obj
                .as_ref()
                .map(|m| m.contains_key("hookEventName") || m.contains_key("terminationReason"))
                .unwrap_or(false);
            let is_claude = hook_obj
                .as_ref()
                .map(|m| m.contains_key("hook_event_name") || m.contains_key("stop_hook_active"))
                .unwrap_or(false);

            if is_agy {
                if let Some(m) = &hook_obj {
                    let cont = m.get("continuationActive").and_then(Value::as_bool).unwrap_or(false);
                    let reason = m
                        .get("terminationReason")
                        .and_then(Value::as_str)
                        .unwrap_or("NO_TOOL_CALL");
                    if cont || reason != "NO_TOOL_CALL" {
                        println!("{}", json!({ "decision": "stop" }));
                        return 0;
                    }
                }
            }
            if is_claude {
                if let Some(m) = &hook_obj {
                    if m.get("stop_hook_active").and_then(Value::as_bool).unwrap_or(false) {
                        return 0;
                    }
                }
            }

            let scan_root = hook_obj
                .as_ref()
                .and_then(|m| m.get("cwd"))
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .map(PathBuf::from)
                .unwrap_or(cwd);

            let mut issues = scan_conflict_and_secret_markers(&scan_root);
            for chk in checks {
                match spawn_shell_string(&chk, &scan_root) {
                    Ok(o) if !o.status.success() => {
                        let code = o.status.code().unwrap_or(1);
                        let combined = format!(
                            "{}{}",
                            String::from_utf8_lossy(&o.stdout),
                            String::from_utf8_lossy(&o.stderr)
                        );
                        let lines: Vec<&str> = combined.lines().collect();
                        let start = lines.len().saturating_sub(12);
                        let tail = lines[start..].join("\n");
                        issues.push(format!("Check failed (`{chk}`, exit {code}):\n{tail}"));
                    }
                    Err(e) => {
                        issues.push(format!("Check failed (`{chk}`): {e}"));
                    }
                    _ => {}
                }
            }

            if is_agy {
                if !issues.is_empty() {
                    println!(
                        "{}",
                        json!({
                            "decision": "continue",
                            "reason": format!("[GUARD STOP-GATE] Cannot finish yet: {}", issues.join("; "))
                        })
                    );
                } else {
                    println!("{}", json!({ "decision": "stop" }));
                }
                return 0;
            }

            if !issues.is_empty() {
                eprintln!("[GUARD STOP-GATE BLOCKED | exit 2] Cannot mark task complete:");
                for item in issues {
                    eprintln!("- {item}");
                }
                return EXIT_BLOCK;
            }
            if !is_claude {
                println!("[guard] stop-gate verified clean.");
            }
            0
        }
    }
}
