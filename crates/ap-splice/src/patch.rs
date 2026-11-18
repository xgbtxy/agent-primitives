use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditOp {
    pub path: String,
    pub op: String,
    #[serde(default)]
    pub old: String,
    #[serde(default)]
    pub new: String,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub all: bool,
}

fn patch_regexes() -> &'static (Regex, Regex, Regex, Regex) {
    static RE: OnceLock<(Regex, Regex, Regex, Regex)> = OnceLock::new();
    RE.get_or_init(|| {
        let hdr =
            Regex::new(r"(?i)^@@\s*(FILE_ALL|FILE|EDIT|REPLACE|SYMBOL|WRITE)\s*:\s*(.+?)\s*$")
                .unwrap();
        let open = Regex::new(r"(?i)^<<<(?:\s*(?:OLD|SEARCH|EOF))?\s*$").unwrap();
        let mid = Regex::new(r"(?i)^===(?:\s*(?:NEW|REPLACE))?\s*$").unwrap();
        let close = Regex::new(r"(?i)^>>>(?:\s*(?:END|REPLACE|EOF))?\s*$").unwrap();
        (hdr, open, mid, close)
    })
}

pub fn parse_text_spec(raw_spec: &str) -> Result<Vec<EditOp>, String> {
    let (hdr_re, open_re, mid_re, close_re) = patch_regexes();
    let norm = raw_spec.replace("\r\n", "\n");
    let lines: Vec<&str> = norm.split('\n').collect();
    let mut ops = Vec::new();
    let mut i = 0usize;
    let n = lines.len();

    while i < n {
        let ln = lines[i];
        if ln.trim().is_empty() {
            i += 1;
            continue;
        }
        let Some(cap) = hdr_re.captures(ln) else {
            return Err(format!(
                "Line {}: expected `@@ FILE: <path>`, `@@ SYMBOL: <path> :: <Sym>`, or `@@ WRITE: <path>`, got: {:?}",
                i + 1,
                ln
            ));
        };
        let kind = cap[1].to_ascii_uppercase();
        let rest = cap[2].trim().to_string();
        i += 1;

        match kind.as_str() {
            "FILE" | "FILE_ALL" | "EDIT" | "REPLACE" => {
                let replace_all = kind == "FILE_ALL";
                let mut hunks = 0usize;
                while i < n {
                    if hdr_re.is_match(lines[i]) {
                        break;
                    }
                    if lines[i].trim().is_empty() {
                        i += 1;
                        continue;
                    }
                    if !open_re.is_match(lines[i]) {
                        return Err(format!(
                            "Line {} ({rest}): expected `<<<` to start search block, got: {:?}",
                            i + 1,
                            lines[i]
                        ));
                    }
                    i += 1;
                    let mut old_buf = Vec::new();
                    while i < n && !mid_re.is_match(lines[i]) {
                        old_buf.push(lines[i]);
                        i += 1;
                    }
                    if i >= n {
                        return Err(format!(
                            "Unclosed `<<<` block in `{rest}`: missing `===` separator."
                        ));
                    }
                    i += 1;
                    let mut new_buf = Vec::new();
                    while i < n && !close_re.is_match(lines[i]) {
                        new_buf.push(lines[i]);
                        i += 1;
                    }
                    if i >= n {
                        return Err(format!(
                            "Unclosed `===` block in `{rest}`: missing `>>>` terminator."
                        ));
                    }
                    i += 1;
                    ops.push(EditOp {
                        path: rest.clone(),
                        op: "replace".to_string(),
                        old: old_buf.join("\n"),
                        new: new_buf.join("\n"),
                        content: None,
                        symbol: String::new(),
                        all: replace_all,
                    });
                    hunks += 1;
                }
                if hunks == 0 {
                    return Err(format!(
                        "`@@ {kind}: {rest}` had no `<<< ... === ... >>>` edit blocks."
                    ));
                }
            }
            "SYMBOL" => {
                let Some((p_raw, s_raw)) = rest.split_once("::") else {
                    return Err(format!(
                        "`@@ SYMBOL: {rest}` must use `@@ SYMBOL: <path> :: <SymbolName>`."
                    ));
                };
                let path = p_raw.trim().to_string();
                let sym = s_raw.trim().to_string();
                while i < n && lines[i].trim().is_empty() {
                    i += 1;
                }
                let mut body_buf = Vec::new();
                if i < n && open_re.is_match(lines[i]) {
                    i += 1;
                    while i < n && !close_re.is_match(lines[i]) {
                        body_buf.push(lines[i]);
                        i += 1;
                    }
                    if i >= n {
                        return Err(format!(
                            "Unclosed `@@ SYMBOL: {path} :: {sym}` block: missing `>>>`."
                        ));
                    }
                    i += 1;
                } else {
                    while i < n && !hdr_re.is_match(lines[i]) {
                        body_buf.push(lines[i]);
                        i += 1;
                    }
                }
                ops.push(EditOp {
                    path,
                    op: "replace_symbol".to_string(),
                    old: String::new(),
                    new: body_buf.join("\n"),
                    content: None,
                    symbol: sym,
                    all: false,
                });
            }
            "WRITE" => {
                let path = rest;
                let mut j = i;
                while j < n && lines[j].trim().is_empty() {
                    j += 1;
                }
                let content_s = if j < n && open_re.is_match(lines[j]) {
                    i = j + 1;
                    let mut w_buf = Vec::new();
                    while i < n && !close_re.is_match(lines[i]) {
                        w_buf.push(lines[i]);
                        i += 1;
                    }
                    if i >= n {
                        return Err(format!("Unclosed `@@ WRITE: {path}` block: missing `>>>`."));
                    }
                    i += 1;
                    format!("{}\n", w_buf.join("\n"))
                } else {
                    let mut w_buf = Vec::new();
                    while i < n && !hdr_re.is_match(lines[i]) {
                        w_buf.push(lines[i]);
                        i += 1;
                    }
                    let mut s = w_buf.join("\n");
                    if !s.is_empty() && !s.ends_with('\n') {
                        s.push('\n');
                    }
                    s
                };
                ops.push(EditOp {
                    path,
                    op: "write".to_string(),
                    old: String::new(),
                    new: String::new(),
                    content: Some(content_s),
                    symbol: String::new(),
                    all: false,
                });
            }
            _ => unreachable!(),
        }
    }
    Ok(ops)
}

pub fn parse_edit_spec(raw_spec: &str) -> Result<Vec<EditOp>, String> {
    let stripped = raw_spec.trim_start_matches(|c: char| c == '\u{feff}' || c.is_whitespace());
    if stripped.starts_with("@@") {
        return parse_text_spec(stripped);
    }
    let val: Value = serde_json::from_str(stripped).map_err(|e| e.to_string())?;
    match val {
        Value::Array(_) => serde_json::from_value::<Vec<EditOp>>(val).map_err(|e| e.to_string()),
        Value::Object(mut map) => {
            if let Some(edits) = map.remove("edits") {
                serde_json::from_value::<Vec<EditOp>>(edits).map_err(|e| e.to_string())
            } else {
                serde_json::from_value::<EditOp>(Value::Object(map))
                    .map(|op| vec![op])
                    .map_err(|e| e.to_string())
            }
        }
        _ => Ok(Vec::new()),
    }
}

pub fn replace_crlf_tolerant(
    content_lf: &str,
    old_lf: &str,
    new_lf: &str,
    replace_all: bool,
) -> Result<(String, usize), String> {
    if old_lf.is_empty() {
        return Err("`old` cannot be empty.".to_string());
    }
    let cnt = content_lf.matches(old_lf).count();
    if cnt == 1 || (cnt > 1 && replace_all) {
        let updated = if replace_all {
            content_lf.replace(old_lf, new_lf)
        } else {
            content_lf.replacen(old_lf, new_lf, 1)
        };
        return Ok((updated, cnt));
    }
    if cnt > 1 && !replace_all {
        return Err(format!(
            "`old` matched {cnt} locations (ambiguous); include 1-2 surrounding lines or use `@@ FILE_ALL:` / `\"all\": true`."
        ));
    }

    let src_lines: Vec<&str> = content_lf.split('\n').collect();
    let old_lines: Vec<&str> = old_lf.split('\n').collect();
    let src_stripped: Vec<&str> = src_lines.iter().map(|l| l.trim_end()).collect();
    let old_stripped: Vec<&str> = old_lines.iter().map(|l| l.trim_end()).collect();
    let n_old = old_stripped.len();
    if n_old > src_stripped.len() {
        return Err("`old` target block not found in file.".to_string());
    }
    let hits: Vec<usize> = (0..=(src_stripped.len() - n_old))
        .filter(|&i| src_stripped[i..i + n_old] == old_stripped[..])
        .collect();
    if hits.len() == 1 {
        let idx = hits[0];
        let mut updated: Vec<&str> = Vec::new();
        updated.extend_from_slice(&src_lines[..idx]);
        updated.extend(new_lf.split('\n'));
        updated.extend_from_slice(&src_lines[idx + n_old..]);
        return Ok((updated.join("\n"), 1));
    }
    if hits.len() > 1 && !replace_all {
        return Err(format!(
            "`old` matched {} locations via rstrip alignment; add context lines.",
            hits.len()
        ));
    }

    // Tier 3 (Aider-style uniform indentation-delta fallback when unambiguous)
    let min_space = |ls: &[&str]| -> usize {
        ls.iter()
            .filter(|l| !l.is_empty())
            .map(|l| l.bytes().take_while(|b| *b == b' ').count())
            .min()
            .unwrap_or(0)
    };
    fn strip_n(l: &str, n: usize) -> &str {
        let take = l.bytes().take_while(|b| *b == b' ').count().min(n);
        &l[take..]
    }

    let has_substance = n_old >= 2
        || old_stripped
            .first()
            .map(|l| l.trim().len() >= 12)
            .unwrap_or(false);
    if has_substance && !replace_all {
        let old_ind = min_space(&old_stripped);
        let old_dedent: Vec<&str> = old_stripped.iter().map(|l| strip_n(l, old_ind)).collect();
        let mut indent_hits: Vec<(usize, usize)> = Vec::new();
        for i in 0..=(src_stripped.len() - n_old) {
            let win = &src_stripped[i..i + n_old];
            let win_ind = min_space(win);
            if win
                .iter()
                .zip(&old_dedent)
                .all(|(w, o)| strip_n(w, win_ind) == *o)
            {
                indent_hits.push((i, win_ind));
            }
        }
        if indent_hits.len() == 1 {
            let (idx, win_ind) = indent_hits[0];
            let delta = (win_ind as isize) - (old_ind as isize);
            let shifted_new: Vec<String> = new_lf
                .split('\n')
                .map(|l| {
                    if l.trim().is_empty() {
                        String::new()
                    } else {
                        let cur = l.bytes().take_while(|b| *b == b' ').count();
                        let next_ind = ((cur as isize) + delta).max(0) as usize;
                        format!("{}{}", " ".repeat(next_ind), &l[cur..])
                    }
                })
                .collect();
            let mut updated: Vec<String> = Vec::with_capacity(src_lines.len() + shifted_new.len());
            updated.extend(src_lines[..idx].iter().map(|s| (*s).to_string()));
            updated.extend(shifted_new);
            updated.extend(src_lines[idx + n_old..].iter().map(|s| (*s).to_string()));
            return Ok((updated.join("\n"), 1));
        }
        if indent_hits.len() > 1 {
            return Err(format!(
                "`old` matched {} locations after indentation normalization; add surrounding lines.",
                indent_hits.len()
            ));
        }
    }

    // SWE-agent ACI diagnostic hint: surface closest matching first line so the LLM fixes without re-reading
    if let Some(first_anchor) = old_stripped.iter().map(|l| l.trim()).find(|l| l.len() >= 6) {
        if let Some((ln_idx, actual)) = src_lines
            .iter()
            .enumerate()
            .find(|(_, l)| l.contains(first_anchor))
        {
            return Err(format!(
                "`old` target block not found (closest anchor at L{}: `{}`).",
                ln_idx + 1,
                actual.trim()
            ));
        }
    }
    Err("`old` target block not found in file.".to_string())
}

pub fn dedent_and_reindent(new_block: &str, col_indent: usize) -> Vec<String> {
    let clean = new_block.replace("\r\n", "\n");
    let trimmed = clean.trim_end_matches('\n');
    let raw_lines: Vec<&str> = trimmed.split('\n').collect();
    let min_indent = raw_lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.chars().take_while(|c| *c == ' ').count())
        .min()
        .unwrap_or(0);
    let pad = " ".repeat(col_indent);
    raw_lines
        .into_iter()
        .map(|l| {
            if l.trim().is_empty() {
                String::new()
            } else {
                let stripped: String = l.chars().skip(min_indent).collect();
                format!("{pad}{stripped}")
            }
        })
        .collect()
}
