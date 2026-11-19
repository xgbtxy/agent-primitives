pub mod coord;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FactEntry {
    pub text: String,
    #[serde(default)]
    pub r#ref: String,
    pub at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeadEndEntry {
    pub text: String,
    #[serde(default)]
    pub reason: String,
    pub at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspaceSnapshot {
    pub cwd: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dirty_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dirty_files: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recent_files: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatonState {
    #[serde(default)]
    pub goal: String,
    #[serde(default)]
    pub constraints: Vec<String>,
    #[serde(default)]
    pub facts: Vec<FactEntry>,
    #[serde(default)]
    pub deadends: Vec<DeadEndEntry>,
    #[serde(default)]
    pub next_step: String,
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    pub workspace: WorkspaceSnapshot,
}

impl Default for BatonState {
    fn default() -> Self {
        Self {
            goal: String::new(),
            constraints: Vec::new(),
            facts: Vec::new(),
            deadends: Vec::new(),
            next_step: String::new(),
            updated_at: now_iso_utc(),
            workspace: WorkspaceSnapshot::default(),
        }
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "baton",
    about = "Compaction-proof state & cross-agent handoff capsule (<500 tokens)"
)]
struct Cli {
    #[command(subcommand)]
    subcmd: SubCmd,
}

#[derive(Subcommand, Debug)]
enum SubCmd {
    /// Set goal, constraints, or immediate next step
    Set {
        #[arg(long)]
        goal: Option<String>,
        #[arg(long = "constraint")]
        constraints: Vec<String>,
        #[arg(long = "next")]
        next_step: Option<String>,
        #[arg(long)]
        reset: bool,
    },
    /// Append a verified ground-truth fact
    Fact {
        text: String,
        #[arg(long = "ref", default_value = "")]
        reference: String,
    },
    /// Record a falsified approach so future turns never repeat it
    Deadend {
        text: String,
        #[arg(long, default_value = "")]
        reason: String,
    },
    /// Snapshot workspace & write history (ideal for PreCompact/Stop hooks)
    Checkpoint {
        #[arg(long, default_value = "")]
        note: String,
    },
    /// Emit compact <500-token handoff packet
    Resume {
        #[arg(long)]
        json: bool,
    },
    /// Reset active baton capsule (archives prior snapshot)
    Clear,
    /// Atomically claim a shared multi-agent resource with TTL expiry (exit 2 if held by another agent)
    Claim {
        resource: String,
        #[arg(long)]
        owner: String,
        #[arg(long, default_value_t = 900)]
        ttl: u64,
        #[arg(long, default_value = "")]
        note: String,
        #[arg(long)]
        json: bool,
    },
    /// Release a claimed shared resource
    Release {
        resource: String,
        #[arg(long)]
        owner: Option<String>,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        json: bool,
    },
    /// List active & expired multi-agent resource claims
    Claims {
        #[arg(long)]
        json: bool,
    },
    /// Sub-15ms hybrid BM25 + CJK bigram semantic near-duplicate finder across ledgers, markdown & JSON indexes
    Dedup {
        query: String,
        files: Vec<PathBuf>,
        #[arg(short = 'k', long, default_value_t = 8)]
        top_k: usize,
        #[arg(long)]
        json: bool,
    },
}

fn truncate_chars(s: &str, max_chars: usize) -> String {
    s.trim().chars().take(max_chars).collect()
}

pub fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = if m <= 2 { y + 1 } else { y };
    (year, m, d)
}

pub fn now_iso_utc() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let days = secs.div_euclid(86400);
    let rem = secs.rem_euclid(86400);
    let (y, m, d) = civil_from_days(days);
    let hh = rem / 3600;
    let mm = (rem % 3600) / 60;
    let ss = rem % 60;
    format!("{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}Z")
}

fn now_compact_stamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_millis())
        .unwrap_or(0);
    let days = (secs as i64).div_euclid(86400);
    let rem = (secs as i64).rem_euclid(86400);
    let (y, m, d) = civil_from_days(days);
    let hh = rem / 3600;
    let mm = (rem % 3600) / 60;
    let ss = rem % 60;
    format!("{y:04}{m:02}{d:02}-{hh:02}{mm:02}{ss:02}-{millis:03}")
}

pub fn baton_dir(cwd: &Path) -> PathBuf {
    let base = match std::env::var("BATON_HOME") {
        Ok(v) if !v.trim().is_empty() => PathBuf::from(v),
        _ => cwd.join(".baton"),
    };
    let _ = fs::create_dir_all(base.join("history"));
    let gi = base.join(".gitignore");
    if !gi.exists() {
        let _ = fs::write(&gi, "*\n");
    }
    base
}

pub fn load_state(base: &Path) -> BatonState {
    let p = base.join("state.json");
    if !p.exists() {
        return BatonState::default();
    }
    fs::read_to_string(&p)
        .ok()
        .and_then(|s| serde_json::from_str::<BatonState>(&s).ok())
        .unwrap_or_default()
}

pub fn probe_workspace(cwd: &Path) -> WorkspaceSnapshot {
    let mut snap = WorkspaceSnapshot {
        cwd: cwd.to_string_lossy().to_string(),
        ..Default::default()
    };
    if let Ok(out) = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(cwd)
        .output()
    {
        if out.status.success() {
            let branch = String::from_utf8_lossy(&out.stdout).trim().to_string();
            snap.git_branch = Some(branch);
            if let Ok(st_out) = Command::new("git")
                .args(["status", "--short"])
                .current_dir(cwd)
                .output()
            {
                let dirty: Vec<String> = String::from_utf8_lossy(&st_out.stdout)
                    .lines()
                    .map(str::trim)
                    .filter(|l| !l.is_empty())
                    .map(String::from)
                    .collect();
                snap.dirty_count = Some(dirty.len());
                snap.dirty_files = Some(dirty.into_iter().take(10).collect());
                return snap;
            }
        }
    }

    let mut files: Vec<(u64, String)> = Vec::new();
    if let Ok(rd) = fs::read_dir(cwd) {
        for entry in rd.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.is_file() && !name.starts_with('.') {
                let mtime = entry
                    .metadata()
                    .and_then(|m| m.modified())
                    .ok()
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                files.push((mtime, name));
            }
        }
    }
    files.sort_by_key(|a| std::cmp::Reverse(a.0));
    snap.recent_files = Some(files.into_iter().take(8).map(|(_, n)| n).collect());
    snap
}

pub fn render_markdown(state: &BatonState) -> String {
    let mut lines = vec![format!("# BATON HANDOFF ({})", state.updated_at)];
    if !state.goal.is_empty() {
        lines.push(format!("**Goal**: {}", state.goal));
    }
    if !state.constraints.is_empty() {
        let start = state.constraints.len().saturating_sub(6);
        lines.push(format!(
            "**Constraints**: {}",
            state.constraints[start..].join(" | ")
        ));
    }
    if !state.next_step.is_empty() {
        lines.push(format!("**Next Immediate Step**: {}", state.next_step));
    }
    if let Some(branch) = &state.workspace.git_branch {
        let cnt = state.workspace.dirty_count.unwrap_or(0);
        let dirty_preview = state
            .workspace
            .dirty_files
            .as_ref()
            .filter(|d| !d.is_empty())
            .map(|d| {
                let take_n = d.len().min(5);
                format!(": {}", d[..take_n].join(", "))
            })
            .unwrap_or_default();
        lines.push(format!(
            "**Git**: `{branch}` ({cnt} modified{dirty_preview})"
        ));
    } else if let Some(recent) = &state.workspace.recent_files {
        if !recent.is_empty() {
            let take_n = recent.len().min(6);
            lines.push(format!("**Recent Files**: {}", recent[..take_n].join(", ")));
        }
    }

    if !state.facts.is_empty() {
        lines.push("\n## Verified Facts".to_string());
        let start = state.facts.len().saturating_sub(8);
        for f in &state.facts[start..] {
            let r = if f.r#ref.is_empty() {
                String::new()
            } else {
                format!(" (`{}`)", f.r#ref)
            };
            lines.push(format!("- {}{r}", f.text));
        }
    }

    if !state.deadends.is_empty() {
        lines.push("\n## Dead-Ends (DO NOT RETRY)".to_string());
        let start = state.deadends.len().saturating_sub(8);
        for d in &state.deadends[start..] {
            let why = if d.reason.is_empty() {
                String::new()
            } else {
                format!(" — {}", d.reason)
            };
            lines.push(format!("- [x] {}{why}", d.text));
        }
    }

    format!("{}\n", lines.join("\n").trim())
}

pub fn save_state(
    base: &Path,
    cwd: &Path,
    state: &mut BatonState,
    snapshot_history: bool,
) -> String {
    state.updated_at = now_iso_utc();
    state.workspace = probe_workspace(cwd);
    if let Ok(js) = serde_json::to_string_pretty(state) {
        let _ = fs::write(base.join("state.json"), &js);
        if snapshot_history {
            let hist_dir = base.join("history");
            let _ = fs::create_dir_all(&hist_dir);
            let _ = fs::write(
                hist_dir.join(format!("baton-{}.json", now_compact_stamp())),
                &js,
            );
            if let Ok(rd) = fs::read_dir(&hist_dir) {
                let mut snaps: Vec<PathBuf> = rd
                    .flatten()
                    .map(|e| e.path())
                    .filter(|p| {
                        p.file_name()
                            .and_then(|n| n.to_str())
                            .map(|s| s.starts_with("baton-") && s.ends_with(".json"))
                            .unwrap_or(false)
                    })
                    .collect();
                snaps.sort();
                if snaps.len() > 20 {
                    for old in &snaps[..snaps.len() - 20] {
                        let _ = fs::remove_file(old);
                    }
                }
            }
        }
    }
    let md = render_markdown(state);
    let _ = fs::write(base.join("HANDOFF.md"), &md);
    md
}

pub fn run_cli(args: &[String]) -> i32 {
    let cli = match Cli::try_parse_from(args) {
        Ok(c) => c,
        Err(e) => {
            let code = if e.use_stderr() { 2 } else { 0 };
            let _ = e.print();
            return code;
        }
    };
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let base = baton_dir(&cwd);

    match cli.subcmd {
        SubCmd::Clear => {
            let mut empty = BatonState::default();
            save_state(&base, &cwd, &mut empty, true);
            println!("[baton] active state cleared (previous snapshot archived).");
            0
        }
        SubCmd::Set {
            goal,
            constraints,
            next_step,
            reset,
        } => {
            let mut state = if reset {
                BatonState::default()
            } else {
                load_state(&base)
            };
            if let Some(g) = goal {
                state.goal = truncate_chars(&g, 240);
            }
            for c in constraints {
                let clean = truncate_chars(&c, 160);
                if !clean.is_empty() && !state.constraints.contains(&clean) {
                    state.constraints.push(clean);
                }
            }
            if let Some(n) = next_step {
                state.next_step = truncate_chars(&n, 240);
            }
            let md = save_state(&base, &cwd, &mut state, false);
            let tok = md.len().div_ceil(4).max(1);
            println!("[baton] updated (~{tok} tokens)");
            0
        }
        SubCmd::Fact { text, reference } => {
            let mut state = load_state(&base);
            let clean_text = truncate_chars(&text, 220);
            if !state.facts.iter().any(|f| f.text == clean_text) {
                state.facts.push(FactEntry {
                    text: clean_text,
                    r#ref: truncate_chars(&reference, 120),
                    at: now_iso_utc(),
                });
                if state.facts.len() > 12 {
                    let drain_n = state.facts.len() - 12;
                    state.facts.drain(0..drain_n);
                }
            }
            let md = save_state(&base, &cwd, &mut state, false);
            let tok = md.len().div_ceil(4).max(1);
            println!(
                "[baton] recorded fact ({} active, ~{tok} tok)",
                state.facts.len()
            );
            0
        }
        SubCmd::Deadend { text, reason } => {
            let mut state = load_state(&base);
            let clean_text = truncate_chars(&text, 220);
            if !state.deadends.iter().any(|d| d.text == clean_text) {
                state.deadends.push(DeadEndEntry {
                    text: clean_text,
                    reason: truncate_chars(&reason, 160),
                    at: now_iso_utc(),
                });
                if state.deadends.len() > 12 {
                    let drain_n = state.deadends.len() - 12;
                    state.deadends.drain(0..drain_n);
                }
            }
            let md = save_state(&base, &cwd, &mut state, false);
            let tok = md.len().div_ceil(4).max(1);
            println!(
                "[baton] recorded dead-end ({} active, ~{tok} tok)",
                state.deadends.len()
            );
            0
        }
        SubCmd::Checkpoint { note } => {
            let mut state = load_state(&base);
            if !note.trim().is_empty() {
                state.next_step = truncate_chars(&note, 240);
            }
            let md = save_state(&base, &cwd, &mut state, true);
            let tok = md.len().div_ceil(4).max(1);
            println!("[baton] checkpoint saved (~{tok} tokens)");
            0
        }
        SubCmd::Resume { json } => {
            let mut state = load_state(&base);
            state.workspace = probe_workspace(&cwd);
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&state).unwrap_or_else(|_| "{}".to_string())
                );
            } else {
                print!("{}", render_markdown(&state));
            }
            0
        }
        SubCmd::Claim {
            resource,
            owner,
            ttl,
            note,
            json,
        } => coord::run_claim(&resource, &owner, ttl, &note, json),
        SubCmd::Release {
            resource,
            owner,
            force,
            json,
        } => coord::run_release(&resource, owner.as_deref(), force, json),
        SubCmd::Claims { json } => coord::run_claims_list(json),
        SubCmd::Dedup {
            query,
            files,
            top_k,
            json,
        } => coord::run_dedup(&query, &files, top_k, json),
    }
}
