use serde_json::{json, Value};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

pub struct CdpSession {
    child: Child,
    profile_dir: PathBuf,
    stream: TcpStream,
    next_id: u64,
    pub js_errors: Vec<String>,
}

fn free_port() -> Result<u16, String> {
    let l = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let p = l.local_addr().map_err(|e| e.to_string())?.port();
    drop(l);
    Ok(p)
}

fn read_exact_bytes(stream: &mut TcpStream, buf: &mut [u8]) -> Result<(), String> {
    stream.read_exact(buf).map_err(|e| format!("ws read: {e}"))
}

fn ws_write_text(stream: &mut TcpStream, text: &str) -> Result<(), String> {
    let payload = text.as_bytes();
    let len = payload.len();
    let mask = [0x37u8, 0xfa, 0x21, 0x3d];
    let mut frame = Vec::with_capacity(len + 14);
    frame.push(0x81); // FIN + text frame
    if len <= 125 {
        frame.push(0x80 | (len as u8));
    } else if len <= 65535 {
        frame.push(0x80 | 126);
        frame.extend_from_slice(&(len as u16).to_be_bytes());
    } else {
        frame.push(0x80 | 127);
        frame.extend_from_slice(&(len as u64).to_be_bytes());
    }
    frame.extend_from_slice(&mask);
    for (i, b) in payload.iter().enumerate() {
        frame.push(b ^ mask[i & 3]);
    }
    stream
        .write_all(&frame)
        .map_err(|e| format!("ws write: {e}"))
}

fn ws_read_message(stream: &mut TcpStream) -> Result<String, String> {
    let mut msg_bytes = Vec::new();
    loop {
        let mut hdr = [0u8; 2];
        read_exact_bytes(stream, &mut hdr)?;
        let fin = (hdr[0] & 0x80) != 0;
        let opcode = hdr[0] & 0x0F;
        let masked = (hdr[1] & 0x80) != 0;
        let mut len = (hdr[1] & 0x7F) as usize;
        if len == 126 {
            let mut b = [0u8; 2];
            read_exact_bytes(stream, &mut b)?;
            len = u16::from_be_bytes(b) as usize;
        } else if len == 127 {
            let mut b = [0u8; 8];
            read_exact_bytes(stream, &mut b)?;
            len = u64::from_be_bytes(b) as usize;
        }
        let mut mask = [0u8; 4];
        if masked {
            read_exact_bytes(stream, &mut mask)?;
        }
        let mut data = vec![0u8; len];
        if len > 0 {
            read_exact_bytes(stream, &mut data)?;
        }
        if masked {
            for (i, b) in data.iter_mut().enumerate() {
                *b ^= mask[i & 3];
            }
        }
        match opcode {
            8 => return Err("CDP websocket closed".to_string()),
            9 => {
                let _ = stream.write_all(&[0x8A, 0x80, 0, 0, 0, 0]);
                continue;
            }
            10 => continue,
            _ => {
                msg_bytes.extend_from_slice(&data);
                if fin {
                    return Ok(String::from_utf8_lossy(&msg_bytes).to_string());
                }
            }
        }
    }
}

fn http_get_json(port: u16, path: &str) -> Result<Value, String> {
    let mut s = TcpStream::connect(("127.0.0.1", port)).map_err(|e| e.to_string())?;
    let _ = s.set_read_timeout(Some(Duration::from_millis(1200)));
    let _ = s.set_write_timeout(Some(Duration::from_millis(1200)));
    let req = format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n");
    s.write_all(req.as_bytes()).map_err(|e| e.to_string())?;
    let mut buf = Vec::new();
    let _ = s.read_to_end(&mut buf);
    let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") else {
        return Err("no http body".to_string());
    };
    serde_json::from_slice(&buf[pos + 4..]).map_err(|e| e.to_string())
}

impl CdpSession {
    pub fn launch(browser_bin: &PathBuf) -> Result<Self, String> {
        let port = free_port()?;
        let pid = std::process::id();
        let profile_dir = std::env::temp_dir().join(format!("ap_cdp_{pid}_{port}"));
        let _ = fs::create_dir_all(&profile_dir);
        let out_log = profile_dir.join("browser.out.log");
        let err_log = profile_dir.join("browser.err.log");
        let out_f = File::create(&out_log).map_err(|e| e.to_string())?;
        let err_f = File::create(&err_log).map_err(|e| e.to_string())?;

        let mut child = Command::new(browser_bin)
            .args([
                "--headless=new",
                "--disable-gpu",
                "--disable-extensions",
                "--no-sandbox",
                "--proxy-bypass-list=<-loopback>",
                "--log-level=3",
                "--no-first-run",
                "--no-default-browser-check",
                &format!("--remote-debugging-port={port}"),
                "--remote-allow-origins=*",
                &format!("--user-data-dir={}", profile_dir.display()),
                "about:blank",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::from(out_f))
            .stderr(Stdio::from(err_f))
            .spawn()
            .map_err(|e| format!("spawn headless browser: {e}"))?;

        let deadline = Instant::now() + Duration::from_millis(8000);
        let mut ws_url = String::new();
        while Instant::now() < deadline {
            if let Ok(Value::Array(items)) = http_get_json(port, "/json/list") {
                for it in items {
                    if it.get("type").and_then(Value::as_str) == Some("page") {
                        if let Some(u) = it.get("webSocketDebuggerUrl").and_then(Value::as_str) {
                            ws_url = u.to_string();
                            break;
                        }
                    }
                }
                if !ws_url.is_empty() {
                    break;
                }
            }
            thread::sleep(Duration::from_millis(60));
        }

        if ws_url.is_empty() {
            kill_browser(&mut child, &profile_dir);
            return Err("Timed out waiting for headless browser CDP /json/list".to_string());
        }

        let after_scheme = ws_url.strip_prefix("ws://").unwrap_or(&ws_url);
        let (addr, ws_path) = after_scheme
            .split_once('/')
            .map(|(a, p)| (a, format!("/{p}")))
            .unwrap_or((after_scheme, "/".to_string()));
        let mut stream = TcpStream::connect(addr).map_err(|e| format!("connect CDP ws: {e}"))?;
        let _ = stream.set_read_timeout(Some(Duration::from_millis(8000)));
        let _ = stream.set_write_timeout(Some(Duration::from_millis(8000)));

        let hs = format!(
            "GET {ws_path} HTTP/1.1\r\nHost: {addr}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nSec-WebSocket-Version: 13\r\n\r\n"
        );
        stream.write_all(hs.as_bytes()).map_err(|e| e.to_string())?;
        let mut resp = Vec::with_capacity(256);
        let mut b1 = [0u8; 1];
        while !resp.ends_with(b"\r\n\r\n") {
            read_exact_bytes(&mut stream, &mut b1)?;
            resp.push(b1[0]);
            if resp.len() > 8192 {
                kill_browser(&mut child, &profile_dir);
                return Err("CDP websocket handshake header overflow".to_string());
            }
        }
        if !String::from_utf8_lossy(&resp).contains("101") {
            kill_browser(&mut child, &profile_dir);
            return Err("CDP websocket upgrade rejected".to_string());
        }

        let mut sess = Self {
            child,
            profile_dir,
            stream,
            next_id: 1,
            js_errors: Vec::new(),
        };
        let _ = sess.call("Page.enable", json!({}));
        let _ = sess.call("Runtime.enable", json!({}));
        let _ = sess.call("Log.enable", json!({}));
        Ok(sess)
    }

    pub fn call(&mut self, method: &str, params: Value) -> Result<Value, String> {
        let id = self.next_id;
        self.next_id += 1;
        let msg = json!({ "id": id, "method": method, "params": params }).to_string();
        ws_write_text(&mut self.stream, &msg)?;
        loop {
            let raw = ws_read_message(&mut self.stream)?;
            let Ok(v) = serde_json::from_str::<Value>(&raw) else {
                continue;
            };
            if let Some(ev) = v.get("method").and_then(Value::as_str) {
                self.record_event(ev, v.get("params").unwrap_or(&Value::Null));
                continue;
            }
            if v.get("id").and_then(Value::as_u64) == Some(id) {
                if let Some(err) = v.get("error") {
                    return Err(format!("CDP {method} error: {err}"));
                }
                return Ok(v.get("result").cloned().unwrap_or(Value::Null));
            }
        }
    }

    fn record_event(&mut self, ev: &str, params: &Value) {
        if ev == "Runtime.exceptionThrown" {
            let desc = params
                .pointer("/exceptionDetails/exception/description")
                .or_else(|| params.pointer("/exceptionDetails/text"))
                .and_then(Value::as_str)
                .unwrap_or("Uncaught JS exception");
            self.js_errors.push(format!(
                "[pageerror] {}",
                desc.lines().next().unwrap_or(desc)
            ));
        } else if ev == "Runtime.consoleAPICalled"
            && params.get("type").and_then(Value::as_str) == Some("error")
        {
            if let Some(args) = params.get("args").and_then(Value::as_array) {
                let parts: Vec<String> = args
                    .iter()
                    .filter_map(|a| a.get("value").or_else(|| a.get("description")))
                    .map(|x| x.to_string().trim_matches('"').to_string())
                    .collect();
                if !parts.is_empty() {
                    self.js_errors
                        .push(format!("[console.error] {}", parts.join(" ")));
                }
            }
        }
    }
}

fn kill_browser(child: &mut Child, profile_dir: &PathBuf) {
    let pid_s = child.id().to_string();
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/PID", &pid_s, "/T", "/F"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    #[cfg(not(windows))]
    {
        let _ = Command::new("pkill")
            .args(["-KILL", "-P", &pid_s])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    let _ = child.kill();
    let _ = child.wait();
    let _ = fs::remove_dir_all(profile_dir);
}

impl Drop for CdpSession {
    fn drop(&mut self) {
        kill_browser(&mut self.child, &self.profile_dir);
    }
}

impl CdpSession {
    pub fn eval_js(&mut self, expr: &str) -> Result<String, String> {
        let wrapped = format!(
            "(async () => {{ const __r = await ({expr}); return typeof __r === 'string' ? __r : JSON.stringify(__r ?? ''); }})()"
        );
        let res = self.call(
            "Runtime.evaluate",
            json!({
                "expression": wrapped,
                "awaitPromise": true,
                "returnByValue": true
            }),
        )?;
        if let Some(exc) = res.get("exceptionDetails") {
            let txt = exc
                .pointer("/exception/description")
                .or_else(|| exc.get("text"))
                .and_then(Value::as_str)
                .unwrap_or("JS evaluation error");
            return Err(txt.lines().next().unwrap_or(txt).to_string());
        }
        Ok(res
            .pointer("/result/value")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string())
    }

    pub fn navigate(&mut self, url: &str) -> Result<String, String> {
        let _ = self.call(
            "Page.addScriptToEvaluateOnNewDocument",
            json!({
                "source": "window.confirm = () => true; window.alert = (m) => { window.__lastAlert = String(m); };"
            }),
        );
        self.call("Page.navigate", json!({ "url": url }))?;
        let deadline = Instant::now() + Duration::from_millis(6000);
        while Instant::now() < deadline {
            if let Ok(st) = self.eval_js("document.readyState") {
                if st == "complete" || st == "interactive" {
                    break;
                }
            }
            thread::sleep(Duration::from_millis(60));
        }
        thread::sleep(Duration::from_millis(320));
        self.dom_snapshot()
    }

    pub fn dom_snapshot(&mut self) -> Result<String, String> {
        self.eval_js("document.documentElement ? (document.documentElement.outerHTML + '\\n---TEXT---\\n' + (document.body ? document.body.innerText : '')) : ''")
    }

    pub fn fill(&mut self, selector: &str, value: &str) -> Result<String, String> {
        let sel_js = serde_json::to_string(selector).unwrap_or_else(|_| "\"\"".to_string());
        let val_js = serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string());
        let script = format!(
            "(() => {{
                const el = document.querySelector({sel_js});
                if (!el) throw new Error('Selector not found: ' + {sel_js});
                el.focus();
                el.value = {val_js};
                el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                return new Promise(r => setTimeout(() => r(document.body.innerText), 160));
            }})()"
        );
        self.eval_js(&script)
    }

    pub fn select_option(&mut self, selector: &str, value: &str) -> Result<String, String> {
        let sel_js = serde_json::to_string(selector).unwrap_or_else(|_| "\"\"".to_string());
        let val_js = serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string());
        let script = format!(
            "(() => {{
                const el = document.querySelector({sel_js});
                if (!el) throw new Error('Select not found: ' + {sel_js});
                let matched = false;
                for (const opt of Array.from(el.options || [])) {{
                    if (opt.value === {val_js} || (opt.textContent || '').includes({val_js})) {{
                        el.value = opt.value;
                        matched = true;
                        break;
                    }}
                }}
                if (!matched) el.value = {val_js};
                el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                return new Promise(r => setTimeout(() => r(document.body.innerText), 180));
            }})()"
        );
        self.eval_js(&script)
    }

    pub fn click(&mut self, target: &str) -> Result<String, String> {
        let script = if let Some(txt) = target.strip_prefix("text=") {
            let t_js = serde_json::to_string(txt.trim()).unwrap_or_else(|_| "\"\"".to_string());
            format!(
                "(() => {{
                    const cands = Array.from(document.querySelectorAll('button, a, [role=\"button\"], input[type=\"submit\"], input[type=\"button\"], label'));
                    const el = cands.find(n => (n.innerText || n.value || n.textContent || '').trim().includes({t_js}));
                    if (!el) throw new Error('Button/link with text not found: ' + {t_js});
                    el.click();
                    return new Promise(r => setTimeout(() => r(document.documentElement.outerHTML + '\\n' + document.body.innerText), 260));
                }})()"
            )
        } else {
            let sel_js = serde_json::to_string(target).unwrap_or_else(|_| "\"\"".to_string());
            format!(
                "(() => {{
                    const el = document.querySelector({sel_js});
                    if (!el) throw new Error('Click target not found: ' + {sel_js});
                    el.click();
                    return new Promise(r => setTimeout(() => r(document.documentElement.outerHTML + '\\n' + document.body.innerText), 260));
                }})()"
            )
        };
        self.eval_js(&script)
    }
}
