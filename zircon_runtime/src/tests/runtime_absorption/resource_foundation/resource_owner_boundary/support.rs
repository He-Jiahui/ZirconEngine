use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;

use super::super::*;

pub(super) fn rust_code_view(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut code = bytes.to_vec();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index..].starts_with(b"//") {
            let start = index;
            index += 2;
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
            code[start..index].fill(b' ');
            continue;
        }
        if bytes[index..].starts_with(b"/*") {
            let start = index;
            index += 2;
            let mut depth = 1usize;
            while index < bytes.len() && depth > 0 {
                if bytes[index..].starts_with(b"/*") {
                    depth += 1;
                    index += 2;
                } else if bytes[index..].starts_with(b"*/") {
                    depth -= 1;
                    index += 2;
                } else {
                    index += 1;
                }
            }
            code[start..index].fill(b' ');
            continue;
        }

        let raw_prefix = if bytes[index] == b'r' {
            Some(index)
        } else if matches!(bytes[index], b'b' | b'c') && bytes.get(index + 1) == Some(&b'r') {
            Some(index + 1)
        } else {
            None
        };
        if let Some(raw_start) = raw_prefix {
            let mut quote = raw_start + 1;
            while bytes.get(quote) == Some(&b'#') {
                quote += 1;
            }
            if bytes.get(quote) == Some(&b'"') {
                let hash_count = quote - raw_start - 1;
                let start = index;
                index = quote + 1;
                while index < bytes.len() {
                    if bytes[index] == b'"'
                        && bytes.get(index + 1..index + 1 + hash_count)
                            == Some(&bytes[raw_start + 1..quote])
                    {
                        index += 1 + hash_count;
                        break;
                    }
                    index += 1;
                }
                code[start..index].fill(b' ');
                continue;
            }
        }

        if bytes[index] == b'"' {
            let start = index;
            index += 1;
            while index < bytes.len() {
                if bytes[index] == b'\\' {
                    index = (index + 2).min(bytes.len());
                } else if bytes[index] == b'"' {
                    index += 1;
                    break;
                } else {
                    index += 1;
                }
            }
            code[start..index].fill(b' ');
            continue;
        }
        index += 1;
    }
    String::from_utf8(code).expect("Rust source remains UTF-8 after lexical filtering")
}

pub(super) fn compact_rust_source(source: &str) -> String {
    rust_code_view(source)
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

pub(super) fn rust_tokens(source: &str) -> Vec<String> {
    let bytes = rust_code_view(source).into_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index].is_ascii_whitespace() {
            index += 1;
            continue;
        }
        if bytes[index..].starts_with(b"::") {
            tokens.push("::".to_owned());
            index += 2;
            continue;
        }
        if matches!(
            bytes[index],
            b'{' | b'}' | b'(' | b')' | b'[' | b']' | b',' | b';' | b'*' | b'#' | b'!'
        ) {
            tokens.push((bytes[index] as char).to_string());
            index += 1;
            continue;
        }

        let start = index;
        if bytes[index..].starts_with(b"r#") {
            index += 2;
        }
        if bytes.get(index).is_some_and(u8::is_ascii_alphabetic) || bytes.get(index) == Some(&b'_')
        {
            index += 1;
            while bytes
                .get(index)
                .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
            {
                index += 1;
            }
            let raw = std::str::from_utf8(&bytes[start..index])
                .expect("Rust identifier token remains UTF-8");
            tokens.push(raw.strip_prefix("r#").unwrap_or(raw).to_owned());
            continue;
        }
        index += 1;
    }
    tokens
}

pub(super) fn has_public_use(source: &str) -> bool {
    let tokens = rust_tokens(source);
    let mut index = 0;
    while index < tokens.len() {
        if tokens[index] != "pub" {
            index += 1;
            continue;
        }
        index += 1;
        if tokens.get(index).is_some_and(|token| token == "(") {
            let mut depth = 1usize;
            index += 1;
            while index < tokens.len() && depth > 0 {
                match tokens[index].as_str() {
                    "(" => depth += 1,
                    ")" => depth -= 1,
                    _ => {}
                }
                index += 1;
            }
        }
        if tokens.get(index).is_some_and(|token| token == "use") {
            return true;
        }
    }
    false
}

pub(super) fn parse_use_tree(
    tokens: &[String],
    mut index: usize,
    prefix: &[String],
    leaves: &mut Vec<(Vec<String>, Option<String>)>,
) -> usize {
    let Some(token) = tokens.get(index).map(String::as_str) else {
        return index;
    };
    if token == "{" {
        index += 1;
        while index < tokens.len() && tokens[index] != "}" {
            if tokens[index] == "," {
                index += 1;
            } else {
                let next = parse_use_tree(tokens, index, prefix, leaves);
                index = next.max(index + 1);
            }
        }
        return (index + 1).min(tokens.len());
    }
    if token == "*" {
        let mut path = prefix.to_vec();
        path.push("*".to_owned());
        leaves.push((path, None));
        return index + 1;
    }
    if matches!(token, "," | "}" | ";") {
        return index + 1;
    }

    let mut path = prefix.to_vec();
    if token != "self" || path.is_empty() {
        path.push(token.to_owned());
    }
    index += 1;
    if tokens.get(index).is_some_and(|token| token == "::") {
        return parse_use_tree(tokens, index + 1, &path, leaves);
    }

    let mut alias = None;
    if tokens.get(index).is_some_and(|token| token == "as") {
        alias = tokens.get(index + 1).cloned();
        index = (index + 2).min(tokens.len());
    }
    leaves.push((path, alias));
    index
}

pub(super) fn rust_use_paths(tokens: &[String]) -> Vec<(Vec<String>, Option<String>)> {
    let mut leaves = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        if tokens[index] != "use" {
            index += 1;
            continue;
        }
        index += 1;
        if tokens.get(index).is_some_and(|token| token == "::") {
            index += 1;
        }
        while index < tokens.len() && tokens[index] != ";" {
            let next = parse_use_tree(tokens, index, &[], &mut leaves);
            index = next.max(index + 1);
        }
        index = (index + 1).min(tokens.len());
    }
    leaves
}

pub(super) fn resolved_paths(
    path: &[String],
    aliases: &HashMap<String, Vec<Vec<String>>>,
) -> Vec<Vec<String>> {
    let mut resolved = Vec::new();
    let mut pending = VecDeque::from([path.to_vec()]);
    let mut seen = HashSet::new();
    while let Some(candidate) = pending.pop_front() {
        if !seen.insert(candidate.clone()) {
            continue;
        }
        let Some(expansions) = candidate.first().and_then(|root| aliases.get(root)) else {
            resolved.push(candidate);
            continue;
        };
        for expansion in expansions {
            let mut next = expansion.clone();
            next.extend_from_slice(&candidate[1..]);
            pending.push_back(next);
        }
    }
    resolved
}

pub(super) fn is_old_resource_management_path(path: &[String]) -> bool {
    let old_owner = ["core", "framework", "asset"];
    let Some(owner_index) = path
        .windows(old_owner.len())
        .position(|window| window.iter().map(String::as_str).eq(old_owner))
    else {
        return false;
    };
    path.iter()
        .skip(owner_index + old_owner.len())
        .any(|segment| segment == "*" || segment.starts_with("ResourceManagement"))
}

fn references_resolved_path(source: &str, matches_path: impl Fn(&[String]) -> bool) -> bool {
    let tokens = rust_tokens(source);
    let use_paths = rust_use_paths(&tokens);
    let aliases = use_paths
        .iter()
        .filter_map(|(path, alias)| {
            let local_name = alias.clone().or_else(|| path.last().cloned())?;
            Some((local_name, path.clone()))
        })
        .fold(
            HashMap::<String, Vec<Vec<String>>>::new(),
            |mut map, row| {
                map.entry(row.0).or_default().push(row.1);
                map
            },
        );
    if use_paths
        .iter()
        .flat_map(|(path, _)| resolved_paths(path, &aliases))
        .any(|path| matches_path(&path))
    {
        return true;
    }

    let mut index = 0;
    while index < tokens.len() {
        if tokens[index] == "use" {
            while index < tokens.len() && tokens[index] != ";" {
                index += 1;
            }
            index += 1;
            continue;
        }
        if !tokens[index]
            .chars()
            .all(|character| character == '_' || character.is_ascii_alphanumeric())
        {
            index += 1;
            continue;
        }
        let mut path = vec![tokens[index].clone()];
        index += 1;
        while tokens.get(index).is_some_and(|token| token == "::")
            && tokens.get(index + 1).is_some()
        {
            path.push(tokens[index + 1].clone());
            index += 2;
        }
        if resolved_paths(&path, &aliases)
            .iter()
            .any(|path| matches_path(path))
        {
            return true;
        }
    }
    false
}

pub(super) fn references_old_resource_management_owner(source: &str) -> bool {
    references_resolved_path(source, is_old_resource_management_path)
}

pub(super) fn normalize_module_path(path: &[String], module_path: &[String]) -> Vec<String> {
    if path.first().is_some_and(|segment| segment == "crate") {
        return path.to_vec();
    }
    if path
        .first()
        .is_some_and(|segment| segment == "zircon_runtime")
    {
        return path.to_vec();
    }
    let mut normalized = module_path.to_vec();
    let mut index = 0;
    if path.first().is_some_and(|segment| segment == "self") {
        index = 1;
    } else {
        while path.get(index).is_some_and(|segment| segment == "super") {
            normalized.pop();
            index += 1;
        }
    }
    normalized.extend_from_slice(&path[index..]);
    normalized
}

pub(super) type ModuleAliasGraph = HashMap<Vec<String>, HashMap<String, Vec<Vec<String>>>>;

pub(super) fn module_context_at(
    tokens: &[String],
    end: usize,
    file_module_path: &[String],
) -> (Vec<String>, bool) {
    let mut module_path = file_module_path.to_vec();
    let mut module_depths = Vec::new();
    let mut brace_depth = 0usize;
    for index in 0..end.min(tokens.len()) {
        match tokens[index].as_str() {
            "{" => {
                brace_depth += 1;
                if index >= 2 && tokens[index - 2] == "mod" {
                    module_path.push(tokens[index - 1].clone());
                    module_depths.push(brace_depth);
                }
            }
            "}" => {
                if module_depths
                    .last()
                    .is_some_and(|depth| *depth == brace_depth)
                {
                    module_depths.pop();
                    module_path.pop();
                }
                brace_depth = brace_depth.saturating_sub(1);
            }
            _ => {}
        }
    }
    let module_scope_depth = module_depths.last().copied().unwrap_or(0);
    (module_path, brace_depth == module_scope_depth)
}

pub(super) fn extend_module_alias_graph(
    aliases: &mut ModuleAliasGraph,
    source: &str,
    file_module_path: &[String],
) {
    let tokens = rust_tokens(source);
    for index in 0..tokens.len() {
        if tokens[index] != "use" {
            continue;
        }
        let (module_path, is_module_scope) = module_context_at(&tokens, index, file_module_path);
        if !is_module_scope {
            continue;
        }
        let mut end = index + 1;
        while end < tokens.len() && tokens[end] != ";" {
            end += 1;
        }
        for (path, alias) in rust_use_paths(&tokens[index..end.min(tokens.len())]) {
            let Some(local_name) = alias.or_else(|| path.last().cloned()) else {
                continue;
            };
            if local_name == "*" {
                continue;
            }
            aliases
                .entry(module_path.clone())
                .or_default()
                .entry(local_name)
                .or_default()
                .push(normalize_module_path(&path, &module_path));
        }
    }
}

pub(super) fn resolved_module_paths(
    path: &[String],
    aliases: &ModuleAliasGraph,
) -> Vec<Vec<String>> {
    let mut resolved = Vec::new();
    let mut pending = VecDeque::from([path.to_vec()]);
    let mut seen = HashSet::new();
    while let Some(candidate) = pending.pop_front() {
        if !seen.insert(candidate.clone()) {
            continue;
        }
        let mut expanded = false;
        for alias_index in 1..candidate.len() {
            let Some(expansions) = aliases
                .get(&candidate[..alias_index])
                .and_then(|scope| scope.get(&candidate[alias_index]))
            else {
                continue;
            };
            for expansion in expansions {
                let mut next = expansion.clone();
                next.extend_from_slice(&candidate[alias_index + 1..]);
                pending.push_back(next);
            }
            expanded = true;
        }
        if !expanded {
            resolved.push(candidate);
        }
    }
    resolved
}

pub(super) fn exposes_resource_owner_path(path: &[String]) -> bool {
    let path = path
        .iter()
        .map(String::as_str)
        .filter(|segment| *segment != "*")
        .collect::<Vec<_>>();
    path == ["crate"]
        || path == ["crate", "core"]
        || path.starts_with(&["crate", "core", "resource"])
}

pub(super) fn reexports_resource_owner(
    source: &str,
    file_module_path: &[String],
    aliases: &ModuleAliasGraph,
) -> bool {
    let tokens = rust_tokens(source);
    let mut index = 0;
    while index < tokens.len() {
        if tokens[index] != "pub" {
            index += 1;
            continue;
        }
        index += 1;
        if tokens.get(index).is_some_and(|token| token == "(") {
            let mut depth = 1usize;
            index += 1;
            while index < tokens.len() && depth > 0 {
                match tokens[index].as_str() {
                    "(" => depth += 1,
                    ")" => depth -= 1,
                    _ => {}
                }
                index += 1;
            }
        }
        if tokens.get(index).is_none_or(|token| token != "use") {
            continue;
        }
        let start = index + 1;
        let mut end = start;
        while end < tokens.len() && tokens[end] != ";" {
            end += 1;
        }
        let (module_path, is_module_scope) =
            module_context_at(&tokens, start - 1, file_module_path);
        if !is_module_scope {
            return true;
        }
        if rust_use_paths(&tokens[index..end.min(tokens.len())])
            .into_iter()
            .any(|(path, _)| {
                resolved_module_paths(&normalize_module_path(&path, &module_path), aliases)
                    .into_iter()
                    .any(|path| exposes_resource_owner_path(&path))
            })
        {
            return true;
        }
        index = end.saturating_add(1);
    }
    false
}

pub(super) fn declares_extern_crate(source: &str) -> bool {
    rust_tokens(source)
        .windows(2)
        .any(|tokens| tokens[0] == "extern" && tokens[1] == "crate")
}

pub(super) fn imports_resource_owner_glob(
    source: &str,
    file_module_path: &[String],
    aliases: &ModuleAliasGraph,
) -> bool {
    rust_use_paths(&rust_tokens(source))
        .into_iter()
        .filter(|(path, _)| path.last().is_some_and(|segment| segment == "*"))
        .flat_map(|(path, _)| {
            resolved_module_paths(&normalize_module_path(&path, file_module_path), aliases)
        })
        .any(|path| exposes_resource_owner_path(&path))
}

pub(super) fn has_source_injection_surface(source: &str) -> bool {
    let tokens = rust_tokens(source);
    if tokens.windows(2).any(|tokens| tokens == ["include", "!"]) {
        return true;
    }
    let mut index = 0;
    while index + 1 < tokens.len() {
        if tokens[index] != "#" || tokens[index + 1] != "[" {
            index += 1;
            continue;
        }
        index += 2;
        let mut depth = 1usize;
        while index < tokens.len() && depth > 0 {
            match tokens[index].as_str() {
                "[" => depth += 1,
                "]" => depth -= 1,
                "path" => return true,
                _ => {}
            }
            index += 1;
        }
    }
    false
}

pub(super) fn framework_module_path(framework_root: &Path, source_path: &Path) -> Vec<String> {
    let mut module_path = vec![
        "crate".to_owned(),
        "core".to_owned(),
        "framework".to_owned(),
    ];
    let Ok(relative) = source_path.strip_prefix(framework_root) else {
        return module_path;
    };
    let mut components = relative
        .components()
        .filter_map(|component| component.as_os_str().to_str().map(str::to_owned))
        .collect::<Vec<_>>();
    if components.last().is_some_and(|name| name == "mod.rs") {
        components.pop();
    } else if let Some(file_name) = components.last_mut() {
        *file_name = file_name
            .strip_suffix(".rs")
            .unwrap_or(file_name)
            .to_owned();
    }
    module_path.extend(components);
    module_path
}

pub(super) fn asset_contract_has_only_resource_management_consumer(source: &str) -> bool {
    let tokens = rust_tokens(source);
    let compact_source = compact_rust_source(source);
    let use_paths = rust_use_paths(&tokens);
    let expected_imports = [
        ["std", "sync", "Arc"].as_slice(),
        ["crate", "core", "resource", "ResourceEventReceiver"].as_slice(),
        ["crate", "core", "resource", "ResourceManagementGeneration"].as_slice(),
        ["crate", "core", "resource", "ResourceRecord"].as_slice(),
        ["crate", "core", "resource", "ResourceState"].as_slice(),
    ];
    let imports_are_exact = use_paths.len() == expected_imports.len()
        && use_paths.iter().all(|(path, alias)| {
            alias.is_none()
                && expected_imports.contains(
                    &path
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>()
                        .as_slice(),
                )
        });
    let management_paths = use_paths
        .iter()
        .map(|(path, _)| path)
        .filter(|path| {
            path.iter()
                .any(|segment| segment.starts_with("ResourceManagement"))
        })
        .collect::<Vec<_>>();
    let expected_path = ["crate", "core", "resource", "ResourceManagementGeneration"];
    imports_are_exact
        && management_paths.len() == 1
        && management_paths[0]
            .iter()
            .map(String::as_str)
            .eq(expected_path)
        && tokens
            .iter()
            .filter(|token| token.as_str() == "ResourceManagementGeneration")
            .count()
            == 2
        && compact_source
            .match_indices(
                "fnresource_management_generation(&self)->Arc<ResourceManagementGeneration>;",
            )
            .count()
            == 1
        && !tokens.iter().any(|token| {
            token.starts_with("ResourceManagement") && token != "ResourceManagementGeneration"
        })
}

pub(super) fn asset_contract_has_no_generated_surface(source: &str) -> bool {
    let tokens = rust_tokens(source);
    if tokens
        .iter()
        .any(|token| matches!(token.as_str(), "mod" | "!"))
    {
        return false;
    }
    let permitted_derive = [["Clone", "Copy", "Debug", "PartialEq", "Eq"].as_slice()];
    let mut index = 0;
    while index < tokens.len() {
        if tokens[index] != "#" {
            index += 1;
            continue;
        }
        if tokens.get(index + 1).is_none_or(|token| token != "[")
            || tokens.get(index + 2).is_none_or(|token| token != "derive")
            || tokens.get(index + 3).is_none_or(|token| token != "(")
        {
            return false;
        }
        let mut names = Vec::new();
        index += 4;
        while index < tokens.len() && tokens[index] != ")" {
            if tokens[index] != "," {
                names.push(tokens[index].as_str());
            }
            index += 1;
        }
        if tokens.get(index).is_none_or(|token| token != ")")
            || tokens.get(index + 1).is_none_or(|token| token != "]")
            || !permitted_derive.contains(&names.as_slice())
        {
            return false;
        }
        index += 2;
    }
    true
}

pub(super) fn asset_contract_has_only_expected_public_items(source: &str) -> bool {
    let tokens = rust_tokens(source);
    let mut public_items = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        if tokens[index] != "pub" {
            index += 1;
            continue;
        }
        index += 1;
        if tokens.get(index).is_some_and(|token| token == "(") {
            let mut depth = 1usize;
            index += 1;
            while index < tokens.len() && depth > 0 {
                match tokens[index].as_str() {
                    "(" => depth += 1,
                    ")" => depth -= 1,
                    _ => {}
                }
                index += 1;
            }
        }
        let Some(kind_or_name) = tokens.get(index).map(String::as_str) else {
            return false;
        };
        let public_item = match kind_or_name {
            "struct" | "trait" => {
                let Some(name) = tokens.get(index + 1).map(String::as_str) else {
                    return false;
                };
                (kind_or_name, name)
            }
            name => ("field", name),
        };
        public_items.push(public_item);
    }
    public_items
        == [
            ("struct", "ResourceCacheIdentity"),
            ("field", "revision"),
            ("field", "state"),
            ("trait", "ResourceManager"),
        ]
}

pub(super) fn framework_root_has_external_asset_module(source: &str) -> bool {
    let tokens = rust_tokens(source);
    let declarations = tokens
        .windows(4)
        .enumerate()
        .filter(|(_, window)| {
            window
                .iter()
                .map(String::as_str)
                .eq(["pub", "mod", "asset", ";"])
        })
        .collect::<Vec<_>>();
    declarations.len() == 1
        && declarations[0]
            .0
            .checked_sub(1)
            .is_none_or(|previous| tokens[previous] != "]")
        && tokens
            .iter()
            .filter(|token| token.as_str() == "asset")
            .count()
            == 1
}

pub(super) fn visit_rust_sources(root: &Path, visit: &mut impl FnMut(&Path, &str)) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let skipped = path.file_name().is_some_and(|name| {
                matches!(
                    name.to_str(),
                    Some("target" | ".git" | ".codex" | "node_modules")
                )
            });
            if !skipped {
                visit_rust_sources(&path, visit);
            }
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            let source = std::fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            visit(&path, &source);
        }
    }
}
