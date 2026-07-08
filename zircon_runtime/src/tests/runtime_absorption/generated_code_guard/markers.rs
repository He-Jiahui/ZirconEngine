use std::fs;
use std::path::Path;

use super::support::{relative_path, rust_source_files};

const GENERATED_MARKER_PREFIX: &str = "// @generated ";
const GENERATED_MARKER_SUFFIX: &str = " - do not edit by hand";

const MARKED_GENERATED_FORBIDDEN_TOKENS: &[&str] = &[
    "impl ",
    "fn ",
    "match ",
    "for ",
    "while ",
    "if let ",
    "EntryRunner::",
    "NativePluginLoader",
    "runtime_plugin_registrations",
    "plugin_registration()",
    "SceneSchedule",
    "CoreRuntime",
];

#[test]
fn generated_marker_format_is_uniform_when_source_files_are_marked() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_root = manifest_root.join("src");
    let files = rust_source_files(&source_root);

    let mut invalid_markers = Vec::new();
    for path in files {
        let relative = relative_path(manifest_root, &path);
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"));
        for (line_index, line) in source.lines().enumerate() {
            if !line.trim_start().starts_with(GENERATED_MARKER_PREFIX) {
                continue;
            }
            let is_valid_first_line = line_index == 0
                && line.starts_with(GENERATED_MARKER_PREFIX)
                && line.ends_with(GENERATED_MARKER_SUFFIX)
                && line.len() > GENERATED_MARKER_PREFIX.len() + GENERATED_MARKER_SUFFIX.len();
            if !is_valid_first_line {
                invalid_markers.push(format!("{}:{}: {}", relative, line_index + 1, line.trim()));
            }
        }
    }

    assert!(
        invalid_markers.is_empty(),
        "generated source markers must use `{GENERATED_MARKER_PREFIX}<generator>{GENERATED_MARKER_SUFFIX}` on line 1:\n{}",
        invalid_markers.join("\n")
    );
}

#[test]
fn marked_generated_source_files_stay_leaf_data_only() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_root = manifest_root.join("src");
    let files = rust_source_files(&source_root);

    let mut behavior_locations = Vec::new();
    for path in files {
        let relative = relative_path(manifest_root, &path);
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"));
        let Some(first_line) = source.lines().next() else {
            continue;
        };
        if !first_line.starts_with(GENERATED_MARKER_PREFIX) {
            continue;
        }
        for (line_index, line) in source.lines().enumerate().skip(1) {
            if let Some(token) = MARKED_GENERATED_FORBIDDEN_TOKENS
                .iter()
                .find(|token| line.contains(**token))
            {
                behavior_locations.push(format!(
                    "{}:{}: `{}` in {}",
                    relative,
                    line_index + 1,
                    token,
                    line.trim()
                ));
            }
        }
    }

    assert!(
        behavior_locations.is_empty(),
        "generated source files must stay leaf data/DTO/table artifacts and cannot own behavior:\n{}",
        behavior_locations.join("\n")
    );
}
