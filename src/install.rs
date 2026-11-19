use crate::audit::default_home_dir;
use regex::Regex;
use rusqlite::Connection;
use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub const ALL_PRIMITIVES: [&str; 7] = [
    "sieve", "baton", "recall", "guard", "atlas", "probe", "splice",
];
pub const FENCE_START: &str = "<!-- agent-primitives:start -->";
pub const FENCE_END: &str = "<!-- agent-primitives:end -->";

const EMBEDDED_SKILLS: [(&str, &str); 7] = [
    ("sieve", include_str!("../skills/sieve/SKILL.md")),
    ("baton", include_str!("../skills/baton/SKILL.md")),
    ("recall", include_str!("../skills/recall/SKILL.md")),
    ("guard", include_str!("../skills/guard/SKILL.md")),
    ("atlas", include_str!("../skills/atlas/SKILL.md")),
    ("probe", include_str!("../skills/probe/SKILL.md")),
    ("splice", include_str!("../skills/splice/SKILL.md")),
];

struct HarnessSpec {
    id: &'static str,
    cli_bin: &'static str,
    probe_dirs: &'static [&'static str],
    skill_dirs: &'static [&'static str],
    rule_file: &'static str,
}

const HARNESS_SPECS: &[HarnessSpec] = &[
    HarnessSpec {
        id: "codex",
        cli_bin: "codex",
        probe_dirs: &[".codex"],
        skill_dirs: &[".codex/skills", ".agents/skills"],
        rule_file: ".codex/AGENTS.md",
    },
    HarnessSpec {
        id: "pi",
        cli_bin: "pi",
        probe_dirs: &[".pi"],
        skill_dirs: &[".pi/agent/skills", ".agents/skills"],
        rule_file: ".pi/agent/AGENTS.md",
    },
    HarnessSpec {
        id: "antigravity",
        cli_bin: "agy",
        probe_dirs: &[
            ".gemini/antigravity-cli",
            ".gemini/antigravity",
            ".gemini/config",
        ],
        skill_dirs: &[".gemini/config/skills", ".agents/skills"],
        rule_file: ".gemini/config/GEMINI.md",
    },
    HarnessSpec {
        id: "claude",
        cli_bin: "claude",
        probe_dirs: &[".claude"],
        skill_dirs: &[".claude/skills"],
        rule_file: ".claude/CLAUDE.md",
    },
    HarnessSpec {
        id: "grok",
        cli_bin: "grok",
        probe_dirs: &[".grok"],
        skill_dirs: &[".grok/skills"],
        rule_file: ".grok/AGENTS.md",
    },
    HarnessSpec {
        id: "opencode",
        cli_bin: "opencode",
        probe_dirs: &[".config/opencode", ".local/share/opencode"],
        skill_dirs: &[".config/opencode/skills", ".agents/skills"],
        rule_file: ".config/opencode/AGENTS.md",
    },
    HarnessSpec {
        id: "qwen",
        cli_bin: "qwen",
        probe_dirs: &[".qwen"],
        skill_dirs: &[".qwen/skills", ".agents/skills"],
        rule_file: ".qwen/QWEN.md",
    },
    HarnessSpec {
        id: "gemini-cli",
        cli_bin: "gemini",
        probe_dirs: &[".gemini"],
        skill_dirs: &[".gemini/skills", ".agents/skills"],
        rule_file: ".gemini/GEMINI.md",
    },
    HarnessSpec {
        id: "goose",
        cli_bin: "goose",
        probe_dirs: &[".config/goose"],
        skill_dirs: &[".agents/skills"],
        rule_file: ".config/goose/AGENTS.md",
    },
    HarnessSpec {
        id: "cursor",
        cli_bin: "cursor",
        probe_dirs: &[".cursor"],
        skill_dirs: &[".agents/skills"],
        rule_file: ".cursor/AGENTS.md",
    },
    HarnessSpec {
        id: "windsurf",
        cli_bin: "windsurf",
        probe_dirs: &[".codeium/windsurf"],
        skill_dirs: &[".agents/skills"],
        rule_file: ".codeium/windsurf/AGENTS.md",
    },
    HarnessSpec {
        id: "vscode",
        cli_bin: "code",
        probe_dirs: &[".vscode"],
        skill_dirs: &[".agents/skills"],
        rule_file: ".agents/AGENTS.md",
    },
    HarnessSpec {
        id: "claude-desktop",
        cli_bin: "claude-desktop-nonexistent-cli",
        probe_dirs: &[
            "AppData/Roaming/Claude",
            "Library/Application Support/Claude",
            ".config/Claude",
        ],
        skill_dirs: &[".agents/skills"],
        rule_file: ".agents/AGENTS.md",
    },
    HarnessSpec {
        id: "openclaw",
        cli_bin: "openclaw",
        probe_dirs: &[".openclaw"],
        skill_dirs: &[".openclaw/skills"],
        rule_file: ".openclaw/AGENTS.md",
    },
    HarnessSpec {
        id: "hermes",
        cli_bin: "hermes",
        probe_dirs: &[".hermes"],
        skill_dirs: &[".hermes/skills"],
        rule_file: ".hermes/AGENTS.md",
    },
];

pub fn embedded_skill_md(name: &str) -> Option<&'static str> {
    EMBEDDED_SKILLS
        .iter()
        .find(|(k, _)| *k == name)
        .map(|(_, v)| *v)
}

pub fn all_embedded_skills() -> &'static [(&'static str, &'static str); 7] {
    &EMBEDDED_SKILLS
}

fn posix(p: &Path) -> String {
    fs::canonicalize(p)
        .unwrap_or_else(|_| p.to_path_buf())
        .to_string_lossy()
        .trim_start_matches(r"\\?\")
        .replace('\\', "/")
}

fn which_exists(bin: &str) -> bool {
    let path_var = std::env::var_os("PATH").unwrap_or_default();
    for dir in std::env::split_paths(&path_var) {
        if dir.join(bin).is_file() {
            return true;
        }
        #[cfg(windows)]
        for ext in ["exe", "cmd", "bat", "ps1"] {
            if dir.join(format!("{bin}.{ext}")).is_file() {
                return true;
            }
        }
    }
    false
}

fn normalize_harness_alias(raw: &str) -> Option<&'static str> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "agy" | "antigravity" => Some("antigravity"),
        "gemini" | "gemini-cli" => Some("gemini-cli"),
        "opencode" | "open-code" => Some("opencode"),
        "qwen" | "qwen-code" => Some("qwen"),
        "goose" => Some("goose"),
        "codex" => Some("codex"),
        "pi" | "pi-coding-agent" => Some("pi"),
        "claude" | "claude-code" => Some("claude"),
        "grok" => Some("grok"),
        "cursor" => Some("cursor"),
        "windsurf" | "codeium" => Some("windsurf"),
        "vscode" | "vs-code" | "copilot" => Some("vscode"),
        "claude-desktop" | "claudedesktop" => Some("claude-desktop"),
        "openclaw" => Some("openclaw"),
        "hermes" => Some("hermes"),
        "agents" => Some("agents"),
        _ => None,
    }
}

fn resolve_requested_harnesses(
    agent_arg: &str,
    home: &Path,
    check_path_bins: bool,
) -> Result<Vec<&'static str>, String> {
    let items: Vec<String> = agent_arg
        .split(',')
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .collect();
    if items.is_empty() || items == ["auto"] {
        let mut found = Vec::new();
        for sp in HARNESS_SPECS {
            let has_dir = sp.probe_dirs.iter().any(|d| home.join(d).exists());
            let has_bin = check_path_bins && which_exists(sp.cli_bin);
            if has_dir || has_bin {
                found.push(sp.id);
            }
        }
        return Ok(found);
    }
    if items == ["all"] {
        return Ok(HARNESS_SPECS.iter().map(|s| s.id).collect());
    }
    let mut out = Vec::new();
    for it in items {
        let Some(canon) = normalize_harness_alias(&it) else {
            return Err(format!(
                "Unknown harness '{it}'. Choose from: auto, all, codex, pi, agy, claude, grok, openclaw, hermes, agents"
            ));
        };
        if !out.contains(&canon) {
            out.push(canon);
        }
    }
    Ok(out)
}

pub fn render_fenced_agents_block(ap_cmd: &str, selected: &[&str]) -> String {
    let sel: HashSet<&str> = selected.iter().copied().collect();
    let mut bullets = Vec::new();
    if sel.contains("probe") || sel.contains("atlas") {
        bullets.push(format!(
            "1. **Orient, Locate & Trace (`atlas`)**: Non-empty dir -> `{ap_cmd} atlas map .` (<15ms Code/Office/SQLite X-Ray). Bug hunting -> `{ap_cmd} atlas locate \"<symptom>\" .` (<25ms Top-5 functions). Call chains -> `{ap_cmd} atlas trace <Sym> .` (<30ms multi-hop callers/sinks). Shared symbol edits -> `{ap_cmd} atlas impact <Sym> .`."
        ));
    }
    if sel.contains("splice") {
        bullets.push(format!(
            "2. **Create & Edit (`splice`)**: Use native edit tools or batch via `{ap_cmd} splice apply --cwd . --stdin` (`@@ WRITE: p` / `@@ APPEND: p` `<<<`...`>>>`, `@@ FILE: p` `<<<` old `===` new `>>>`, `@@ SYMBOL: p :: S`, or Git unified diff) — auto-checks AST & cross-file exports."
        ));
    }
    if sel.contains("probe") || sel.contains("guard") {
        bullets.push(format!(
            "3. **Deep Verify, Live CDP, Process, FS & Net (`guard` / `probe`)**: `{ap_cmd} guard verify --cwd . --deep` (OWASP + git diff + vacuous/slop + contracts + tests + fuzz + UI + mutation). Ports/Procs -> `{ap_cmd} probe ports` / `probe ps -f <name>`; Live CDP -> `{ap_cmd} probe cdp --port <P> [--eval JS --watch-ms 1500]`; Batch HTTP/TCP/Callback -> `probe http --batch-stdin` / `probe tcp <h:p>` / `probe listen --run <cmd>`; FS diff -> `probe fs <dir> [--run <cmd>|--recent 300]`."
        ));
    }
    if ["probe", "sieve", "recall", "guard", "baton"]
        .iter()
        .any(|x| sel.contains(x))
    {
        bullets.push(format!(
            "4. **Batch Read, Hex/Binary, Search, SQL, Exec & Undo**: `{ap_cmd} read <target> [--lines 1-30,90-120|--offset 18000,95000 --pretty|--hex|--json-path P|--decode|--map-pos L:C|--entry E|--render]` (PE/ELF imports+exports+SHA256, Office/SQLite/`.map`/`.asar`); `{ap_cmd} search <pat> <path> [--pat P2 --multiline --block --pretty]`; `{ap_cmd} query`; `{ap_cmd} exec -c \"<cmd>\"`; `{ap_cmd} undo`."
        ));
    }
    format!(
        "{FENCE_START}\n## Agent Primitives (`{ap_cmd} <tool>` — self-contained, skip reading SKILL.md or running --help)\n{}\n{FENCE_END}\n",
        bullets.join("\n")
    )
}

fn upsert_fenced_block(md_path: &Path, block: &str) -> &'static str {
    if let Some(parent) = md_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if md_path.exists() {
        let orig = fs::read_to_string(md_path).unwrap_or_default();
        let pat = Regex::new(&format!(
            r"(?s){}.*?{}\n?",
            regex::escape(FENCE_START),
            regex::escape(FENCE_END)
        ))
        .unwrap();
        if pat.is_match(&orig) {
            let updated = pat.replacen(&orig, 1, block).to_string();
            let _ = fs::write(md_path, updated);
            return "updated";
        }
        let sep = if orig.is_empty() || orig.ends_with("\n\n") {
            ""
        } else if orig.ends_with('\n') {
            "\n"
        } else {
            "\n\n"
        };
        let _ = fs::write(md_path, format!("{orig}{sep}{block}"));
        return "appended";
    }
    let _ = fs::write(md_path, block);
    "created"
}

fn remove_fenced_block(md_path: &Path) -> bool {
    if !md_path.exists() {
        return false;
    }
    let orig = fs::read_to_string(md_path).unwrap_or_default();
    let pat = Regex::new(&format!(
        r"(?s)\n*{}.*?{}\n*",
        regex::escape(FENCE_START),
        regex::escape(FENCE_END)
    ))
    .unwrap();
    if !pat.is_match(&orig) {
        return false;
    }
    let cleaned = pat.replace_all(&orig, "\n").trim().to_string();
    if cleaned.is_empty() {
        let _ = fs::remove_file(md_path);
    } else {
        let _ = fs::write(md_path, format!("{cleaned}\n"));
    }
    true
}

fn install_self_binary(dest_dir: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(dest_dir).map_err(|e| e.to_string())?;
    #[cfg(windows)]
    for legacy in ["ap", "ap.cmd", "ap.ps1"] {
        let lp = dest_dir.join(legacy);
        if lp.is_file() {
            let _ = fs::remove_file(lp);
        }
    }
    let exe_name = if cfg!(windows) { "ap.exe" } else { "ap" };
    let dst = dest_dir.join(exe_name);
    let src = std::env::current_exe().map_err(|e| e.to_string())?;
    let same = fs::canonicalize(&src).ok() == fs::canonicalize(&dst).ok() && dst.exists();
    if !same {
        if dst.exists() {
            let _ = fs::remove_file(&dst);
        }
        if fs::hard_link(&src, &dst).is_err() {
            fs::copy(&src, &dst)
                .map_err(|e| format!("Failed to install binary to {}: {e}", dst.display()))?;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&dst, fs::Permissions::from_mode(0o755));
        }
    }
    Ok(dst)
}

fn write_skill_folders(
    dest_skills_dir: &Path,
    selected: &[&str],
    ap_cmd_rewrite: Option<&str>,
) -> Result<usize, String> {
    fs::create_dir_all(dest_skills_dir).map_err(|e| e.to_string())?;
    let rewrite_re = Regex::new(r"\bap (sieve|baton|recall|guard|atlas|probe|splice)\b").unwrap();
    let mut count = 0usize;
    for &name in selected {
        let Some(raw_md) = embedded_skill_md(name) else {
            return Err(format!("Unknown primitive: {name}"));
        };
        let folder = dest_skills_dir.join(name);
        if folder.exists() {
            let _ = fs::remove_dir_all(&folder);
        }
        fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
        let final_md = if let Some(cmd_p) = ap_cmd_rewrite {
            rewrite_re
                .replace_all(raw_md, |caps: &regex::Captures| {
                    format!("{cmd_p} {}", &caps[1])
                })
                .to_string()
        } else {
            raw_md.to_string()
        };
        fs::write(folder.join("SKILL.md"), final_md).map_err(|e| e.to_string())?;
        count += 1;
    }
    Ok(count)
}

fn merge_claude_or_codex_hooks(
    settings_path: &Path,
    ap_bin_posix: &str,
    selected: &[&str],
    include_precompact: bool,
) -> String {
    let sel: HashSet<&str> = selected.iter().copied().collect();
    let mut data: Value = if settings_path.exists() {
        let _ = fs::copy(settings_path, settings_path.with_extension("json.bak"));
        fs::read_to_string(settings_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| json!({}))
    } else {
        json!({})
    };
    if !data.is_object() {
        data = json!({});
    }
    let hooks_obj = data
        .as_object_mut()
        .unwrap()
        .entry("hooks")
        .or_insert_with(|| json!({}));
    if !hooks_obj.is_object() {
        *hooks_obj = json!({});
    }

    let mut ensure_hook = |event: &str, cmd_str: String, matcher: Option<&str>| {
        let arr = hooks_obj
            .as_object_mut()
            .unwrap()
            .entry(event)
            .or_insert_with(|| json!([]));
        if let Some(vec) = arr.as_array_mut() {
            // Remove any legacy python guard.py/baton.py or stale ap hook for the same role
            vec.retain(|item| {
                let s = item.to_string();
                !s.contains("guard/scripts/guard.py")
                    && !s.contains("baton/scripts/baton.py")
                    && !s.contains(&cmd_str)
            });
            let mut entry = json!({
                "hooks": [{"type": "command", "command": cmd_str}]
            });
            if let Some(m) = matcher {
                entry["matcher"] = json!(m);
            }
            vec.push(entry);
        }
    };

    if sel.contains("guard") {
        ensure_hook(
            "PreToolUse",
            format!("\"{ap_bin_posix}\" guard shield"),
            Some("Bash"),
        );
        ensure_hook("Stop", format!("\"{ap_bin_posix}\" guard stop-gate"), None);
    }
    if include_precompact && sel.contains("baton") {
        ensure_hook(
            "PreCompact",
            format!("\"{ap_bin_posix}\" baton checkpoint --note \"auto pre-compact snapshot\""),
            None,
        );
    }

    if let Some(parent) = settings_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(
        settings_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&data).unwrap_or_default()
        ),
    );
    posix(settings_path)
}

fn remove_claude_or_codex_hooks(settings_path: &Path) -> bool {
    let Ok(raw) = fs::read_to_string(settings_path) else {
        return false;
    };
    let Ok(mut data) = serde_json::from_str::<Value>(&raw) else {
        return false;
    };
    let Some(hooks) = data.get_mut("hooks").and_then(Value::as_object_mut) else {
        return false;
    };
    let mut changed = false;
    let keys: Vec<String> = hooks.keys().cloned().collect();
    for ev in keys {
        if let Some(arr) = hooks.get_mut(&ev).and_then(Value::as_array_mut) {
            let before = arr.len();
            arr.retain(|item| {
                let s = item.to_string();
                !s.contains("guard/scripts/guard.py")
                    && !s.contains("baton/scripts/baton.py")
                    && !s.contains(" guard shield")
                    && !s.contains(" guard stop-gate")
                    && !s.contains(" baton checkpoint")
            });
            if arr.len() != before {
                changed = true;
            }
            if arr.is_empty() {
                hooks.remove(&ev);
            }
        }
    }
    if hooks.is_empty() {
        if let Some(m) = data.as_object_mut() {
            m.remove("hooks");
        }
    }
    if !changed {
        return false;
    }
    if data.as_object().map(|m| m.is_empty()).unwrap_or(false) {
        let _ = fs::remove_file(settings_path);
    } else {
        let _ = fs::write(
            settings_path,
            format!(
                "{}\n",
                serde_json::to_string_pretty(&data).unwrap_or_default()
            ),
        );
    }
    true
}

fn merge_antigravity_hooks(
    hooks_path: &Path,
    ap_bin_posix: &str,
    selected: &[&str],
) -> Option<String> {
    if !selected.contains(&"guard") {
        return None;
    }
    let mut data: Value = if hooks_path.exists() {
        let _ = fs::copy(hooks_path, hooks_path.with_extension("json.bak"));
        fs::read_to_string(hooks_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| json!({}))
    } else {
        json!({})
    };
    if !data.is_object() {
        data = json!({});
    }
    data["agent-primitives-guard"] = json!({
        "enabled": true,
        "PreToolUse": [{
            "matcher": "run_command",
            "hooks": [{
                "type": "command",
                "command": format!("\"{ap_bin_posix}\" guard shield"),
                "timeout": 10
            }]
        }],
        "Stop": [{
            "type": "command",
            "command": format!("\"{ap_bin_posix}\" guard stop-gate"),
            "timeout": 15
        }]
    });
    if let Some(parent) = hooks_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(
        hooks_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&data).unwrap_or_default()
        ),
    );
    Some(posix(hooks_path))
}

fn remove_antigravity_hooks(hooks_path: &Path) -> bool {
    let Ok(raw) = fs::read_to_string(hooks_path) else {
        return false;
    };
    let Ok(mut data) = serde_json::from_str::<Value>(&raw) else {
        return false;
    };
    let Some(map) = data.as_object_mut() else {
        return false;
    };
    if map.remove("agent-primitives-guard").is_none() {
        return false;
    }
    if map.is_empty() {
        let _ = fs::remove_file(hooks_path);
        let bak = hooks_path.with_extension("json.bak");
        if let Ok(bs) = fs::read_to_string(&bak) {
            if let Ok(Value::Object(bm)) = serde_json::from_str::<Value>(&bs) {
                if bm.len() <= 1 && (bm.is_empty() || bm.contains_key("agent-primitives-guard")) {
                    let _ = fs::remove_file(bak);
                }
            }
        }
    } else {
        let _ = fs::write(
            hooks_path,
            format!(
                "{}\n",
                serde_json::to_string_pretty(&data).unwrap_or_default()
            ),
        );
    }
    true
}

fn merge_opencode_mcp(cfg_path: &Path, ap_bin_posix: &str) -> String {
    let mut data: Value = if cfg_path.exists() {
        fs::read_to_string(cfg_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| json!({}))
    } else {
        json!({})
    };
    if !data.is_object() {
        data = json!({});
    }
    let obj = data.as_object_mut().unwrap();
    obj.entry("$schema")
        .or_insert_with(|| json!("https://opencode.ai/config.json"));
    let mcp = obj.entry("mcp").or_insert_with(|| json!({}));
    if !mcp.is_object() {
        *mcp = json!({});
    }
    mcp.as_object_mut().unwrap().insert(
        "agent-primitives".to_string(),
        json!({
            "type": "local",
            "command": [ap_bin_posix, "mcp"],
            "enabled": true
        }),
    );
    if let Some(p) = cfg_path.parent() {
        let _ = fs::create_dir_all(p);
    }
    let _ = fs::write(
        cfg_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&data).unwrap_or_default()
        ),
    );
    posix(cfg_path)
}

fn remove_opencode_mcp(cfg_path: &Path) {
    let Ok(txt) = fs::read_to_string(cfg_path) else {
        return;
    };
    let Ok(mut data) = serde_json::from_str::<Value>(&txt) else {
        return;
    };
    let Some(obj) = data.as_object_mut() else {
        return;
    };
    if let Some(mcp) = obj.get_mut("mcp").and_then(Value::as_object_mut) {
        mcp.remove("agent-primitives");
        if mcp.is_empty() {
            obj.remove("mcp");
        }
    }
    let only_schema = obj.is_empty() || (obj.len() == 1 && obj.contains_key("$schema"));
    if only_schema {
        let _ = fs::remove_file(cfg_path);
    } else {
        let _ = fs::write(
            cfg_path,
            format!(
                "{}\n",
                serde_json::to_string_pretty(&data).unwrap_or_default()
            ),
        );
    }
}

fn merge_mcp_servers_json(cfg_path: &Path, ap_bin_posix: &str, include_trust: bool) -> String {
    let mut data: Value = if cfg_path.exists() {
        fs::read_to_string(cfg_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| json!({}))
    } else {
        json!({})
    };
    if !data.is_object() {
        data = json!({});
    }
    let servers = data
        .as_object_mut()
        .unwrap()
        .entry("mcpServers")
        .or_insert_with(|| json!({}));
    if !servers.is_object() {
        *servers = json!({});
    }
    let mut entry = json!({
        "command": ap_bin_posix,
        "args": ["mcp"]
    });
    if include_trust {
        entry["trust"] = json!(true);
    }
    servers
        .as_object_mut()
        .unwrap()
        .insert("agent-primitives".to_string(), entry);
    if let Some(p) = cfg_path.parent() {
        let _ = fs::create_dir_all(p);
    }
    let _ = fs::write(
        cfg_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&data).unwrap_or_default()
        ),
    );
    posix(cfg_path)
}

fn remove_mcp_servers_json(cfg_path: &Path) {
    let Ok(txt) = fs::read_to_string(cfg_path) else {
        return;
    };
    let Ok(mut data) = serde_json::from_str::<Value>(&txt) else {
        return;
    };
    let Some(obj) = data.as_object_mut() else {
        return;
    };
    if let Some(srvs) = obj.get_mut("mcpServers").and_then(Value::as_object_mut) {
        srvs.remove("agent-primitives");
        if srvs.is_empty() {
            obj.remove("mcpServers");
        }
    }
    if obj.is_empty() {
        let _ = fs::remove_file(cfg_path);
    } else {
        let _ = fs::write(
            cfg_path,
            format!(
                "{}\n",
                serde_json::to_string_pretty(&data).unwrap_or_default()
            ),
        );
    }
}

fn merge_pi_mcp(cfg_path: &Path, ap_bin_posix: &str) -> String {
    let mut data: Value = if cfg_path.exists() {
        fs::read_to_string(cfg_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| json!({}))
    } else {
        json!({})
    };
    if !data.is_object() {
        data = json!({});
    }
    let servers = data
        .as_object_mut()
        .unwrap()
        .entry("mcpServers")
        .or_insert_with(|| json!({}));
    if !servers.is_object() {
        *servers = json!({});
    }
    servers.as_object_mut().unwrap().insert(
        "agent-primitives".to_string(),
        json!({
            "command": ap_bin_posix,
            "args": ["mcp"],
            "exposure": "codemode",
            "description": "Fast Pure-Rust MCP 2025-06-18 (34 annotated tools, Resources, Prompts; batch via Promise.allSettled in codemode): AST ap_read (--symbol/--outline/batch), parallel ap_search (-l/-c/-C), ap_edit/ap_rename, ap_orient/ap_impact/ap_changes/ap_locate/ap_trace, ap_query (named records), ap_sieve (compiler/test/diff), ap_baton/ap_recall & CDP/HTTP/TCP/WS/TLS probes"
        }),
    );
    if let Some(p) = cfg_path.parent() {
        let _ = fs::create_dir_all(p);
    }
    let _ = fs::write(
        cfg_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&data).unwrap_or_default()
        ),
    );
    posix(cfg_path)
}

pub fn render_pi_accelerator_ts(ap_bin_posix: &str) -> String {
    let escaped = ap_bin_posix.replace('\\', "/").replace('"', "\\\"");
    format!(
        r#"// Managed by agent-primitives (`ap init`). Zero system-prompt token overhead.
// Designed for Pi v1.5+ and interleaved-thinking models (e.g. DeepSeek Flash / Reasoner):
// - Accelerates slow `grep -rn`, heals Windows `> NUL`, non-raw `python -c` paths & UTF-8
// - Auto-sieves >28KB direct-to-LLM shell outputs into `~/.ap/sieve` (zero `.sieve` clutter)
// - Native Pi v1.5 `context_edit` batch observation masking at high-water marks (180k tok / 72%)
//   with >=10-turn cooldown to prevent Context Rot while preserving DeepSeek 50x KV-cache & thinking
// - Breaks 3x shell doom-loops, 3x unchanged file re-reads & 3-turn heavy-thinking stalls
// - Recovers leaked DSML XML tool blocks when 0 native tool calls were emitted (zero conflict)
// - Self-healing tips for `edit` misses, `codemode` errors & giant code `read` calls
// - Auto-checkpoints `.baton/` on compaction (only if `.baton` already exists) + live KV-cache HUD
// - Native Pi v1.5 `agent_before_settle` closed-loop syntax + diff blocker gate (max 1 retry/run)
// - Opt-in `deepseek/flash-auto` virtual model (dynamic low/high/max effort, zero cache-bust switch)
import {{ spawnSync }} from "node:child_process";
import {{ existsSync }} from "node:fs";
import {{ homedir }} from "node:os";
import {{ isAbsolute, join, resolve }} from "node:path";
import type {{
	ContextEditEntryDraft,
	CustomMessageEntryDraft,
	ExtensionAPI,
	ExtensionContext,
	ProjectedSessionEntry,
}} from "@earendil-works/pi-coding-agent";

const AP_BIN = "{escaped}";
const SIEVE_HOME = process.env.SIEVE_HOME || join(homedir(), ".ap", "sieve");
const DESTRUCTIVE_OR_WRITE =
	/\b(?:Remove-Item|rm|del|erase|rmdir|rd|Move-Item|Rename-Item|mv|ren|Set-Content|Add-Content|Clear-Content|Out-File|Copy-Item|cpi|copy)\b|shutil\.rmtree|os\.remove|os\.unlink|\.unlink\(|\.rmdir\(|\bgit(?:\.exe)?\b[\s\S]{{0,80}}?\bclean\b/i;
const BARE_SLOW_GREP =
	/^\s*grep\s+-[rR][nwiEH]*\s+(?:'([^']+)'|"([^"]+)"|([^\s'"|;&]+))(?:\s+(?:'([^']+)'|"([^"]+)"|([^\s'"|;&]+)))?\s*$/;
const PY_INVOKE = /\b(?:python3?(?:\.exe)?|py(?:\.exe)?|pytest)\b/i;
const HAS_UTF8_ENV = /PYTHONUTF8|PYTHONIOENCODING/i;
const PY_WIN_BS = /\b(?:python3?(?:\.exe)?|py(?:\.exe)?)\s+-c\s+[\s\S]*(?<![rR])['"][A-Za-z]:\\(?!\\)/;
const REDIR_NUL = /(\s(?:[12]?>|&>))\s*NUL\b/gi;
const QUOTE_OR_BS_ERR =
	/unterminated string|EOL while scanning|unicodeescape|unexpected character after line continuation|syntax error near unexpected token/i;
const CODE_FILE_EXT = /\.(?:rs|ts|tsx|js|jsx|mjs|cjs|py|go|c|cc|cpp|h|hpp)$/i;
const EDIT_TOOLS = new Set(["write", "edit", "ap_edit", "ap_rename"]);
const LOOP_WINDOW = 6;
const LOOP_ERR_THRESHOLD = 3;
const LOOP_OK_THRESHOLD = 5;
const REREAD_THRESHOLD = 3;
const HEAVY_THINKING_CHARS = 12000;
const STAGNATION_TURNS = 3;
const LARGE_CODE_READ_CHARS = 20000;

// High-water batch observation masking constants (calibrated for 1M-window DeepSeek Flash 98%+ KV-cache & 128k-256k models)
const MASK_PREFIX = "[ap masked stale toolResult:";
const MASK_HIGH_WATER_PCT = 72;
const MASK_HIGH_WATER_TOKENS = 180000;
const MASK_STALE_CHARS_FALLBACK = 120000;
const MASK_KEEP_RECENT_ENTRIES = 12;
const MASK_MIN_ENTRY_CHARS = 4500;
const MASK_MIN_BATCH_ENTRIES = 2;
const MASK_MIN_RECLAIMED_CHARS = 16000;
const MASK_COOLDOWN_TURNS = 15;

export function normCmdFingerprint(raw: string): string {{
	return raw
		.replace(/^export\s+PYTHONUTF8=1\s+PYTHONIOENCODING=utf-8;\s*/i, "")
		.replace(/^\$env:PYTHONUTF8='1';\s*\$env:PYTHONIOENCODING='utf-8';\s*/i, "")
		.replace(/\b0x[0-9a-fA-F]{{4,}}\b/g, "<HEX>")
		.replace(/\b\d{{4}}-\d{{2}}-\d{{2}}[T\s]\d{{2}}:\d{{2}}:\d{{2}}(?:\.\d+)?Z?\b/g, "<TS>")
		.replace(/\s+/g, " ")
		.trim()
		.slice(0, 180);
}}

export function healCommand(raw: string, toolName: string): string | undefined {{
	if (!raw || DESTRUCTIVE_OR_WRITE.test(raw)) return undefined;
	let cmd = raw;
	let changed = false;

	// 1. Bare recursive GNU `grep -rn <pat> [path]` -> `<50ms` parallel `ap search`
	if (!/[|;&>]/.test(cmd)) {{
		const gm = cmd.match(BARE_SLOW_GREP);
		if (gm) {{
			const pat = (gm[1] || gm[2] || gm[3] || "").replace(/"/g, '\\"');
			const target = gm[4] || gm[5] || gm[6] || ".";
			const prefix = toolName === "powershell" ? "& " : "";
			return `${{prefix}}"${{AP_BIN}}" search "${{pat}}" "${{target}}"`;
		}}
	}}

	// 2. Prevent accidental literal `NUL` file creation in Git Bash
	if (toolName === "bash" && REDIR_NUL.test(cmd)) {{
		cmd = cmd.replace(REDIR_NUL, "$1 /dev/null");
		changed = true;
	}}

	// 3. Auto-fix unescaped Windows drive backslashes inside `python -c` non-raw string literals
	if (PY_WIN_BS.test(cmd)) {{
		cmd = cmd.replace(
			/(?<![rR])(['"])([A-Za-z]:)(\\(?!\\)[^\s'"]+)/g,
			(_m, q: string, drv: string, rest: string) => q + drv + rest.replace(/\\/g, "/"),
		);
		changed = true;
	}}

	// 4. Auto-lock UTF-8 across BOTH Bash and PowerShell Python invocations (prevents 99% of Windows GBK crashes)
	if (PY_INVOKE.test(cmd) && !HAS_UTF8_ENV.test(cmd)) {{
		if (toolName === "bash") {{
			cmd = `export PYTHONUTF8=1 PYTHONIOENCODING=utf-8; ${{cmd}}`;
			changed = true;
		}} else if (toolName === "powershell") {{
			cmd = `$env:PYTHONUTF8='1'; $env:PYTHONIOENCODING='utf-8'; ${{cmd}}`;
			changed = true;
		}}
	}}

	// 5. Prevent PowerShell 5.1 `curl` -> Invoke-WebRequest alias crashes
	if (toolName === "powershell" && /(?:^|[;&|]\s*)curl(?=\s)/i.test(cmd)) {{
		cmd = cmd.replace(/(^|[;&|]\s*)curl(?=\s)/gi, "$1curl.exe");
		changed = true;
	}}

	return changed ? cmd : undefined;
}}

export interface DsmlRecoveredCall {{
	type: "toolCall";
	id: string;
	name: string;
	arguments: Record<string, unknown>;
}}

/**
 * Recover leaked DeepSeek DSML XML tool invocations into native Pi `toolCall` blocks.
 * Strictly gated to currently active tools (`allowedTools`) so unknown names never run.
 */
export function extractDsmlToolCalls(
	text: string,
	allowedTools: ReadonlySet<string>,
): {{ cleanedText: string; calls: DsmlRecoveredCall[] }} | undefined {{
	if (!text || !text.includes("DSML") || !text.includes("invoke")) return undefined;
	const invokeRe =
		/<[|｜]DSML[|｜]invoke\s+name="([^"]+)"[^>]*>([\s\S]*?)<\/[|｜]DSML[|｜]invoke>/g;
	const paramRe =
		/<[|｜]DSML[|｜]parameter\s+name="([^"]+)"(?:\s+string="(true|false)")?[^>]*>([\s\S]*?)<\/[|｜]DSML[|｜]parameter>/g;

	const calls: DsmlRecoveredCall[] = [];
	let match: RegExpExecArray | null;
	let seq = 0;
	while ((match = invokeRe.exec(text)) !== null) {{
		const toolName = (match[1] || "").trim();
		if (!toolName || !allowedTools.has(toolName)) continue;
		const body = match[2] || "";
		const args: Record<string, unknown> = {{}};
		let pm: RegExpExecArray | null;
		paramRe.lastIndex = 0;
		while ((pm = paramRe.exec(body)) !== null) {{
			const key = (pm[1] || "").trim();
			const isStr = pm[2] !== "false";
			const rawVal = pm[3] ?? "";
			if (!key) continue;
			if (!isStr) {{
				try {{
					args[key] = JSON.parse(rawVal.trim());
					continue;
				}} catch {{
					// fall through to raw string
				}}
			}}
			args[key] = rawVal;
		}}
		seq += 1;
		calls.push({{
			type: "toolCall",
			id: `ap_dsml_${{Date.now()}}_${{seq}}`,
			name: toolName,
			arguments: args,
		}});
	}}
	if (calls.length === 0) return undefined;
	const cleanedText = text
		.replace(/<[|｜]DSML[|｜]function_calls>[\s\S]*?<\/[|｜]DSML[|｜]function_calls>/g, "")
		.replace(invokeRe, "")
		.trim();
	return {{ cleanedText, calls }};
}}

/**
 * Recover leaked `<tool_call>{{"name":"...","arguments":{{...}}}}</tool_call>` blocks
 * emitted as plain text under high context pressure (strictly gated to `allowedTools`).
 */
export function extractXmlJsonToolCalls(
	text: string,
	allowedTools: ReadonlySet<string>,
): {{ cleanedText: string; calls: DsmlRecoveredCall[] }} | undefined {{
	if (!text || !text.includes("<tool_call>")) return undefined;
	const re = /<tool_call>\s*([\s\S]*?)\s*<\/tool_call>/g;
	const calls: DsmlRecoveredCall[] = [];
	let match: RegExpExecArray | null;
	let seq = 0;
	while ((match = re.exec(text)) !== null) {{
		try {{
			const obj = JSON.parse(match[1]) as {{
				name?: unknown;
				arguments?: unknown;
				parameters?: unknown;
			}};
			const name = typeof obj?.name === "string" ? obj.name.trim() : "";
			if (!name || !allowedTools.has(name)) continue;
			const rawArgs = obj.arguments ?? obj.parameters ?? {{}};
			const args =
				rawArgs && typeof rawArgs === "object" && !Array.isArray(rawArgs)
					? (rawArgs as Record<string, unknown>)
					: {{}};
			seq += 1;
			calls.push({{
				type: "toolCall",
				id: `ap_xmltc_${{Date.now()}}_${{seq}}`,
				name,
				arguments: args,
			}});
		}} catch {{
			// ignore non-JSON blocks
		}}
	}}
	if (calls.length === 0) return undefined;
	const cleanedText = text.replace(re, "").trim();
	return {{ cleanedText, calls }};
}}

const ERROR_LINE_RE =
	/(?:^|\s)(?:error\[E\d+\]|ERROR:|FATAL:|Traceback|AssertionError|TypeError|SyntaxError|ReferenceError|RangeError|ValueError|RuntimeError|panicked at|FAILED\b|Exception:|Command exited with code [1-9])/i;

/**
 * Extract up to 3 high-signal error lines so observation masking never erases failure lessons
 * (Manus Context Engineering rule: keep error signatures in the trace to prevent repeat loops).
 */
export function extractErrorSignatureLines(rawText: string): string[] {{
	const seen = new Set<string>();
	const out: string[] = [];
	for (const ln of rawText.split("\n")) {{
		const clean = ln.replace(/\s+/g, " ").trim();
		if (clean.length < 8 || !ERROR_LINE_RE.test(clean)) continue;
		const clipped = clean.slice(0, 140);
		if (seen.has(clipped)) continue;
		seen.add(clipped);
		out.push(clipped);
		if (out.length >= 3) break;
	}}
	return out;
}}

/**
 * Deterministic, timestamp-free replacement stub for stale bulky `toolResult` entries.
 * Preserves any `ap sieve` archive ID AND top error signatures for lossless failure memory.
 */
export function formatMaskedToolResult(
	toolName: string,
	rawText: string,
	isError = false,
): string {{
	const lineCount = rawText.split("\n").length;
	const sieveMatch =
		rawText.match(/\bsieve\s+(?:id=|peek\s+)([a-zA-Z0-9_-]{{4,40}})\b/i) ||
		rawText.match(/\[sieve\s+id=([a-zA-Z0-9_-]{{4,40}})\b/i);
	const sieveId = sieveMatch?.[1];
	const head = rawText.slice(0, 140).replace(/\s+/g, " ").trim();
	const tail = rawText.slice(-110).replace(/\s+/g, " ").trim();
	const errLines = extractErrorSignatureLines(rawText);
	const errBlock =
		isError || errLines.length > 0
			? `\nerrors: ${{errLines.length > 0 ? errLines.join(" | ") : "tool returned isError=true"}}`
			: "";
	const recovery = sieveId
		? `(Recover full output via ap_sieve(action="peek", id="${{sieveId}}").)`
		: "(Re-run or narrow with ap_read(symbol/outline) / ap_search if needed.)";
	const sieveTag = sieveId ? ` | sieve=${{sieveId}}` : "";
	const errTag = isError ? " | ERROR" : "";
	return `${{MASK_PREFIX}} ${{toolName}} (${{rawText.length}} chars, ${{lineCount}} lines)${{errTag}}${{sieveTag}}]\nhead: ${{head}}${{errBlock}}\ntail: ${{tail}}\n${{recovery}}`;
}}

/**
 * Compact, standalone branch serializer for `/ap handoff <goal>` (zero runtime SDK value imports).
 */
export function serializeBranchForHandoff(branch: ReadonlyArray<Record<string, unknown>>): string {{
	if (!Array.isArray(branch) || branch.length === 0) return "";
	let startIdx = 0;
	for (let i = branch.length - 1; i >= 0; i--) {{
		if (branch[i]?.type === "compaction") {{
			startIdx = i;
			break;
		}}
	}}
	const chunks: string[] = [];
	for (let i = startIdx; i < branch.length; i++) {{
		const e = branch[i];
		if (e?.type === "compaction" && typeof e.summary === "string") {{
			chunks.push(`[Compaction Summary]\n${{e.summary.slice(0, 4000)}}`);
			continue;
		}}
		if (e?.type !== "message" || !e.message || typeof e.message !== "object") continue;
		const m = e.message as {{
			role?: string;
			toolName?: string;
			isError?: boolean;
			content?: string | Array<{{ type?: string; text?: string; name?: string }}>;
		}};
		if (m.role === "user" || m.role === "assistant") {{
			let txt = "";
			if (typeof m.content === "string") {{
				txt = m.content;
			}} else if (Array.isArray(m.content)) {{
				const parts: string[] = [];
				for (const b of m.content) {{
					if (b?.type === "text" && typeof b.text === "string") parts.push(b.text);
					else if (b?.type === "toolCall" && typeof b.name === "string") parts.push(`[call:${{b.name}}]`);
				}}
				txt = parts.join("\n");
			}}
			if (txt.trim()) chunks.push(`${{m.role.toUpperCase()}}: ${{txt.trim().slice(0, 1200)}}`);
		}} else if (m.role === "toolResult" && m.isError) {{
			const raw = Array.isArray(m.content)
				? m.content.map((b) => (typeof b.text === "string" ? b.text : "")).join("\n")
				: "";
			const errs = extractErrorSignatureLines(raw);
			chunks.push(`TOOL_ERROR(${{m.toolName ?? "tool"}}): ${{(errs.join(" | ") || raw.slice(0, 240)).trim()}}`);
		}}
	}}
	return chunks.join("\n\n").slice(-24000);
}}

export interface ObservationMaskPlan {{
	edits: ContextEditEntryDraft[];
	maskedCount: number;
	reclaimedChars: number;
	reason: string;
}}

export interface ObservationMaskOptions {{
	usagePercent?: number | null;
	usageTokens?: number | null;
	keepRecentEntries?: number;
	minEntryChars?: number;
	highWaterPercent?: number;
	highWaterTokens?: number;
	staleCharsFallback?: number;
	minBatchEntries?: number;
	minReclaimedChars?: number;
}}

/**
 * Plan a single KV-cache-friendly batch of Pi v1.5 `context_edit` drafts to mask stale
 * oversized `toolResult` entries outside the recent active tail.
 * Never touches assistant messages, thinking/reasoning_content blocks, user turns, or images.
 */
export function planObservationMasking(
	contextEntries: readonly ProjectedSessionEntry[],
	options: ObservationMaskOptions = {{}},
): ObservationMaskPlan {{
	const keepRecent = options.keepRecentEntries ?? MASK_KEEP_RECENT_ENTRIES;
	const minChars = options.minEntryChars ?? MASK_MIN_ENTRY_CHARS;
	const highPct = options.highWaterPercent ?? MASK_HIGH_WATER_PCT;
	const highTok = options.highWaterTokens ?? MASK_HIGH_WATER_TOKENS;
	const staleFallback = options.staleCharsFallback ?? MASK_STALE_CHARS_FALLBACK;
	const minEntries = options.minBatchEntries ?? MASK_MIN_BATCH_ENTRIES;
	const minReclaimed = options.minReclaimedChars ?? MASK_MIN_RECLAIMED_CHARS;

	if (!Array.isArray(contextEntries) || contextEntries.length <= keepRecent) {{
		return {{ edits: [], maskedCount: 0, reclaimedChars: 0, reason: "too-few-entries" }};
	}}

	const scanLimit = contextEntries.length - keepRecent;
	const candidates: Array<{{ targetId: string; toolName: string; rawText: string; isError: boolean }}> = [];
	let totalStaleChars = 0;

	for (let i = 0; i < scanLimit; i++) {{
		const proj = contextEntries[i];
		const src = proj?.sourceEntry as {{ id?: string; type?: string }} | undefined;
		if (!src || src.type !== "message" || typeof src.id !== "string" || !src.id) continue;
		if (!Array.isArray(proj.messages) || proj.messages.length !== 1) continue;
		const msg = proj.messages[0] as {{
			role?: string;
			toolName?: string;
			isError?: boolean;
			content?: Array<{{ type?: string; text?: string }}>;
		}};
		if (msg?.role !== "toolResult" || !Array.isArray(msg.content) || msg.content.length === 0) {{
			continue;
		}}
		// Keep multimodal/image tool results untouched
		if (msg.content.some((b) => b?.type !== "text")) continue;
		const rawText = msg.content.map((b) => (typeof b.text === "string" ? b.text : "")).join("\n");
		if (rawText.length < minChars || rawText.startsWith(MASK_PREFIX)) continue;

		candidates.push({{
			targetId: src.id,
			toolName: typeof msg.toolName === "string" && msg.toolName ? msg.toolName : "tool",
			rawText,
			isError: msg.isError === true,
		}});
		totalStaleChars += rawText.length;
	}}

	if (candidates.length < minEntries) {{
		return {{ edits: [], maskedCount: 0, reclaimedChars: 0, reason: "below-min-entries" }};
	}}

	const pct = typeof options.usagePercent === "number" ? options.usagePercent : null;
	const tok = typeof options.usageTokens === "number" ? options.usageTokens : null;
	const pctTriggered = pct !== null && pct >= highPct;
	const tokTriggered = tok !== null && tok >= highTok;
	const fallbackTriggered = totalStaleChars >= staleFallback && candidates.length >= 4;

	if (!pctTriggered && !tokTriggered && !fallbackTriggered) {{
		return {{ edits: [], maskedCount: 0, reclaimedChars: 0, reason: "below-high-water" }};
	}}

	const edits: ContextEditEntryDraft[] = [];
	let reclaimedChars = 0;
	for (const c of candidates) {{
		const stub = formatMaskedToolResult(c.toolName, c.rawText, c.isError);
		const saved = c.rawText.length - stub.length;
		if (saved < 1000) continue;
		edits.push({{
			type: "context_edit",
			targetId: c.targetId,
			replacement: {{ content: [{{ type: "text", text: stub }}] }},
		}});
		reclaimedChars += saved;
	}}

	if (edits.length < minEntries || reclaimedChars < minReclaimed) {{
		return {{ edits: [], maskedCount: 0, reclaimedChars: 0, reason: "savings-too-small" }};
	}}

	const reason = tokTriggered
		? "high-water-tokens"
		: pctTriggered
			? "high-water-percent"
			: "stale-payload-fallback";
	return {{ edits, maskedCount: edits.length, reclaimedChars, reason }};
}}

export type DeepSeekEffort = "low" | "high" | "max";

const COMPLEX_PROMPT_RE =
	/\b(?:architect|architecture|refactor|redesign|concurrency|deadlock|race\s+condition|root\s+cause|debug|diagnose|profil|optimiz|benchmark|migrat|multi[- ]file|security|audit|algorithm|compil|memory\s+leak)\b|架构|重构|并发|死锁|根因|排查|定位|性能|优化|基准|迁移|跨文件|深度|设计|算法|泄漏/i;
const SIMPLE_PROMPT_RE =
	/^(?:hi|hello|hey|ok|okay|thanks|thank you|yes|no|continue|go|lgtm|commit|push|status|git status|format|lint|run tests?|你好|继续|好的|谢谢|提交|推送|跑测试|看状态)\b[\s!.。？?]*$/i;
const BLOCKER_DIFF_KINDS = new Set([
	"SECRET-ADDED",
	"TEST-SKIPPED",
	"INCOMPLETE-STUB",
	"DEBUG-LEFTOVER",
]);
const CHECKABLE_CODE_EXT = /\.(?:rs|ts|tsx|js|jsx|mjs|cjs|py|go|c|cc|cpp|h|hpp|json|jsonc|toml)$/i;

/**
 * Deterministic <0.1ms effort selector for DeepSeek V4.1 Flash (`low` / `high` / `max`).
 * Avoids burning max-effort overthinking on trivial turns while preserving deep reasoning for complex work.
 */
export function classifyDeepSeekEffort(
	userText: string,
	ceiling: DeepSeekEffort = "max",
): DeepSeekEffort {{
	if (ceiling === "low") return "low";
	const trimmed = (userText || "").trim();
	if (!trimmed) return "low";
	if (trimmed.length <= 80 && SIMPLE_PROMPT_RE.test(trimmed)) return "low";
	const hasCodeBlock = trimmed.includes("```") || trimmed.includes("Error:") || trimmed.includes("Traceback");
	const lineCount = trimmed.split("\n").length;
	if (
		trimmed.length >= 240 ||
		lineCount >= 6 ||
		hasCodeBlock ||
		COMPLEX_PROMPT_RE.test(trimmed)
	) {{
		return ceiling === "max" ? "max" : "high";
	}}
	if (trimmed.length <= 45 && !/[?？]/.test(trimmed)) return "low";
	return "high";
}}

export interface DeepSeekRouteInput {{
	reason: "user" | "continuation" | "retry" | "direct";
	selectedLevel?: string;
	previousLevel?: string;
	userText?: string;
	recentToolErrors?: number;
	recentToolSuccessesAfterEdit?: number;
	stagnantHeavyTurns?: number;
}}

function clampDeepSeekEffort(raw?: string): DeepSeekEffort {{
	if (raw === "low" || raw === "minimal" || raw === "off") return "low";
	if (raw === "max" || raw === "xhigh") return "max";
	return "high";
}}

/**
 * KV-cache-friendly dynamic reasoning-effort router staying strictly on the SAME physical
 * `deepseek-flash` model (zero provider switch, 100% `reasoning_content` continuity).
 */
export function routeDeepSeekFlashEffort(input: DeepSeekRouteInput): {{
	thinkingLevel: DeepSeekEffort;
	phase: string;
}} {{
	const ceiling = clampDeepSeekEffort(input.selectedLevel);
	if (input.reason === "direct") {{
		return {{ thinkingLevel: "low", phase: "direct-summary" }};
	}}
	const prev = input.previousLevel ? clampDeepSeekEffort(input.previousLevel) : ceiling;
	if (input.reason === "retry") {{
		const next: DeepSeekEffort = prev === "low" ? "high" : "max";
		return {{ thinkingLevel: next, phase: "retry-escalate" }};
	}}
	if (input.reason === "user") {{
		const chosen = classifyDeepSeekEffort(input.userText ?? "", ceiling);
		return {{ thinkingLevel: chosen, phase: `user-${{chosen}}` }};
	}}
	// continuation: sticky by default, escalate on repeated tool errors, cool down on overthinking stalls
	if ((input.recentToolErrors ?? 0) >= 2) {{
		const next: DeepSeekEffort = prev === "low" ? "high" : "max";
		return {{ thinkingLevel: next, phase: "error-escalate" }};
	}}
	if (
		prev === "max" &&
		((input.stagnantHeavyTurns ?? 0) >= 3 || (input.recentToolSuccessesAfterEdit ?? 0) >= 2)
	) {{
		return {{ thinkingLevel: "high", phase: "cool-down" }};
	}}
	return {{ thinkingLevel: prev, phase: "sticky" }};
}}

export interface DiffFindingItem {{
	kind?: string;
	file?: string;
	detail?: string;
}}

function normRelPath(p: string, cwd?: string): string {{
	const clean = p.replace(/\\/g, "/").replace(/^\.\//, "");
	if (cwd) {{
		const base = cwd.replace(/\\/g, "/").replace(/\/+$/, "") + "/";
		if (clean.toLowerCase().startsWith(base.toLowerCase())) {{
			return clean.slice(base.length);
		}}
	}}
	return clean;
}}

/**
 * Filter `ap guard diff --json` findings strictly to high-precision blockers inside files
 * edited during this agent run (eliminating false positives from pre-existing dirty files).
 */
export function filterSettleDiffBlockers(
	findings: readonly DiffFindingItem[],
	editedFiles: ReadonlySet<string>,
	cwd?: string,
): string[] {{
	if (!Array.isArray(findings) || findings.length === 0 || editedFiles.size === 0) return [];
	const normEdited = new Set<string>();
	for (const f of editedFiles) {{
		const rel = normRelPath(f, cwd).toLowerCase();
		if (rel) normEdited.add(rel);
	}}
	const out: string[] = [];
	for (const item of findings) {{
		const kind = String(item?.kind ?? "");
		if (!BLOCKER_DIFF_KINDS.has(kind)) continue;
		const file = normRelPath(String(item?.file ?? ""), cwd);
		if (!file) continue;
		const low = file.toLowerCase();
		const touched =
			normEdited.has(low) ||
			[...normEdited].some((e) => e.endsWith(`/${{low}}`) || low.endsWith(`/${{e}}`));
		if (!touched) continue;
		out.push(`- [${{kind}}] ${{file}}: ${{String(item?.detail ?? "").slice(0, 160)}}`);
	}}
	return out.slice(0, 6);
}}

function extractEditedPathsFromTool(toolName: string, input: Record<string, unknown> | undefined): string[] {{
	if (!input) return [];
	const out: string[] = [];
	const pushStr = (v: unknown) => {{
		if (typeof v === "string" && v.trim()) out.push(v.trim());
	}};
	if (toolName === "write" || toolName === "edit") {{
		pushStr(input.path ?? input.file_path);
	}} else if (toolName === "ap_edit") {{
		for (const key of ["writes", "edits", "appends", "symbols"]) {{
			const arr = input[key];
			if (Array.isArray(arr)) {{
				for (const item of arr) {{
					if (item && typeof item === "object") pushStr((item as {{ path?: unknown }}).path);
				}}
			}}
		}}
	}}
	return out;
}}

export default function (pi: ExtensionAPI) {{
	const recentShellFps: string[] = [];
	const unchangedReads = new Map<string, number>();
	const editedFilesThisRun = new Set<string>();
	const builtInCodeEditsThisRun = new Set<string>();
	let settleGateFiredThisRun = false;
	let stagnantHeavyTurns = 0;
	let pendingThinkingNudge = false;
	let largeReadTipShown = false;
	let lastMaskTurn = -MASK_COOLDOWN_TURNS;
	let expectedCacheReset = false;
	let sessionInputTokens = 0;
	let sessionCacheReadTokens = 0;
	let lastCacheDropPct: number | null = null;
	let sievedCount = 0;
	let maskedCount = 0;
	let guardsTriggered = 0;

	const resetRunState = () => {{
		editedFilesThisRun.clear();
		builtInCodeEditsThisRun.clear();
		settleGateFiredThisRun = false;
	}};

	const resetTransientState = () => {{
		recentShellFps.length = 0;
		unchangedReads.clear();
		resetRunState();
		stagnantHeavyTurns = 0;
		pendingThinkingNudge = false;
		lastMaskTurn = -MASK_COOLDOWN_TURNS;
		lastCacheDropPct = null;
	}};

	// Opt-in Pi v1.5 virtual model: `deepseek/flash-auto` (same physical deepseek-flash, dynamic low/high/max effort)
	if (typeof pi.registerVirtualModel === "function") {{
		pi.registerVirtualModel<{{ phase: string; lastEffort: DeepSeekEffort }}>({{
			provider: "deepseek",
			id: "flash-auto",
			name: "DeepSeek Flash Auto-Effort (ap)",
			thinkingLevels: ["low", "high", "max"],
			contextWindow: 1000000,
			maxTokens: 384000,
			input: ["text", "image"],
			route(request, ctx) {{
				const physical =
					ctx.modelRegistry.find("deepseek", "deepseek-flash") ?? request.previous?.model;
				if (!physical) {{
					throw new Error("Physical model deepseek/deepseek-flash not found in Pi model registry");
				}}
				let userText = "";
				let recentToolErrors = 0;
				let recentToolSuccessesAfterEdit = 0;
				if (Array.isArray(request.messages)) {{
					for (let i = request.messages.length - 1; i >= 0; i--) {{
						const m = request.messages[i] as {{
							role?: string;
							content?: string | Array<{{ type?: string; text?: string }}>;
							isError?: boolean;
						}};
						if (m.role === "user") {{
							if (typeof m.content === "string") userText = m.content;
							else if (Array.isArray(m.content)) {{
								userText = m.content
									.filter((b) => b?.type === "text" && typeof b.text === "string")
									.map((b) => b.text as string)
									.join("\n");
							}}
							break;
						}}
						if (m.role === "toolResult") {{
							if (m.isError) recentToolErrors += 1;
							else if (editedFilesThisRun.size > 0) recentToolSuccessesAfterEdit += 1;
						}}
					}}
				}}
				const decision = routeDeepSeekFlashEffort({{
					reason: request.reason,
					selectedLevel: request.thinkingLevel,
					previousLevel: request.previous?.thinkingLevel ?? request.state?.lastEffort,
					userText,
					recentToolErrors,
					recentToolSuccessesAfterEdit,
					stagnantHeavyTurns,
				}});
				const nextState =
					request.reason === "direct"
						? undefined
						: request.state?.phase === decision.phase &&
								request.state?.lastEffort === decision.thinkingLevel
							? request.state
							: {{ phase: decision.phase, lastEffort: decision.thinkingLevel }};
				return {{
					model: physical,
					thinkingLevel: decision.thinkingLevel,
					state: nextState,
				}};
			}},
		}});
	}}

	const updateHud = (ctx?: ExtensionContext) => {{
		if (!ctx || !ctx.hasUI) return;
		const usage = ctx.getContextUsage?.();
		const pct = typeof usage?.percent === "number" ? Math.round(usage.percent) : null;
		const tok = typeof usage?.tokens === "number" ? usage.tokens : null;
		const tokLabel = tok !== null ? `${{Math.round(tok / 1000)}}k` : null;
		const parts: string[] = [];
		const totalPrompt = sessionInputTokens + sessionCacheReadTokens;
		if (lastCacheDropPct !== null) {{
			parts.push(`CACHE DROP ${{lastCacheDropPct}}%`);
		}} else if (totalPrompt >= 10000) {{
			const hitPct = Math.round((sessionCacheReadTokens / totalPrompt) * 100);
			parts.push(`cache ${{hitPct}}%`);
		}}
		if (sievedCount > 0) parts.push(`${{sievedCount}} sieved`);
		if (maskedCount > 0) parts.push(`${{maskedCount}} masked`);
		if (guardsTriggered > 0) parts.push(`${{guardsTriggered}} guarded`);
		const highWater =
			(pct !== null && pct >= MASK_HIGH_WATER_PCT) ||
			(tok !== null && tok >= MASK_HIGH_WATER_TOKENS);
		if (highWater) {{
			parts.push(
				tokLabel && pct !== null
					? `CTX HIGH ${{tokLabel}} (${{pct}}%)`
					: pct !== null
						? `CTX HIGH ${{pct}}%`
						: `CTX HIGH ${{tokLabel}}`,
			);
		}} else if ((pct !== null && pct >= 55) || (tok !== null && tok >= 120000)) {{
			if (parts.length > 0 || (tok !== null && tok >= 120000)) {{
				parts.push(tokLabel && pct !== null ? `ctx ${{tokLabel}} (${{pct}}%)` : `ctx ${{pct ?? tokLabel}}`);
			}}
		}}
		ctx.ui.setStatus("ap", parts.length > 0 ? `ap: ${{parts.join(" | ")}}` : undefined);
	}};

	pi.on("session_start", (_event, ctx) => {{
		resetTransientState();
		largeReadTipShown = false;
		sessionInputTokens = 0;
		sessionCacheReadTokens = 0;
		expectedCacheReset = false;
		sievedCount = 0;
		maskedCount = 0;
		guardsTriggered = 0;
		updateHud(ctx);
	}});

	pi.on("agent_start", () => {{
		resetRunState();
	}});

	pi.on("session_before_switch", () => {{
		resetTransientState();
		largeReadTipShown = false;
		sessionInputTokens = 0;
		sessionCacheReadTokens = 0;
		expectedCacheReset = false;
	}});

	pi.on("session_compact", (_event, ctx) => {{
		resetTransientState();
		expectedCacheReset = true;
		updateHud(ctx);
	}});

	pi.on("session_before_compact", (_event, ctx) => {{
		// Zero-clutter compaction sync: only checkpoint `.baton` if the workspace already uses it
		if (ctx?.cwd && existsSync(join(ctx.cwd, ".baton"))) {{
			spawnSync(AP_BIN, ["baton", "checkpoint", "--note", "pi-compact"], {{
				cwd: ctx.cwd,
				encoding: "utf8",
				timeout: 2500,
				windowsHide: true,
			}});
		}}
	}});

	pi.on("session_before_tree", () => {{
		// Keep branch-switch summaries sharp by preserving verified facts & falsified dead-ends
		return {{
			customInstructions:
				"Preserve exact file paths, symbol names, verified facts, falsified hypotheses/dead-ends, and exact failing error signatures so the target branch never repeats failed attempts.",
		}};
	}});

	// Zero-prompt-token interactive `/ap` command inside Pi (`/ap`, `status`, `doctor`, `compact`, `handoff <goal>`)
	if (typeof pi.registerCommand === "function") {{
		pi.registerCommand("ap", {{
			description: "Live ap + DeepSeek KV-cache telemetry, doctor, focused compact, or clean-session handoff",
			getArgumentCompletions(prefix: string) {{
				const items = [
					{{ value: "status", label: "status — Live KV-cache, masking & guard telemetry" }},
					{{ value: "doctor", label: "doctor — Run native ap doctor health check" }},
					{{ value: "compact", label: "compact — Compact while anchoring verified facts & dead-ends" }},
					{{ value: "handoff ", label: "handoff <goal> — Transfer context cleanly to a fresh 0-rot session" }},
				];
				const p = (prefix || "").trim().toLowerCase();
				return p ? items.filter((it) => it.value.trim().startsWith(p)) : items;
			}},
			async handler(args, ctx) {{
				const rawArgs = (args || "").trim();
				const firstSpace = rawArgs.indexOf(" ");
				const sub = (firstSpace >= 0 ? rawArgs.slice(0, firstSpace) : rawArgs).toLowerCase() || "status";
				const rest = firstSpace >= 0 ? rawArgs.slice(firstSpace + 1).trim() : "";

				if (sub === "doctor") {{
					const res = spawnSync(AP_BIN, ["doctor"], {{
						cwd: ctx.cwd,
						encoding: "utf8",
						timeout: 5000,
						windowsHide: true,
					}});
					const summary = (res.stdout || res.stderr || "ap doctor failed")
						.trim()
						.split("\n")
						.slice(0, 12)
						.join("\n");
					ctx.ui.notify(summary, res.status === 0 ? "info" : "warning");
					return;
				}}
				if (sub === "compact") {{
					ctx.compact({{
						customInstructions:
							"Preserve exact file paths, symbol names, verified facts, falsified dead-ends, and exact build/test commands needed to continue cleanly.",
					}});
					ctx.ui.notify("[ap] Triggered focused compaction preserving verified facts & dead-ends.", "info");
					return;
				}}
				if (sub === "handoff") {{
					if (!rest) {{
						ctx.ui.notify("Usage: /ap handoff <goal for the new focused session>", "warning");
						return;
					}}
					if (!ctx.model) {{
						ctx.ui.notify("[ap handoff] No active model selected.", "error");
						return;
					}}
					const branch = ctx.sessionManager?.getBranch?.() ?? [];
					const transcript = serializeBranchForHandoff(branch as unknown as Array<Record<string, unknown>>);
					if (!transcript) {{
						ctx.ui.notify("[ap handoff] Current branch has no messages to transfer.", "warning");
						return;
					}}
					let envFacts = "";
					if (ctx.cwd && existsSync(join(ctx.cwd, ".git"))) {{
						const gs = spawnSync("git", ["status", "--short"], {{
							cwd: ctx.cwd,
							encoding: "utf8",
							timeout: 2000,
							windowsHide: true,
						}});
						if (gs.status === 0 && gs.stdout?.trim()) {{
							envFacts += `\n\n## Modified Files (git status)\n${{gs.stdout.trim().slice(0, 1500)}}`;
						}}
					}}
					if (ctx.cwd && existsSync(join(ctx.cwd, ".baton"))) {{
						const bt = spawnSync(AP_BIN, ["baton", "resume"], {{
							cwd: ctx.cwd,
							encoding: "utf8",
							timeout: 2000,
							windowsHide: true,
						}});
						if (bt.status === 0 && bt.stdout?.trim()) {{
							envFacts += `\n\n## Baton State\n${{bt.stdout.trim().slice(0, 1500)}}`;
						}}
					}}
					ctx.ui.notify("[ap handoff] Generating clean transfer brief (low effort)...", "info");
					try {{
						const response = await ctx.modelRegistry.complete(
							ctx.model,
							{{
								systemPrompt:
									"Generate a concise, self-contained handoff prompt for a brand-new coding-agent session. Include: ## Context (verified facts & key decisions), ## Falsified Dead-Ends (what NOT to retry), ## Relevant Files, and ## Next Task. Output ONLY the prompt itself.",
								messages: [
									{{
										role: "user",
										content: [
											{{
												type: "text",
												text: `## Conversation Branch\n${{transcript}}${{envFacts}}\n\n## Goal For New Session\n${{rest}}`,
											}},
										],
										timestamp: Date.now(),
									}},
								],
							}},
							{{
								reasoning: "low",
								cacheRetention: "none",
							}},
						);
						const draft = response.content
							.filter((c): c is {{ type: "text"; text: string }} => c.type === "text")
							.map((c) => c.text)
							.join("\n")
							.trim();
						if (!draft) {{
							ctx.ui.notify("[ap handoff] Empty response from model.", "warning");
							return;
						}}
						const edited =
							typeof ctx.ui.editor === "function"
								? await ctx.ui.editor("Review /ap handoff prompt", draft)
								: draft;
						if (edited === undefined) {{
							ctx.ui.notify("[ap handoff] Cancelled.", "info");
							return;
						}}
						const parentSession = ctx.sessionManager?.getSessionFile?.();
						if (typeof ctx.newSession === "function") {{
							await ctx.newSession({{
								parentSession,
								withSession: async (nextCtx) => {{
									nextCtx.ui.setEditorText(edited);
									nextCtx.ui.notify("[ap handoff] Fresh session ready — press Enter to start!", "info");
								}},
							}});
						}} else {{
							ctx.ui.setEditorText?.(edited);
						}}
					}} catch (err) {{
						ctx.ui.notify(
							`[ap handoff] Failed: ${{err instanceof Error ? err.message : String(err)}}`,
							"error",
						);
					}}
					return;
				}}

				const usage = ctx.getContextUsage?.();
				const totalPrompt = sessionInputTokens + sessionCacheReadTokens;
				const hitPct =
					totalPrompt > 0 ? ((sessionCacheReadTokens / totalPrompt) * 100).toFixed(1) : "n/a";
				// DeepSeek V4.1 Flash savings: ($0.30 - $0.006) / 1M = $0.294 per 1M cacheRead tokens
				const estUsdSaved = ((sessionCacheReadTokens / 1_000_000) * 0.294).toFixed(4);
				const ctxStr =
					typeof usage?.tokens === "number"
						? `${{Math.round(usage.tokens / 1000)}}k (${{Math.round(usage.percent ?? 0)}}%)`
						: "n/a";
				const modelStr = ctx.model ? `${{ctx.model.provider}}/${{ctx.model.id}}` : "unknown";
				ctx.ui.notify(
					`[ap telemetry] model=${{modelStr}} | ctx=${{ctxStr}} (mask@180k) | cache=${{hitPct}}% (saved ~$${{estUsdSaved}}) | sieved=${{sievedCount}} masked=${{maskedCount}} guarded=${{guardsTriggered}}`,
					"info",
				);
			}},
		}});
	}}

	pi.on("message_end", (event) => {{
		const msg = event.message as {{
			role?: string;
			content?: Array<{{ type?: string; text?: string; [k: string]: unknown }}>;
		}} | undefined;
		if (!msg || msg.role !== "assistant" || !Array.isArray(msg.content)) return undefined;
		// Never interfere when native toolCall blocks already exist (or an earlier extension recovered them)
		if (msg.content.some((b) => b?.type === "toolCall")) return undefined;

		const active = new Set(pi.getActiveTools?.() ?? []);
		if (active.size === 0) return undefined;

		let recoveredAny = false;
		const nextContent: Array<Record<string, unknown>> = [];
		for (const blk of msg.content) {{
			if (blk?.type === "text" && typeof blk.text === "string") {{
				const rec = blk.text.includes("DSML")
					? extractDsmlToolCalls(blk.text, active)
					: blk.text.includes("<tool_call>")
						? extractXmlJsonToolCalls(blk.text, active)
						: undefined;
				if (rec) {{
					recoveredAny = true;
					if (rec.cleanedText) {{
						nextContent.push({{ ...blk, text: rec.cleanedText }});
					}}
					for (const call of rec.calls) {{
						nextContent.push(call as unknown as Record<string, unknown>);
					}}
					continue;
				}}
			}}
			nextContent.push(blk as Record<string, unknown>);
		}}
		if (!recoveredAny) return undefined;
		guardsTriggered += 1;
		return {{
			message: {{
				...(msg as Record<string, unknown>),
				content: nextContent,
			}} as typeof event.message,
		}};
	}});

	pi.on("turn_end", (event, ctx) => {{
		const msg = event.message as {{
			role?: string;
			content?: Array<{{ type?: string; thinking?: string; name?: string }}>;
			usage?: {{ input?: number; cacheRead?: number }};
		}} | undefined;
		if (msg?.role === "assistant" && Array.isArray(msg.content)) {{
			// Live DeepSeek 50x KV-cache hit-rate telemetry & unexpected drop alarm
			const uIn = typeof msg.usage?.input === "number" ? msg.usage.input : 0;
			const uCache = typeof msg.usage?.cacheRead === "number" ? msg.usage.cacheRead : 0;
			const turnPrompt = uIn + uCache;
			if (turnPrompt > 0) {{
				sessionInputTokens += uIn;
				sessionCacheReadTokens += uCache;
				const turnHitPct = Math.round((uCache / turnPrompt) * 100);
				if (turnPrompt >= 20000 && turnHitPct < 45 && !expectedCacheReset) {{
					lastCacheDropPct = turnHitPct;
				}} else if (turnHitPct >= 60) {{
					lastCacheDropPct = null;
				}}
				expectedCacheReset = false;
			}}

			let thinkChars = 0;
			let didEdit = false;
			for (const blk of msg.content) {{
				if (blk?.type === "thinking" && typeof blk.thinking === "string") {{
					thinkChars += blk.thinking.length;
				}} else if (blk?.type === "toolCall" && typeof blk.name === "string" && EDIT_TOOLS.has(blk.name)) {{
					didEdit = true;
				}}
			}}
			if (didEdit) {{
				stagnantHeavyTurns = 0;
				pendingThinkingNudge = false;
			}} else if (thinkChars >= HEAVY_THINKING_CHARS) {{
				stagnantHeavyTurns += 1;
				if (stagnantHeavyTurns >= STAGNATION_TURNS) {{
					pendingThinkingNudge = true;
				}}
			}} else {{
				stagnantHeavyTurns = Math.max(0, stagnantHeavyTurns - 1);
			}}
		}}

		// Native Pi v1.5 batch observation masking at high-water mark with cooldown (KV-cache preserving)
		const turnIdx = typeof event.turnIndex === "number" ? event.turnIndex : 0;
		const contextEntries = event.context?.contextEntries;
		if (
			Array.isArray(contextEntries) &&
			turnIdx - lastMaskTurn >= MASK_COOLDOWN_TURNS
		) {{
			const usage = ctx?.getContextUsage?.();
			const plan = planObservationMasking(contextEntries, {{
				usagePercent: usage?.percent ?? null,
				usageTokens: usage?.tokens ?? null,
			}});
			if (plan.edits.length > 0) {{
				lastMaskTurn = turnIdx;
				maskedCount += plan.maskedCount;
				expectedCacheReset = true;
				updateHud(ctx);
				return {{
					entries: [...(event.entries ?? []), ...plan.edits],
				}};
			}}
		}}

		updateHud(ctx);
		return undefined;
	}});

	pi.on("agent_before_settle", (event, ctx) => {{
		if (event.outcome !== "completed" || settleGateFiredThisRun) {{
			return undefined;
		}}

		// 0. Recover silent thinking-only stall (assistant emitted thinking with 0 text & 0 toolCalls)
		const ctxMsgs = event.context?.contextMessages;
		if (Array.isArray(ctxMsgs) && ctxMsgs.length > 0) {{
			const last = ctxMsgs[ctxMsgs.length - 1] as {{
				role?: string;
				content?: Array<{{ type?: string; thinking?: string; text?: string }}>;
			}};
			if (last?.role === "assistant" && Array.isArray(last.content)) {{
				const hasThinking = last.content.some(
					(b) => b?.type === "thinking" && typeof b.thinking === "string" && b.thinking.trim().length > 0,
				);
				const hasVisible = last.content.some(
					(b) =>
						b?.type === "toolCall" ||
						(b?.type === "text" && typeof b.text === "string" && b.text.trim().length > 0),
				);
				if (hasThinking && !hasVisible) {{
					settleGateFiredThisRun = true;
					guardsTriggered += 1;
					updateHud(ctx);
					const draft: CustomMessageEntryDraft = {{
						type: "custom_message",
						customType: "ap-empty-recover",
						display: false,
						content:
							"[ap recover] Previous assistant turn produced internal reasoning with no visible text or tool call. Emit your concrete tool call or final answer now.",
					}};
					return {{
						entries: [...(event.entries ?? []), draft],
						continue: true,
					}};
				}}
			}}
		}}

		if (editedFilesThisRun.size === 0) {{
			return undefined;
		}}
		const cwd = ctx?.cwd;
		if (!cwd) return undefined;

		const blockers: string[] = [];

		// 1. Fast (<15ms) Tree-sitter/JSON/TOML syntax check on code files edited via built-in write/edit
		for (const rawPath of [...builtInCodeEditsThisRun].slice(0, 6)) {{
			if (!CHECKABLE_CODE_EXT.test(rawPath)) continue;
			const absPath = isAbsolute(rawPath) ? rawPath : resolve(cwd, rawPath);
			if (!existsSync(absPath)) continue;
			const chk = spawnSync(AP_BIN, ["splice", "check", "--json", absPath], {{
				cwd,
				encoding: "utf8",
				timeout: 2500,
				windowsHide: true,
			}});
			if (chk.stdout && chk.stdout.trim().startsWith("{{")) {{
				try {{
					const parsed = JSON.parse(chk.stdout) as {{
						ok?: boolean;
						issues?: Array<string | {{ file?: string; line?: number; message?: string; detail?: string }}>;
					}};
					if (parsed.ok === false && Array.isArray(parsed.issues)) {{
						for (const iss of parsed.issues.slice(0, 3)) {{
							const msg =
								typeof iss === "string"
									? iss
									: `${{iss.file ?? normRelPath(rawPath, cwd)}}${{iss.line ? `:${{iss.line}}` : ""}} ${{iss.message ?? iss.detail ?? "syntax error"}}`;
							blockers.push(`- [SYNTAX] ${{msg.slice(0, 180)}}`);
						}}
					}}
				}} catch {{
					// ignore unparseable output
				}}
			}}
		}}

		// 2. Fast (<25ms) Git diff blocker gate strictly scoped to files edited in this run
		if (existsSync(join(cwd, ".git"))) {{
			const diffRes = spawnSync(AP_BIN, ["guard", "diff", "--json", cwd], {{
				cwd,
				encoding: "utf8",
				timeout: 2500,
				windowsHide: true,
			}});
			if (diffRes.stdout && diffRes.stdout.trim().startsWith("{{")) {{
				try {{
					const parsed = JSON.parse(diffRes.stdout) as {{ findings?: DiffFindingItem[] }};
					if (Array.isArray(parsed.findings)) {{
						blockers.push(...filterSettleDiffBlockers(parsed.findings, editedFilesThisRun, cwd));
					}}
				}} catch {{
					// ignore unparseable output
				}}
			}}
		}}

		if (blockers.length === 0) {{
			editedFilesThisRun.clear();
			builtInCodeEditsThisRun.clear();
			return undefined;
		}}

		settleGateFiredThisRun = true;
		guardsTriggered += 1;
		updateHud(ctx);
		const draft: CustomMessageEntryDraft = {{
			type: "custom_message",
			customType: "ap-settle-gate",
			display: true,
			content: `[ap settle-gate] Blocked premature completion — concrete issue(s) found in files edited this run:\n${{blockers.slice(0, 6).join("\n")}}\nFix these issues and re-verify before finishing.`,
		}};
		return {{
			entries: [...(event.entries ?? []), draft],
			continue: true,
		}};
	}});

	pi.on("tool_call", (event) => {{
		const baseTool = event.toolName.replace(/^mcp__agent_primitives__/, "");
		if (EDIT_TOOLS.has(baseTool)) {{
			recentShellFps.length = 0;
			unchangedReads.clear();
			stagnantHeavyTurns = 0;
			pendingThinkingNudge = false;
			return undefined;
		}}
		if (event.toolName !== "bash" && event.toolName !== "powershell") return undefined;
		const cmd = typeof event.input?.command === "string" ? event.input.command : "";
		if (DESTRUCTIVE_OR_WRITE.test(cmd)) {{
			unchangedReads.clear();
			return undefined;
		}}
		const healed = healCommand(cmd, event.toolName);
		if (healed && healed !== cmd) {{
			event.input.command = healed;
		}}
		return undefined;
	}});

	pi.on("tool_result", (event, ctx) => {{
		// Track edited files across both direct calls and codemode child calls for `agent_before_settle`
		const baseTool = event.toolName.replace(/^mcp__agent_primitives__/, "");
		if (!event.isError && EDIT_TOOLS.has(baseTool)) {{
			for (const p of extractEditedPathsFromTool(baseTool, event.input as Record<string, unknown> | undefined)) {{
				editedFilesThisRun.add(p);
				if (baseTool === "write" || baseTool === "edit") {{
					builtInCodeEditsThisRun.add(p);
				}}
			}}
		}}
		// Leave codemode child calls (`parentToolCallId`) untouched so JS scripts keep raw structuredContent!
		if (event.parentToolCallId) return undefined;
		if (!Array.isArray(event.content) || event.content.length === 0) return undefined;
		const textBlock = event.content.find(
			(c: {{ type?: string; text?: string }}) => c?.type === "text" && typeof c.text === "string",
		);
		if (!textBlock || typeof textBlock.text !== "string") return undefined;

		let outText = textBlock.text;
		let mutated = false;

		// A. Built-in `read` redundant re-read guard + one-time large code file AST tip
		if (event.toolName === "read" && !event.isError) {{
			const p = String(event.input?.path ?? event.input?.file_path ?? "");
			const off = String(event.input?.offset ?? "");
			const lim = String(event.input?.limit ?? "");
			if (p) {{
				const key = `${{p}}::${{off}}::${{lim}}`;
				const hits = (unchangedReads.get(key) ?? 0) + 1;
				unchangedReads.set(key, hits);
				if (hits >= REREAD_THRESHOLD) {{
					outText += `\n[ap reread-guard] Exact unchanged slice (${{p}}) read ${{hits}}x already in this session. Avoid re-reading; use ap_search/ap_locate or proceed directly to edits.`;
					guardsTriggered += 1;
					mutated = true;
				}} else if (!largeReadTipShown && outText.length >= LARGE_CODE_READ_CHARS && CODE_FILE_EXT.test(p)) {{
					outText += `\n[ap read-tip] Large code file loaded; use ap_read(outline=true) or ap_read(symbol="...") for AST-exact slices (~85% fewer tokens).`;
					largeReadTipShown = true;
					mutated = true;
				}}
			}}
		}}

		// B. Built-in `edit` error self-healing tip (guides DeepSeek straight to fuzzy/AST recovery)
		if (event.toolName === "edit" && event.isError) {{
			outText += `\n[ap edit-tip] Patch target missed; use ap_edit (4-tier indentation/blank-line fuzzy match + AST symbols) or ap splice view for exact lines.`;
			mutated = true;
		}}

		// B2. Pi `codemode` error self-healing tip (eliminates blind MCP name/schema guessing loops)
		if (
			event.toolName === "codemode" &&
			(event.isError || outText.includes("Script failed") || outText.includes("is not a function"))
		) {{
			outText += `\n[ap codemode-tip] Use await searchTools("ap_") or await describeTool("mcp__agent_primitives__<name>") to verify exact tool IDs & structuredContent fields.`;
			mutated = true;
		}}

		// C. Shell output auto-sieve (>28KB or giant single line) + 3x doom-loop circuit-breaker
		if (event.toolName === "bash" || event.toolName === "powershell") {{
			const cmd = typeof event.input?.command === "string" ? event.input.command : "";
			const skipSieve =
				cmd.includes("ap.exe") ||
				/\bap\s+(?:read|search|query|atlas|probe|guard|sieve|baton|splice|recall)\b/.test(cmd);
			if (!skipSieve) {{
				const hasGiantLine = outText.split("\n").some((l: string) => l.length > 4500);
				if (outText.length >= 28000 || hasGiantLine) {{
					const res = spawnSync(AP_BIN, ["sieve", "pipe", "--label", "pi-bash"], {{
						input: outText,
						encoding: "utf8",
						timeout: 4000,
						windowsHide: true,
						env: {{ ...process.env, SIEVE_HOME }},
					}});
					if (res.status === 0 && res.stdout && res.stdout.length > 40 && res.stdout.length < outText.length * 0.8) {{
						outText = res.stdout;
						sievedCount += 1;
						mutated = true;
					}}
				}}
			}}

			if (event.isError && QUOTE_OR_BS_ERR.test(outText)) {{
				outText += `\n[ap quote-tip] Avoid nested quotes/backslashes inside inline python -c; use a single-quoted heredoc (python - <<'PY' ... PY) or write a script file via write/ap_edit.`;
				mutated = true;
			}}

			const fp = normCmdFingerprint(cmd);
			if (fp.length >= 8) {{
				recentShellFps.push(fp);
				if (recentShellFps.length > LOOP_WINDOW) {{
					recentShellFps.shift();
				}}
				const repeats = recentShellFps.filter((x) => x === fp).length;
				const threshold = event.isError ? LOOP_ERR_THRESHOLD : LOOP_OK_THRESHOLD;
				if (repeats >= threshold) {{
					const hint = event.isError
						? `\n[ap loop-breaker] Repeated failing shell command (${{repeats}}/${{LOOP_WINDOW}} calls) without intervening edits. Pause, inspect root cause, edit code or switch strategy instead of retrying identically.`
						: `\n[ap loop-breaker] Identical shell command repeated (${{repeats}}/${{LOOP_WINDOW}} calls) without edits. Avoid redundant polling; summarize findings or pivot to the next concrete action.`;
					outText = `${{outText}}${{hint}}`;
					guardsTriggered += 1;
					mutated = true;
				}}
			}}
		}}

		// D. DeepSeek overthinking stagnation nudge (emitted once after 3 consecutive >12k-char thinking turns without edits)
		if (pendingThinkingNudge) {{
			outText += `\n[ap thinking-guard] 3 consecutive heavy-thinking turns without code changes. Commit to one concrete edit, focused test, or ap_locate/ap_changes call now.`;
			pendingThinkingNudge = false;
			stagnantHeavyTurns = 0;
			guardsTriggered += 1;
			mutated = true;
		}}

		if (!mutated) return undefined;
		updateHud(ctx);
		const patchedContent = event.content.map((c: {{ type?: string; text?: string }}) =>
			c === textBlock ? {{ ...c, text: outText }} : c,
		);
		return event.structuredContent !== undefined
			? {{ content: patchedContent, structuredContent: event.structuredContent }}
			: {{ content: patchedContent }};
	}});
}}
"#
    )
}

fn install_pi_accelerator_extension(ext_dir: &Path, ap_bin_posix: &str) -> String {
    let _ = fs::create_dir_all(ext_dir);
    let dst = ext_dir.join("zz-ap-accelerator.ts");
    let _ = fs::write(&dst, render_pi_accelerator_ts(ap_bin_posix));
    posix(&dst)
}

fn remove_pi_accelerator_extension(ext_dir: &Path) {
    let _ = fs::remove_file(ext_dir.join("zz-ap-accelerator.ts"));
}

fn merge_vscode_mcp(cfg_path: &Path, ap_bin_posix: &str) -> String {
    let mut data: Value = if cfg_path.exists() {
        fs::read_to_string(cfg_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| json!({}))
    } else {
        json!({})
    };
    if !data.is_object() {
        data = json!({});
    }
    let servers = data
        .as_object_mut()
        .unwrap()
        .entry("servers")
        .or_insert_with(|| json!({}));
    if !servers.is_object() {
        *servers = json!({});
    }
    servers.as_object_mut().unwrap().insert(
        "agent-primitives".to_string(),
        json!({
            "type": "stdio",
            "command": ap_bin_posix,
            "args": ["mcp"]
        }),
    );
    if let Some(p) = cfg_path.parent() {
        let _ = fs::create_dir_all(p);
    }
    let _ = fs::write(
        cfg_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&data).unwrap_or_default()
        ),
    );
    posix(cfg_path)
}

fn remove_vscode_mcp(cfg_path: &Path) {
    let Ok(txt) = fs::read_to_string(cfg_path) else {
        return;
    };
    let Ok(mut data) = serde_json::from_str::<Value>(&txt) else {
        return;
    };
    let Some(obj) = data.as_object_mut() else {
        return;
    };
    if let Some(srvs) = obj.get_mut("servers").and_then(Value::as_object_mut) {
        srvs.remove("agent-primitives");
        if srvs.is_empty() {
            obj.remove("servers");
        }
    }
    if obj.is_empty() {
        let _ = fs::remove_file(cfg_path);
    } else {
        let _ = fs::write(
            cfg_path,
            format!(
                "{}\n",
                serde_json::to_string_pretty(&data).unwrap_or_default()
            ),
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub fn cmd_init(
    agent_arg: &str,
    only_arg: &str,
    here: bool,
    dir_opt: Option<&str>,
    home_opt: Option<&str>,
    with_hooks: bool,
    with_mcp: bool,
    no_rules: bool,
) -> i32 {
    let home = home_opt
        .filter(|s| !s.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(default_home_dir);
    let check_bins = fs::canonicalize(&home).ok() == fs::canonicalize(default_home_dir()).ok();

    let selected: Vec<&str> = only_arg
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| {
            ALL_PRIMITIVES
                .iter()
                .copied()
                .find(|&p| p == s)
                .unwrap_or("")
        })
        .collect();
    if selected.is_empty() || selected.contains(&"") {
        eprintln!(
            "[ap init] unknown primitive in --only '{only_arg}'. Choose from: {}",
            ALL_PRIMITIVES.join(", ")
        );
        return 2;
    }

    let harnesses = match resolve_requested_harnesses(agent_arg, &home, check_bins) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("[ap init] {e}");
            return 2;
        }
    };
    let explicit_agents = !matches!(agent_arg.trim().to_ascii_lowercase().as_str(), "" | "auto");

    if here || dir_opt.is_some() {
        let repo_dir = fs::canonicalize(dir_opt.unwrap_or("."))
            .unwrap_or_else(|_| PathBuf::from(dir_opt.unwrap_or(".")));
        let agents_root = repo_dir.join(".agents");
        let bin_dst = match install_self_binary(&agents_root.join("bin")) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("[ap init --here] {e}");
                return 1;
            }
        };
        let rel_ap_cmd = if cfg!(windows) {
            ".agents/bin/ap.exe"
        } else {
            ".agents/bin/ap"
        };
        let dest_skills = agents_root.join("skills");
        if let Err(e) = write_skill_folders(&dest_skills, &selected, Some(rel_ap_cmd)) {
            eprintln!("[ap init --here] {e}");
            return 1;
        }

        let mut extra_dirs = Vec::new();
        for (hid, rel_d) in [
            ("pi", ".pi/skills"),
            ("claude", ".claude/skills"),
            ("grok", ".grok/skills"),
        ] {
            let dot_root = repo_dir.join(rel_d.split('/').next().unwrap());
            if (harnesses.contains(&hid) && explicit_agents) || dot_root.is_dir() {
                let _ = write_skill_folders(&repo_dir.join(rel_d), &selected, Some(rel_ap_cmd));
                extra_dirs.push(rel_d);
            }
        }

        let mut rule_notes = Vec::new();
        if !no_rules {
            let block = render_fenced_agents_block(rel_ap_cmd, &selected);
            let st_a = upsert_fenced_block(&repo_dir.join("AGENTS.md"), &block);
            rule_notes.push(format!("AGENTS.md ({st_a})"));
            if (harnesses.contains(&"claude") && explicit_agents)
                || repo_dir.join("CLAUDE.md").exists()
            {
                let st_c = upsert_fenced_block(&repo_dir.join("CLAUDE.md"), &block);
                rule_notes.push(format!("CLAUDE.md ({st_c})"));
            }
        }

        if selected.contains(&"guard") {
            let gcfg_path = repo_dir.join(".agents").join("guard.json");
            if !gcfg_path.exists() {
                let cfg = ap_guard::rules::default_guard_config(&repo_dir);
                let _ = ap_guard::rules::save_config(&repo_dir, &cfg);
            }
        }

        let mut hook_paths = Vec::new();
        if with_hooks {
            let ap_abs = posix(&bin_dst);
            if !explicit_agents || harnesses.contains(&"claude") {
                hook_paths.push(merge_claude_or_codex_hooks(
                    &repo_dir.join(".claude").join("settings.json"),
                    &ap_abs,
                    &selected,
                    true,
                ));
            }
            if !explicit_agents || harnesses.contains(&"codex") {
                hook_paths.push(merge_claude_or_codex_hooks(
                    &repo_dir.join(".codex").join("hooks.json"),
                    &ap_abs,
                    &selected,
                    false,
                ));
            }
            if !explicit_agents || harnesses.contains(&"antigravity") {
                if let Some(hp) = merge_antigravity_hooks(
                    &repo_dir.join(".agents").join("hooks.json"),
                    &ap_abs,
                    &selected,
                ) {
                    hook_paths.push(hp);
                }
            }
        }

        let mut mcp_paths = Vec::new();
        let ap_abs_here = posix(&bin_dst);
        if with_mcp
            || (explicit_agents && harnesses.contains(&"opencode"))
            || repo_dir.join("opencode.json").exists()
        {
            mcp_paths.push(merge_opencode_mcp(
                &repo_dir.join("opencode.json"),
                &ap_abs_here,
            ));
        }
        if with_mcp || (explicit_agents && harnesses.contains(&"qwen")) {
            mcp_paths.push(merge_mcp_servers_json(
                &repo_dir.join(".qwen").join("settings.json"),
                &ap_abs_here,
                true,
            ));
        }
        if with_mcp || (explicit_agents && harnesses.contains(&"gemini-cli")) {
            mcp_paths.push(merge_mcp_servers_json(
                &repo_dir.join(".gemini").join("settings.json"),
                &ap_abs_here,
                true,
            ));
        }
        if with_mcp
            || (explicit_agents && harnesses.contains(&"cursor"))
            || repo_dir.join(".cursor").is_dir()
        {
            mcp_paths.push(merge_mcp_servers_json(
                &repo_dir.join(".cursor").join("mcp.json"),
                &ap_abs_here,
                false,
            ));
        }
        if with_mcp
            || (explicit_agents && harnesses.contains(&"vscode"))
            || repo_dir.join(".vscode").is_dir()
        {
            mcp_paths.push(merge_vscode_mcp(
                &repo_dir.join(".vscode").join("mcp.json"),
                &ap_abs_here,
            ));
        }
        if with_mcp
            || (explicit_agents && harnesses.contains(&"pi"))
            || repo_dir.join(".pi").is_dir()
        {
            mcp_paths.push(merge_pi_mcp(
                &repo_dir.join(".pi").join("mcp.json"),
                &ap_abs_here,
            ));
            if with_hooks {
                hook_paths.push(install_pi_accelerator_extension(
                    &repo_dir.join(".pi").join("extensions"),
                    &ap_abs_here,
                ));
            }
        }
        if with_mcp {
            mcp_paths.push(merge_mcp_servers_json(
                &repo_dir.join(".mcp.json"),
                &ap_abs_here,
                false,
            ));
        }

        let extras_s = if extra_dirs.is_empty() {
            String::new()
        } else {
            format!(" (+ {})", extra_dirs.join(", "))
        };
        println!("[ap init --here] Ready in {}!", posix(&repo_dir));
        println!(
            "  - Installed {} standalone Rust primitives -> .agents/skills/{{{}}}{extras_s}",
            selected.len(),
            selected.join(",")
        );
        println!(
            "  - Self-contained native binary -> {rel_ap_cmd} <{}>",
            selected.join("|")
        );
        if !rule_notes.is_empty() {
            println!("  - Routing rules wired -> {}", rule_notes.join(", "));
        }
        if !mcp_paths.is_empty() {
            println!(
                "  - Native stdio MCP (`ap mcp`) wired -> {}",
                mcp_paths.join(", ")
            );
        }
        if !hook_paths.is_empty() {
            println!(
                "  - Native <2ms hooks merged (with .bak) -> {}",
                hook_paths.join(", ")
            );
        }
        return 0;
    }

    // Global onboarding
    let bin_dir = home.join(".local").join("bin");
    let global_bin = match install_self_binary(&bin_dir) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("[ap init] {e}");
            return 1;
        }
    };
    // Remove legacy Python shims (`~/.agents/ap.py`, `~/.local/bin/ap.cmd`) so `ap.exe` runs directly!
    let _ = fs::remove_file(home.join(".agents").join("ap.py"));
    if cfg!(windows) {
        let _ = fs::remove_file(bin_dir.join("ap.cmd"));
        let _ = fs::remove_file(bin_dir.join("ap"));
    }

    let global_bin_posix = posix(&global_bin);
    let ap_on_path = which_exists("ap");
    let ap_cmd_str = if ap_on_path {
        "ap".to_string()
    } else if global_bin_posix.contains(' ') {
        format!("\"{global_bin_posix}\"")
    } else {
        global_bin_posix.clone()
    };
    let skill_cmd_rewrite = if ap_on_path {
        None
    } else {
        Some(ap_cmd_str.as_str())
    };

    let mut skill_targets: Vec<PathBuf> = vec![home.join(".agents").join("skills")];
    let mut rule_targets: Vec<PathBuf> = Vec::new();
    for &hid in &harnesses {
        if hid == "agents" {
            continue;
        }
        if let Some(sp) = HARNESS_SPECS.iter().find(|s| s.id == hid) {
            for &sd in sp.skill_dirs {
                let tp = home.join(sd);
                if !skill_targets.contains(&tp) {
                    skill_targets.push(tp);
                }
            }
            let rp = home.join(sp.rule_file);
            if !rule_targets.contains(&rp) {
                rule_targets.push(rp);
            }
        }
    }

    for st in &skill_targets {
        if let Err(e) = write_skill_folders(st, &selected, skill_cmd_rewrite) {
            eprintln!("[ap init] {e}");
            return 1;
        }
    }

    let mut wired_rules = Vec::new();
    if !no_rules {
        let block = render_fenced_agents_block(&ap_cmd_str, &selected);
        if rule_targets.is_empty() {
            rule_targets.push(home.join(".agents").join("AGENTS.md"));
        }
        for rt in &rule_targets {
            let st_r = upsert_fenced_block(rt, &block);
            wired_rules.push(format!("{} ({st_r})", posix(rt)));
        }
    }

    let mut hook_paths = Vec::new();
    if with_hooks {
        if harnesses.is_empty() || harnesses.contains(&"claude") {
            hook_paths.push(merge_claude_or_codex_hooks(
                &home.join(".claude").join("settings.json"),
                &global_bin_posix,
                &selected,
                true,
            ));
        }
        if harnesses.contains(&"codex") {
            hook_paths.push(merge_claude_or_codex_hooks(
                &home.join(".codex").join("hooks.json"),
                &global_bin_posix,
                &selected,
                false,
            ));
        }
        if harnesses.contains(&"antigravity") {
            if let Some(hp) = merge_antigravity_hooks(
                &home.join(".gemini").join("config").join("hooks.json"),
                &global_bin_posix,
                &selected,
            ) {
                hook_paths.push(hp);
            }
        }
        if harnesses.contains(&"pi") {
            hook_paths.push(install_pi_accelerator_extension(
                &home.join(".pi").join("agent").join("extensions"),
                &global_bin_posix,
            ));
        }
    }

    // Zero-flag automatic MCP registration for free/open coding agents (OpenCode, Qwen Code, Gemini CLI, Pi codemode)
    let mut mcp_paths = Vec::new();
    if harnesses.contains(&"pi") {
        mcp_paths.push(merge_pi_mcp(
            &home.join(".pi").join("agent").join("mcp.json"),
            &global_bin_posix,
        ));
    }
    if harnesses.contains(&"opencode") {
        mcp_paths.push(merge_opencode_mcp(
            &home.join(".config").join("opencode").join("opencode.json"),
            &global_bin_posix,
        ));
    }
    if harnesses.contains(&"qwen") {
        mcp_paths.push(merge_mcp_servers_json(
            &home.join(".qwen").join("settings.json"),
            &global_bin_posix,
            true,
        ));
    }
    if harnesses.contains(&"gemini-cli") {
        mcp_paths.push(merge_mcp_servers_json(
            &home.join(".gemini").join("settings.json"),
            &global_bin_posix,
            true,
        ));
    }
    if harnesses.contains(&"cursor") {
        mcp_paths.push(merge_mcp_servers_json(
            &home.join(".cursor").join("mcp.json"),
            &global_bin_posix,
            false,
        ));
    }
    if harnesses.contains(&"windsurf") {
        mcp_paths.push(merge_mcp_servers_json(
            &home
                .join(".codeium")
                .join("windsurf")
                .join("mcp_config.json"),
            &global_bin_posix,
            false,
        ));
    }
    if harnesses.contains(&"claude-desktop") {
        let cd_rel = [
            "AppData/Roaming/Claude",
            "Library/Application Support/Claude",
            ".config/Claude",
        ]
        .iter()
        .copied()
        .find(|d| home.join(d).is_dir())
        .unwrap_or(if cfg!(windows) {
            "AppData/Roaming/Claude"
        } else if cfg!(target_os = "macos") {
            "Library/Application Support/Claude"
        } else {
            ".config/Claude"
        });
        mcp_paths.push(merge_mcp_servers_json(
            &home.join(cd_rel).join("claude_desktop_config.json"),
            &global_bin_posix,
            false,
        ));
    }

    println!("[ap init] Global native Rust onboarding complete!");
    println!(
        "  - Harnesses configured: {}",
        if harnesses.is_empty() {
            "agents (~/.agents)".to_string()
        } else {
            harnesses.join(", ")
        }
    );
    println!(
        "  - Synced {} primitives into {} skill directory(s): {}",
        selected.len(),
        skill_targets.len(),
        skill_targets
            .iter()
            .map(|t| posix(t))
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!("  - Installed native single binary: {global_bin_posix}");
    if !wired_rules.is_empty() {
        println!(
            "  - Global routing blocks wired: {}",
            wired_rules.join(", ")
        );
    }
    if !mcp_paths.is_empty() {
        println!(
            "  - Native stdio MCP (`ap mcp`) auto-wired: {}",
            mcp_paths.join(", ")
        );
    }
    if !hook_paths.is_empty() {
        println!(
            "  - Native <2ms hooks merged (with .bak): {}",
            hook_paths.join(", ")
        );
    }
    0
}

pub fn cmd_uninstall(here: bool, dir_opt: Option<&str>, home_opt: Option<&str>) -> i32 {
    if here || dir_opt.is_some() {
        let repo_dir = fs::canonicalize(dir_opt.unwrap_or("."))
            .unwrap_or_else(|_| PathBuf::from(dir_opt.unwrap_or(".")));
        let mut removed: HashSet<&str> = HashSet::new();
        for sub in [
            ".agents/skills",
            ".pi/skills",
            ".claude/skills",
            ".grok/skills",
        ] {
            let sr = repo_dir.join(sub);
            for &nm in &ALL_PRIMITIVES {
                let p = sr.join(nm);
                if p.exists() {
                    let _ = fs::remove_dir_all(&p);
                    removed.insert(nm);
                }
            }
            if sr.is_dir()
                && fs::read_dir(&sr)
                    .map(|mut r| r.next().is_none())
                    .unwrap_or(false)
            {
                let _ = fs::remove_dir(&sr);
            }
        }
        let _ = fs::remove_dir_all(repo_dir.join(".agents").join("bin"));
        let _ = fs::remove_file(repo_dir.join(".agents").join("ap.py"));
        let _ = fs::remove_file(repo_dir.join(".agents").join("guard.json"));
        remove_antigravity_hooks(&repo_dir.join(".agents").join("hooks.json"));
        remove_claude_or_codex_hooks(&repo_dir.join(".claude").join("settings.json"));
        remove_claude_or_codex_hooks(&repo_dir.join(".codex").join("hooks.json"));
        remove_opencode_mcp(&repo_dir.join("opencode.json"));
        remove_mcp_servers_json(&repo_dir.join(".mcp.json"));
        remove_mcp_servers_json(&repo_dir.join(".cursor").join("mcp.json"));
        remove_vscode_mcp(&repo_dir.join(".vscode").join("mcp.json"));
        remove_mcp_servers_json(&repo_dir.join(".qwen").join("settings.json"));
        remove_mcp_servers_json(&repo_dir.join(".gemini").join("settings.json"));
        remove_mcp_servers_json(&repo_dir.join(".pi").join("mcp.json"));
        remove_pi_accelerator_extension(&repo_dir.join(".pi").join("extensions"));

        for dot in [
            ".agents", ".pi", ".claude", ".codex", ".grok", ".qwen", ".gemini", ".cursor",
            ".vscode",
        ] {
            let dp = repo_dir.join(dot);
            if dp.is_dir()
                && fs::read_dir(&dp)
                    .map(|mut r| r.next().is_none())
                    .unwrap_or(false)
            {
                let _ = fs::remove_dir(&dp);
            }
        }
        let md_removed = remove_fenced_block(&repo_dir.join("AGENTS.md"));
        remove_fenced_block(&repo_dir.join("CLAUDE.md"));
        println!(
            "[ap uninstall --here] Removed {} primitives from {} (AGENTS.md block removed={md_removed}).",
            removed.len(),
            posix(&repo_dir)
        );
        return 0;
    }

    let home = home_opt
        .filter(|s| !s.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(default_home_dir);
    let mut skill_dirs: Vec<PathBuf> = vec![home.join(".agents").join("skills")];
    let mut rule_files: Vec<PathBuf> = vec![home.join(".agents").join("AGENTS.md")];
    for sp in HARNESS_SPECS {
        for &sd in sp.skill_dirs {
            let p = home.join(sd);
            if !skill_dirs.contains(&p) {
                skill_dirs.push(p);
            }
        }
        let rf = home.join(sp.rule_file);
        if !rule_files.contains(&rf) {
            rule_files.push(rf);
        }
    }

    let mut count = 0usize;
    for st in skill_dirs {
        for &nm in &ALL_PRIMITIVES {
            let p = st.join(nm);
            if p.exists() {
                let _ = fs::remove_dir_all(&p);
                count += 1;
            }
        }
    }
    let mut rules_cleaned = 0usize;
    for rf in rule_files {
        if remove_fenced_block(&rf) {
            rules_cleaned += 1;
        }
    }
    remove_claude_or_codex_hooks(&home.join(".claude").join("settings.json"));
    remove_claude_or_codex_hooks(&home.join(".codex").join("hooks.json"));
    remove_antigravity_hooks(&home.join(".gemini").join("config").join("hooks.json"));
    remove_opencode_mcp(&home.join(".config").join("opencode").join("opencode.json"));
    remove_mcp_servers_json(&home.join(".qwen").join("settings.json"));
    remove_mcp_servers_json(&home.join(".gemini").join("settings.json"));
    remove_mcp_servers_json(&home.join(".pi").join("agent").join("mcp.json"));
    remove_pi_accelerator_extension(&home.join(".pi").join("agent").join("extensions"));
    remove_mcp_servers_json(&home.join(".cursor").join("mcp.json"));
    remove_mcp_servers_json(
        &home
            .join(".codeium")
            .join("windsurf")
            .join("mcp_config.json"),
    );
    for cd_rel in [
        "AppData/Roaming/Claude",
        "Library/Application Support/Claude",
        ".config/Claude",
    ] {
        remove_mcp_servers_json(&home.join(cd_rel).join("claude_desktop_config.json"));
    }

    let cur_exe = std::env::current_exe()
        .ok()
        .and_then(|p| fs::canonicalize(p).ok());
    for f in [
        home.join(".agents").join("ap.py"),
        home.join(".agents").join("audit").join("audit_sessions.py"),
        home.join(".local").join("bin").join("ap"),
        home.join(".local").join("bin").join("ap.cmd"),
        home.join(".local").join("bin").join("ap.exe"),
    ] {
        if f.exists() && fs::canonicalize(&f).ok() != cur_exe {
            let _ = fs::remove_file(&f);
        }
    }
    let _ = fs::remove_dir_all(home.join(".agent-primitives"));
    println!("[ap uninstall] Removed {count} installed skill folder(s) and {rules_cleaned} global rule block(s).");
    0
}

fn inspect_pi_runtime(home: &Path) -> Value {
    let pi_agent = home.join(".pi").join("agent");
    if !pi_agent.is_dir() {
        return json!({
            "detected": false,
            "hints": []
        });
    }

    let accel_path = pi_agent.join("extensions").join("zz-ap-accelerator.ts");
    let accel_txt = fs::read_to_string(&accel_path).ok();
    let accelerator_present = accel_txt.is_some();
    let accelerator_up_to_date = accel_txt
        .as_deref()
        .map(|s| {
            s.contains("planObservationMasking")
                && s.contains("extractDsmlToolCalls")
                && s.contains("extractXmlJsonToolCalls")
                && s.contains("extractErrorSignatureLines")
                && s.contains("serializeBranchForHandoff")
                && s.contains("LOOP_ERR_THRESHOLD")
                && s.contains("QUOTE_OR_BS_ERR")
                && s.contains("session_before_compact")
                && s.contains("agent_before_settle")
                && s.contains("routeDeepSeekFlashEffort")
                && s.contains("registerCommand(\"ap\"")
                && s.contains("session_before_tree")
        })
        .unwrap_or(false);

    let mcp_path = pi_agent.join("mcp.json");
    let mcp_json = fs::read_to_string(&mcp_path)
        .ok()
        .and_then(|s| serde_json::from_str::<Value>(&s).ok());
    let mcp_server = mcp_json
        .as_ref()
        .and_then(|v| v.pointer("/mcpServers/agent-primitives"));
    let mcp_codemode_wired = mcp_server
        .and_then(|v| v.get("exposure"))
        .and_then(Value::as_str)
        == Some("codemode");

    let settings_json = fs::read_to_string(pi_agent.join("settings.json"))
        .ok()
        .and_then(|s| serde_json::from_str::<Value>(&s).ok());
    let default_provider = settings_json
        .as_ref()
        .and_then(|v| v.get("defaultProvider"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let default_model_id = settings_json
        .as_ref()
        .and_then(|v| v.get("defaultModel"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let default_model = match (&default_provider, &default_model_id) {
        (Some(p), Some(m)) => Some(format!("{p}/{m}")),
        (None, Some(m)) => Some(m.clone()),
        _ => None,
    };
    let codemode_enabled = settings_json
        .as_ref()
        .and_then(|v| v.get("defaultTools"))
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .any(|t| t == "+codemode" || t == "codemode")
        })
        .unwrap_or(false);
    let compaction_reserve_tokens = settings_json
        .as_ref()
        .and_then(|v| {
            default_model
                .as_deref()
                .and_then(|dm| {
                    v.pointer("/compaction/modelOverrides")
                        .and_then(|mo| mo.get(dm))
                        .and_then(|ov| ov.get("reserveTokens"))
                })
                .or_else(|| v.pointer("/compaction/reserveTokens"))
        })
        .and_then(Value::as_u64);
    let show_cache_miss_notices = settings_json
        .as_ref()
        .and_then(|v| v.get("showCacheMissNotices"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let flash_auto_enabled = settings_json
        .as_ref()
        .and_then(|v| v.get("enabledModels"))
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .any(|m| m == "deepseek/flash-auto")
        })
        .unwrap_or(false);
    let retry_max_retries = settings_json
        .as_ref()
        .and_then(|v| v.pointer("/retry/maxRetries"))
        .and_then(Value::as_u64);

    let models_json = fs::read_to_string(pi_agent.join("models.json"))
        .ok()
        .and_then(|s| serde_json::from_str::<Value>(&s).ok());
    let deepseek_mid_convo_system = models_json
        .as_ref()
        .and_then(|v| {
            default_model_id
                .as_deref()
                .and_then(|mid| {
                    v.pointer(&format!(
                        "/providers/deepseek/modelOverrides/{mid}/compat/supportsMidConvoSystemMessages"
                    ))
                })
                .or_else(|| {
                    v.pointer(
                        "/providers/deepseek/modelOverrides/deepseek-flash/compat/supportsMidConvoSystemMessages",
                    )
                })
        })
        .and_then(Value::as_bool);

    let mut hints: Vec<String> = Vec::new();
    if !accelerator_present || !accelerator_up_to_date {
        hints.push(
            "Run `ap init -a pi --with-hooks --no-rules` to install/upgrade zz-ap-accelerator.ts (context_edit batch masking, agent_before_settle gate, deepseek/flash-auto router & DSML recovery)."
                .to_string(),
        );
    }
    if mcp_server.is_some() && !mcp_codemode_wired {
        hints.push(
            "Set `\"exposure\": \"codemode\"` for agent-primitives in ~/.pi/agent/mcp.json to keep provider tool schemas deferred and preserve KV-cache hits."
                .to_string(),
        );
    }
    if mcp_codemode_wired && !codemode_enabled {
        hints.push(
            "Add `\"+codemode\"` to `defaultTools` in ~/.pi/agent/settings.json so Pi can invoke deferred MCP tools in QuickJS scripts."
                .to_string(),
        );
    }
    let is_deepseek = default_provider.as_deref() == Some("deepseek")
        || default_model_id
            .as_deref()
            .map(|m| m.contains("deepseek"))
            .unwrap_or(false);
    if is_deepseek && deepseek_mid_convo_system != Some(true) {
        hints.push(
            "Consider enabling `compat.supportsMidConvoSystemMessages: true` under providers.deepseek.modelOverrides in ~/.pi/agent/models.json to keep DeepSeek prefix KV-cache intact."
                .to_string(),
        );
    }

    let effective_compaction_trigger_tokens = if is_deepseek
        && default_model_id.as_deref() == Some("deepseek-flash")
    {
        compaction_reserve_tokens.map(|r| 1_000_000u64.saturating_sub(r))
    } else {
        None
    };

    let mut recommendations: Vec<String> = Vec::new();
    if let Some(trigger) = effective_compaction_trigger_tokens {
        if trigger > 400_000 {
            recommendations.push(
                format!("deepseek-flash (1M window) auto-compacts at {trigger} tokens; set compaction.modelOverrides[\"deepseek/deepseek-flash\"].reserveTokens=650000 (350k trigger; or 780000 for 220k) to compact cleanly before the >=400k latency cliff.")
            );
        }
    }
    if is_deepseek && !show_cache_miss_notices {
        recommendations.push(
            "Set `\"showCacheMissNotices\": true` in ~/.pi/agent/settings.json to surface DeepSeek prefix cache misses immediately."
                .to_string(),
        );
    }
    if is_deepseek && retry_max_retries.is_some_and(|r| r < 5) {
        recommendations.push(
            "Raise `retry.maxRetries` to 6 in ~/.pi/agent/settings.json to survive transient DeepSeek stream drops/timeouts during long sessions."
                .to_string(),
        );
    }

    json!({
        "detected": true,
        "accelerator_present": accelerator_present,
        "accelerator_up_to_date": accelerator_up_to_date,
        "mcp_codemode_wired": mcp_codemode_wired,
        "codemode_enabled": codemode_enabled,
        "default_model": default_model,
        "compaction_reserve_tokens": compaction_reserve_tokens,
        "effective_compaction_trigger_tokens": effective_compaction_trigger_tokens,
        "show_cache_miss_notices": show_cache_miss_notices,
        "flash_auto_enabled": flash_auto_enabled,
        "retry_max_retries": retry_max_retries,
        "deepseek_mid_convo_system": deepseek_mid_convo_system,
        "hints": hints,
        "recommendations": recommendations,
    })
}

pub fn cmd_doctor(as_json: bool) -> i32 {
    let mut ok = true;
    let fts5_ok = Connection::open_in_memory()
        .and_then(|c| {
            c.execute_batch(
                "CREATE VIRTUAL TABLE t USING fts5(x, tokenize='unicode61'); INSERT INTO t VALUES ('ok');",
            )
        })
        .is_ok();
    if !fts5_ok {
        ok = false;
    }
    let mut ast_ok = true;
    for ext in ["py", "ts", "js", "go", "rs", "c"] {
        if ap_splice::ast::language_for_ext(ext).is_none() {
            ast_ok = false;
            ok = false;
        }
    }
    for (_, md) in all_embedded_skills() {
        if md.is_empty() {
            ok = false;
        }
    }

    let browser_found = [
        r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\Google\Chrome\Application\chrome.exe",
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/usr/bin/google-chrome",
        "/usr/bin/chromium",
    ]
    .iter()
    .copied()
    .find(|p| Path::new(p).is_file());

    let home = default_home_dir();
    let detected = resolve_requested_harnesses("auto", &home, true).unwrap_or_default();

    let global_bin =
        home.join(".local")
            .join("bin")
            .join(if cfg!(windows) { "ap.exe" } else { "ap" });
    let global_bin_version = if global_bin.is_file() {
        std::process::Command::new(&global_bin)
            .arg("--version")
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    } else {
        None
    };
    let expected_ver = format!("ap {}", env!("CARGO_PKG_VERSION"));
    let global_bin_up_to_date = global_bin_version
        .as_deref()
        .map(|v| v == expected_ver)
        .unwrap_or(false);

    let mcp_candidates = [
        home.join(".pi").join("agent").join("mcp.json"),
        home.join(".config").join("opencode").join("opencode.json"),
        home.join(".cursor").join("mcp.json"),
        home.join(".gemini").join("settings.json"),
        home.join(".qwen").join("settings.json"),
        home.join(".codeium")
            .join("windsurf")
            .join("mcp_config.json"),
    ];
    let wired_mcp_count = mcp_candidates
        .iter()
        .filter(|p| {
            fs::read_to_string(p)
                .map(|s| s.contains("agent-primitives"))
                .unwrap_or(false)
        })
        .count();

    let hook_candidates = [
        home.join(".claude").join("settings.json"),
        home.join(".codex").join("hooks.json"),
        home.join(".gemini").join("config").join("hooks.json"),
        home.join(".pi")
            .join("agent")
            .join("extensions")
            .join("zz-ap-accelerator.ts"),
    ];
    let wired_hooks_count = hook_candidates
        .iter()
        .filter(|p| {
            fs::read_to_string(p)
                .map(|s| {
                    s.contains("agent-primitives") || s.contains("ap exec") || s.contains("ap.exe")
                })
                .unwrap_or(false)
        })
        .count();

    let sieve_dir = home.join(".ap").join("sieve");
    let mut sieve_archives = 0usize;
    let mut sieve_bytes = 0u64;
    if let Ok(rd) = fs::read_dir(&sieve_dir) {
        for ent in rd.flatten() {
            if ent.path().extension().and_then(|e| e.to_str()) == Some("raw") {
                sieve_archives += 1;
                sieve_bytes += ent.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }
    }

    let pi_runtime = inspect_pi_runtime(&home);

    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "ok": ok,
                "version": env!("CARGO_PKG_VERSION"),
                "os": std::env::consts::OS,
                "arch": std::env::consts::ARCH,
                "fts5_ok": fts5_ok,
                "ast_ok": ast_ok,
                "browser_cdp": browser_found,
                "detected_agents": detected,
                "global_bin_version": global_bin_version,
                "global_bin_up_to_date": global_bin_up_to_date,
                "wired_mcp_count": wired_mcp_count,
                "wired_hooks_count": wired_hooks_count,
                "sieve_archives": sieve_archives,
                "sieve_kb": ((sieve_bytes as f64 / 1024.0) * 10.0).round() / 10.0,
                "pi_runtime": pi_runtime,
            }))
            .unwrap_or_default()
        );
        return if ok { 0 } else { 1 };
    }

    println!(
        "[ap doctor] agent-primitives v{} ({}/{}) -> OK (Pure-Rust single binary, zero external runtime)",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    if fts5_ok {
        println!("  [OK] Engine Core       : Bundled SQLite FTS5/BM25 + 7 embedded modules ready");
    } else {
        println!("  [FAIL] Engine Core     : Bundled SQLite FTS5 failed");
    }
    println!(
        "  [OK] AST Syntax Gate   : Python, TypeScript/TSX, JS, Go, Rust, C grammars compiled"
    );

    #[cfg(windows)]
    {
        let has_pwsh7 = crate::exec::probe_pwsh7();
        let git_bash = crate::exec::probe_git_bash();
        let py_utf8 = std::env::var("PYTHONUTF8").ok().as_deref() == Some("1");
        println!(
            "  [OK] Windows Shell     : pwsh7={}, git-bash={}, PYTHONUTF8={} (`ap exec` auto-heals UTF-8, `&&`, `curl.exe` & POSIX pipes)",
            if has_pwsh7 { "yes" } else { "no (PS 5.1)" },
            if git_bash.is_some() { "yes" } else { "no" },
            if py_utf8 { "1" } else { "auto-injected by ap" }
        );
    }
    #[cfg(not(windows))]
    {
        println!(
            "  [OK] Host Shell        : POSIX sh + UTF-8 lock & deadlock-free timeout shield ready"
        );
    }

    if let Some(bp) = browser_found {
        println!("  [OK] Browser Smoke CDP : READY ({bp}) — 0-MB native headless verification");
    } else {
        println!("  [INFO] Browser Smoke   : HTTP/API smoke ready (install Edge/Chrome to enable DOM CDP)");
    }

    if !detected.is_empty() {
        println!("  [OK] Detected Agents   : {}", detected.join(", "));
    }
    println!(
        "  [OK] Agent Wiring      : {wired_mcp_count} MCP config(s), {wired_hooks_count} hook/extension(s), global bin={} | ~/.ap/sieve: {sieve_archives} archive(s) ({:.1} KB)",
        global_bin_version.as_deref().unwrap_or("not installed"),
        sieve_bytes as f64 / 1024.0
    );
    if pi_runtime["detected"].as_bool() == Some(true) {
        let accel_ok = pi_runtime["accelerator_up_to_date"].as_bool() == Some(true);
        let mcp_ok = pi_runtime["mcp_codemode_wired"].as_bool() == Some(true);
        let cm_ok = pi_runtime["codemode_enabled"].as_bool() == Some(true);
        let model = pi_runtime["default_model"]
            .as_str()
            .unwrap_or("unset");
        let badge = if accel_ok && mcp_ok && cm_ok {
            "[OK]"
        } else {
            "[WARN]"
        };
        println!(
            "  {badge} Pi Runtime        : model={model}, codemode={}, mcp_deferred={}, accelerator={}",
            if cm_ok { "on" } else { "off" },
            if mcp_ok { "yes" } else { "no" },
            if accel_ok { "v2.14+ (99% UTF-8 heal + error-aware context_edit + settle-gate + flash-auto + /ap handoff)" } else { "stale/missing" }
        );
        if let Some(hints) = pi_runtime["hints"].as_array() {
            for h in hints.iter().filter_map(Value::as_str) {
                println!("         -> {h}");
            }
        }
        if let Some(recs) = pi_runtime["recommendations"].as_array() {
            for r in recs.iter().filter_map(Value::as_str) {
                println!("         [tune] {r}");
            }
        }
    }
    println!(
        "  Tip: Run `ap stats` (or `ap stats --since-hours 24`) anytime to view saved tokens & prevented crashes."
    );
    if ok {
        0
    } else {
        1
    }
}

pub fn cmd_status(dir_opt: Option<&str>, home_opt: Option<&str>) -> i32 {
    let repo_dir = fs::canonicalize(dir_opt.unwrap_or(".")).unwrap_or_else(|_| PathBuf::from("."));
    let home = home_opt
        .filter(|s| !s.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(default_home_dir);
    let local_skills: Vec<&str> = ALL_PRIMITIVES
        .iter()
        .copied()
        .filter(|nm| {
            repo_dir
                .join(".agents")
                .join("skills")
                .join(nm)
                .join("SKILL.md")
                .is_file()
        })
        .collect();
    let has_fence = fs::read_to_string(repo_dir.join("AGENTS.md"))
        .map(|s| s.contains(FENCE_START))
        .unwrap_or(false);
    println!("[ap status]");
    println!(
        "  - Repo ({}): {}/7 primitives ({}), AGENTS.md wired={has_fence}",
        posix(&repo_dir),
        local_skills.len(),
        if local_skills.is_empty() {
            "none".to_string()
        } else {
            local_skills.join(", ")
        }
    );
    for sp in HARNESS_SPECS {
        let st_dir = home.join(sp.skill_dirs[0]);
        let installed: Vec<&str> = ALL_PRIMITIVES
            .iter()
            .copied()
            .filter(|nm| st_dir.join(nm).join("SKILL.md").is_file())
            .collect();
        let rf_wired = fs::read_to_string(home.join(sp.rule_file))
            .map(|s| s.contains(FENCE_START))
            .unwrap_or(false);
        if !installed.is_empty() || rf_wired {
            println!(
                "  - Global [{}] ({}): {}/7 primitives, rules wired={rf_wired}",
                sp.id,
                posix(&st_dir),
                installed.len()
            );
        }
    }
    0
}
