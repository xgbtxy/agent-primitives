use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

fn unescape_payload(s: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            match chars[i + 1] {
                'r' => {
                    out.push(b'\r');
                    i += 2;
                    continue;
                }
                'n' => {
                    out.push(b'\n');
                    i += 2;
                    continue;
                }
                't' => {
                    out.push(b'\t');
                    i += 2;
                    continue;
                }
                '0' => {
                    out.push(0);
                    i += 2;
                    continue;
                }
                '\\' => {
                    out.push(b'\\');
                    i += 2;
                    continue;
                }
                _ => {}
            }
        }
        let mut buf = [0u8; 4];
        out.extend_from_slice(chars[i].encode_utf8(&mut buf).as_bytes());
        i += 1;
    }
    out
}

fn decode_hex_str(s: &str) -> Result<Vec<u8>, String> {
    let clean: String = s
        .chars()
        .filter(|c| !c.is_whitespace() && *c != ':')
        .collect();
    if clean.len() % 2 != 0 {
        return Err("hex string must have even length".to_string());
    }
    let mut out = Vec::with_capacity(clean.len() / 2);
    for idx in (0..clean.len()).step_by(2) {
        let b = u8::from_str_radix(&clean[idx..idx + 2], 16)
            .map_err(|e| format!("invalid hex at {idx}: {e}"))?;
        out.push(b);
    }
    Ok(out)
}

fn mini_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .take(256)
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Deadline-bounded raw TCP banner/send/recv probe (replaces `nc`/`telnet` & hanging Python sockets).
pub fn run_tcp_cli(
    addr_str: &str,
    send_txt: Option<&str>,
    send_hex: Option<&str>,
    wait_ms: u64,
    timeout_ms: u64,
    as_json: bool,
) -> i32 {
    let t0 = Instant::now();
    let Ok(mut addrs) = addr_str.to_socket_addrs() else {
        eprintln!("[probe tcp] Failed to resolve `{addr_str}`");
        return 2;
    };
    let Some(sock_addr) = addrs.next() else {
        eprintln!("[probe tcp] No socket address resolved for `{addr_str}`");
        return 2;
    };
    let conn_dur = Duration::from_millis(timeout_ms.clamp(100, 30_000));
    let mut stream = match TcpStream::connect_timeout(&sock_addr, conn_dur) {
        Ok(s) => s,
        Err(e) => {
            let elapsed_ms = (t0.elapsed().as_secs_f64() * 1000.0).round();
            if as_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "ok": false,
                        "addr": addr_str,
                        "connected": false,
                        "elapsed_ms": elapsed_ms,
                        "error": e.to_string(),
                    }))
                    .unwrap_or_default()
                );
            } else {
                eprintln!("[probe tcp] FAIL `{addr_str}` ({elapsed_ms}ms): {e}");
            }
            return 1;
        }
    };

    let _ = stream.set_write_timeout(Some(Duration::from_millis(3000)));
    if let Some(hx) = send_hex.filter(|s| !s.trim().is_empty()) {
        match decode_hex_str(hx) {
            Ok(bytes) => {
                if let Err(e) = stream.write_all(&bytes) {
                    eprintln!("[probe tcp] write error: {e}");
                    return 1;
                }
            }
            Err(e) => {
                eprintln!("[probe tcp] {e}");
                return 2;
            }
        }
    } else if let Some(txt) = send_txt {
        let bytes = unescape_payload(txt);
        if let Err(e) = stream.write_all(&bytes) {
            eprintln!("[probe tcp] write error: {e}");
            return 1;
        }
    }

    let bounded_wait = wait_ms.clamp(50, 10_000);
    let deadline = Instant::now() + Duration::from_millis(bounded_wait);
    let _ = stream.set_read_timeout(Some(Duration::from_millis(60)));
    let mut received = Vec::new();
    let mut buf = [0u8; 2048];
    while Instant::now() < deadline && received.len() < 16_384 {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                received.extend_from_slice(&buf[..n]);
                if received.len() >= 16_384 {
                    break;
                }
            }
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                if !received.is_empty() {
                    break;
                }
                continue;
            }
            Err(_) => break,
        }
    }

    let elapsed_ms = (t0.elapsed().as_secs_f64() * 1000.0).round();
    let text_preview = String::from_utf8_lossy(&received)
        .chars()
        .take(1600)
        .collect::<String>();
    let hex_preview = mini_hex(&received);

    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "ok": true,
                "addr": addr_str,
                "connected": true,
                "elapsed_ms": elapsed_ms,
                "bytes_read": received.len(),
                "text": text_preview,
                "hex": hex_preview,
            }))
            .unwrap_or_default()
        );
    } else {
        println!(
            "[probe tcp] CONNECTED `{addr_str}` ({elapsed_ms}ms | {} B received)",
            received.len()
        );
        if !text_preview.trim().is_empty() {
            println!("  text: {}", text_preview.trim());
        }
        if !hex_preview.is_empty() {
            println!("  hex : {hex_preview}");
        }
    }
    0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedHttpReq {
    pub method: String,
    pub path: String,
    pub headers: Vec<String>,
    pub body: String,
}

fn handle_one_http(
    mut stream: TcpStream,
    reply_status: u16,
    reply_type: &str,
    reply_body: &str,
) -> Option<CapturedHttpReq> {
    let _ = stream.set_read_timeout(Some(Duration::from_millis(800)));
    let mut raw = Vec::new();
    let mut chunk = [0u8; 4096];
    let mut hdr_end: Option<usize> = None;
    let mut content_len: usize = 0;

    while raw.len() < 65_536 {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                raw.extend_from_slice(&chunk[..n]);
                if hdr_end.is_none() {
                    if let Some(pos) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
                        hdr_end = Some(pos + 4);
                        let htxt = String::from_utf8_lossy(&raw[..pos]);
                        for ln in htxt.lines().skip(1) {
                            if let Some((k, v)) = ln.split_once(':') {
                                if k.trim().eq_ignore_ascii_case("content-length") {
                                    content_len = v.trim().parse::<usize>().unwrap_or(0);
                                }
                            }
                        }
                    }
                }
                if let Some(he) = hdr_end {
                    if raw.len() >= he + content_len {
                        break;
                    }
                }
            }
            Err(_) => break,
        }
    }
    if raw.is_empty() {
        return None;
    }

    let (hdr_part, body_part) = if let Some(he) = hdr_end {
        (
            String::from_utf8_lossy(&raw[..he.saturating_sub(4)]).to_string(),
            String::from_utf8_lossy(&raw[he..]).to_string(),
        )
    } else {
        (String::from_utf8_lossy(&raw).to_string(), String::new())
    };
    let mut lines = hdr_part.lines();
    let first = lines.next().unwrap_or("");
    let mut fp = first.split_whitespace();
    let method = fp.next().unwrap_or("GET").to_string();
    let path = fp.next().unwrap_or("/").to_string();
    let headers: Vec<String> = lines
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .take(32)
        .collect();

    let reason = match reply_status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        302 => "Found",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        _ => "OK",
    };
    let resp = format!(
        "HTTP/1.1 {reply_status} {reason}\r\nContent-Type: {reply_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{reply_body}",
        reply_body.len()
    );
    let _ = stream.write_all(resp.as_bytes());
    let _ = stream.flush();

    Some(CapturedHttpReq {
        method,
        path,
        headers,
        body: body_part.chars().take(4000).collect(),
    })
}

/// Bounded ephemeral HTTP callback/webhook capture server (auto-exits after `max_reqs` or `wait_ms` — zero hanging server leaks!).
#[allow(clippy::too_many_arguments)]
pub fn run_listen_cli(
    port: u16,
    wait_ms: u64,
    max_reqs: usize,
    reply_status: u16,
    reply_type: &str,
    reply_body: &str,
    run_cmd: Option<&str>,
    as_json: bool,
) -> i32 {
    let t0 = Instant::now();
    let listener = match TcpListener::bind(("127.0.0.1", port)) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[probe listen] Failed to bind 127.0.0.1:{port}: {e}");
            return 1;
        }
    };
    let bound_port = listener.local_addr().map(|a| a.port()).unwrap_or(port);
    let _ = listener.set_nonblocking(true);

    let bg_handle = run_cmd.filter(|s| !s.trim().is_empty()).map(|raw| {
        let expanded = raw.replace("{{PORT}}", &bound_port.to_string());
        thread::spawn(move || {
            let out = if cfg!(windows) {
                Command::new("pwsh.exe")
                    .args(["-NoLogo", "-NoProfile", "-Command", &expanded])
                    .output()
                    .or_else(|_| {
                        Command::new("powershell.exe")
                            .args(["-NoLogo", "-NoProfile", "-Command", &expanded])
                            .output()
                    })
            } else {
                Command::new("sh").args(["-c", &expanded]).output()
            };
            match out {
                Ok(o) => (
                    o.status.code().unwrap_or(-1),
                    String::from_utf8_lossy(&o.stdout)
                        .chars()
                        .take(400)
                        .collect::<String>(),
                ),
                Err(e) => (-1, format!("spawn error: {e}")),
            }
        })
    });

    let deadline = Instant::now() + Duration::from_millis(wait_ms.clamp(200, 30_000));
    let target_reqs = max_reqs.clamp(1, 50);
    let mut captured = Vec::new();
    while Instant::now() < deadline && captured.len() < target_reqs {
        match listener.accept() {
            Ok((stream, _)) => {
                let _ = stream.set_nonblocking(false);
                if let Some(req) = handle_one_http(stream, reply_status, reply_type, reply_body) {
                    captured.push(req);
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(_) => break,
        }
    }

    let run_res = bg_handle.and_then(|h| h.join().ok());
    let elapsed_ms = (t0.elapsed().as_secs_f64() * 1000.0).round();

    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "ok": !captured.is_empty(),
                "port": bound_port,
                "elapsed_ms": elapsed_ms,
                "captured_count": captured.len(),
                "requests": captured,
                "run_exit": run_res.as_ref().map(|(c, _)| *c),
                "run_output": run_res.as_ref().map(|(_, o)| o.as_str()),
            }))
            .unwrap_or_default()
        );
    } else {
        println!(
            "[probe listen] 127.0.0.1:{bound_port} captured {} request(s) in {elapsed_ms}ms:",
            captured.len()
        );
        for (i, r) in captured.iter().enumerate() {
            println!(
                "  {}. {} {} ({} headers, {} B body)",
                i + 1,
                r.method,
                r.path,
                r.headers.len(),
                r.body.len()
            );
            for h in r.headers.iter().take(6) {
                println!("     {h}");
            }
            if !r.body.is_empty() {
                println!(
                    "     body: {}",
                    r.body.chars().take(240).collect::<String>()
                );
            }
        }
    }
    if captured.is_empty() {
        1
    } else {
        0
    }
}
