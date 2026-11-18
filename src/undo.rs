use crate::audit::default_home_dir;
use base64::Engine;
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileBackupEntry {
    pub path: String,
    pub existed: bool,
    pub orig_lines: usize,
    pub orig_b64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoTx {
    pub id: String,
    pub ts: u64,
    pub files: Vec<FileBackupEntry>,
}

#[derive(Parser, Debug)]
#[command(
    name = "undo",
    about = "Atomically restore files modified by the most recent agent edit (works even outside Git!)"
)]
struct UndoCli {
    /// List saved undo checkpoints without restoring
    #[arg(long)]
    list: bool,
    /// Emit structured JSON output
    #[arg(long)]
    json: bool,
}

pub fn undo_dir() -> PathBuf {
    if let Ok(ap_home) = std::env::var("AP_HOME") {
        if !ap_home.trim().is_empty() {
            return PathBuf::from(ap_home).join("undo");
        }
    }
    default_home_dir().join(".agent-primitives").join("undo")
}

fn sorted_tx_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(rd) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = rd
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("json"))
        .collect();
    files.sort();
    files
}

pub fn record_pre_edit_snapshot(cwd: &Path, rel_or_abs_paths: &[String]) -> Option<UndoTx> {
    if rel_or_abs_paths.is_empty() {
        return None;
    }
    let cwd_canon = fs::canonicalize(cwd).unwrap_or_else(|_| cwd.to_path_buf());
    let mut entries = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for raw in rel_or_abs_paths {
        let p = Path::new(raw);
        let full = if p.is_absolute() {
            p.to_path_buf()
        } else {
            cwd_canon.join(p)
        };
        let key = full.to_string_lossy().replace('\\', "/");
        if !seen.insert(key.clone()) {
            continue;
        }
        if full.is_file() {
            let bytes = fs::read(&full).unwrap_or_default();
            let orig_lines = bytes.iter().filter(|&&b| b == b'\n').count() + 1;
            let orig_b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
            entries.push(FileBackupEntry {
                path: key,
                existed: true,
                orig_lines,
                orig_b64,
            });
        } else {
            entries.push(FileBackupEntry {
                path: key,
                existed: false,
                orig_lines: 0,
                orig_b64: String::new(),
            });
        }
    }

    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let ts = (now_ms / 1000) as u64;
    let id = format!("{now_ms:015}_{}", std::process::id());
    let tx = UndoTx {
        id: id.clone(),
        ts,
        files: entries,
    };

    let dir = undo_dir();
    if fs::create_dir_all(&dir).is_ok() {
        let dst = dir.join(format!("tx_{id}.json"));
        if let Ok(txt) = serde_json::to_string(&tx) {
            let _ = fs::write(dst, txt);
        }
        let all = sorted_tx_files(&dir);
        if all.len() > 25 {
            for old in &all[..all.len() - 25] {
                let _ = fs::remove_file(old);
            }
        }
    }
    Some(tx)
}

pub fn check_truncation_warnings(tx: &UndoTx) -> Vec<String> {
    let mut warns = Vec::new();
    for f in &tx.files {
        if !f.existed || f.orig_lines < 60 {
            continue;
        }
        if let Ok(after) = fs::read(&f.path) {
            let new_lines = after.iter().filter(|&&b| b == b'\n').count() + 1;
            if new_lines * 4 < f.orig_lines {
                warns.push(format!(
                    "[TRUNCATION WARNING] `{}` shrunk from {} -> {} lines (>75% loss). Run `ap undo` immediately if unintended!",
                    f.path, f.orig_lines, new_lines
                ));
            }
        }
    }
    warns
}

pub fn run_cli(args: &[String]) -> i32 {
    let cli = match UndoCli::try_parse_from(args) {
        Ok(c) => c,
        Err(e) => {
            let code = if e.use_stderr() { 2 } else { 0 };
            let _ = e.print();
            return code;
        }
    };
    let dir = undo_dir();
    let files = sorted_tx_files(&dir);
    if cli.list {
        let mut items = Vec::new();
        for f in files.iter().rev().take(10) {
            if let Ok(txt) = fs::read_to_string(f) {
                if let Ok(tx) = serde_json::from_str::<UndoTx>(&txt) {
                    let paths: Vec<String> = tx.files.iter().map(|e| e.path.clone()).collect();
                    items.push(json!({"id": tx.id, "ts": tx.ts, "files": paths}));
                }
            }
        }
        if cli.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&items).unwrap_or_default()
            );
        } else if items.is_empty() {
            println!("[ap undo] No undo snapshots recorded yet.");
        } else {
            println!("[ap undo] Saved non-Git safety checkpoints (newest first):");
            for it in items {
                println!(
                    "  - {} -> {:?}",
                    it["id"].as_str().unwrap_or(""),
                    it["files"]
                );
            }
        }
        return 0;
    }

    let Some(latest_p) = files.last() else {
        if cli.json {
            println!("{}", json!({"ok": false, "error": "no_snapshots"}));
        } else {
            eprintln!("[ap undo] Nothing to undo (no snapshots found).");
        }
        return 1;
    };

    let Ok(raw) = fs::read_to_string(latest_p) else {
        return 1;
    };
    let Ok(tx) = serde_json::from_str::<UndoTx>(&raw) else {
        return 1;
    };

    let mut restored = Vec::new();
    for entry in &tx.files {
        let p = Path::new(&entry.path);
        if entry.existed {
            if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(&entry.orig_b64) {
                if let Some(parent) = p.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                if fs::write(p, bytes).is_ok() {
                    restored.push(format!("restored {}", entry.path));
                }
            }
        } else if p.exists() && fs::remove_file(p).is_ok() {
            restored.push(format!("removed newly-created {}", entry.path));
        }
    }
    let _ = fs::remove_file(latest_p);

    if cli.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({"ok": true, "id": tx.id, "actions": restored}))
                .unwrap_or_default()
        );
    } else {
        println!(
            "[ap undo] Restored checkpoint {} ({} file(s)):\n  - {}",
            tx.id,
            restored.len(),
            restored.join("\n  - ")
        );
    }
    0
}
