//! 核对接口 crate 的依赖边界、受审例外以及 Runtime API 的目录归属。
//!
//! 生产文件过滤与文本扫描按本仓源码约定工作，不是完整 Rust 属性求值或语法解析。
//! 模块路径必须解析到本 crate 的源码；实现路径的说明性注释不算源码引入。
//! 其他禁用文本与能力例外仍按准确来源路径检查。

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// 接口契约允许依赖的 crate 名称；生产依赖必须逐项留在此清单。
const ALLOWED_DEPENDENCIES: &[&str] = &[
    "bincode-next",
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

/// 当前不接受开发依赖；新测试依赖需要显式边界审查。
const ALLOWED_DEV_DEPENDENCIES: &[&str] = &[];
/// 生成接口清单仅允许已登记的构建依赖。
const ALLOWED_BUILD_DEPENDENCIES: &[&str] = &["serde_json"];

/// 生产文本中的实现引入、操作系统访问和其他禁用标记；例外按路径另行审核。
const FORBIDDEN_SOURCE_NEEDLES: &[&str] = &[
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

/// 规范文本磁盘排序已审查的三个系统能力，仍须与实际命中精确相等。
const CANONICAL_TEXT_SPOOL_OS_NEEDLES: &[&str] = &["std::fs", "std::process", "std::sync"];
/// 磁盘排序例外的唯一来源路径。
const CANONICAL_TEXT_SPOOL_PATH: &str = "src/serialization/text/canonical_spool.rs";
/// 最近项目存储允许的文件与线程能力，不放行任意系统访问。
const SHARED_RECENT_PROJECTS_STORE_OS_NEEDLES: &[&str] = &["std::fs", "std::thread"];
/// 最近项目存储例外的唯一来源路径。
const SHARED_RECENT_PROJECTS_STORE_PATH: &str = "src/hub_protocol/recent_projects/store.rs";
/// 编译期输入例外必须同时匹配路径和对应包含标记。
const REVIEWED_COMPILE_TIME_INPUTS: &[(&str, &str)] = &[
    ("src/project/template_pack/embedded.rs", "include_bytes!("),
    ("src/runtime_build_set/interface_spec.rs", "include_str!("),
    ("src/runtime_build_set/payload_schema.rs", "include_bytes!("),
];
/// 已审核同步原语来源；具体可用符号仍由逐路径映射限制。
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
/// Runtime API 门面必须声明并显式重导出的五个归属域。
const EXPECTED_RUNTIME_API_DOMAINS: &[&str] = &["abi", "constants", "frame", "host", "session"];
/// 具体归属文件的冻结路径清单，按集合与真实文件树比较。
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
    "session/ime_compose_capability_v2.rs",
    "session/ime_composition_v2.rs",
    "session/operation.rs",
    "session/plugin_event_mirror.rs",
    "session/requests.rs",
    "session/session.rs",
    "session/session_identity.rs",
    "session/translated_events.rs",
    "session/viewport.rs",
];
/// 顶层门面非空行数上限；注释也计入。
const RUNTIME_API_FACADE_LINE_BUDGET: usize = 220;
/// 顶层门面公开重导出语句行数上限。
const RUNTIME_API_FACADE_REEXPORT_BUDGET: usize = 6;
/// 每个具体归属文件的总行数上限；空行和注释同样计入。
const RUNTIME_API_CHILD_LINE_BUDGET: usize = 700;
/// 域门面不得使用通配重导出的路径清单。
const RUNTIME_API_DOMAIN_FACADE_PATHS: &[&str] = &[
    "abi/mod.rs",
    "frame/mod.rs",
    "host/mod.rs",
    "session/mod.rs",
];

#[test]
/// 比较顶层三类依赖名，将各类未许可项集中报告。
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
/// 扫描过滤后的生产全文，只有逐路径例外才能消除禁用标记的违规项。
fn production_source_does_not_include_or_import_implementation_crates() {
    let sources = production_rust_sources();
    let mut violations = Vec::new();

    for source in sources {
        let text = std::fs::read_to_string(&source).expect("read interface source");
        let production_text = without_cfg_test_items(&text);
        violations.extend(local_module_path_violations(&source, &production_text));
        for needle in FORBIDDEN_SOURCE_NEEDLES {
            if reviewed_compile_time_input(&source, needle)
                || canonical_text_spool_os_access_is_reviewed(&source, needle)
                || shared_recent_projects_store_os_access_is_reviewed(&source, needle)
                || std_sync_contract_primitive_is_reviewed(&source, &production_text, needle)
            {
                continue;
            }
            let present = if matches!(*needle, "zircon_runtime/src" | "zircon_editor/src") {
                production_text
                    .match_indices(*needle)
                    .any(|(offset, _)| is_rust_code_position(&production_text, offset))
            } else {
                production_text.contains(needle)
            };
            if present {
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

/// 显式模块挂载只允许普通相对路径，并验证其真实目标仍属于接口源码。
/// 条件路径属性需要单独审查，不能借条件编译绕过模块归属检查。
pub(super) fn local_module_path_violations(source: &Path, text: &str) -> Vec<String> {
    let root = manifest_dir()
        .join("src")
        .canonicalize()
        .expect("resolve interface source root");
    let mut violations = Vec::new();
    for (offset, _) in text.match_indices('#') {
        if !is_rust_code_position(text, offset) {
            continue;
        }
        let Some(remainder) = text[offset + 1..].trim_start().strip_prefix('[') else {
            continue;
        };
        let remainder = remainder.trim_start();
        let Some(end) = remainder.find(']') else {
            continue;
        };
        let attribute = remainder[..end].trim();
        if attribute.starts_with("cfg_attr") && attribute.contains("path") {
            violations.push(format!(
                "{} has an unreviewed conditional module path",
                relative_to_manifest(source).display()
            ));
            continue;
        }
        let Some(value) = attribute.strip_prefix("path") else {
            continue;
        };
        if value
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        {
            continue;
        }
        let relative = value
            .trim_start()
            .strip_prefix('=')
            .map(str::trim)
            .and_then(|value| value.strip_prefix('"'))
            .and_then(|value| value.strip_suffix('"'));
        let valid = relative.is_some_and(|relative| {
            !relative.is_empty()
                && !relative.contains([':', '\\', '"'])
                && !Path::new(relative).is_absolute()
                && source
                    .parent()
                    .and_then(|parent| parent.join(relative).canonicalize().ok())
                    .is_some_and(|target| {
                        target.starts_with(&root)
                            && target.is_file()
                            && target.extension() == Some(OsStr::new("rs"))
                    })
        });
        if !valid {
            violations.push(format!(
                "{} has a module path outside its contract source owner: {attribute}",
                relative_to_manifest(source).display()
            ));
        }
    }
    violations
}

#[test]
fn module_paths_stay_within_the_physical_contract_source_owner() {
    let source = manifest_dir().join("src/lib.rs");
    for text in [
        "#[path = \"ui/mod.rs\"] mod ui;",
        "#[ path = \"world_sync/mod.rs\"] mod world_sync;",
        "# [ path = \"ui/mod.rs\"] mod ui;",
        "// #[path = \"../../zircon_runtime/src/lib.rs\"]",
        "const NOTE: &str = \"#[path = \\\"../../zircon_runtime/src/lib.rs\\\"]\";",
    ] {
        assert!(local_module_path_violations(&source, text).is_empty());
    }
    for text in [
        "#[path = \"../../zircon_runtime/src/lib.rs\"] mod runtime;",
        "# [ path = \"../../zircon_runtime/src/lib.rs\"] mod runtime;",
        "#[path = \"E:/Git/ZirconEngine/zircon_runtime/src/lib.rs\"] mod runtime;",
        "#[path = \"missing.rs\"] mod missing;",
        "#[path = concat!(\"ui/\", \"mod.rs\")] mod ui;",
        "#[cfg_attr(feature = \"runtime\", path = \"../../zircon_runtime/src/lib.rs\")] mod runtime;",
    ] {
        assert_eq!(local_module_path_violations(&source, text).len(), 1);
    }
    let raw_note_then_external_path = r###"const NOTE: &str = r##"quote " #[path = "ui/mod.rs"]"##;
#[path = "../../zircon_runtime/src/lib.rs"] mod runtime;"###;
    assert_eq!(
        local_module_path_violations(&source, raw_note_then_external_path).len(),
        1
    );
}

// These exact exceptions cover reviewed compile-time contract inputs, the bounded canonical
// text spool, and the shared recent-project transaction owner. All other interface production
// sources remain subject to the contract-layer I/O and implementation-dependency boundary.
/// 判断 `source` 与 `needle` 是否同时匹配某个已审编译期输入；不按目录整体放行。
fn reviewed_compile_time_input(source: &Path, needle: &str) -> bool {
    REVIEWED_COMPILE_TIME_INPUTS
        .iter()
        .any(|(path, reviewed_needle)| {
            needle == *reviewed_needle && relative_to_manifest(source) == Path::new(path)
        })
}

/// 仅在规范文本排序的准确路径上允许已列系统标记。
fn canonical_text_spool_os_access_is_reviewed(source: &Path, needle: &str) -> bool {
    relative_to_manifest(source) == Path::new(CANONICAL_TEXT_SPOOL_PATH)
        && CANONICAL_TEXT_SPOOL_OS_NEEDLES.contains(&needle)
}

/// 返回去掉已识别测试专属项的生产文本，保留可能用于生产的条件项。
///
/// `source` 必须按守卫认识的属性写法组织；移除范围通过属性后的分号或配对括号确定。
/// 注释中的属性不作为代码处理，返回值仍含生产注释，不代表完整条件编译结果。
fn without_cfg_test_items(source: &str) -> String {
    /// 仅识别列出的测试专属属性文本，其他条件项继续参与生产检查。
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

/// 在 `source` 的字节偏移 `open` 起寻找配对结束括号，跳过已识别的注释与字面量。
///
/// 返回结束括号的字节偏移；扫描不到配对时返回空值，不尝试补全非法源码。
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
        if let Some(end) = rust_raw_string_end(source, index) {
            index = end;
            continue;
        }
        if byte == b'\'' {
            if let Some(end) = rust_char_literal_end(source, index) {
                index = end;
                continue;
            }
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
/// 测试专属项被移除，生产导入与可用于生产的条件项仍参与边界守卫。
fn cfg_test_source_filter_preserves_production_boundary_guards() {
    let source = "#[cfg(test)]\nmod tests { const ONLY_TEST: &str = \"std::fs\"; }\nuse std::fs;";
    let production = without_cfg_test_items(source);
    assert!(!production.contains("ONLY_TEST"));
    assert!(production.contains("use std::fs;"));

    let source = r###"#[cfg(test)]
mod tests { const RAW: &str = r##"} \" #[path = "ignored.rs"]"##; let _ = '{'; }
use std::fs;"###;
    let production = without_cfg_test_items(source);
    assert!(!production.contains("const RAW"));
    assert!(production.contains("use std::fs;"));

    let source = r###"const RAW: &str = r##"\" #[cfg(test)] mod ignored;"##;
use std::fs;"###;
    assert_eq!(without_cfg_test_items(source), source);

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
/// 编译期输入和最近项目例外同时限制路径与能力，邻近路径不得借用许可。
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

/// 只允许最近项目存储准确路径上的两个已列系统标记。
fn shared_recent_projects_store_os_access_is_reviewed(source: &Path, needle: &str) -> bool {
    relative_to_manifest(source) == Path::new(SHARED_RECENT_PROJECTS_STORE_PATH)
        && SHARED_RECENT_PROJECTS_STORE_OS_NEEDLES.contains(&needle)
}

/// 校验指定路径的同步导入是否全部限制在其专属符号清单内。
///
/// `needle` 须为同步标记；`text` 中至少有一个真实导入且每个导入均通过才返回真。
/// 路径分隔符统一后比较，未知路径或其他标记直接拒绝。
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

/// 借用返回 `text` 中从行首开始且位于代码区间的完整同步导入语句。
///
/// 返回切片的生命周期依附输入文本；遇到无法闭合的候选语句时停止收集。
fn reviewed_std_sync_use_statements(text: &str) -> Vec<&str> {
    /// 候选导入的固定起始文本，后续仍须验证位置与语句结束。
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

/// 扫描到字节位置 `target`，判断它是否处于已识别注释和字符串之外。
///
/// 调用方提供输入内的合法偏移；此局部状态机不代替完整 Rust 词法分析。
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
        } else if let Some(end) = rust_raw_string_end(text, index) {
            if end > target {
                return false;
            }
            index = end;
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

/// 跳过 Rust 原始字符串，包括字节串与 C 字符串的前缀；内嵌引号不结束扫描。
fn rust_raw_string_end(text: &str, start: usize) -> Option<usize> {
    let remainder = text.get(start..)?;
    let prefix_len = if remainder.starts_with("br") || remainder.starts_with("cr") {
        2
    } else if remainder.starts_with('r') {
        1
    } else {
        return None;
    };
    let hashes = remainder[prefix_len..]
        .bytes()
        .take_while(|byte| *byte == b'#')
        .count();
    let quote = start + prefix_len + hashes;
    if text.as_bytes().get(quote) != Some(&b'"') {
        return None;
    }
    let closing = format!("\"{}", "#".repeat(hashes));
    let body_start = quote + 1;
    Some(body_start + text[body_start..].find(&closing)? + closing.len())
}

/// 从单引号位置 `start` 识别单字符或已支持的转义，并返回结束引号后的字节位置。
///
/// 无法识别闭合字符字面量时返回空值，使生命周期标记等单引号保留普通扫描路径。
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

/// 返回 `start` 起导入语句在顶层分号之后的字节位置。
///
/// 分组括号、已识别注释和字符串中的分号不终止扫描；括号不匹配或无结束时返回空值。
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

/// 按 `allowed_symbols` 校验单条导入，同时优先拒绝列出的强同步或渠道标识符。
///
/// 直接导入与分组导入走不同文本分支；返回值仅用于本守卫支持的源码导入形式。
fn reviewed_std_sync_use_statement_is_allowed(statement: &str, allowed_symbols: &[&str]) -> bool {
    /// 即使出现在分组内层，也须按完整标识符边界拒绝的能力名称。
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

/// 仅在顶层逗号处拆分 `grouped`，并要求每项通过符号校验。
///
/// 允许尾逗号；空的中间项或不配对的嵌套括号返回假。
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

/// 校验单个分组项：原子域可继续分组，其他已允许符号仅接受裸名或别名后缀。
///
/// `item` 先去掉首尾空白；空项或未登记的首个符号返回假。
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

/// 检查 `identifier` 的匹配两侧是否均为非 ASCII 标识符字符，避免误拒绝包含它的长名。
///
/// 这是文本标识符边界检查，不区分匹配是否出现在注释或字面量内。
fn rust_identifier_is_present(text: &str, identifier: &str) -> bool {
    text.match_indices(identifier).any(|(index, _)| {
        let before = text.as_bytes().get(index.wrapping_sub(1)).copied();
        let after = text.as_bytes().get(index + identifier.len()).copied();
        !before.is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            && !after.is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    })
}

#[test]
/// 实际已审源码及 Windows 路径均被接受，未许可原语和嵌套禁用项仍被拒绝。
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
/// 磁盘排序源码实际命中的系统能力必须恰好等于已审三项集合。
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
/// Runtime API 门面、归属路径、重导出形式及行数共同受约束，避免声明重新堆回门面。
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

/// 返回 `module_root` 下归属文件的标准化相对路径集合，去重并提供稳定比较顺序。
fn runtime_api_owner_sources(module_root: &Path) -> BTreeSet<String> {
    let mut owner_paths = BTreeSet::new();
    collect_runtime_api_owner_sources(module_root, module_root, &mut owner_paths);
    owner_paths
}

/// 递归追加归属路径，仅跳过测试目录、域门面和指定测试后缀的文件。
///
/// `path` 必须位于 `module_root` 内；路径去前缀失败或目录读取失败时 panic。
/// 不解析模块声明，因此准确文件名仍须通过调用方的冻结清单核对。
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

/// 返回接口源码中的生产 Rust 路径，保留根入口和门面供边界全文扫描。
fn production_rust_sources() -> Vec<PathBuf> {
    let source_root = manifest_dir().join("src");
    let mut sources = Vec::new();
    collect_rust_sources(&source_root, &mut sources);
    sources
}

/// 递归追加 Rust 文件，排除 tests 目录及两种测试文件名后缀。
///
/// `sources` 沿用目录遍历顺序；不按模块是否可达筛选，读取失败直接终止测试。
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

/// 使用编译时 crate 根目录，使源码扫描不依赖测试进程的当前目录。
fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// 去掉接口 crate 根前缀用于诊断；前缀不匹配时保留原路径。
fn relative_to_manifest(path: &Path) -> PathBuf {
    path.strip_prefix(manifest_dir())
        .unwrap_or(path)
        .to_path_buf()
}
