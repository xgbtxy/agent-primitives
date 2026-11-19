use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
#[cfg(windows)]
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::process::Command;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortEntry {
    pub port: u16,
    pub bind: String,
    pub pid: u32,
    pub process: String,
    pub cmdline: Option<String>,
    pub cdp_browser: Option<String>,
}

fn probe_cdp_version(port: u16) -> Option<String> {
    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().ok()?;
    let mut s = TcpStream::connect_timeout(&addr, Duration::from_millis(70)).ok()?;
    let _ = s.set_read_timeout(Some(Duration::from_millis(120)));
    let _ = s.set_write_timeout(Some(Duration::from_millis(120)));
    let req = format!(
        "GET /json/version HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
    );
    s.write_all(req.as_bytes()).ok()?;
    let mut buf = Vec::new();
    let _ = s.take(4096).read_to_end(&mut buf);
    let pos = buf.windows(4).position(|w| w == b"\r\n\r\n")?;
    let v: Value = serde_json::from_slice(&buf[pos + 4..]).ok()?;
    if v.get("webSocketDebuggerUrl").is_some() || v.get("Browser").is_some() {
        let br = v
            .get("Browser")
            .and_then(Value::as_str)
            .unwrap_or("CDP")
            .to_string();
        return Some(br);
    }
    None
}

#[cfg(windows)]
fn list_raw_listening_ports() -> Vec<(u16, String, u32, String)> {
    let mut pid_names: HashMap<u32, String> = HashMap::new();
    if let Ok(tl) = Command::new("tasklist")
        .args(["/FO", "CSV", "/NH"])
        .output()
    {
        for ln in String::from_utf8_lossy(&tl.stdout).lines() {
            let parts: Vec<&str> = ln.split("\",\"").collect();
            if parts.len() >= 2 {
                let name = parts[0].trim_matches('"').to_string();
                if let Ok(pid) = parts[1].trim_matches('"').parse::<u32>() {
                    pid_names.insert(pid, name);
                }
            }
        }
    }
    let mut out = Vec::new();
    if let Ok(ns) = Command::new("netstat").args(["-ano", "-p", "TCP"]).output() {
        for ln in String::from_utf8_lossy(&ns.stdout).lines() {
            let toks: Vec<&str> = ln.split_whitespace().collect();
            if toks.len() >= 5
                && toks[0].eq_ignore_ascii_case("TCP")
                && toks[3].eq_ignore_ascii_case("LISTENING")
            {
                if let Some((bind, p_str)) = toks[1].rsplit_once(':') {
                    if let (Ok(port), Ok(pid)) = (p_str.parse::<u16>(), toks[4].parse::<u32>()) {
                        let proc_name = pid_names
                            .get(&pid)
                            .cloned()
                            .unwrap_or_else(|| "unknown".to_string());
                        out.push((port, bind.to_string(), pid, proc_name));
                    }
                }
            }
        }
    }
    out
}

#[cfg(not(windows))]
fn list_raw_listening_ports() -> Vec<(u16, String, u32, String)> {
    let mut out = Vec::new();
    if let Ok(lo) = Command::new("lsof")
        .args(["-nP", "-iTCP", "-sTCP:LISTEN", "-F", "pcn"])
        .output()
    {
        if lo.status.success() {
            let mut cur_pid = 0u32;
            let mut cur_cmd = String::new();
            for ln in String::from_utf8_lossy(&lo.stdout).lines() {
                if let Some(p) = ln.strip_prefix('p') {
                    cur_pid = p.parse().unwrap_or(0);
                } else if let Some(c) = ln.strip_prefix('c') {
                    cur_cmd = c.to_string();
                } else if let Some(n) = ln.strip_prefix('n') {
                    if let Some((bind, p_str)) = n.rsplit_once(':') {
                        if let Ok(port) = p_str.parse::<u16>() {
                            out.push((port, bind.to_string(), cur_pid, cur_cmd.clone()));
                        }
                    }
                }
            }
            if !out.is_empty() {
                return out;
            }
        }
    }
    if let Ok(ss) = Command::new("ss").args(["-ltnp"]).output() {
        for ln in String::from_utf8_lossy(&ss.stdout).lines().skip(1) {
            let toks: Vec<&str> = ln.split_whitespace().collect();
            if toks.len() >= 4 {
                if let Some((bind, p_str)) = toks[3].rsplit_once(':') {
                    if let Ok(port) = p_str.parse::<u16>() {
                        let rest = toks.get(5).copied().unwrap_or("");
                        let pid = rest
                            .split("pid=")
                            .nth(1)
                            .and_then(|s| s.split(',').next())
                            .and_then(|s| s.parse::<u32>().ok())
                            .unwrap_or(0);
                        let pname = rest.split('"').nth(1).unwrap_or("unknown").to_string();
                        out.push((port, bind.to_string(), pid, pname));
                    }
                }
            }
        }
    }
    out
}

/// Return a map of `PID -> sorted unique listening TCP ports` for process enrichment.
pub fn list_pid_ports_map() -> BTreeMap<u32, Vec<u16>> {
    let mut map: BTreeMap<u32, Vec<u16>> = BTreeMap::new();
    for (port, _bind, pid, _proc) in list_raw_listening_ports() {
        if pid == 0 {
            continue;
        }
        let ent = map.entry(pid).or_default();
        if !ent.contains(&port) {
            ent.push(port);
        }
    }
    for v in map.values_mut() {
        v.sort_unstable();
    }
    map
}

pub fn run_ports_radar(port_filter: Option<u16>, probe_cdp: bool, as_json: bool) -> i32 {
    let t0 = Instant::now();
    let raw = list_raw_listening_ports();
    let mut dedup: BTreeMap<(u16, u32), PortEntry> = BTreeMap::new();
    for (port, bind, pid, process) in raw {
        if let Some(pf) = port_filter {
            if port != pf {
                continue;
            }
        }
        dedup.entry((port, pid)).or_insert(PortEntry {
            port,
            bind,
            pid,
            process,
            cmdline: None,
            cdp_browser: None,
        });
    }

    let mut entries: Vec<PortEntry> = dedup.into_values().collect();
    if port_filter.is_some() {
        for e in &mut entries {
            if e.pid > 0 {
                e.cmdline = crate::ps::lookup_pid_cmdline(e.pid);
            }
        }
    }
    if probe_cdp {
        let mut probed = 0usize;
        for e in &mut entries {
            let is_local = e.bind == "127.0.0.1"
                || e.bind == "0.0.0.0"
                || e.bind == "[::1]"
                || e.bind == "[::]"
                || e.bind == "*";
            if is_local && e.port >= 1024 && (port_filter.is_some() || probed < 30) {
                probed += 1;
                e.cdp_browser = probe_cdp_version(e.port);
            }
        }
    }

    let elapsed_ms = (t0.elapsed().as_secs_f64() * 1000.0).round();
    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "ok": true,
                "elapsed_ms": elapsed_ms,
                "count": entries.len(),
                "ports": entries,
            }))
            .unwrap_or_default()
        );
    } else {
        println!(
            "[probe ports] {} listening TCP port(s) in {}ms:",
            entries.len(),
            elapsed_ms
        );
        for e in &entries {
            let cdp_tag = e
                .cdp_browser
                .as_deref()
                .map(|b| format!("  [CDP READY: {b}]"))
                .unwrap_or_default();
            let cmd_tag = e
                .cmdline
                .as_deref()
                .map(|c| format!("  cmd=`{}`", c.chars().take(160).collect::<String>()))
                .unwrap_or_default();
            println!(
                "  - :{:<5} ({:<15}) PID={:<6} {}{}{}",
                e.port, e.bind, e.pid, e.process, cdp_tag, cmd_tag
            );
        }
    }
    0
}
