use ignore::WalkBuilder;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io::{Cursor, Read};
use std::path::Path;
use std::sync::OnceLock;
use std::time::Instant;
use zip::ZipArchive;

pub const ALL_AXES: &[&str] = &[
    "exec_spawn",
    "net_listen_rpc",
    "ipc_pipe_proto",
    "fs_path_io",
    "deserialize_parse",
    "loader_plugin_ext",
    "auth_token_cred",
    "webview_browser",
    "service_priv_acl",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceItem {
    pub id: String,
    pub axis: String,
    pub module: String,
    pub target: String,
    pub line: usize,
    pub symbol: String,
    pub snippet: String,
    pub score: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisSummary {
    pub axis: String,
    pub total_hits: usize,
    pub distinct_files: usize,
    pub distinct_modules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceMap {
    pub root: String,
    pub elapsed_ms: f64,
    pub scanned_files: usize,
    pub scanned_archives: usize,
    pub axes_present: Vec<String>,
    pub axes_absent: Vec<String>,
    pub axis_summaries: Vec<AxisSummary>,
    pub items: Vec<SurfaceItem>,
}

struct AxisRule {
    axis: &'static str,
    base_score: u32,
    re: Regex,
}

fn axis_rules() -> &'static [AxisRule] {
    static RULES: OnceLock<Vec<AxisRule>> = OnceLock::new();
    RULES.get_or_init(|| {
        vec![
            AxisRule {
                axis: "exec_spawn",
                base_score: 90,
                re: Regex::new(
                    r"\b(?:child_process|execFileSync|execFile|execSync|spawnSync|spawn|CreateProcess[AW]?|ShellExecute(?:Ex)?[AW]?|WinExec|ProcessBuilder|Runtime\.getRuntime\(\)\.exec|subprocess\.(?:Popen|run|call|check_output)|os\.system|os/exec|exec\.Command|Command::new)\b",
                )
                .unwrap(),
            },
            AxisRule {
                axis: "net_listen_rpc",
                base_score: 92,
                re: Regex::new(
                    r"\b(?:createServer|ServerSocket|TcpListener::bind|net\.Listen|WebSocketServer|http\.HandleFunc|ListenAndServe|uvicorn\.run|FastAPI|JsonRpc|LanguageServer|DebugAdapter|startServer|bind\s*\(\s*\(?['\x22]?(?:0\.0\.0\.0|127\.0\.0\.1|localhost)|\.listen\s*\()",
                )
                .unwrap(),
            },
            AxisRule {
                axis: "ipc_pipe_proto",
                base_score: 88,
                re: Regex::new(
                    r"\b(?:ipcMain\.(?:handle|on)|contextBridge\.exposeInMainWorld|setAsDefaultProtocolClient|register(?:File|Http|Buffer|Stream|String)?Protocol|protocol\.handle|CreateNamedPipe[AW]?|ConnectNamedPipe|CallNamedPipe|\\\\\.\\pipe\\|postMessage|addEventListener\s*\(\s*['\x22]message['\x22]|registerUriHandler|open-url|second-instance)\b",
                )
                .unwrap(),
            },
            AxisRule {
                axis: "fs_path_io",
                base_score: 82,
                re: Regex::new(
                    r"\b(?:Files\.newBufferedReader|Files\.readAllBytes|Files\.write|File\.delete|ZipInputStream|ZipFile|extractAllTo|extractTo|tar\.extract|shutil\.(?:unpack_archive|copy|rmtree)|writeFileSync|readFileSync|createWriteStream|createReadStream|fs\.unlink|fs\.rmSync|fs::write|fs::copy|symlinkSync|MoveFileEx[AW]?|CopyFile[AW]?)\b",
                )
                .unwrap(),
            },
            AxisRule {
                axis: "deserialize_parse",
                base_score: 86,
                re: Regex::new(
                    r"\b(?:ObjectInputStream|readObject|yaml\.load|pickle\.loads?|marshal\.loads?|vm\.runIn(?:New|This)?Context|new\s+Function\s*\(|\beval\s*\(|DocumentBuilderFactory|SAXParserFactory|XMLDecoder|XStream|innerHTML|outerHTML|dangerouslySetInnerHTML|insertAdjacentHTML|DOMParser)\b",
                )
                .unwrap(),
            },
            AxisRule {
                axis: "loader_plugin_ext",
                base_score: 84,
                re: Regex::new(
                    r"\b(?:LoadLibrary(?:Ex)?[AW]?|dlopen|URLClassLoader|importlib\.import_module|__import__|require\.resolve|contributes|extensionPath|pluginDir|loadPlugin|executablePath|commandLineArguments|settings\.json)\b",
                )
                .unwrap(),
            },
            AxisRule {
                axis: "auth_token_cred",
                base_score: 80,
                re: Regex::new(
                    r"\b(?:rejectUnauthorized\s*:\s*false|InsecureSkipVerify|X509TrustManager|checkServerTrusted|NODE_TLS_REJECT_UNAUTHORIZED|CredRead[AW]?|CryptUnprotectData|Authorization|X-Auth-Token|cookie|sessionToken|refreshToken|accessToken|jwt\.decode)\b",
                )
                .unwrap(),
            },
            AxisRule {
                axis: "webview_browser",
                base_score: 85,
                re: Regex::new(
                    r"\b(?:BrowserWindow|webPreferences|nodeIntegration|contextIsolation|sandbox\s*:\s*false|webSecurity\s*:\s*false|allowRunningInsecureContent|setWindowOpenHandler|will-navigate|shell\.openExternal|createWebviewPanel|asWebviewUri|preload)\b",
                )
                .unwrap(),
            },
            AxisRule {
                axis: "service_priv_acl",
                base_score: 87,
                re: Regex::new(
                    r"\b(?:CreateService[AW]?|ChangeServiceConfig|StartService[AW]?|RegSetValue(?:Ex)?[AW]?|RegCreateKey(?:Ex)?[AW]?|SetNamedSecurityInfo|ConvertStringSecurityDescriptorToSecurityDescriptor|icacls|schtasks|sudo\b|setuid|chmod\s+777|runas|ShellExecute.*runas|NETWORK SERVICE|LOCAL SYSTEM)\b",
                )
                .unwrap(),
            },
        ]
    })
}

fn fnv1a_hex(s: &str) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{:08x}", (h ^ (h >> 32)) as u32)
}

fn derive_module(rel: &str) -> String {
    let clean = rel.replace('\\', "/");
    let parts: Vec<&str> = clean.split('/').filter(|p| !p.is_empty()).collect();
    if parts.is_empty() {
        return "root".to_string();
    }
    if parts.len() == 1 {
        return parts[0].to_string();
    }
    // Group e.g. extensions/foo/bar.js -> extensions/foo; resources/app.asar!/... -> app.asar/...
    if parts[0].eq_ignore_ascii_case("extensions")
        || parts[0].eq_ignore_ascii_case("plugins")
        || parts[0].eq_ignore_ascii_case("packages")
        || parts[0].eq_ignore_ascii_case("crates")
        || parts[0].eq_ignore_ascii_case("resources")
    {
        return format!("{}/{}", parts[0], parts[1]);
    }
    if let Some((arch, inner)) = clean.split_once("!/") {
        let arch_name = Path::new(arch)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(arch);
        let first_inner = inner.split('/').next().unwrap_or("");
        if first_inner.is_empty() {
            return arch_name.to_string();
        }
        return format!("{arch_name}/{first_inner}");
    }
    format!("{}/{}", parts[0], parts[1])
}

fn clip_snippet(text: &str, byte_start: usize, byte_end: usize) -> (usize, String) {
    let line_start = text[..byte_start].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_end = text[byte_end..]
        .find('\n')
        .map(|i| byte_end + i)
        .unwrap_or(text.len());
    let line_no = text[..byte_start].bytes().filter(|&b| b == b'\n').count() + 1;
    let line = &text[line_start..line_end];
    if line.len() <= 160 {
        return (line_no, line.trim().to_string());
    }
    // Minified single-line bundle: take a clean UTF-8 boundary window around the match
    let mut win_start = byte_start.saturating_sub(48).max(line_start);
    while win_start < byte_start && !text.is_char_boundary(win_start) {
        win_start += 1;
    }
    let mut win_end = (byte_end + 80).min(line_end);
    while win_end > byte_end && !text.is_char_boundary(win_end) {
        win_end -= 1;
    }
    let slice = text[win_start..win_end].replace(['\r', '\n', '\t'], " ");
    (line_no, slice.trim().to_string())
}

fn is_scannable_text_ext(ext: &str, file_name: &str) -> bool {
    let low_name = file_name.to_ascii_lowercase();
    if matches!(
        low_name.as_str(),
        "package.json" | "manifest.json" | "plugin.xml" | "androidmanifest.xml" | "config.json"
    ) {
        return true;
    }
    matches!(
        ext,
        "js" | "mjs"
            | "cjs"
            | "ts"
            | "tsx"
            | "jsx"
            | "py"
            | "go"
            | "rs"
            | "c"
            | "cc"
            | "cpp"
            | "h"
            | "hpp"
            | "java"
            | "kt"
            | "cs"
            | "rb"
            | "php"
            | "sh"
            | "ps1"
            | "yaml"
            | "yml"
            | "toml"
    )
}

fn is_archive_ext(ext: &str) -> bool {
    matches!(ext, "asar" | "vsix" | "zip" | "jar")
}

fn scan_text_into(
    rel_target: &str,
    text: &str,
    rules: &[AxisRule],
    axis_hits: &mut HashMap<String, usize>,
    axis_files: &mut HashMap<String, HashSet<String>>,
    axis_modules: &mut HashMap<String, HashSet<String>>,
    candidates: &mut Vec<SurfaceItem>,
) {
    let module = derive_module(rel_target);
    let is_manifest = rel_target.ends_with("package.json")
        || rel_target.ends_with("manifest.json")
        || rel_target.ends_with("plugin.xml");
    let low_rel = rel_target.to_ascii_lowercase();
    let is_test = low_rel.contains("/test/")
        || low_rel.contains("/tests/")
        || low_rel.contains(".test.")
        || low_rel.contains(".spec.")
        || low_rel.contains("_test.");
    if is_test {
        return;
    }

    for rule in rules {
        let mut seen_symbols: HashSet<String> = HashSet::new();
        let mut file_axis_hits = 0usize;
        for m in rule.re.find_iter(text) {
            // Skip obvious comment-only lines on non-minified files
            let (line_no, snippet) = clip_snippet(text, m.start(), m.end());
            if snippet.starts_with("//") || snippet.starts_with('#') || snippet.starts_with('*') {
                continue;
            }
            let sym = m.as_str().trim().to_string();
            *axis_hits.entry(rule.axis.to_string()).or_insert(0) += 1;
            axis_files
                .entry(rule.axis.to_string())
                .or_default()
                .insert(rel_target.to_string());
            axis_modules
                .entry(rule.axis.to_string())
                .or_default()
                .insert(module.clone());

            file_axis_hits += 1;
            if file_axis_hits > 3 || !seen_symbols.insert(sym.to_ascii_lowercase()) {
                continue;
            }

            let mut score = rule.base_score;
            if rel_target.contains("!/") {
                score += 4;
            }
            if is_manifest {
                score = score.saturating_sub(6);
            }
            if low_rel.contains("node_modules/") {
                score = score.saturating_sub(18);
            }
            let key = format!("{}|{}|{}", rule.axis, rel_target, sym.to_ascii_lowercase());
            let id = format!("{}:{}", rule.axis, fnv1a_hex(&key));
            candidates.push(SurfaceItem {
                id,
                axis: rule.axis.to_string(),
                module: module.clone(),
                target: rel_target.to_string(),
                line: line_no,
                symbol: sym,
                snippet,
                score,
            });
        }
    }
}

fn collect_asar_files(node: &Value, prefix: &str, out: &mut Vec<(String, u64, u64)>, limit: usize) {
    if out.len() >= limit {
        return;
    }
    let Some(files) = node.get("files").and_then(|v| v.as_object()) else {
        return;
    };
    for (name, val) in files {
        if out.len() >= limit {
            break;
        }
        let next = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}/{name}")
        };
        if val.get("files").is_some() {
            if name == "node_modules" && prefix.contains("node_modules") {
                continue;
            }
            collect_asar_files(val, &next, out, limit);
        } else if let (Some(off_v), Some(size_v)) = (val.get("offset"), val.get("size")) {
            let off = off_v
                .as_str()
                .and_then(|s| s.parse::<u64>().ok())
                .or_else(|| off_v.as_u64());
            let sz = size_v.as_u64();
            if let (Some(o), Some(s)) = (off, sz) {
                if s > 0 && s <= 512_000 {
                    let ext = Path::new(&next)
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_ascii_lowercase();
                    if is_scannable_text_ext(&ext, name) {
                        out.push((next, o, s));
                    }
                }
            }
        }
    }
}

fn scan_asar_archive(
    archive_path: &Path,
    rel_archive: &str,
    rules: &[AxisRule],
    axis_hits: &mut HashMap<String, usize>,
    axis_files: &mut HashMap<String, HashSet<String>>,
    axis_modules: &mut HashMap<String, HashSet<String>>,
    candidates: &mut Vec<SurfaceItem>,
) -> usize {
    let Ok(bytes) = fs::read(archive_path) else {
        return 0;
    };
    if bytes.len() < 16 {
        return 0;
    }
    let header_size = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
    let json_len = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;
    let header_end = 8usize.saturating_add(header_size);
    let json_end = 16usize.saturating_add(json_len);
    if json_end > bytes.len() || header_end > bytes.len() || json_end > header_end + 8 {
        return 0;
    }
    let Ok(header_str) = std::str::from_utf8(&bytes[16..json_end]) else {
        return 0;
    };
    let Ok(header_json) = serde_json::from_str::<Value>(header_str) else {
        return 0;
    };
    let mut entries = Vec::new();
    collect_asar_files(&header_json, "", &mut entries, 160);
    let mut scanned = 0usize;
    for (inner_path, off, sz) in entries {
        let start = header_end.saturating_add(off as usize);
        let end = start.saturating_add(sz as usize);
        if end > bytes.len() {
            continue;
        }
        let text = String::from_utf8_lossy(&bytes[start..end]);
        let vpath = format!("{rel_archive}!/{inner_path}");
        scan_text_into(
            &vpath,
            &text,
            rules,
            axis_hits,
            axis_files,
            axis_modules,
            candidates,
        );
        scanned += 1;
    }
    scanned
}

fn scan_zip_archive(
    archive_path: &Path,
    rel_archive: &str,
    rules: &[AxisRule],
    axis_hits: &mut HashMap<String, usize>,
    axis_files: &mut HashMap<String, HashSet<String>>,
    axis_modules: &mut HashMap<String, HashSet<String>>,
    candidates: &mut Vec<SurfaceItem>,
) -> usize {
    let Ok(meta) = archive_path.metadata() else {
        return 0;
    };
    if meta.len() > 48 * 1024 * 1024 {
        return 0;
    }
    let Ok(bytes) = fs::read(archive_path) else {
        return 0;
    };
    let Ok(mut zip) = ZipArchive::new(Cursor::new(bytes)) else {
        return 0;
    };
    let mut scanned = 0usize;
    for i in 0..zip.len() {
        if scanned >= 120 {
            break;
        }
        let Ok(mut entry) = zip.by_index(i) else {
            continue;
        };
        if entry.is_dir() || entry.size() == 0 || entry.size() > 512_000 {
            continue;
        }
        let name = entry.name().to_string();
        let file_name = Path::new(&name)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(&name);
        let ext = Path::new(&name)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !is_scannable_text_ext(&ext, file_name) {
            continue;
        }
        let mut buf = Vec::with_capacity(entry.size().min(512_000) as usize);
        if entry.read_to_end(&mut buf).is_err() {
            continue;
        }
        let text = String::from_utf8_lossy(&buf);
        let vpath = format!("{rel_archive}!/{name}");
        scan_text_into(
            &vpath,
            &text,
            rules,
            axis_hits,
            axis_files,
            axis_modules,
            candidates,
        );
        scanned += 1;
    }
    scanned
}

pub fn build_surface_map(
    root: &Path,
    max_files: usize,
    max_per_axis: usize,
    include_vendor: bool,
) -> SurfaceMap {
    let t0 = Instant::now();
    let rules = axis_rules();
    let mut axis_hits: HashMap<String, usize> = HashMap::new();
    let mut axis_files: HashMap<String, HashSet<String>> = HashMap::new();
    let mut axis_modules: HashMap<String, HashSet<String>> = HashMap::new();
    let mut candidates: Vec<SurfaceItem> = Vec::new();
    let mut scanned_files = 0usize;
    let mut scanned_archives = 0usize;

    let hard_skip = [
        ".git",
        ".hg",
        ".svn",
        "__pycache__",
        ".venv",
        "venv",
        "target",
        ".sieve",
        ".baton",
        ".recall",
        ".guard",
        ".probe",
        ".splice",
        "coverage",
    ];

    if root.is_file() {
        let rel = root
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("target")
            .to_string();
        let ext = root
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if ext == "asar" {
            scanned_files += scan_asar_archive(
                root,
                &rel,
                rules,
                &mut axis_hits,
                &mut axis_files,
                &mut axis_modules,
                &mut candidates,
            );
            scanned_archives += 1;
        } else if is_archive_ext(&ext) {
            scanned_files += scan_zip_archive(
                root,
                &rel,
                rules,
                &mut axis_hits,
                &mut axis_files,
                &mut axis_modules,
                &mut candidates,
            );
            scanned_archives += 1;
        } else if let Ok(raw) = fs::read(root) {
            let cap = raw.len().min(1_048_576);
            let text = String::from_utf8_lossy(&raw[..cap]);
            scan_text_into(
                &rel,
                &text,
                rules,
                &mut axis_hits,
                &mut axis_files,
                &mut axis_modules,
                &mut candidates,
            );
            scanned_files += 1;
        }
    } else {
        let walker = WalkBuilder::new(root)
            .hidden(false)
            .git_ignore(!include_vendor)
            .git_global(false)
            .git_exclude(false)
            .filter_entry(move |e| {
                if e.depth() == 0 {
                    return true;
                }
                let nm = e.file_name().to_string_lossy();
                if hard_skip.iter().any(|s| *s == nm) {
                    return false;
                }
                if !include_vendor && (nm == "node_modules" || nm == "vendor") {
                    return false;
                }
                true
            })
            .build();

        for entry in walker.flatten() {
            if scanned_files >= max_files {
                break;
            }
            let p = entry.path();
            if !p.is_file() {
                continue;
            }
            let rel = p
                .strip_prefix(root)
                .unwrap_or(p)
                .to_string_lossy()
                .replace('\\', "/");
            let file_name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
            let ext = p
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();

            if ext == "asar" && scanned_archives < 16 {
                scanned_files += scan_asar_archive(
                    p,
                    &rel,
                    rules,
                    &mut axis_hits,
                    &mut axis_files,
                    &mut axis_modules,
                    &mut candidates,
                );
                scanned_archives += 1;
                continue;
            }
            if is_archive_ext(&ext) && scanned_archives < 24 {
                scanned_files += scan_zip_archive(
                    p,
                    &rel,
                    rules,
                    &mut axis_hits,
                    &mut axis_files,
                    &mut axis_modules,
                    &mut candidates,
                );
                scanned_archives += 1;
                continue;
            }
            if !is_scannable_text_ext(&ext, file_name) {
                continue;
            }
            let Ok(meta) = p.metadata() else { continue };
            if meta.len() == 0 || meta.len() > 2 * 1024 * 1024 {
                continue;
            }
            let Ok(raw) = fs::read(p) else { continue };
            let cap = raw.len().min(768_000);
            let text = String::from_utf8_lossy(&raw[..cap]);
            scan_text_into(
                &rel,
                &text,
                rules,
                &mut axis_hits,
                &mut axis_files,
                &mut axis_modules,
                &mut candidates,
            );
            scanned_files += 1;
        }
    }

    // Rank & diversify per axis (module diversity first, then score)
    let mut by_axis: BTreeMap<String, Vec<SurfaceItem>> = BTreeMap::new();
    for item in candidates {
        by_axis.entry(item.axis.clone()).or_default().push(item);
    }
    let mut final_items: Vec<SurfaceItem> = Vec::new();
    for ax in ALL_AXES {
        let Some(mut list) = by_axis.remove(*ax) else {
            continue;
        };
        list.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| a.target.cmp(&b.target))
                .then_with(|| a.line.cmp(&b.line))
        });
        // Pick up to max_per_axis with module round-robin so one huge file doesn't crowd out other modules
        let mut by_mod: BTreeMap<String, Vec<SurfaceItem>> = BTreeMap::new();
        for it in list {
            by_mod.entry(it.module.clone()).or_default().push(it);
        }
        let mut picked = 0usize;
        let mut round = 0usize;
        while picked < max_per_axis {
            let mut added_this_round = false;
            for vec in by_mod.values() {
                if let Some(it) = vec.get(round) {
                    final_items.push(it.clone());
                    picked += 1;
                    added_this_round = true;
                    if picked >= max_per_axis {
                        break;
                    }
                }
            }
            if !added_this_round {
                break;
            }
            round += 1;
        }
    }

    let mut axes_present = Vec::new();
    let mut axes_absent = Vec::new();
    let mut axis_summaries = Vec::new();
    for ax in ALL_AXES {
        let hits = *axis_hits.get(*ax).unwrap_or(&0);
        if hits > 0 {
            axes_present.push((*ax).to_string());
            let mut mods: Vec<String> = axis_modules
                .get(*ax)
                .map(|s| s.iter().cloned().collect())
                .unwrap_or_default();
            mods.sort();
            mods.truncate(8);
            axis_summaries.push(AxisSummary {
                axis: (*ax).to_string(),
                total_hits: hits,
                distinct_files: axis_files.get(*ax).map(|s| s.len()).unwrap_or(0),
                distinct_modules: mods,
            });
        } else {
            axes_absent.push((*ax).to_string());
        }
    }

    SurfaceMap {
        root: root.to_string_lossy().to_string(),
        elapsed_ms: (t0.elapsed().as_secs_f64() * 1000.0 * 10.0).round() / 10.0,
        scanned_files,
        scanned_archives,
        axes_present,
        axes_absent,
        axis_summaries,
        items: final_items,
    }
}

pub fn render_surface_md(map: &SurfaceMap) -> String {
    let mut out = Vec::new();
    out.push(format!(
        "# ATLAS SURFACE MAP ({} files, {} archives, {:.1}ms)",
        map.scanned_files, map.scanned_archives, map.elapsed_ms
    ));
    out.push(format!("**Root**: `{}`", map.root));
    out.push(format!(
        "**Axes Present ({}/{})**: {}",
        map.axes_present.len(),
        ALL_AXES.len(),
        if map.axes_present.is_empty() {
            "(none)".to_string()
        } else {
            map.axes_present.join(", ")
        }
    ));
    if !map.axes_absent.is_empty() {
        out.push(format!("**Axes Absent**: {}", map.axes_absent.join(", ")));
    }
    if !map.axis_summaries.is_empty() {
        out.push("\n## Axis Coverage".to_string());
        for s in &map.axis_summaries {
            out.push(format!(
                "- `{}`: {} hits across {} files (modules: {})",
                s.axis,
                s.total_hits,
                s.distinct_files,
                s.distinct_modules.join(", ")
            ));
        }
    }
    if !map.items.is_empty() {
        out.push("\n## Top Entrypoint / Sink Surfaces".to_string());
        for it in map.items.iter().take(36) {
            out.push(format!(
                "- `{}` [{}] `{}:{}` (`{}`) — {}",
                it.id,
                it.axis,
                it.target,
                it.line,
                it.symbol,
                it.snippet.chars().take(100).collect::<String>()
            ));
        }
    }
    format!("{}\n", out.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn detects_multiple_orthogonal_axes_and_zip_archives() {
        let dir = std::env::temp_dir().join(format!("ap_surface_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(
            dir.join("src/server.js"),
            "const net = require('http');\nconst s = net.createServer((req,res)=>{});\ns.listen(8080);\n",
        )
        .unwrap();
        fs::write(
            dir.join("src/ipc.ts"),
            "ipcMain.handle('open', async (_e, p) => fs.readFileSync(p));\n",
        )
        .unwrap();
        fs::write(
            dir.join("src/java_sock.java"),
            "ServerSocket ss = new ServerSocket(0);\nFiles.newBufferedReader(Paths.get(p));\n",
        )
        .unwrap();

        // Also write a zip archive containing a webview surface
        let zip_path = dir.join("ext.vsix");
        {
            let file = fs::File::create(&zip_path).unwrap();
            let mut zw = zip::ZipWriter::new(file);
            let opts = zip::write::SimpleFileOptions::default();
            zw.start_file("extension/main.js", opts).unwrap();
            zw.write_all(b"new BrowserWindow({ webPreferences: { nodeIntegration: true } });\n")
                .unwrap();
            zw.finish().unwrap();
        }

        let map = build_surface_map(&dir, 500, 10, false);
        assert!(map.axes_present.contains(&"net_listen_rpc".to_string()));
        assert!(map.axes_present.contains(&"ipc_pipe_proto".to_string()));
        assert!(map.axes_present.contains(&"fs_path_io".to_string()));
        assert!(map.axes_present.contains(&"webview_browser".to_string()));
        assert_eq!(map.scanned_archives, 1);
        assert!(map
            .items
            .iter()
            .any(|it| it.target.contains("ext.vsix!/extension/main.js")));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn scans_electron_asar_in_memory() {
        let dir = std::env::temp_dir().join(format!("ap_surface_asar_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let body = b"const { shell } = require('electron'); shell.openExternal(url);";
        let header_json = format!(
            r#"{{"files":{{"main.js":{{"size":{},"offset":"0"}}}}}}"#,
            body.len()
        );
        let json_bytes = header_json.as_bytes();
        let pad = (4 - (json_bytes.len() % 4)) % 4;
        let aligned_len = json_bytes.len() + pad;
        let header_size = aligned_len + 8;
        let mut asar = Vec::new();
        asar.extend_from_slice(&4u32.to_le_bytes());
        asar.extend_from_slice(&(header_size as u32).to_le_bytes());
        asar.extend_from_slice(&((aligned_len + 4) as u32).to_le_bytes());
        asar.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
        asar.extend_from_slice(json_bytes);
        asar.resize(asar.len() + pad, 0);
        asar.extend_from_slice(body);
        fs::write(dir.join("app.asar"), &asar).unwrap();

        let map = build_surface_map(&dir, 100, 10, false);
        assert_eq!(map.scanned_archives, 1);
        assert!(map.axes_present.contains(&"webview_browser".to_string()));
        assert!(map.items.iter().any(|it| it.target == "app.asar!/main.js"));
        let _ = fs::remove_dir_all(&dir);
    }
}
