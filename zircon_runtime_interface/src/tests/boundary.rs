use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

const ALLOWED_DEPENDENCIES: &[&str] = &[
    "bincode",
    "blake3",
    "glam",
    "semver",
    "serde",
    "serde_json",
    "sha2",
    "thiserror",
    "toml",
    "unicode-segmentation",
    "uuid",
    "zr_math",
];

const ALLOWED_DEV_DEPENDENCIES: &[&str] = &[];
const ALLOWED_BUILD_DEPENDENCIES: &[&str] = &["serde_json"];

const FORBIDDEN_SOURCE_NEEDLES: &[&str] = &[
    "#[path",
    "include_str!(",
    "include_bytes!(",
    "zircon_runtime/src",
    "zircon_editor/src",
    "zircon_runtime::",
    "zircon_editor::",
    "wgpu::",
    "winit::",
    concat!("sli", "nt::"),
    "libloading::",
    "tokio::",
    "std::fs",
    "std::net",
    "std::process",
    "std::thread",
    "std::sync",
];

const CANONICAL_TEXT_SPOOL_OS_NEEDLES: &[&str] = &["std::fs", "std::process", "std::sync"];
const CANONICAL_TEXT_SPOOL_PATH: &str = "src/serialization/text/canonical_spool.rs";
const SHARED_RECENT_PROJECTS_STORE_OS_NEEDLES: &[&str] = &["std::fs", "std::thread"];
const SHARED_RECENT_PROJECTS_STORE_PATH: &str = "src/hub_protocol/recent_projects/store.rs";
const REVIEWED_COMPILE_TIME_INPUTS: &[(&str, &str)] = &[
    ("src/project/template_pack/embedded.rs", "include_bytes!("),
    ("src/runtime_build_set/interface_spec.rs", "include_str!("),
    ("src/runtime_build_set/payload_schema.rs", "include_bytes!("),
];
const REVIEWED_STD_SYNC_PATHS: &[&str] = &[
    "src/project/activation_operation_id/generator.rs",
    "src/project/template_pack/descriptor.rs",
    "src/reflect/schema_catalog/mod.rs",
    "src/runtime_api/session/session_identity.rs",
    "src/ui/dispatch/input/effect.rs",
    "src/ui/dispatch/input/event.rs",
    "src/ui/surface/frame.rs",
    "src/ui/surface/render/text_geometry/source_map/line.rs",
    "src/ui/surface/render/text_layout.rs",
    "src/ui/text/rich_link_target.rs",
    "src/ui/window/input/normalization.rs",
];
const EXPECTED_RUNTIME_API_DOMAINS: &[&str] = &["abi", "constants", "frame", "host", "session"];
const EXPECTED_RUNTIME_API_OWNER_PATHS: &[&str] = &[
    "abi/api_shape.rs",
    "abi/api_table.rs",
    "abi/host_api_shape.rs",
    "constants.rs",
    "frame/frame_demand.rs",
    "frame/frame_shape.rs",
    "frame/highlight_set.rs",
    "frame/viewport_pick.rs",
    "host/clipboard.rs",
    "host/host_requests.rs",
    "host/ui_action.rs",
    "host/ui_host_request.rs",
    "session/app_session_configuration_v2.rs",
    "session/camera.rs",
    "session/editor_transform.rs",
    "session/events.rs",
    "session/ime_composition_capability_v2.rs",
    "session/ime_composition_v2.rs",
    "session/operation.rs",
    "session/plugin_event_mirror.rs",
    "session/requests.rs",
    "session/session.rs",
    "session/session_identity.rs",
    "session/translated_events.rs",
    "session/viewport.rs",
];
const RUNTIME_API_FACADE_LINE_BUDGET: usize = 220;
const RUNTIME_API_FACADE_REEXPORT_BUDGET: usize = 6;
const RUNTIME_API_CHILD_LINE_BUDGET: usize = 700;
const RUNTIME_API_DOMAIN_FACADE_PATHS: &[&str] = &[
    "abi/mod.rs",
    "frame/mod.rs",
    "host/mod.rs",
    "session/mod.rs",
];

#[test]
fn manifest_dependencies_stay_contract_only() {
    let manifest_path = manifest_dir().join("Cargo.toml");
    let manifest = std::fs::read_to_string(&manifest_path).expect("read interface manifest");
    let manifest: toml::Value = toml::from_str(&manifest).expect("parse interface manifest");
    let dependencies = manifest
        .get("dependencies")
        .and_then(toml::Value::as_table)
        .expect("interface manifest dependencies table");
    let allowed: BTreeSet<_> = ALLOWED_DEPENDENCIES.iter().copied().collect();
    let actual: BTreeSet<_> = dependencies.keys().map(String::as_str).collect();
    let unexpected: Vec<_> = actual.difference(&allowed).copied().collect();

    assert!(
        unexpected.is_empty(),
        "zircon_runtime_interface may only depend on contract/serialization crates; unexpected dependencies: {unexpected:?}"
    );
    let allowed_build: BTreeSet<_> = ALLOWED_BUILD_DEPENDENCIES.iter().copied().collect();
    let actual_build: BTreeSet<_> = manifest
        .get("build-dependencies")
        .and_then(toml::Value::as_table)
        .into_iter()
        .flat_map(|dependencies| dependencies.keys().map(String::as_str))
        .collect();
    let unexpected_build: Vec<_> = actual_build.difference(&allowed_build).copied().collect();
    assert!(
        unexpected_build.is_empty(),
        "zircon_runtime_interface build dependencies require explicit boundary review; unexpected: {unexpected_build:?}"
    );
    let allowed_dev: BTreeSet<_> = ALLOWED_DEV_DEPENDENCIES.iter().copied().collect();
    let actual_dev: BTreeSet<_> = manifest
        .get("dev-dependencies")
        .and_then(toml::Value::as_table)
        .into_iter()
        .flat_map(|dependencies| dependencies.keys().map(String::as_str))
        .collect();
    let unexpected_dev: Vec<_> = actual_dev.difference(&allowed_dev).copied().collect();
    assert!(
        unexpected_dev.is_empty(),
        "zircon_runtime_interface dev dependencies require explicit boundary review; unexpected: {unexpected_dev:?}"
    );
}

#[test]
fn production_source_does_not_include_or_import_implementation_crates() {
    let sources = production_rust_sources();
    let mut violations = Vec::new();

    for source in sources {
        let text = std::fs::read_to_string(&source).expect("read interface source");
        let production_text = without_cfg_test_items(&text);
        for needle in FORBIDDEN_SOURCE_NEEDLES {
            if reviewed_compile_time_input(&source, needle)
                || canonical_text_spool_os_access_is_reviewed(&source, needle)
                || shared_recent_projects_store_os_access_is_reviewed(&source, needle)
                || std_sync_contract_primitive_is_reviewed(&source, &production_text, needle)
            {
                continue;
            }
            if production_text.contains(needle) {
                violations.push(format!(
                    "{} contains forbidden boundary marker `{needle}`",
                    relative_to_manifest(&source).display()
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "zircon_runtime_interface source must stay ABI/DTO/serialization-only:\n{}",
        violations.join("\n")
    );
}

// These exact exceptions cover reviewed compile-time contract inputs, the bounded canonical
// text spool, and the shared recent-project transaction owner. All other interface production
// sources remain subject to the contract-layer I/O and implementation-dependency boundary.
fn reviewed_compile_time_input(source: &Path, needle: &str) -> bool {
    REVIEWED_COMPILE_TIME_INPUTS
        .iter()
        .any(|(path, reviewed_needle)| {
            needle == *reviewed_needle && relative_to_manifest(source) == Path::new(path)
        })
}

fn canonical_text_spool_os_access_is_reviewed(source: &Path, needle: &str) -> bool {
    relative_to_manifest(source) == Path::new(CANONICAL_TEXT_SPOOL_PATH)
        && CANONICAL_TEXT_SPOOL_OS_NEEDLES.contains(&needle)
}

fn without_cfg_test_items(source: &str) -> String {
    const TEST_ATTRIBUTES: &[&str] = &["#[cfg(test)]", "#[cfg(all(test, windows))]"];
    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;

    while let Some((relative_start, attribute)) = TEST_ATTRIBUTES
        .iter()
        .filter_map(|attribute| {
            source[cursor..]
                .match_indices(*attribute)
                .find(|(start, _)| is_rust_code_position(source, cursor + start))
        })
        .min_by_key(|(start, _)| *start)
    {
        let attribute_start = cursor + relative_start;
        output.push_str(&source[cursor..attribute_start]);
        let item_start = attribute_start + attribute.len();
        let remainder = &source[item_start..];
        let brace_start = remainder.find('{');
        let semicolon_start = remainder.find(';');
        let item_end = match (brace_start, semicolon_start) {
            (Some(brace), Some(semicolon)) if semicolon < brace => item_start + semicolon + 1,
            (Some(brace), _) => find_matching_brace(source, item_start + brace)
                .map_or(item_start, |close| close + 1),
            (None, Some(semicolon)) => item_start + semicolon + 1,
            (None, None) => item_start,
        };

        if item_end == item_start {
            output.push_str(&source[attribute_start..item_start]);
        } else {
            output.push('\n');
        }
        cursor = item_end;
    }
    output.push_str(&source[cursor..]);
    output
}

fn find_matching_brace(source: &str, open: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut depth = 0usize;
    let mut index = open;
    let mut string_delimiter = None;
    let mut line_comment = false;
    let mut block_comment_depth = 0usize;
    let mut escaped = false;

    while index < bytes.len() {
        let byte = bytes[index];
        if line_comment {
            if byte == b'\n' {
                line_comment = false;
            }
            index += 1;
            continue;
        }
        if block_comment_depth > 0 {
            if bytes.get(index..index.saturating_add(2)) == Some(b"/*") {
                block_comment_depth += 1;
                index += 2;
            } else if bytes.get(index..index.saturating_add(2)) == Some(b"*/") {
                block_comment_depth -= 1;
                index += 2;
            } else {
                index += 1;
            }
            continue;
        }
        if let Some(delimiter) = string_delimiter {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == delimiter {
                string_delimiter = None;
            }
            index += 1;
            continue;
        }
        if bytes.get(index..index.saturating_add(2)) == Some(b"//") {
            line_comment = true;
            index += 2;
            continue;
        }
        if bytes.get(index..index.saturating_add(2)) == Some(b"/*") {
            block_comment_depth = 1;
            index += 2;
            continue;
        }
        if byte == b'"' {
            string_delimiter = Some(byte);
            index += 1;
            continue;
        }
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

#[test]
fn cfg_test_source_filter_preserves_production_boundary_guards() {
    let source = "#[cfg(test)]\nmod tests { const ONLY_TEST: &str = \"std::fs\"; }\nuse std::fs;";
    let production = without_cfg_test_items(source);
    assert!(!production.contains("ONLY_TEST"));
    assert!(production.contains("use std::fs;"));

    let source = "#[cfg(all(test, windows))]\n#[path = \"tests/windows.rs\"]\nmod windows_tests;\nuse std::fs;";
    let production = without_cfg_test_items(source);
    assert!(!production.contains("#[path"));
    assert!(production.contains("use std::fs;"));

    let source =
        "#[cfg(any(test, feature = \"production\"))]\n#[path = \"production.rs\"]\nmod production;";
    assert_eq!(without_cfg_test_items(source), source);

    let source = "// #[cfg(test)]\nuse std::fs;";
    assert_eq!(without_cfg_test_items(source), source);

    let source = "fn write_quote() { let _ = '\"'; }\n#[cfg(test)]\n#[path = \"tests/quote.rs\"]\nmod tests;\nuse std::fs;";
    let production = without_cfg_test_items(source);
    assert!(!production.contains("#[path"));
    assert!(production.contains("use std::fs;"));
}

#[test]
fn reviewed_boundary_exceptions_stay_path_scoped() {
    assert!(reviewed_compile_time_input(
        Path::new("src/runtime_build_set/interface_spec.rs"),
        "include_str!("
    ));
    assert!(!reviewed_compile_time_input(
        Path::new("src/runtime_build_set/payload_schema.rs"),
        "include_str!("
    ));
    assert!(shared_recent_projects_store_os_access_is_reviewed(
        Path::new(SHARED_RECENT_PROJECTS_STORE_PATH),
        "std::fs"
    ));
    assert!(shared_recent_projects_store_os_access_is_reviewed(
        Path::new(SHARED_RECENT_PROJECTS_STORE_PATH),
        "std::thread"
    ));
    assert!(!shared_recent_projects_store_os_access_is_reviewed(
        Path::new(SHARED_RECENT_PROJECTS_STORE_PATH),
        "std::process"
    ));
}

fn shared_recent_projects_store_os_access_is_reviewed(source: &Path, needle: &str) -> bool {
    relative_to_manifest(source) == Path::new(SHARED_RECENT_PROJECTS_STORE_PATH)
        && SHARED_RECENT_PROJECTS_STORE_OS_NEEDLES.contains(&needle)
}

fn std_sync_contract_primitive_is_reviewed(source: &Path, text: &str, needle: &str) -> bool {
    let relative = relative_to_manifest(source);
    let relative = relative.to_string_lossy().replace('\\', "/");
    let allowed_symbols: &[&str] = match relative.as_str() {
        "src/project/activation_operation_id/generator.rs" => &["atomic"],
        "src/project/template_pack/descriptor.rs" => &["OnceLock"],
        "src/reflect/schema_catalog/mod.rs" => &["OnceLock"],
        "src/runtime_api/session/session_identity.rs" => &["Arc"],
        "src/ui/dispatch/input/effect.rs" => &["Arc"],
        "src/ui/dispatch/input/event.rs" => &["Arc"],
        "src/ui/surface/frame.rs" => &["Arc"],
        "src/ui/surface/render/text_geometry/source_map/line.rs" => &["atomic", "OnceLock"],
        "src/ui/surface/render/text_layout.rs" => &["Arc"],
        "src/ui/text/rich_link_target.rs" => &["Arc"],
        "src/ui/window/input/normalization.rs" => &["Arc"],
        _ => return false,
    };
    if needle != "std::sync" {
        return false;
    }

    let statements = reviewed_std_sync_use_statements(text);
    !statements.is_empty()
        && statements
            .into_iter()
            .all(|statement| reviewed_std_sync_use_statement_is_allowed(statement, allowed_symbols))
}

fn reviewed_std_sync_use_statements(text: &str) -> Vec<&str> {
    const PREFIX: &str = "use std::sync";
    let mut statements = Vec::new();
    let mut cursor = 0usize;
    while let Some(relative_index) = text[cursor..].find(PREFIX) {
        let start = cursor + relative_index;
        let line_start = text[..start].rfind('\n').map_or(0, |index| index + 1);
        if text[line_start..start].trim().is_empty() && is_rust_code_position(text, start) {
            if let Some(end) = find_use_statement_end(text, start) {
                statements.push(&text[start..end]);
                cursor = end;
                continue;
            }
            break;
        }
        cursor = start + PREFIX.len();
    }
    statements
}

fn is_rust_code_position(text: &str, target: usize) -> bool {
    let bytes = text.as_bytes();
    let mut index = 0usize;
    let mut line_comment = false;
    let mut block_comment_depth = 0usize;
    let mut string_literal = false;
    let mut escaped = false;

    while index < target {
        let byte = bytes[index];
        if line_comment {
            if byte == b'\n' {
                line_comment = false;
            }
            index += 1;
            continue;
        }
        if block_comment_depth != 0 {
            if bytes.get(index..index.saturating_add(2)) == Some(b"/*") {
                block_comment_depth += 1;
                index += 2;
            } else if bytes.get(index..index.saturating_add(2)) == Some(b"*/") {
                block_comment_depth -= 1;
                index += 2;
            } else {
                index += 1;
            }
            continue;
        }
        if string_literal {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                string_literal = false;
            }
            index += 1;
            continue;
        }
        if bytes.get(index..index.saturating_add(2)) == Some(b"//") {
            line_comment = true;
            index += 2;
        } else if bytes.get(index..index.saturating_add(2)) == Some(b"/*") {
            block_comment_depth = 1;
            index += 2;
        } else if byte == b'\'' && rust_char_literal_end(text, index).is_some() {
            index = rust_char_literal_end(text, index).expect("checked character literal");
        } else if byte == b'"' {
            string_literal = true;
            index += 1;
        } else {
            index += 1;
        }
    }
    !line_comment && block_comment_depth == 0 && !string_literal
}

fn rust_char_literal_end(text: &str, start: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let value_start = start.checked_add(1)?;
    let first = text.get(value_start..)?.chars().next()?;
    let value_end = if first != '\\' {
        value_start + first.len_utf8()
    } else {
        match bytes.get(value_start + 1)? {
            b'x' => value_start + 4,
            b'u' if bytes.get(value_start + 2) == Some(&b'{') => {
                value_start + 3 + text.get(value_start + 3..)?.find('}')? + 1
            }
            _ => value_start + 2,
        }
    };
    (bytes.get(value_end) == Some(&b'\'')).then_some(value_end + 1)
}

fn find_use_statement_end(text: &str, start: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut index = start;
    let mut brace_depth = 0usize;
    let mut line_comment = false;
    let mut block_comment_depth = 0usize;
    let mut string_literal = false;
    let mut escaped = false;

    while index < bytes.len() {
        let byte = bytes[index];
        if line_comment {
            if byte == b'\n' {
                line_comment = false;
            }
            index += 1;
            continue;
        }
        if block_comment_depth != 0 {
            if bytes.get(index..index.saturating_add(2)) == Some(b"/*") {
                block_comment_depth += 1;
                index += 2;
            } else if bytes.get(index..index.saturating_add(2)) == Some(b"*/") {
                block_comment_depth -= 1;
                index += 2;
            } else {
                index += 1;
            }
            continue;
        }
        if string_literal {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                string_literal = false;
            }
            index += 1;
            continue;
        }
        if bytes.get(index..index.saturating_add(2)) == Some(b"//") {
            line_comment = true;
            index += 2;
            continue;
        }
        if bytes.get(index..index.saturating_add(2)) == Some(b"/*") {
            block_comment_depth = 1;
            index += 2;
            continue;
        }
        match byte {
            b'"' => string_literal = true,
            b'{' => brace_depth += 1,
            b'}' => brace_depth = brace_depth.checked_sub(1)?,
            b';' if brace_depth == 0 => return Some(index + 1),
            _ => {}
        }
        index += 1;
    }
    None
}

fn reviewed_std_sync_use_statement_is_allowed(statement: &str, allowed_symbols: &[&str]) -> bool {
    const FORBIDDEN_SYMBOLS: &[&str] = &[
        "Barrier", "LazyLock", "Weak", "ArcSwap", "Mutex", "RwLock", "Condvar", "mpsc",
    ];
    if FORBIDDEN_SYMBOLS
        .iter()
        .any(|symbol| rust_identifier_is_present(statement, symbol))
    {
        return false;
    }

    let suffix = statement
        .strip_prefix("use std::sync")
        .and_then(|suffix| suffix.strip_suffix(';'))
        .unwrap_or_default();
    if suffix.starts_with("::{") {
        let Some(open) = statement.find('{') else {
            return false;
        };
        return find_matching_brace(statement, open)
            .map(|close| grouped_imports_are_allowed(&statement[open + 1..close], allowed_symbols))
            .unwrap_or(false);
    }

    let direct = suffix.strip_prefix("::").unwrap_or_default();
    let symbol_end = direct
        .find(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .unwrap_or(direct.len());
    let symbol = &direct[..symbol_end];
    !symbol.is_empty() && allowed_symbols.contains(&symbol)
}

fn grouped_imports_are_allowed(grouped: &str, allowed_symbols: &[&str]) -> bool {
    let mut depth = 0usize;
    let mut item_start = 0usize;
    for (index, character) in grouped.char_indices() {
        match character {
            '{' => depth += 1,
            '}' => match depth.checked_sub(1) {
                Some(next_depth) => depth = next_depth,
                None => return false,
            },
            ',' if depth == 0 => {
                let item = &grouped[item_start..index];
                if item.trim().is_empty() || !grouped_import_is_allowed(item, allowed_symbols) {
                    return false;
                }
                item_start = index + character.len_utf8();
            }
            _ => {}
        }
    }
    if depth != 0 {
        return false;
    }
    let final_item = grouped[item_start..].trim();
    final_item.is_empty() && item_start != 0
        || !final_item.is_empty() && grouped_import_is_allowed(final_item, allowed_symbols)
}

fn grouped_import_is_allowed(item: &str, allowed_symbols: &[&str]) -> bool {
    let item = item.trim();
    if item.is_empty() {
        return false;
    }
    let symbol_end = item
        .find(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .unwrap_or(item.len());
    let symbol = &item[..symbol_end];
    if !allowed_symbols.contains(&symbol) {
        return false;
    }
    let rest = item[symbol_end..].trim_start();
    (symbol == "atomic" && (rest.is_empty() || rest.starts_with("::{")))
        || (symbol != "atomic" && (rest.is_empty() || rest.starts_with(" as ")))
}

fn rust_identifier_is_present(text: &str, identifier: &str) -> bool {
    text.match_indices(identifier).any(|(index, _)| {
        let before = text.as_bytes().get(index.wrapping_sub(1)).copied();
        let after = text.as_bytes().get(index + identifier.len()).copied();
        !before.is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            && !after.is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    })
}

#[test]
fn reviewed_std_sync_admission_uses_actual_sources_and_preserves_negative_guards() {
    for relative in REVIEWED_STD_SYNC_PATHS {
        let source = manifest_dir().join(relative);
        let text = std::fs::read_to_string(&source).expect("read reviewed std::sync source");
        assert!(
            std_sync_contract_primitive_is_reviewed(&source, &text, "std::sync"),
            "actual reviewed source must retain its exact std::sync import form: {relative}"
        );
        #[cfg(windows)]
        {
            let windows_source = manifest_dir().join(relative.replace('/', "\\"));
            assert!(std_sync_contract_primitive_is_reviewed(
                &windows_source,
                &text,
                "std::sync"
            ));
        }
    }

    let arc_source = Path::new("src/ui/dispatch/input/effect.rs");
    assert!(!std_sync_contract_primitive_is_reviewed(
        arc_source,
        "use std::sync::Barrier;\nfn trailing_source() {}",
        "std::sync"
    ));
    assert!(!std_sync_contract_primitive_is_reviewed(
        arc_source,
        "use std::sync::LazyLock;\nfn trailing_source() {}",
        "std::sync"
    ));
    assert!(!std_sync_contract_primitive_is_reviewed(
        arc_source,
        "use std::sync::{Arc, Weak};\nfn trailing_source() {}",
        "std::sync"
    ));
    assert!(!std_sync_contract_primitive_is_reviewed(
        arc_source,
        "use std::sync::{Arc, atomic::{AtomicU8, Barrier}};\nfn trailing_source() {}",
        "std::sync"
    ));
}

#[test]
fn canonical_text_spool_os_exception_stays_exact() {
    let source = manifest_dir().join(CANONICAL_TEXT_SPOOL_PATH);
    let text = std::fs::read_to_string(source).expect("read canonical text spool");
    let text = without_cfg_test_items(&text);
    let actual: Vec<_> = FORBIDDEN_SOURCE_NEEDLES
        .iter()
        .copied()
        .filter(|needle| text.contains(needle))
        .collect();

    assert_eq!(
        actual, CANONICAL_TEXT_SPOOL_OS_NEEDLES,
        "canonical text sorting may use only its reviewed disk-spool OS primitives"
    );
}

#[test]
fn runtime_api_surface_stays_folder_backed_by_abi_owner() {
    let legacy_root_path = manifest_dir().join("src").join("runtime_api.rs");
    assert!(
        !legacy_root_path.exists(),
        "runtime_api.rs was superseded by runtime_api/mod.rs and must not be restored"
    );

    let root_path = manifest_dir()
        .join("src")
        .join("runtime_api")
        .join("mod.rs");
    let root_text = std::fs::read_to_string(&root_path).expect("read runtime_api facade");
    let facade_lines = root_text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();

    assert!(
        facade_lines <= RUNTIME_API_FACADE_LINE_BUDGET,
        "runtime_api/mod.rs must stay a small facade over owner modules; found {facade_lines} non-empty lines"
    );
    let facade_reexport_statements = root_text
        .lines()
        .filter(|line| line.trim_start().starts_with("pub use "))
        .count();
    assert!(
        facade_reexport_statements <= RUNTIME_API_FACADE_REEXPORT_BUDGET,
        "runtime_api/mod.rs must group its curated re-exports by domain; found {facade_reexport_statements} statements"
    );
    for forbidden in [
        "#[repr(",
        "pub struct ",
        "pub enum ",
        "pub const ",
        "pub type ",
    ] {
        assert!(
            !root_text.contains(forbidden),
            "runtime_api/mod.rs must not own ABI declarations directly; found `{forbidden}`"
        );
    }

    let module_root = manifest_dir().join("src").join("runtime_api");
    let expected_files: BTreeSet<_> = EXPECTED_RUNTIME_API_OWNER_PATHS
        .iter()
        .map(|path| (*path).to_string())
        .collect();
    let actual_files = runtime_api_owner_sources(&module_root);

    assert_eq!(
        actual_files, expected_files,
        "runtime_api owner paths changed; update the boundary review when adding/removing ABI owner files"
    );

    for domain in EXPECTED_RUNTIME_API_DOMAINS {
        assert!(
            root_text.contains(&format!("mod {domain};")),
            "runtime_api/mod.rs must declare `{domain}` as an ABI owner domain"
        );
        assert!(
            root_text.contains(&format!("pub use {domain}::{{")),
            "runtime_api/mod.rs must explicitly re-export `{domain}` through runtime_api::*"
        );
    }

    assert!(
        !root_text
            .lines()
            .any(|line| line.trim_start().starts_with("pub use ") && line.contains("::*;")),
        "runtime_api/mod.rs must not use glob re-exports"
    );

    for owner_path in EXPECTED_RUNTIME_API_OWNER_PATHS {
        let module_path = module_root.join(owner_path);
        let module_text = std::fs::read_to_string(&module_path).expect("read runtime_api owner");
        let module_lines = module_text.lines().count();
        assert!(
            module_lines <= RUNTIME_API_CHILD_LINE_BUDGET,
            "{} must be split before it becomes another support hot spot; found {module_lines} lines",
            relative_to_manifest(&module_path).display()
        );
    }

    for facade_path in RUNTIME_API_DOMAIN_FACADE_PATHS {
        let facade_path = module_root.join(facade_path);
        let facade_text =
            std::fs::read_to_string(&facade_path).expect("read runtime_api domain facade");
        assert!(
            !facade_text
                .lines()
                .any(|line| { line.trim_start().starts_with("pub use ") && line.contains("::*;") }),
            "{} must explicitly re-export its ABI owner surface",
            relative_to_manifest(&facade_path).display()
        );
    }
}

fn runtime_api_owner_sources(module_root: &Path) -> BTreeSet<String> {
    let mut owner_paths = BTreeSet::new();
    collect_runtime_api_owner_sources(module_root, module_root, &mut owner_paths);
    owner_paths
}

fn collect_runtime_api_owner_sources(
    module_root: &Path,
    path: &Path,
    owner_paths: &mut BTreeSet<String>,
) {
    for entry in std::fs::read_dir(path).expect("read runtime_api owner directory") {
        let entry = entry.expect("read runtime_api owner entry");
        let path = entry.path();
        if path.is_dir() {
            if path
                .file_name()
                .is_some_and(|name| name == OsStr::new("tests"))
            {
                continue;
            }
            collect_runtime_api_owner_sources(module_root, &path, owner_paths);
        } else if path
            .extension()
            .is_some_and(|extension| extension == OsStr::new("rs"))
            && path
                .file_name()
                .is_some_and(|name| name != OsStr::new("mod.rs"))
            && !path
                .file_stem()
                .is_some_and(|stem| stem.to_string_lossy().ends_with("_tests"))
        {
            owner_paths.insert(
                path.strip_prefix(module_root)
                    .expect("runtime_api owner stays below its module root")
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        }
    }
}

fn production_rust_sources() -> Vec<PathBuf> {
    let source_root = manifest_dir().join("src");
    let mut sources = Vec::new();
    collect_rust_sources(&source_root, &mut sources);
    sources
}

fn collect_rust_sources(path: &Path, sources: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(path).expect("read interface source directory") {
        let entry = entry.expect("read interface source entry");
        let path = entry.path();
        if path.is_dir() {
            if path
                .file_name()
                .is_some_and(|name| name == OsStr::new("tests"))
            {
                continue;
            }
            collect_rust_sources(&path, sources);
        } else if path
            .extension()
            .is_some_and(|extension| extension == OsStr::new("rs"))
            && !path.file_stem().is_some_and(|stem| {
                let stem = stem.to_string_lossy();
                stem.ends_with("_tests") || stem.ends_with("_test")
            })
        {
            sources.push(path);
        }
    }
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn relative_to_manifest(path: &Path) -> PathBuf {
    path.strip_prefix(manifest_dir())
        .unwrap_or(path)
        .to_path_buf()
}
