use regex::Regex;
use serde_json::Value;
use std::io::{Cursor, Read};
use zip::ZipArchive;

/// Mini-beautifier for single-line / minified JS & JSON snippets (zero dependencies).
pub fn pretty_format_js_snippet(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len() + raw.len() / 4);
    let mut indent: usize = 0;
    let mut in_str: Option<char> = None;
    let mut escape = false;
    let mut paren_depth: usize = 0;
    let chars: Vec<char> = raw.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if let Some(q) = in_str {
            out.push(ch);
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == q {
                in_str = None;
            }
            i += 1;
            continue;
        }
        match ch {
            '"' | '\'' | '`' => {
                in_str = Some(ch);
                out.push(ch);
            }
            '(' => {
                paren_depth += 1;
                out.push(ch);
            }
            ')' => {
                paren_depth = paren_depth.saturating_sub(1);
                out.push(ch);
            }
            '{' => {
                out.push('{');
                indent = (indent + 1).min(8);
                out.push('\n');
                out.push_str(&"  ".repeat(indent));
            }
            '}' => {
                indent = indent.saturating_sub(1);
                if !out.ends_with('\n') {
                    out.push('\n');
                }
                out.push_str(&"  ".repeat(indent));
                out.push('}');
                if i + 1 < chars.len() && !matches!(chars[i + 1], ';' | ',' | ')' | '\n') {
                    out.push('\n');
                    out.push_str(&"  ".repeat(indent));
                }
            }
            ';' if paren_depth == 0 => {
                out.push(';');
                out.push('\n');
                out.push_str(&"  ".repeat(indent));
            }
            '\n' | '\r' => {
                if !out.ends_with('\n') {
                    out.push('\n');
                    out.push_str(&"  ".repeat(indent));
                }
            }
            _ => out.push(ch),
        }
        i += 1;
    }
    out.trim().to_string()
}

#[derive(Debug, Clone)]
pub struct AsarEntry {
    pub path: String,
    pub offset: usize,
    pub size: usize,
    pub unpacked: bool,
}

fn read_u32_le(b: &[u8], off: usize) -> Option<u32> {
    let s = b.get(off..off + 4)?;
    Some(u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
}

fn read_u16_le(b: &[u8], off: usize) -> Option<u16> {
    let s = b.get(off..off + 2)?;
    Some(u16::from_le_bytes([s[0], s[1]]))
}

fn walk_asar_tree(node: &Value, prefix: &str, out: &mut Vec<AsarEntry>) {
    let Some(files) = node.get("files").and_then(Value::as_object) else {
        return;
    };
    for (name, val) in files {
        let full = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}/{name}")
        };
        if val.get("files").is_some() {
            walk_asar_tree(val, &full, out);
        } else if let Some(sz) = val.get("size").and_then(Value::as_u64) {
            let unpacked = val
                .get("unpacked")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let off = val
                .get("offset")
                .and_then(|v| {
                    v.as_u64()
                        .or_else(|| v.as_str().and_then(|s| s.parse::<u64>().ok()))
                })
                .unwrap_or(0) as usize;
            out.push(AsarEntry {
                path: full,
                offset: off,
                size: sz as usize,
                unpacked,
            });
        }
    }
}

/// Parse Electron `.asar` Chromium Pickle header -> (base_data_offset, sorted entries).
pub fn parse_asar_index(bytes: &[u8]) -> Result<(usize, Vec<AsarEntry>), String> {
    if bytes.len() < 16 {
        return Err("invalid .asar file (too short)".to_string());
    }
    let header_pickle_sz = read_u32_le(bytes, 4).ok_or("asar header truncated")? as usize;
    let json_len = read_u32_le(bytes, 12).ok_or("asar json_len truncated")? as usize;
    let json_end = 16usize.saturating_add(json_len);
    if json_end > bytes.len() {
        return Err("invalid .asar header JSON bounds".to_string());
    }
    let header_json: Value = serde_json::from_slice(&bytes[16..json_end])
        .map_err(|e| format!("invalid .asar JSON header: {e}"))?;
    let base_offset = 8usize.saturating_add(header_pickle_sz);
    let mut entries = Vec::new();
    walk_asar_tree(&header_json, "", &mut entries);
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok((base_offset, entries))
}

pub fn read_asar_member(bytes: &[u8], entry_path: &str) -> Result<Vec<u8>, String> {
    let (base, entries) = parse_asar_index(bytes)?;
    let norm = entry_path.trim_start_matches("./").trim_start_matches('/');
    let hit = entries
        .iter()
        .find(|e| e.path.eq_ignore_ascii_case(norm) || e.path.ends_with(&format!("/{norm}")))
        .ok_or_else(|| format!("entry `{entry_path}` not found in .asar"))?;
    if hit.unpacked {
        return Err(format!(
            "entry `{}` is marked unpacked (lives in app.asar.unpacked)",
            hit.path
        ));
    }
    let start = base.saturating_add(hit.offset);
    let end = start.saturating_add(hit.size);
    if end > bytes.len() {
        return Err(format!("entry `{}` offset out of bounds", hit.path));
    }
    Ok(bytes[start..end].to_vec())
}

pub fn list_asar_md(bytes: &[u8], query: Option<&str>) -> Result<String, String> {
    let (_, entries) = parse_asar_index(bytes)?;
    let q_low = query.map(|q| q.to_ascii_lowercase());
    let mut lines = vec![format!(
        "## Electron ASAR Archive ({} files — pass `--entry <path>` to read without extracting)",
        entries.len()
    )];
    let mut shown = 0usize;
    for e in &entries {
        if let Some(ref q) = q_low {
            if !e.path.to_ascii_lowercase().contains(q) {
                continue;
            }
        }
        let tag = if e.unpacked { " [unpacked]" } else { "" };
        lines.push(format!("- `{}` ({} B){tag}", e.path, e.size));
        shown += 1;
        if shown >= 200 {
            lines.push(format!(
                "... ({} more entries; narrow with `--query <substr>`)",
                entries.len().saturating_sub(shown)
            ));
            break;
        }
    }
    Ok(lines.join("\n"))
}

pub fn read_zip_member(bytes: &[u8], entry_path: &str) -> Result<Vec<u8>, String> {
    let mut za = ZipArchive::new(Cursor::new(bytes)).map_err(|e| format!("zip error: {e}"))?;
    let norm = entry_path.trim_start_matches("./").trim_start_matches('/');
    let mut matched_name = None;
    for i in 0..za.len() {
        if let Ok(f) = za.by_index(i) {
            let nm = f.name().replace('\\', "/");
            if nm.eq_ignore_ascii_case(norm) || nm.ends_with(&format!("/{norm}")) {
                matched_name = Some(f.name().to_string());
                break;
            }
        }
    }
    let Some(name) = matched_name else {
        return Err(format!("entry `{entry_path}` not found in archive"));
    };
    let f = za.by_name(&name).map_err(|e| format!("zip entry: {e}"))?;
    let mut buf = Vec::new();
    let _ = f.take(16 * 1024 * 1024).read_to_end(&mut buf);
    Ok(buf)
}

pub fn is_binary_magic(bytes: &[u8]) -> bool {
    bytes.starts_with(b"MZ")
        || bytes.starts_with(b"\x7FELF")
        || bytes.starts_with(b"\x00asm")
        || bytes.starts_with(&[0xCF, 0xFA, 0xED, 0xFE])
        || bytes.starts_with(&[0xCE, 0xFA, 0xED, 0xFE])
        || bytes.starts_with(&[0xCA, 0xFE, 0xBA, 0xBE])
}

/// Inspect PE / ELF / Mach-O / WASM headers + ASCII & Windows UTF-16LE wide strings.
pub fn inspect_binary_md(bytes: &[u8], query: Option<&str>, max_strings: usize) -> String {
    let mut out = Vec::new();
    if bytes.starts_with(b"MZ") && bytes.len() > 0x44 {
        let pe_off = read_u32_le(bytes, 0x3C).unwrap_or(0) as usize;
        if bytes.get(pe_off..pe_off + 4) == Some(b"PE\0\0") {
            let machine = read_u16_le(bytes, pe_off + 4).unwrap_or(0);
            let n_sec = read_u16_le(bytes, pe_off + 6).unwrap_or(0) as usize;
            let opt_sz = read_u16_le(bytes, pe_off + 20).unwrap_or(0) as usize;
            let arch = match machine {
                0x8664 => "x86_64",
                0x014c => "x86",
                0xaa64 => "ARM64",
                _ => "other",
            };
            let mut secs = Vec::new();
            let sec_base = pe_off + 24 + opt_sz;
            for s in 0..n_sec.min(16) {
                let off = sec_base + s * 40;
                if let Some(nm) = bytes.get(off..off + 8) {
                    let sname = String::from_utf8_lossy(nm)
                        .trim_end_matches('\0')
                        .to_string();
                    if !sname.is_empty() {
                        secs.push(sname);
                    }
                }
            }
            out.push(format!(
                "## Binary Header: Windows PE ({arch}, {} B, sections: [{}])",
                bytes.len(),
                secs.join(", ")
            ));
        } else {
            out.push(format!("## Binary Header: DOS/MZ ({} B)", bytes.len()));
        }
    } else if bytes.starts_with(b"\x7FELF") && bytes.len() >= 20 {
        let bits = if bytes[4] == 2 { "64-bit" } else { "32-bit" };
        let mach = read_u16_le(bytes, 18).unwrap_or(0);
        let arch = match mach {
            0x3E => "x86_64",
            0xB7 => "AArch64",
            0x03 => "x86",
            0x28 => "ARM",
            0xF3 => "RISC-V",
            _ => "other",
        };
        out.push(format!(
            "## Binary Header: Linux ELF {bits} ({arch}, {} B)",
            bytes.len()
        ));
    } else if bytes.starts_with(b"\x00asm") {
        out.push(format!("## Binary Header: WebAssembly ({} B)", bytes.len()));
    } else if bytes.starts_with(&[0xCF, 0xFA, 0xED, 0xFE])
        || bytes.starts_with(&[0xCA, 0xFE, 0xBA, 0xBE])
    {
        out.push(format!(
            "## Binary Header: macOS Mach-O ({} B)",
            bytes.len()
        ));
    } else {
        out.push(format!("## Binary File ({} B)", bytes.len()));
    }

    let q_re = query.and_then(|q| {
        Regex::new(&format!("(?i){q}"))
            .or_else(|_| Regex::new(&format!("(?i){}", regex::escape(q))))
            .ok()
    });
    let mut hits: Vec<String> = Vec::new();
    let scan_lim = bytes.len().min(8 * 1024 * 1024);
    let data = &bytes[..scan_lim];

    // 1. ASCII printable strings (len >= 6)
    let mut i = 0usize;
    while i < data.len() && hits.len() < max_strings {
        if (0x20..=0x7E).contains(&data[i]) {
            let st = i;
            while i < data.len() && (0x20..=0x7E).contains(&data[i]) && i - st < 240 {
                i += 1;
            }
            if i - st >= 6 {
                let s = String::from_utf8_lossy(&data[st..i]);
                if q_re.as_ref().map(|r| r.is_match(&s)).unwrap_or(true) {
                    hits.push(format!("[ascii @0x{st:x}] {s}"));
                }
            }
        } else {
            i += 1;
        }
    }

    // 2. Windows UTF-16LE wide strings (len >= 5 chars)
    let mut j = 0usize;
    while j + 1 < data.len() && hits.len() < max_strings {
        if (0x20..=0x7E).contains(&data[j]) && data[j + 1] == 0 {
            let st = j;
            let mut s = String::new();
            while j + 1 < data.len()
                && (0x20..=0x7E).contains(&data[j])
                && data[j + 1] == 0
                && s.len() < 200
            {
                s.push(data[j] as char);
                j += 2;
            }
            if s.len() >= 5 && q_re.as_ref().map(|r| r.is_match(&s)).unwrap_or(true) {
                hits.push(format!("[utf16le @0x{st:x}] {s}"));
            }
        } else {
            j += 1;
        }
    }

    out.push(format!(
        "### Extracted Strings ({} shown; scans ASCII + Windows UTF-16LE — narrow with `--query <regex>`)",
        hits.len()
    ));
    if hits.is_empty() {
        out.push("*(no matching printable ASCII/UTF-16LE strings)*".to_string());
    } else {
        out.extend(hits);
    }
    out.join("\n")
}
