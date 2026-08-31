use std::path::{Path, PathBuf};

use super::{runtime_src_path, rust_source_view::production_section};

const EXPECTED_RUNTIME_C_ABI_ENTRIES: usize = 42;
const EXPECTED_EXPORT_TEMPLATE_ENTRIES: usize = 13;

#[test]
fn runtime_production_ffi_entries_are_classified_and_panic_guarded() {
    let runtime_src = runtime_src_path("");
    let mut rust_files = Vec::new();
    collect_rust_files(&runtime_src, &mut rust_files);
    rust_files.sort();

    let mut entries = Vec::new();
    for path in rust_files {
        let relative = path
            .strip_prefix(&runtime_src)
            .expect("runtime source should stay below its root");
        if is_test_source(relative)
            || relative == Path::new("plugin/export_build_plan/platform_host_files.rs")
        {
            continue;
        }
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        let production = production_section(&source);
        for entry in ffi_entry_names(&production) {
            let relative = normalized_path(relative);
            let window = function_window(&production, &entry);
            let expected_guard = expected_runtime_guard(&relative, &entry);
            assert!(
                window.contains(expected_guard),
                "production FFI entry `{relative}::{entry}` must call `{expected_guard}` at its boundary"
            );
            entries.push((relative.clone(), entry));
        }
    }

    entries.sort();
    assert_eq!(
        entries.len(),
        EXPECTED_RUNTIME_C_ABI_ENTRIES,
        "Runtime C ABI inventory changed; classify every new or retired entry explicitly: {entries:#?}"
    );
    assert_eq!(
        entries
            .iter()
            .filter(|(path, _)| path == "dynamic_api/exports.rs")
            .count(),
        27,
        "dynamic API C ABI inventory drifted"
    );
    assert_eq!(
        entries
            .iter()
            .filter(|(path, _)| path.starts_with("plugin/native_plugin_loader/"))
            .count(),
        15,
        "native plugin host C ABI inventory drifted"
    );
}

#[test]
fn generated_export_host_entries_are_classified_and_panic_guarded() {
    let template = include_str!("../../../plugin/export_build_plan/platform_host_files.rs");
    let entries = ffi_entry_names(template);

    assert_eq!(
        entries.len(),
        EXPECTED_EXPORT_TEMPLATE_ENTRIES,
        "generated export-host ABI inventory changed; classify every new or retired entry explicitly: {entries:#?}"
    );
    for entry in entries {
        assert!(
            function_window(template, &entry).contains("zircon_export_ffi_guard"),
            "generated export-host FFI entry `{entry}` must call `zircon_export_ffi_guard`"
        );
    }
}

fn expected_runtime_guard(path: &str, entry: &str) -> &'static str {
    if path == "dynamic_api/exports.rs" {
        if entry == "zircon_runtime_get_api_v8" {
            "catch_unwind(AssertUnwindSafe"
        } else {
            "catch_ffi_panic"
        }
    } else if path == "plugin/native_plugin_loader/host_callbacks.rs" {
        "catch_native_plugin_host_callback_panic"
    } else if path.starts_with("plugin/native_plugin_loader/host_api_adapter/") {
        "catch_native_host_api_panic"
    } else if path == "plugin/native_plugin_loader/behavior_calls/output_sink.rs" {
        "catch_native_plugin_output_sink_panic"
    } else {
        panic!("unclassified production FFI owner `{path}::{entry}`")
    }
}

fn ffi_entry_names(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| {
            ["extern \"C\" fn ", "extern \"system\" fn "]
                .into_iter()
                .find_map(|marker| {
                    let suffix = line.split_once(marker)?.1.trim_start();
                    let name = suffix.split_once('(')?.0.trim();
                    (!name.is_empty()
                        && name
                            .bytes()
                            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_'))
                    .then(|| name.to_string())
                })
        })
        .collect()
}

fn function_window<'a>(source: &'a str, entry: &str) -> &'a str {
    let start = source
        .find(&format!("fn {entry}"))
        .unwrap_or_else(|| panic!("missing FFI entry body for `{entry}`"));
    let suffix = &source[start..];
    let end = suffix
        .match_indices('\n')
        .nth(32)
        .map_or(suffix.len(), |(offset, _)| offset);
    &suffix[..end]
}

fn collect_rust_files(directory: &Path, output: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("failed to enumerate {}: {error}", directory.display()));
    for entry in entries {
        let path = entry.expect("runtime source directory entry").path();
        if path.is_dir() {
            collect_rust_files(&path, output);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            output.push(path);
        }
    }
}

fn is_test_source(relative: &Path) -> bool {
    relative
        .components()
        .any(|component| component.as_os_str() == "tests")
        || relative.file_name().is_some_and(|name| name == "tests.rs")
}

fn normalized_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}
