use crate::ports::list_pid_ports_map;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::process::Command;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcEntry {
    pub pid: u32,
    pub ppid: u32,
    pub name: String,
    pub exe: String,
    pub cmdline: String,
    pub ports: Vec<u16>,
}

#[cfg(windows)]
fn collect_processes() -> Vec<ProcEntry> {
    let script = "[Console]::OutputEncoding=[System.Text.Encoding]::UTF8; Get-CimInstance Win32_Process | Select-Object ProcessId,ParentProcessId,Name,ExecutablePath,CommandLine | ConvertTo-Json -Compress";
    let shell = ["pwsh.exe", "powershell.exe"].into_iter().find_map(|bin| {
        Command::new(bin)
            .args([
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                script,
            ])
            .output()
            .ok()
            .filter(|o| o.status.success() && !o.stdout.is_empty())
    });
    let Some(out) = shell else {
        return Vec::new();
    };
    let raw = String::from_utf8_lossy(&out.stdout);
    let parsed: Value = match serde_json::from_str(raw.trim()) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let items: Vec<&Value> = match &parsed {
        Value::Array(arr) => arr.iter().collect(),
        Value::Object(_) => vec![&parsed],
        _ => Vec::new(),
    };
    let mut procs = Vec::with_capacity(items.len());
    for it in items {
        let pid = it.get("ProcessId").and_then(Value::as_u64).unwrap_or(0) as u32;
        if pid == 0 {
            continue;
        }
        let ppid = it
            .get("ParentProcessId")
            .and_then(Value::as_u64)
            .unwrap_or(0) as u32;
        let name = it
            .get("Name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let exe = it
            .get("ExecutablePath")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let cmdline = it
            .get("CommandLine")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        procs.push(ProcEntry {
            pid,
            ppid,
            name,
            exe,
            cmdline,
            ports: Vec::new(),
        });
    }
    procs
}

#[cfg(not(windows))]
fn collect_processes() -> Vec<ProcEntry> {
    let Ok(out) = Command::new("ps")
        .args(["-axo", "pid=,ppid=,comm=,args="])
        .output()
    else {
        return Vec::new();
    };
    let mut procs = Vec::new();
    for ln in String::from_utf8_lossy(&out.stdout).lines() {
        let t = ln.trim();
        if t.is_empty() {
            continue;
        }
        let mut parts = t.split_whitespace();
        let pid = parts
            .next()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        let ppid = parts
            .next()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        let comm = parts.next().unwrap_or("").to_string();
        if pid == 0 || comm.is_empty() {
            continue;
        }
        // Extract remaining command line after comm
        let cmdline = t
            .splitn(4, char::is_whitespace)
            .nth(3)
            .unwrap_or(&comm)
            .trim()
            .to_string();
        let exe = cmdline
            .split_whitespace()
            .next()
            .unwrap_or(&comm)
            .to_string();
        procs.push(ProcEntry {
            pid,
            ppid,
            name: comm,
            exe,
            cmdline,
            ports: Vec::new(),
        });
    }
    let _ = Value::Null; // keep serde_json::Value imported uniformly across targets
    procs
}

/// Fast single-PID command line lookup for enriching `ap probe ports --port <P>`.
pub fn lookup_pid_cmdline(pid: u32) -> Option<String> {
    if pid == 0 {
        return None;
    }
    #[cfg(windows)]
    {
        let script = format!(
            "[Console]::OutputEncoding=[System.Text.Encoding]::UTF8; (Get-CimInstance Win32_Process -Filter 'ProcessId = {pid}').CommandLine"
        );
        let out = ["pwsh.exe", "powershell.exe"].into_iter().find_map(|bin| {
            Command::new(bin)
                .args([
                    "-NoLogo",
                    "-NoProfile",
                    "-NonInteractive",
                    "-Command",
                    &script,
                ])
                .output()
                .ok()
                .filter(|o| o.status.success())
        })?;
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    }
    #[cfg(not(windows))]
    {
        let out = Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "args="])
            .output()
            .ok()?;
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    }
}

pub fn run_ps_cli(
    filter: Option<&str>,
    pid_filter: Option<u32>,
    include_ports: bool,
    limit: usize,
    as_json: bool,
) -> i32 {
    let t0 = Instant::now();
    let mut procs = collect_processes();
    let ports_map: BTreeMap<u32, Vec<u16>> = if include_ports {
        list_pid_ports_map()
    } else {
        BTreeMap::new()
    };

    let filt_low = filter
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_ascii_lowercase());

    procs.retain(|p| {
        if let Some(target_pid) = pid_filter {
            if p.pid != target_pid && p.ppid != target_pid {
                return false;
            }
        }
        if let Some(ref f) = filt_low {
            let hit = p.name.to_ascii_lowercase().contains(f)
                || p.exe.to_ascii_lowercase().contains(f)
                || p.cmdline.to_ascii_lowercase().contains(f);
            if !hit {
                return false;
            }
        }
        true
    });

    for p in &mut procs {
        if let Some(ps) = ports_map.get(&p.pid) {
            p.ports = ps.clone();
        }
    }

    // Prioritize processes with open ports or longer command lines when unfiltered
    procs.sort_by(|a, b| {
        b.ports
            .len()
            .cmp(&a.ports.len())
            .then_with(|| a.pid.cmp(&b.pid))
    });
    let total = procs.len();
    let lim = limit.clamp(1, 200);
    procs.truncate(lim);
    let elapsed_ms = (t0.elapsed().as_secs_f64() * 1000.0).round();

    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "ok": true,
                "elapsed_ms": elapsed_ms,
                "total_matched": total,
                "shown": procs.len(),
                "processes": procs,
            }))
            .unwrap_or_default()
        );
    } else {
        println!(
            "[probe ps] {} shown / {} matched process(es) in {}ms:",
            procs.len(),
            total,
            elapsed_ms
        );
        for p in &procs {
            let ptag = if p.ports.is_empty() {
                String::new()
            } else {
                let list: Vec<String> = p.ports.iter().map(|n| format!(":{n}")).collect();
                format!(" [ports={}]", list.join(","))
            };
            let cmd_preview: String = if p.cmdline.is_empty() {
                p.exe.clone()
            } else if p.cmdline.chars().count() > 160 {
                format!("{}...", p.cmdline.chars().take(160).collect::<String>())
            } else {
                p.cmdline.clone()
            };
            println!(
                "  - PID={:<6} PPID={:<6} {}{} — {}",
                p.pid, p.ppid, p.name, ptag, cmd_preview
            );
        }
    }
    0
}
