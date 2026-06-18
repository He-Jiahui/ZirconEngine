use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const EXPECTED_RUNTIME_06_SOURCE_FILES: &[&str] = &[
    "src/plugin/mod.rs",
    "src/plugin/native.rs",
    "src/plugin/native_plugin_loader/mod.rs",
    "src/plugin/native_plugin_loader/abi_declarations.rs",
    "src/plugin/native_plugin_loader/native_plugin_abi.rs",
    "src/plugin/native_plugin_loader/native_plugin_live_host/lifecycle.rs",
    "src/plugin/native_plugin_loader/native_plugin_live_host/hot_reload.rs",
    "src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_failures.rs",
    "src/script/vm/backend/zr_vm_project_backend/real_backend/instance.rs",
    "src/script/vm/tests.rs",
    "src/script/vm/tests/lifecycle_failures.rs",
    "src/tests/runtime_absorption/plan_status/cargo_gates/early.rs",
    "src/tests/runtime_absorption/plugin_surface_lifecycle.rs",
    "../zircon_plugins/native_dynamic_fixture/native/src/lib.rs",
];

const EXPECTED_RUNTIME_06_MIRROR_DOCS: &[&str] = &[
    "../docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md",
    "../docs/plans/zircon_runtime/runtime/index.md",
    "../docs/engine-architecture/native-plugin-boundary.md",
    "../docs/engine-architecture/runtime-interface-convergence.md",
    "../docs/engine-architecture/runtime-architecture-review-m0.md",
];

const V1_V2_PATTERNS: &[&str] = &[
    "NativePluginAbiV1",
    "NativePluginAbiV2",
    "DESCRIPTOR_SYMBOL_V1",
    "DESCRIPTOR_SYMBOL_V2",
    "ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V1",
    "ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V2",
];

const NATIVE_LOADER_TEST_PATTERNS: &[&str] = &[
    "NativePluginAbi",
    "NativePluginEntryReport",
    "NativePluginBehavior",
    "NativePluginLoader",
    "ZIRCON_NATIVE_PLUGIN_STATUS",
];

const LIFECYCLE_FALLBACK_TESTS: &[&str] = &[
    "vm_lifecycle_fallback_activate_bad_entry_module_surfaces_vm_error",
    "vm_lifecycle_fallback_missing_optional_export_returns_none_not_error",
    "vm_lifecycle_fallback_deactivate_is_idempotent_after_unload",
    "vm_lifecycle_fallback_empty_arguments_do_not_require_real_backend",
];

#[test]
fn runtime_06_plugin_surface_lifecycle_mirror_docs_match_structure_audit_counts() {
    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert_eq!(
        EXPECTED_RUNTIME_06_SOURCE_FILES.len(),
        14,
        "Runtime 06 source inventory should mirror plugin_surface_lifecycle_boundary"
    );
    for source_file in EXPECTED_RUNTIME_06_SOURCE_FILES {
        assert!(
            runtime_root.join(source_file).exists(),
            "Runtime 06 source owner `{source_file}` is missing; update plugin_surface_lifecycle_boundary before changing plugin lifecycle coverage"
        );
    }

    assert_eq!(
        EXPECTED_RUNTIME_06_MIRROR_DOCS.len(),
        5,
        "Runtime 06 mirror-doc inventory should mirror plugin_surface_lifecycle_boundary"
    );
    for doc_file in EXPECTED_RUNTIME_06_MIRROR_DOCS {
        assert!(
            runtime_root.join(doc_file).exists(),
            "Runtime 06 mirror doc `{doc_file}` is missing"
        );
    }

    let plan_doc = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md"
    );
    assert!(
        plan_doc.contains("status: in_progress"),
        "Runtime 06 should stay in_progress until plugin/native/app/plugins validation closes"
    );
    assert!(
        plan_doc.contains("last_refined: 2026-06-16"),
        "Runtime 06 last_refined should cover the latest mirror-doc row"
    );

    let plugin_root_source = include_str!("../../plugin/mod.rs");
    assert!(
        plugin_root_source.contains("pub mod native;"),
        "plugin root should expose the hard-cutover plugin::native namespace"
    );
    assert!(
        !plugin_root_source.contains("pub use native_plugin_loader::{"),
        "plugin root should not re-export native loader symbols after the M2.1 hard-cutover"
    );

    let native_namespace_symbols = native_plugin_namespace_reexport_symbols();
    assert_eq!(
        native_namespace_symbols.len(),
        60,
        "native plugin namespace re-export count changed; update native_plugin_public_surface and Runtime 06 mirror docs"
    );
    for required_symbol in [
        "NativePluginAbiV3",
        "NativePluginBridgeMethodTableV3",
        "NativeHostBridgeCallScope",
        "NativePluginLoader",
        "ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V3",
    ] {
        assert!(
            native_namespace_symbols.contains(&required_symbol.to_string()),
            "native plugin public-surface mirror should classify `{required_symbol}` through plugin::native"
        );
    }

    let workspace_root = runtime_root
        .parent()
        .expect("zircon_runtime manifest should be inside the workspace root");
    let app_native_plugin_files = files_containing(
        &workspace_root.join("zircon_app").join("src"),
        &["NativePlugin"],
    );
    assert_eq!(
        app_native_plugin_files.len(),
        7,
        "zircon_app NativePlugin call-site file count changed without Runtime 06 audit sync"
    );

    let native_loader_v1_v2_files = files_containing(
        &runtime_root
            .join("src")
            .join("plugin")
            .join("native_plugin_loader"),
        V1_V2_PATTERNS,
    );
    assert_eq!(
        native_loader_v1_v2_files.len(),
        0,
        "native loader V1/V2 implementation file count changed without Runtime 06 audit sync"
    );

    let plugin_v1_v2_usage_files =
        files_containing(&workspace_root.join("zircon_plugins"), V1_V2_PATTERNS);
    let expected_plugin_v1_v2_usage = BTreeSet::<String>::new();
    assert_eq!(
        plugin_v1_v2_usage_files, expected_plugin_v1_v2_usage,
        "Runtime 06 expects V1/V2 plugin usage to stay limited to the native dynamic fixture"
    );

    let export_build_plan_v1_v2_usage = location_count(
        &runtime_root
            .join("src")
            .join("plugin")
            .join("export_build_plan"),
        V1_V2_PATTERNS,
    );
    assert_eq!(
        export_build_plan_v1_v2_usage, 0,
        "export_build_plan should not reference retired native ABI V1/V2 symbols"
    );

    let mirror_docs = [
        ("Runtime 06 plan", plan_doc),
        (
            "runtime index",
            include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md"),
        ),
        (
            "native plugin boundary",
            include_str!("../../../../docs/engine-architecture/native-plugin-boundary.md"),
        ),
        (
            "interface convergence",
            include_str!("../../../../docs/engine-architecture/runtime-interface-convergence.md"),
        ),
        (
            "M0 review",
            include_str!("../../../../docs/engine-architecture/runtime-architecture-review-m0.md"),
        ),
    ];

    for (doc_name, doc_source) in mirror_docs {
        for required_anchor in [
            "plugin_surface_lifecycle_boundary",
            "expected_source_file_count = 14",
            "expected_doc_file_count = 5",
            "fallback lifecycle failure tests 4/4",
            "root_reexport_count = 0",
            "native_namespace_reexport_count = 60",
            "native root re-export 0/0",
            "native namespace re-export 60/60",
            "M4 gate `classified-and-clear`",
            "debt groups 0/0",
            "native namespace symbol groups 5/5",
            "unclassified native root symbols 0/0",
            "unclassified native namespace symbols 0/0",
            "root public native re-export locations 0/0",
            "public native namespace re-export locations 1/1",
            "app NativePlugin current call-site files: 7",
            "native loader V1/V2 implementation files 0/0",
            "`zircon_plugins` V1/V2 usage files 0/0",
            "export_build_plan V1/V2 usage 0/0",
            "unknown ABI rejection",
            "hot reload failure injection",
            "native loader test files 3/3",
            "native test namespace import files 2/2",
            "native test root import leaks 0/0",
            "runtime_06_vm_lifecycle_fallback_failure_tests_are_folder_backed",
            "runtime_06_native_loader_tests_use_isolated_plugin_native_namespace",
            "mirror_docs_guard_present = true",
            "risks = []",
            "runtime_06_plugin_surface_lifecycle_mirror_docs_match_structure_audit_counts",
        ] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should mirror Runtime 06 plugin lifecycle audit anchor `{required_anchor}`"
            );
        }
    }
}

#[test]
fn runtime_06_vm_lifecycle_fallback_failure_tests_are_folder_backed() {
    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let vm_tests_source = include_str!("../../script/vm/tests.rs");
    assert!(
        vm_tests_source.contains("mod lifecycle_failures;"),
        "Runtime 06 M1.2 fallback lifecycle test owner should be mounted by script/vm/tests.rs"
    );

    let lifecycle_tests_path = runtime_root.join("src/script/vm/tests/lifecycle_failures.rs");
    assert!(
        lifecycle_tests_path.exists(),
        "Runtime 06 M1.2 fallback lifecycle tests should live in a folder-backed script/vm test owner"
    );

    let lifecycle_tests_source =
        fs::read_to_string(&lifecycle_tests_path).unwrap_or_else(|error| {
            panic!("failed to read {}: {error}", lifecycle_tests_path.display())
        });
    for test_name in LIFECYCLE_FALLBACK_TESTS {
        assert!(
            lifecycle_tests_source.contains(test_name),
            "Runtime 06 M1.2 fallback lifecycle test `{test_name}` is missing"
        );
    }
    assert!(
        lifecycle_tests_source.contains("lifecycle:fallback"),
        "Runtime 06 M1.2 fallback lifecycle tests should not require the real ZrVM backend"
    );
}

#[test]
fn runtime_06_native_loader_tests_use_isolated_plugin_native_namespace() {
    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let plugin_extension_tests = runtime_root
        .join("src")
        .join("tests")
        .join("plugin_extensions");

    let native_loader_test_files =
        files_containing(&plugin_extension_tests, NATIVE_LOADER_TEST_PATTERNS);
    let expected_native_loader_test_files = BTreeSet::from([
        "zircon_runtime/src/tests/plugin_extensions/export_build_plan.rs".to_string(),
        "zircon_runtime/src/tests/plugin_extensions/export_build_plan_native_dynamic.rs"
            .to_string(),
        "zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs".to_string(),
    ]);
    assert_eq!(
        native_loader_test_files, expected_native_loader_test_files,
        "native loader test files should stay isolated under plugin_extensions and mirror Runtime 06 M2.2"
    );

    let namespace_import_files = files_containing(
        &plugin_extension_tests,
        &[
            "crate::plugin::native::",
            "zircon_runtime::plugin::native::",
        ],
    );
    let expected_namespace_import_files = BTreeSet::from([
        "zircon_runtime/src/tests/plugin_extensions/export_build_plan_native_dynamic.rs"
            .to_string(),
        "zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs".to_string(),
    ]);
    assert_eq!(
        namespace_import_files, expected_namespace_import_files,
        "tests that import native loader symbols should use the isolated plugin::native namespace"
    );

    let native_root_import_leaks = native_root_import_leak_files(&plugin_extension_tests);
    assert!(
        native_root_import_leaks.is_empty(),
        "native loader tests must not import NativePlugin or ZIRCON_NATIVE_PLUGIN symbols from the plugin root: {native_root_import_leaks:?}"
    );
}

fn native_plugin_namespace_reexport_symbols() -> Vec<String> {
    let source = include_str!("../../plugin/native.rs");
    let start_marker = "pub use super::native_plugin_loader::{";
    let start = source
        .find(start_marker)
        .expect("plugin::native should expose the native loader public namespace");
    let body_start = start + start_marker.len();
    let body_end = source[body_start..]
        .find("};")
        .map(|offset| body_start + offset)
        .expect("native namespace re-export block should terminate");

    source[body_start..body_end]
        .replace('\n', " ")
        .split(',')
        .map(str::trim)
        .filter(|symbol| !symbol.is_empty())
        .map(String::from)
        .collect()
}

fn native_root_import_leak_files(root: &Path) -> BTreeSet<String> {
    let workspace_root = workspace_root_from(root);
    rust_source_files(root)
        .into_iter()
        .filter(|path| {
            let source = fs::read_to_string(path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            has_native_root_import_leak(&source)
        })
        .map(|path| relative_path(&workspace_root, &path))
        .collect()
}

fn has_native_root_import_leak(source: &str) -> bool {
    if source.contains("crate::plugin::NativePlugin")
        || source.contains("crate::plugin::ZIRCON_NATIVE_PLUGIN")
        || source.contains("zircon_runtime::plugin::NativePlugin")
        || source.contains("zircon_runtime::plugin::ZIRCON_NATIVE_PLUGIN")
    {
        return true;
    }

    for marker in ["use crate::plugin::", "use zircon_runtime::plugin::"] {
        let mut search_start = 0;
        while let Some(relative_start) = source[search_start..].find(marker) {
            let statement_start = search_start + relative_start;
            let statement_tail = &source[statement_start..];
            if statement_tail.starts_with("use crate::plugin::native::")
                || statement_tail.starts_with("use zircon_runtime::plugin::native::")
            {
                search_start = statement_start + marker.len();
                continue;
            }

            let statement_end = statement_tail.find(';').unwrap_or(statement_tail.len());
            let statement = &statement_tail[..statement_end];
            if statement.contains("NativePlugin") || statement.contains("ZIRCON_NATIVE_PLUGIN") {
                return true;
            }
            search_start = statement_start + statement_end + 1;
        }
    }

    false
}

fn files_containing(root: &Path, patterns: &[&str]) -> BTreeSet<String> {
    let workspace_root = workspace_root_from(root);
    rust_source_files(root)
        .into_iter()
        .filter(|path| {
            let source = fs::read_to_string(path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            patterns.iter().any(|pattern| source.contains(pattern))
        })
        .map(|path| relative_path(&workspace_root, &path))
        .collect()
}

fn location_count(root: &Path, patterns: &[&str]) -> usize {
    rust_source_files(root)
        .into_iter()
        .map(|path| {
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            patterns
                .iter()
                .map(|pattern| source.matches(pattern).count())
                .sum::<usize>()
        })
        .sum()
}

fn rust_source_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rust_source_files(root, &mut files);
    files.sort();
    files
}

fn collect_rust_source_files(root: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()))
    {
        let entry = entry.unwrap_or_else(|error| panic!("failed to read entry: {error}"));
        let path = entry.path();
        if path.is_dir() {
            collect_rust_source_files(&path, files);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

fn workspace_root_from(path: &Path) -> PathBuf {
    path.ancestors()
        .find(|ancestor| ancestor.join("zircon_runtime").is_dir())
        .unwrap_or(path)
        .to_path_buf()
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
