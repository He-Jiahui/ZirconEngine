use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const EXPECTED_RUNTIME_06_SOURCE_FILES: &[&str] = &[
    "src/plugin/mod.rs",
    "src/plugin/native_plugin_loader/mod.rs",
    "src/plugin/native_plugin_loader/abi_declarations.rs",
    "src/plugin/native_plugin_loader/native_plugin_abi.rs",
    "src/plugin/native_plugin_loader/native_plugin_live_host/lifecycle.rs",
    "src/plugin/native_plugin_loader/native_plugin_live_host/hot_reload.rs",
    "src/script/vm/backend/zr_vm_project_backend/real_backend/instance.rs",
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

#[test]
fn runtime_06_plugin_surface_lifecycle_mirror_docs_match_structure_audit_counts() {
    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert_eq!(
        EXPECTED_RUNTIME_06_SOURCE_FILES.len(),
        10,
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
        plan_doc.contains("last_refined: 2026-06-14"),
        "Runtime 06 last_refined should cover the latest mirror-doc row"
    );

    let plugin_root_symbols = native_plugin_root_reexport_symbols();
    assert_eq!(
        plugin_root_symbols.len(),
        68,
        "native plugin root re-export count changed; update native_plugin_public_surface and Runtime 06 mirror docs"
    );
    for required_symbol in [
        "NativePluginAbiV1",
        "NativePluginAbiV2",
        "NativePluginAbiV3",
        "NativePluginBridgeMethodTableV3",
        "NativeHostBridgeCallScope",
        "NativePluginLoader",
        "ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V1",
        "ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V2",
    ] {
        assert!(
            plugin_root_symbols.contains(&required_symbol.to_string()),
            "native plugin public-surface mirror should still classify `{required_symbol}` while M4 debt is present"
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
        6,
        "native loader V1/V2 implementation file count changed without Runtime 06 audit sync"
    );

    let plugin_v1_v2_usage_files =
        files_containing(&workspace_root.join("zircon_plugins"), V1_V2_PATTERNS);
    let expected_plugin_v1_v2_usage = ["zircon_plugins/native_dynamic_fixture/native/src/lib.rs"]
        .into_iter()
        .map(String::from)
        .collect::<BTreeSet<_>>();
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
            "expected_source_file_count = 10",
            "expected_doc_file_count = 5",
            "native root re-export 70/70",
            "M4 gate `migration-debt-present`",
            "debt groups 5/5",
            "unclassified native symbols 0/0",
            "public native re-export locations 1/1",
            "app NativePlugin current call-site files: 7",
            "native loader V1/V2 implementation files 6/6",
            "`zircon_plugins` V1/V2 usage files 1/1",
            "export_build_plan V1/V2 usage 0/0",
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

fn native_plugin_root_reexport_symbols() -> Vec<String> {
    let source = include_str!("../../plugin/mod.rs");
    let start_marker = "pub use native_plugin_loader::{";
    let start = source
        .find(start_marker)
        .expect("plugin root should still expose the native loader re-export block while M4 debt is present");
    let body_start = start + start_marker.len();
    let body_end = source[body_start..]
        .find("};")
        .map(|offset| body_start + offset)
        .expect("native loader re-export block should terminate");

    source[body_start..body_end]
        .replace('\n', " ")
        .split(',')
        .map(str::trim)
        .filter(|symbol| !symbol.is_empty())
        .map(String::from)
        .collect()
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
