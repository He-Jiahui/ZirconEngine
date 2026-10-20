//! Fail-closed declaration identities and cfg reachability for the unchanged runtime root.
use std::collections::BTreeSet;

const HEADER: &str = r#"#![cfg_attr(all(test,windows),recursion_limit="256")]"#;
const OTHER_ITEMS: [&str; 5] = [
    "extern crate self as zircon_runtime;",
    r#"#[cfg(all(feature="native-host",target_arch="wasm32"))]compile_error!("native-host requires a native target; use --no-default-features --features core-min for the portable Wasm core");"#,
    r#"#[cfg(feature="dev-dynamic-linking")]#[allow(unused_imports,clippy::single_component_path_imports)]use zr_dev_deps_dylib as _;"#,
    "pub use crate::core::resource;",
    "pub use zircon_runtime_reflection_macros::{zircon_host_function,zircon_host_module,ZirconScriptType,};",
];

fn tokens(source: &str) -> Result<Vec<String>, String> {
    let bytes = source.as_bytes();
    let (mut result, mut i) = (Vec::new(), 0);
    while i < bytes.len() {
        if b" \t\r\n".contains(&bytes[i]) {
            i += 1;
        } else if bytes[i..].starts_with(b"//") {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
        } else if bytes[i..].starts_with(b"/*") {
            let mut depth = 1;
            i += 2;
            while i < bytes.len() && depth > 0 {
                if bytes[i..].starts_with(b"/*") {
                    depth += 1;
                    i += 2;
                } else if bytes[i..].starts_with(b"*/") {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            if depth != 0 {
                return Err("unterminated root comment".into());
            }
        } else {
            let start = i;
            if bytes[i] == b'"' {
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    i += if bytes[i] == b'\\' { 2 } else { 1 };
                }
                if i >= bytes.len() {
                    return Err("unterminated root string".into());
                }
                i += 1;
            } else if bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' {
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
            } else if b"#![](){},;:=".contains(&bytes[i]) {
                i += 1;
            } else {
                return Err(format!("unknown root token at {i}"));
            }
            result.push(source[start..i].to_owned());
        }
    }
    Ok(result)
}

fn items(source: &str) -> Result<Vec<Vec<String>>, String> {
    let all = tokens(source)?;
    let header = tokens(HEADER)?;
    if !all.starts_with(&header) {
        return Err("root header identity changed".into());
    }
    let all = &all[header.len()..];
    let (mut result, mut stack, mut start) = (Vec::new(), Vec::new(), 0);
    for (i, token) in all.iter().enumerate() {
        match token.as_str() {
            "[" | "(" | "{" => stack.push(token.as_str()),
            "]" | ")" | "}" => {
                let expected = match token.as_str() {
                    "]" => "[",
                    ")" => "(",
                    _ => "{",
                };
                if stack.pop() != Some(expected) {
                    return Err("unbalanced root item".into());
                }
            }
            ";" if stack.is_empty() => {
                result.push(all[start..=i].to_vec());
                start = i + 1;
            }
            _ => {}
        }
    }
    if !stack.is_empty() || start != all.len() {
        return Err("unknown or incomplete root item".into());
    }
    Ok(result)
}

fn registry(canonical: &str) -> Result<Vec<Vec<&str>>, String> {
    let start = "<!-- root-declaration-identities:start -->";
    let end = "<!-- root-declaration-identities:end -->";
    if canonical.matches(start).count() != 1 || canonical.matches(end).count() != 1 {
        return Err("missing or duplicate root identity table".into());
    }
    let body = canonical
        .split_once(start)
        .unwrap()
        .1
        .split_once(end)
        .ok_or("root identity table order")?
        .0;
    let mut rows: Vec<Vec<_>> = body
        .lines()
        .filter(|line| line.starts_with("| "))
        .map(|line| line.trim_matches('|').split('|').map(str::trim).collect())
        .collect();
    if rows.first().map(Vec::as_slice) != Some(&["visibility", "name", "path", "cfg"][..])
        || rows.get(1).map(Vec::as_slice) != Some(&["---", "---", "---", "---"][..])
    {
        return Err("root identity table header".into());
    }
    rows.drain(..2);
    if rows.len() != 28
        || rows.iter().any(|row| row.len() != 4)
        || rows.iter().collect::<BTreeSet<_>>().len() != 28
    {
        return Err("root identity table must have 28 unique declarations".into());
    }
    if rows.iter().any(|row| {
        row[1].is_empty()
            || !row[1]
                .bytes()
                .enumerate()
                .all(|(i, b)| b == b'_' || b.is_ascii_alphabetic() || (i > 0 && b.is_ascii_digit()))
            || (row[2] != "-"
                && (!row[2].ends_with(".rs")
                    || !row[2]
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"_/.".contains(&b))
                    || row[2]
                        .split('/')
                        .any(|part| ["", ".", ".."].contains(&part))))
    }) {
        return Err("invalid root module identifier/path".into());
    }
    if rows
        .iter()
        .any(|row| !["pub", "pub(crate)", "private"].contains(&row[0]))
        || rows.iter().filter(|row| row[2] != "-").count() != 27
        || rows.iter().filter(|row| row[3] != "true").count() != 23
    {
        return Err("root visibility/path/cfg attachment inventory".into());
    }
    let public: Vec<_> = rows
        .iter()
        .filter(|row| row[0] == "pub")
        .map(|row| row[1])
        .collect();
    if public.len() != 24 || public.iter().collect::<BTreeSet<_>>().len() != 22 {
        return Err("root public declaration/distinct inventory".into());
    }
    Ok(rows)
}

fn cfg_value(
    tokens: &[String],
    pos: &mut usize,
    features: &BTreeSet<String>,
    wasm: bool,
) -> Result<bool, String> {
    let token = tokens.get(*pos).ok_or("incomplete root cfg")?.as_str();
    *pos += 1;
    match token {
        "true" => Ok(true),
        "test" => Ok(false),
        "windows" => Ok(!wasm),
        "feature" | "target_arch" => {
            if tokens.get(*pos).map(String::as_str) != Some("=") {
                return Err("invalid root cfg predicate".into());
            }
            let literal = tokens.get(*pos + 1).ok_or("invalid root cfg value")?;
            *pos += 2;
            let literal = literal
                .strip_prefix('"')
                .and_then(|s| s.strip_suffix('"'))
                .ok_or("invalid root cfg value")?;
            if token == "feature" {
                Ok(features.contains(literal))
            } else if literal == "wasm32" {
                Ok(wasm)
            } else {
                Err("unknown root target cfg".into())
            }
        }
        "all" | "any" | "not" => {
            if tokens.get(*pos).map(String::as_str) != Some("(") {
                return Err("invalid root cfg grammar".into());
            }
            *pos += 1;
            let mut values = Vec::new();
            while tokens.get(*pos).map(String::as_str) != Some(")") {
                values.push(cfg_value(tokens, pos, features, wasm)?);
                match tokens.get(*pos).map(String::as_str) {
                    Some(",") => *pos += 1,
                    Some(")") => {}
                    _ => return Err("invalid root cfg separator".into()),
                }
            }
            *pos += 1;
            if values.is_empty() || (token == "not" && values.len() != 1) {
                return Err("invalid root cfg arity".into());
            }
            Ok(match token {
                "all" => values.iter().all(|v| *v),
                "any" => values.iter().any(|v| *v),
                _ => !values[0],
            })
        }
        _ => Err("unknown root cfg grammar".into()),
    }
}

fn cfg(expression: &str, features: &BTreeSet<String>, wasm: bool) -> Result<bool, String> {
    let tokens = tokens(expression)?;
    let mut pos = 0;
    let value = cfg_value(&tokens, &mut pos, features, wasm)?;
    if pos != tokens.len() {
        return Err("trailing root cfg tokens".into());
    }
    Ok(value)
}

fn closure(
    features: &toml::map::Map<String, toml::Value>,
    selected: &str,
) -> Result<BTreeSet<String>, String> {
    let (mut result, mut pending) = (BTreeSet::new(), vec![selected.to_owned()]);
    while let Some(name) = pending.pop() {
        if result.contains(&name) {
            continue;
        }
        let entries = features
            .get(&name)
            .and_then(toml::Value::as_array)
            .ok_or("missing Cargo feature")?;
        result.insert(name);
        for entry in entries {
            let entry = entry.as_str().ok_or("invalid Cargo feature")?;
            if features.contains_key(entry) {
                pending.push(entry.to_owned());
            } else if !entry.starts_with("dep:") && !entry.contains('/') {
                return Err("unknown Cargo feature dependency".into());
            }
        }
    }
    Ok(result)
}

pub(super) fn validate(source: &str, canonical: &str, cargo: &str) -> Result<Vec<String>, String> {
    if source.lines().count() > 127 {
        return Err("root exceeds 127 physical lines".into());
    }
    let rows = registry(canonical)?;
    let mut expected = Vec::new();
    for row in &rows {
        let mut item = String::new();
        if row[3] != "true" {
            item.push_str(&format!("#[cfg({})]", row[3]));
        }
        if row[2] != "-" {
            item.push_str(&format!("#[path=\"{}\"]", row[2]));
        }
        if row[0] != "private" {
            item.push_str(row[0]);
            item.push(' ');
        }
        item.push_str(&format!("mod {};", row[1]));
        expected.push(tokens(&item)?);
    }
    let mut allowed = OTHER_ITEMS
        .iter()
        .map(|item| tokens(item))
        .collect::<Result<Vec<_>, _>>()?;
    let mut remaining = Vec::new();
    for item in items(source)? {
        if let Some(index) = allowed.iter().position(|other| *other == item) {
            allowed.remove(index);
        } else {
            remaining.push(item);
        }
    }
    if !allowed.is_empty() || remaining != expected {
        return Err("root declaration/path/cfg/item identity changed".into());
    }
    let cargo: toml::Value =
        toml::from_str::<toml::Value>(cargo).map_err(|_| "invalid Cargo manifest")?;
    let features = cargo
        .get("features")
        .and_then(toml::Value::as_table)
        .ok_or("missing Cargo features")?;
    let all_features = features.keys().cloned().collect();
    for row in &rows {
        cfg(row[3], &all_features, false)?;
        for triple in tokens(row[3])?.windows(3) {
            if triple[0] == "feature"
                && triple[1] == "="
                && !features.contains_key(triple[2].trim_matches('"'))
            {
                return Err("unknown root Cargo feature predicate".into());
            }
        }
    }
    for (name, wasm, count) in [
        ("core-min", false, 4),
        ("asset-types", false, 5),
        ("graphics", false, 14),
        ("target-client", false, 21),
        ("target-web", true, 17),
    ] {
        let features = closure(features, name)?;
        let mut active = Vec::new();
        for row in &rows {
            if row[0] == "pub" && cfg(row[3], &features, wasm)? {
                active.push(row[1]);
            }
        }
        if active.len() != count
            || active.iter().collect::<BTreeSet<_>>().len() != count
            || (wasm && features.contains("native-host"))
            || active.contains(&"browser") != wasm
        {
            return Err(format!("root cfg/profile identity changed: {name}"));
        }
    }
    let browser = rows
        .iter()
        .find(|row| row[1] == "browser")
        .ok_or("missing browser identity")?;
    for bits in 0..8 {
        let features: BTreeSet<_> = ["target-web", "native-host"]
            .into_iter()
            .enumerate()
            .filter(|(bit, _)| bits & (1 << bit) != 0)
            .map(|(_, name)| name.to_owned())
            .collect();
        let wasm = bits & 4 != 0;
        if cfg(browser[3], &features, wasm)?
            != (wasm && features.contains("target-web") && !features.contains("native-host"))
        {
            return Err("root browser predicate truth table changed".into());
        }
    }
    for bits in 0..8 {
        let features: BTreeSet<_> = ["native-host", "asset-types", "graphics"]
            .into_iter()
            .enumerate()
            .filter(|(bit, _)| bits & (1 << bit) != 0)
            .map(|(_, name)| name.to_owned())
            .collect();
        for alias in ["asset", "plugin"] {
            let mut count = 0;
            for row in &rows {
                if row[0] == "pub" && row[1] == alias && cfg(row[3], &features, false)? {
                    count += 1;
                }
            }
            if count > 1 {
                return Err(format!("root alias predicates overlap: {alias}"));
            }
        }
    }
    Ok(rows
        .iter()
        .filter(|row| row[0] == "pub")
        .map(|row| row[1].to_owned())
        .collect())
}
