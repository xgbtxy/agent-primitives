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
            "3. **1-Call Deep Verify & Live CDP (`guard` / `probe`)**: Run `{ap_cmd} guard verify --cwd . --deep` (OWASP sec + git diff anti-test-cheating + vacuous/anti-slop + contracts + tests + API fuzz + Desktop/Mobile UI + active-file AST mutation in 1 pass). Discover local servers/CDP ports -> `{ap_cmd} probe ports`. Attach to live Chrome/Electron CDP -> `{ap_cmd} probe cdp --port <P> [--list|--target T --eval JS]`."
        ));
    }
    if ["probe", "sieve", "recall", "guard", "baton"]
        .iter()
        .any(|x| sel.contains(x))
    {
        bullets.push(format!(
            "4. **Read, Search, SQL, Exec & Undo**: `{ap_cmd} read <target> [--json-path P|--decode|--map-pos L:C|--entry E|--offset N --pretty|--render]` (Office/PDF/SQLite/`.map`/`.asar`/`.vsix`/PE UTF-16LE/minified JS/SPA); `{ap_cmd} search <pat> <path> [--multiline --block --pretty]` (AST blocks + `.map`/`.asar`/minified/binaries); `{ap_cmd} query`; `{ap_cmd} exec -c \"<cmd>\"`; `{ap_cmd} undo`."
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
    }

    // Zero-flag automatic MCP registration for free/open coding agents (OpenCode, Qwen Code, Gemini CLI)
    let mut mcp_paths = Vec::new();
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

pub fn cmd_doctor() -> i32 {
    println!(
        "[ap doctor] agent-primitives v{} ({}/{}) -> OK (Pure-Rust single binary, zero external runtime)",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    let mut ok = true;
    match Connection::open_in_memory().and_then(|c| {
        c.execute_batch("CREATE VIRTUAL TABLE t USING fts5(x, tokenize='unicode61'); INSERT INTO t VALUES ('ok');")
    }) {
        Ok(()) => println!("  [OK] Engine Core       : Bundled SQLite FTS5/BM25 + 7 embedded modules ready"),
        Err(e) => {
            println!("  [FAIL] Engine Core     : Bundled SQLite FTS5 failed ({e})");
            ok = false;
        }
    }
    for ext in ["py", "ts", "js", "go", "rs", "c"] {
        if ap_splice::ast::language_for_ext(ext).is_none() {
            ok = false;
        }
    }
    for (_, md) in all_embedded_skills() {
        if md.is_empty() {
            ok = false;
        }
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
    if let Some(bp) = browser_found {
        println!("  [OK] Browser Smoke CDP : READY ({bp}) — 0-MB native headless verification");
    } else {
        println!("  [INFO] Browser Smoke   : HTTP/API smoke ready (install Edge/Chrome to enable DOM CDP)");
    }

    let home = default_home_dir();
    if let Ok(detected) = resolve_requested_harnesses("auto", &home, true) {
        if !detected.is_empty() {
            println!("  [OK] Detected Agents   : {}", detected.join(", "));
        }
    }
    println!(
        "  Tip: Run `ap stats` (or `ap gain`) anytime to view saved tokens & prevented crashes."
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
