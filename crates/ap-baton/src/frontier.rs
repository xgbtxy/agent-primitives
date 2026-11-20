use crate::now_iso_utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FrontierStatus {
    Unvisited,
    Probed,
    Stopped,
    Confirmed,
    Deferred,
}

impl FrontierStatus {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "unvisited" | "todo" | "open" => Some(Self::Unvisited),
            "probed" | "visited" | "active" => Some(Self::Probed),
            "stopped" | "deadend" | "closed" | "skip" => Some(Self::Stopped),
            "confirmed" | "done" | "verified" => Some(Self::Confirmed),
            "deferred" | "later" => Some(Self::Deferred),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unvisited => "unvisited",
            Self::Probed => "probed",
            Self::Stopped => "stopped",
            Self::Confirmed => "confirmed",
            Self::Deferred => "deferred",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontierItem {
    pub id: String,
    pub axis: String,
    #[serde(default)]
    pub module: String,
    pub target: String,
    #[serde(default)]
    pub line: usize,
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub snippet: String,
    #[serde(default = "default_score")]
    pub score: u32,
    pub status: FrontierStatus,
    #[serde(default)]
    pub reason: String,
    #[serde(default)]
    pub evidence_ref: String,
    #[serde(default)]
    pub updated_at: String,
}

fn default_score() -> u32 {
    80
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FrontierState {
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    pub source_roots: Vec<String>,
    #[serde(default)]
    pub items: Vec<FrontierItem>,
}

fn fnv1a_hex(s: &str) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{:08x}", (h ^ (h >> 32)) as u32)
}

fn clip(s: &str, max_chars: usize) -> String {
    s.trim().chars().take(max_chars).collect()
}

pub fn resolve_store_path(cwd: &Path, store: Option<&str>) -> PathBuf {
    if let Some(s) = store.map(str::trim).filter(|s| !s.is_empty()) {
        let p = PathBuf::from(s);
        return if p.is_absolute() { p } else { cwd.join(p) };
    }
    crate::baton_dir(cwd).join("frontier.json")
}

pub fn load_frontier(store_path: &Path) -> FrontierState {
    if !store_path.exists() {
        return FrontierState::default();
    }
    fs::read_to_string(store_path)
        .ok()
        .and_then(|s| serde_json::from_str::<FrontierState>(&s).ok())
        .unwrap_or_default()
}

pub fn save_frontier(store_path: &Path, state: &mut FrontierState) -> Result<(), String> {
    state.updated_at = now_iso_utc();
    if let Some(parent) = store_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let tmp = store_path.with_extension(format!("json.{}.tmp", std::process::id()));
    let body = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    fs::write(&tmp, body).map_err(|e| e.to_string())?;
    if store_path.exists() {
        let _ = fs::remove_file(store_path);
    }
    fs::rename(&tmp, store_path).map_err(|e| e.to_string())
}

fn upsert_item(state: &mut FrontierState, item: FrontierItem) -> bool {
    if let Some(existing) = state.items.iter_mut().find(|x| {
        x.id == item.id
            || (x.axis == item.axis && x.target == item.target && x.symbol == item.symbol)
    }) {
        // Preserve already-explored status; only refresh metadata/snippet if empty
        if existing.snippet.is_empty() && !item.snippet.is_empty() {
            existing.snippet = item.snippet;
        }
        if existing.module.is_empty() && !item.module.is_empty() {
            existing.module = item.module;
        }
        return false;
    }
    state.items.push(item);
    true
}

pub fn seed_from_json_str(state: &mut FrontierState, raw: &str) -> Result<(usize, usize), String> {
    let val: Value = serde_json::from_str(raw).map_err(|e| format!("invalid JSON: {e}"))?;
    if let Some(root) = val
        .get("root")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
    {
        if !state.source_roots.iter().any(|r| r == root) {
            state.source_roots.push(root.to_string());
        }
    }
    let arr = if let Some(items) = val.get("items").and_then(|v| v.as_array()) {
        items.clone()
    } else if let Some(items) = val.as_array() {
        items.clone()
    } else {
        return Err("expected JSON object with `items` array or top-level array".to_string());
    };
    let now = now_iso_utc();
    let mut added = 0usize;
    let mut skipped = 0usize;
    for row in arr {
        let axis = row
            .get("axis")
            .and_then(|v| v.as_str())
            .unwrap_or("general")
            .trim()
            .to_string();
        let target = row
            .get("target")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if target.is_empty() {
            continue;
        }
        let symbol = row
            .get("symbol")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let module = row
            .get("module")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let line = row.get("line").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let snippet = clip(
            row.get("snippet")
                .or_else(|| row.get("note"))
                .and_then(|v| v.as_str())
                .unwrap_or(""),
            180,
        );
        let score = row.get("score").and_then(|v| v.as_u64()).unwrap_or(80) as u32;
        let id = row
            .get("id")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| {
                format!(
                    "{}:{}",
                    axis,
                    fnv1a_hex(&format!(
                        "{}|{}|{}",
                        axis,
                        target,
                        symbol.to_ascii_lowercase()
                    ))
                )
            });
        let inserted = upsert_item(
            state,
            FrontierItem {
                id,
                axis,
                module,
                target,
                line,
                symbol,
                snippet,
                score,
                status: FrontierStatus::Unvisited,
                reason: String::new(),
                evidence_ref: String::new(),
                updated_at: now.clone(),
            },
        );
        if inserted {
            added += 1;
        } else {
            skipped += 1;
        }
    }
    Ok((added, skipped))
}

pub fn pick_next_diverse<'a>(
    state: &'a FrontierState,
    top_k: usize,
    exclude_axes: &[String],
) -> Vec<&'a FrontierItem> {
    let excluded: HashSet<String> = exclude_axes
        .iter()
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    // Count how many already-explored (probed/stopped/confirmed) entries exist per axis
    let mut explored_per_axis: HashMap<String, usize> = HashMap::new();
    for it in &state.items {
        if matches!(
            it.status,
            FrontierStatus::Probed | FrontierStatus::Stopped | FrontierStatus::Confirmed
        ) {
            *explored_per_axis.entry(it.axis.clone()).or_insert(0) += 1;
        }
    }

    let mut unvisited: Vec<&FrontierItem> = state
        .items
        .iter()
        .filter(|it| it.status == FrontierStatus::Unvisited)
        .collect();
    if unvisited.is_empty() {
        return Vec::new();
    }

    // Prefer non-excluded axes whenever any exist
    let has_non_excluded = unvisited
        .iter()
        .any(|it| !excluded.contains(&it.axis.to_ascii_lowercase()));

    unvisited.sort_by(|a, b| {
        let a_ex = if has_non_excluded && excluded.contains(&a.axis.to_ascii_lowercase()) {
            1u8
        } else {
            0u8
        };
        let b_ex = if has_non_excluded && excluded.contains(&b.axis.to_ascii_lowercase()) {
            1u8
        } else {
            0u8
        };
        let a_cov = *explored_per_axis.get(&a.axis).unwrap_or(&0);
        let b_cov = *explored_per_axis.get(&b.axis).unwrap_or(&0);
        a_ex.cmp(&b_ex)
            .then_with(|| a_cov.cmp(&b_cov))
            .then_with(|| b.score.cmp(&a.score))
            .then_with(|| a.axis.cmp(&b.axis))
            .then_with(|| a.target.cmp(&b.target))
    });

    // Diversify across axes & modules while strictly respecting exploration-count tiers
    // (0-explored axes ALWAYS precede already-explored axes, even if they share a module)
    let mut picked: Vec<&FrontierItem> = Vec::new();
    let mut used_ids: HashSet<&str> = HashSet::new();
    let mut used_axes: HashSet<&str> = HashSet::new();
    let mut used_modules: HashSet<&str> = HashSet::new();

    // Pass 1a: strictly unexplored axes (cov == 0) + distinct axis + distinct module
    for it in &unvisited {
        if picked.len() >= top_k {
            break;
        }
        let cov = *explored_per_axis.get(&it.axis).unwrap_or(&0);
        if cov == 0
            && !used_axes.contains(it.axis.as_str())
            && !used_modules.contains(it.module.as_str())
        {
            picked.push(*it);
            used_ids.insert(it.id.as_str());
            used_axes.insert(it.axis.as_str());
            if !it.module.is_empty() {
                used_modules.insert(it.module.as_str());
            }
        }
    }
    // Pass 1b: remaining strictly unexplored axes (cov == 0), distinct axis even if same module
    for it in &unvisited {
        if picked.len() >= top_k {
            break;
        }
        let cov = *explored_per_axis.get(&it.axis).unwrap_or(&0);
        if cov == 0 && !used_ids.contains(it.id.as_str()) && !used_axes.contains(it.axis.as_str()) {
            picked.push(*it);
            used_ids.insert(it.id.as_str());
            used_axes.insert(it.axis.as_str());
        }
    }
    // Pass 2: distinct axis + distinct module across all remaining tiers
    for it in &unvisited {
        if picked.len() >= top_k {
            break;
        }
        if !used_ids.contains(it.id.as_str())
            && !used_axes.contains(it.axis.as_str())
            && !used_modules.contains(it.module.as_str())
        {
            picked.push(*it);
            used_ids.insert(it.id.as_str());
            used_axes.insert(it.axis.as_str());
            if !it.module.is_empty() {
                used_modules.insert(it.module.as_str());
            }
        }
    }
    // Pass 3: distinct axis across remaining tiers
    for it in &unvisited {
        if picked.len() >= top_k {
            break;
        }
        if !used_ids.contains(it.id.as_str()) && !used_axes.contains(it.axis.as_str()) {
            picked.push(*it);
            used_ids.insert(it.id.as_str());
            used_axes.insert(it.axis.as_str());
        }
    }
    // Pass 4: fill remaining slots by rank
    for it in &unvisited {
        if picked.len() >= top_k {
            break;
        }
        if !used_ids.contains(it.id.as_str()) {
            picked.push(*it);
            used_ids.insert(it.id.as_str());
        }
    }
    picked
}

fn extract_distinct_tokens(text: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    let generic: HashSet<&str> = [
        "src",
        "dist",
        "lib",
        "main",
        "index",
        "app",
        "file",
        "path",
        "json",
        "js",
        "ts",
        "py",
        "go",
        "rs",
        "java",
        "ext",
        "extension",
        "extensions",
        "node_modules",
        "resources",
        "package",
        "config",
        "server",
        "client",
        "service",
        "utils",
        "common",
        "core",
        "default",
        "require",
        "import",
        "const",
        "function",
        "return",
        "async",
        "await",
        "spawn",
        "exec",
    ]
    .into_iter()
    .collect();
    let mut buf = String::new();
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            buf.push(ch.to_ascii_lowercase());
        } else {
            if buf.len() >= 4 && !generic.contains(buf.as_str()) {
                out.insert(buf.clone());
            }
            buf.clear();
        }
    }
    if buf.len() >= 4 && !generic.contains(buf.as_str()) {
        out.insert(buf);
    }
    out
}

pub fn sync_stopped_from_text(state: &mut FrontierState, source_label: &str, text: &str) -> usize {
    let lines: Vec<String> = text
        .lines()
        .map(|l| l.trim().to_ascii_lowercase())
        .filter(|l| l.len() >= 8)
        .collect();
    if lines.is_empty() {
        return 0;
    }
    let now = now_iso_utc();
    let mut marked = 0usize;
    for it in &mut state.items {
        if it.status != FrontierStatus::Unvisited {
            continue;
        }
        let target_low = it.target.to_ascii_lowercase();
        let file_stem = Path::new(&it.target)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let sym_low = it.symbol.to_ascii_lowercase();
        let item_tokens =
            extract_distinct_tokens(&format!("{} {} {}", it.module, it.target, it.symbol));

        let hit = lines.iter().any(|ln| {
            // Exact target path match
            if !target_low.is_empty() && target_low.len() >= 10 && ln.contains(&target_low) {
                return true;
            }
            // File name + symbol match on the same line
            if file_stem.len() >= 6
                && sym_low.len() >= 5
                && ln.contains(&file_stem)
                && ln.contains(&sym_low)
            {
                return true;
            }
            // High-precision >=3 distinct token overlap on the same line
            if item_tokens.len() >= 3 {
                let hits = item_tokens
                    .iter()
                    .filter(|t| ln.contains(t.as_str()))
                    .count();
                if hits >= 3 {
                    return true;
                }
            }
            false
        });

        if hit {
            it.status = FrontierStatus::Stopped;
            it.reason = format!("matched prior record ({})", clip(source_label, 48));
            it.updated_at = now.clone();
            marked += 1;
        }
    }
    marked
}

pub fn status_summary_json(state: &FrontierState, top_k: usize) -> Value {
    let mut per_axis: BTreeMap<String, (usize, usize, usize, usize)> = BTreeMap::new();
    let mut counts: HashMap<&'static str, usize> = HashMap::new();
    for it in &state.items {
        *counts.entry(it.status.as_str()).or_insert(0) += 1;
        let e = per_axis.entry(it.axis.clone()).or_insert((0, 0, 0, 0));
        match it.status {
            FrontierStatus::Unvisited => e.0 += 1,
            FrontierStatus::Probed => e.1 += 1,
            FrontierStatus::Stopped => e.2 += 1,
            FrontierStatus::Confirmed => e.3 += 1,
            FrontierStatus::Deferred => e.0 += 1,
        }
    }
    let mut unexplored_axes = Vec::new();
    let mut explored_axes = Vec::new();
    let mut axis_matrix = Vec::new();
    for (ax, (unv, prb, stp, cnf)) in &per_axis {
        if *prb + *stp + *cnf == 0 && *unv > 0 {
            unexplored_axes.push(ax.clone());
        } else {
            explored_axes.push(ax.clone());
        }
        axis_matrix.push(json!({
            "axis": ax,
            "unvisited": unv,
            "probed": prb,
            "stopped": stp,
            "confirmed": cnf,
        }));
    }
    let next_items: Vec<Value> = pick_next_diverse(state, top_k, &[])
        .into_iter()
        .map(|it| serde_json::to_value(it).unwrap_or(Value::Null))
        .collect();
    json!({
        "updated_at": state.updated_at,
        "source_roots": state.source_roots,
        "total": state.items.len(),
        "unvisited": *counts.get("unvisited").unwrap_or(&0),
        "probed": *counts.get("probed").unwrap_or(&0),
        "stopped": *counts.get("stopped").unwrap_or(&0),
        "confirmed": *counts.get("confirmed").unwrap_or(&0),
        "deferred": *counts.get("deferred").unwrap_or(&0),
        "explored_axes": explored_axes,
        "unexplored_axes": unexplored_axes,
        "axis_matrix": axis_matrix,
        "next": next_items,
    })
}

pub fn render_status_md(state: &FrontierState, top_k: usize) -> String {
    let summary = status_summary_json(state, top_k);
    let total = summary["total"].as_u64().unwrap_or(0);
    let unv = summary["unvisited"].as_u64().unwrap_or(0);
    let prb = summary["probed"].as_u64().unwrap_or(0);
    let stp = summary["stopped"].as_u64().unwrap_or(0);
    let cnf = summary["confirmed"].as_u64().unwrap_or(0);
    let unexp: Vec<&str> = summary["unexplored_axes"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    let exp: Vec<&str> = summary["explored_axes"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    let mut out = Vec::new();
    out.push(format!(
        "# FRONTIER COVERAGE (total={total}: unvisited={unv}, probed={prb}, stopped={stp}, confirmed={cnf})"
    ));
    out.push(format!(
        "**Unexplored Axes ({})**: {}",
        unexp.len(),
        if unexp.is_empty() {
            "(none)".to_string()
        } else {
            unexp.join(", ")
        }
    ));
    if !exp.is_empty() {
        out.push(format!(
            "**Explored Axes ({})**: {}",
            exp.len(),
            exp.join(", ")
        ));
    }
    let next = pick_next_diverse(state, top_k, &[]);
    if !next.is_empty() {
        out.push("\n## Next Diverse Unvisited Targets".to_string());
        for it in next {
            out.push(format!(
                "- `{}` [{}] `{}:{}` (`{}`) — {}",
                it.id,
                it.axis,
                it.target,
                it.line,
                it.symbol,
                clip(&it.snippet, 100)
            ));
        }
    }
    format!("{}\n", out.join("\n"))
}

pub struct FrontierSeedOpts<'a> {
    pub store: Option<&'a str>,
    pub from_json: Option<&'a str>,
    pub from_stdin: bool,
    pub axis: Option<&'a str>,
    pub target: Option<&'a str>,
    pub module: Option<&'a str>,
    pub note: Option<&'a str>,
    pub score: u32,
    pub as_json: bool,
}

pub fn run_frontier_seed(cwd: &Path, opts: &FrontierSeedOpts<'_>) -> i32 {
    let store_path = resolve_store_path(cwd, opts.store);
    let mut state = load_frontier(&store_path);
    let (added, skipped) = if opts.from_stdin {
        let mut buf = String::new();
        if std::io::stdin().read_to_string(&mut buf).is_err() {
            eprintln!("[baton frontier] failed to read stdin");
            return 1;
        }
        match seed_from_json_str(&mut state, &buf) {
            Ok(pair) => pair,
            Err(e) => {
                eprintln!("[baton frontier] {e}");
                return 1;
            }
        }
    } else if let Some(path_str) = opts.from_json {
        let raw = match fs::read_to_string(path_str) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[baton frontier] cannot read {path_str}: {e}");
                return 1;
            }
        };
        match seed_from_json_str(&mut state, &raw) {
            Ok(pair) => pair,
            Err(e) => {
                eprintln!("[baton frontier] {e}");
                return 1;
            }
        }
    } else if let Some(tgt) = opts.target.map(str::trim).filter(|s| !s.is_empty()) {
        let ax = opts
            .axis
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("general");
        let id = format!("{}:{}", ax, fnv1a_hex(&format!("{ax}|{tgt}")));
        let ins = upsert_item(
            &mut state,
            FrontierItem {
                id,
                axis: ax.to_string(),
                module: opts.module.unwrap_or("").trim().to_string(),
                target: tgt.to_string(),
                line: 0,
                symbol: String::new(),
                snippet: clip(opts.note.unwrap_or(""), 180),
                score: opts.score,
                status: FrontierStatus::Unvisited,
                reason: String::new(),
                evidence_ref: String::new(),
                updated_at: now_iso_utc(),
            },
        );
        if ins {
            (1, 0)
        } else {
            (0, 1)
        }
    } else {
        eprintln!("[baton frontier] seed requires --from-json, --stdin, or --target");
        return 2;
    };

    if let Err(e) = save_frontier(&store_path, &mut state) {
        eprintln!("[baton frontier] save failed: {e}");
        return 1;
    }
    if opts.as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "ok": true,
                "added": added,
                "skipped_existing": skipped,
                "total": state.items.len(),
                "store": store_path.to_string_lossy(),
            }))
            .unwrap_or_default()
        );
    } else {
        println!(
            "[baton frontier] seeded +{added} new ({skipped} existing kept), total={}",
            state.items.len()
        );
    }
    0
}

pub fn run_frontier_mark(
    cwd: &Path,
    store: Option<&str>,
    query: &str,
    status_str: &str,
    reason: &str,
    evidence_ref: &str,
    as_json: bool,
) -> i32 {
    let Some(status) = FrontierStatus::parse(status_str) else {
        eprintln!("[baton frontier] invalid status '{status_str}' (use unvisited|probed|stopped|confirmed|deferred)");
        return 2;
    };
    let store_path = resolve_store_path(cwd, store);
    let mut state = load_frontier(&store_path);
    let q_low = query.trim().to_ascii_lowercase();
    let now = now_iso_utc();
    let mut updated_ids = Vec::new();
    for it in &mut state.items {
        let id_low = it.id.to_ascii_lowercase();
        let tgt_low = it.target.to_ascii_lowercase();
        if id_low == q_low
            || id_low.starts_with(&q_low)
            || (!q_low.is_empty() && q_low.len() >= 4 && tgt_low.contains(&q_low))
        {
            it.status = status.clone();
            if !reason.trim().is_empty() {
                it.reason = clip(reason, 180);
            }
            if !evidence_ref.trim().is_empty() {
                it.evidence_ref = clip(evidence_ref, 120);
            }
            it.updated_at = now.clone();
            updated_ids.push(it.id.clone());
        }
    }
    if updated_ids.is_empty() {
        if as_json {
            println!("{}", json!({"ok": false, "matched": 0, "query": query}));
        } else {
            println!("[baton frontier] no entry matched '{query}'");
        }
        return 1;
    }
    if let Err(e) = save_frontier(&store_path, &mut state) {
        eprintln!("[baton frontier] save failed: {e}");
        return 1;
    }
    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "ok": true,
                "status": status.as_str(),
                "matched": updated_ids.len(),
                "ids": updated_ids,
            }))
            .unwrap_or_default()
        );
    } else {
        println!(
            "[baton frontier] marked {} item(s) -> {} ({})",
            updated_ids.len(),
            status.as_str(),
            updated_ids.join(", ")
        );
    }
    0
}

pub fn run_frontier_next(
    cwd: &Path,
    store: Option<&str>,
    top_k: usize,
    exclude_axes: &[String],
    as_json: bool,
) -> i32 {
    let store_path = resolve_store_path(cwd, store);
    let state = load_frontier(&store_path);
    let picked = pick_next_diverse(&state, top_k, exclude_axes);
    let summary = status_summary_json(&state, top_k);
    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "total": state.items.len(),
                "unvisited": summary["unvisited"],
                "unexplored_axes": summary["unexplored_axes"],
                "explored_axes": summary["explored_axes"],
                "items": picked,
            }))
            .unwrap_or_default()
        );
    } else if picked.is_empty() {
        println!(
            "[baton frontier] no unvisited items remaining (total={}).",
            state.items.len()
        );
    } else {
        println!(
            "# NEXT DIVERSE FRONTIER ({} shown, {} unvisited total)",
            picked.len(),
            summary["unvisited"].as_u64().unwrap_or(0)
        );
        for it in picked {
            println!(
                "- `{}` [{}] `{}:{}` (`{}`) — {}",
                it.id,
                it.axis,
                it.target,
                it.line,
                it.symbol,
                clip(&it.snippet, 110)
            );
        }
    }
    0
}

pub fn run_frontier_status(cwd: &Path, store: Option<&str>, top_k: usize, as_json: bool) -> i32 {
    let store_path = resolve_store_path(cwd, store);
    let state = load_frontier(&store_path);
    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&status_summary_json(&state, top_k)).unwrap_or_default()
        );
    } else {
        print!("{}", render_status_md(&state, top_k));
    }
    0
}

pub fn run_frontier_sync_stopped(
    cwd: &Path,
    store: Option<&str>,
    text_files: &[PathBuf],
    as_json: bool,
) -> i32 {
    let store_path = resolve_store_path(cwd, store);
    let mut state = load_frontier(&store_path);
    let mut total_marked = 0usize;
    for tf in text_files {
        let Ok(txt) = fs::read_to_string(tf) else {
            continue;
        };
        let label = tf.file_name().and_then(|s| s.to_str()).unwrap_or("notes");
        total_marked += sync_stopped_from_text(&mut state, label, &txt);
    }
    if total_marked > 0 {
        let _ = save_frontier(&store_path, &mut state);
    }
    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "ok": true,
                "newly_stopped": total_marked,
                "total": state.items.len(),
            }))
            .unwrap_or_default()
        );
    } else {
        println!("[baton frontier] synced {total_marked} already-covered item(s) as stopped.");
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diverse_scheduler_prioritizes_unexplored_axes() {
        let mut st = FrontierState::default();
        let sample = json!({
            "root": "/tmp/app",
            "items": [
                {"id": "exec_spawn:1", "axis": "exec_spawn", "module": "ext/a", "target": "ext/a/run.js", "line": 10, "symbol": "spawnSync", "snippet": "spawnSync(x)", "score": 95},
                {"id": "exec_spawn:2", "axis": "exec_spawn", "module": "ext/b", "target": "ext/b/run.js", "line": 20, "symbol": "execFile", "snippet": "execFile(y)", "score": 94},
                {"id": "net_listen_rpc:1", "axis": "net_listen_rpc", "module": "ls/java", "target": "ls/java/SbtServer.java", "line": 42, "symbol": "ServerSocket", "snippet": "new ServerSocket(0)", "score": 92},
                {"id": "fs_path_io:1", "axis": "fs_path_io", "module": "ls/java", "target": "ls/java/Reader.java", "line": 88, "symbol": "Files.newBufferedReader", "snippet": "Files.newBufferedReader(p)", "score": 82}
            ]
        });
        let (added, _) = seed_from_json_str(&mut st, &sample.to_string()).unwrap();
        assert_eq!(added, 4);
        // Mark exec_spawn:1 as stopped -> exec_spawn now has 1 explored, while net_listen_rpc & fs_path_io have 0!
        st.items[0].status = FrontierStatus::Stopped;
        let next = pick_next_diverse(&st, 2, &[]);
        assert_eq!(next.len(), 2);
        // Must surface unexplored axes FIRST, NOT exec_spawn:2 despite higher score!
        assert_eq!(next[0].axis, "net_listen_rpc");
        assert_eq!(next[1].axis, "fs_path_io");
    }

    #[test]
    fn sync_stopped_marks_exact_matches_only() {
        let mut st = FrontierState::default();
        let sample = json!({
            "items": [
                {"id": "exec_spawn:1", "axis": "exec_spawn", "module": "ext/wizard", "target": "ext/wizard/git_runner.js", "line": 10, "symbol": "spawnSync", "score": 90},
                {"id": "net_listen_rpc:1", "axis": "net_listen_rpc", "module": "ls/sbt", "target": "ls/sbt/SbtPort.java", "line": 42, "symbol": "ServerSocket", "score": 92}
            ]
        });
        seed_from_json_str(&mut st, &sample.to_string()).unwrap();
        let notes = "- ext/wizard/git_runner.js spawnSync git -v checked and verified safe\n";
        let n = sync_stopped_from_text(&mut st, "audit_notes.md", notes);
        assert_eq!(n, 1);
        assert_eq!(st.items[0].status, FrontierStatus::Stopped);
        assert_eq!(st.items[1].status, FrontierStatus::Unvisited);
    }
}
