use std::collections::BTreeSet;
use std::path::Path;

use super::inventory::{
    EXPECTED_RUNTIME_06_MIRROR_DOCS, EXPECTED_RUNTIME_06_SOURCE_FILES, V1_V2_PATTERNS,
};
use super::support::{files_containing, location_count, native_plugin_namespace_reexport_symbols};

#[test]
fn runtime_06_plugin_surface_lifecycle_mirror_docs_match_structure_audit_counts() {
    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert_eq!(
        EXPECTED_RUNTIME_06_SOURCE_FILES.len(),
        20,
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
        "../../../../../docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md"
    );
    assert!(
        plan_doc.contains("status: in_progress"),
        "Runtime 06 should stay in_progress until plugin/native/app/plugins validation closes"
    );
    assert!(
        plan_doc.contains("last_refined: 2026-08-24"),
        "Runtime 06 last_refined should cover the latest mirror-doc row"
    );

    let plugin_root_source = include_str!("../../../plugin/mod.rs");
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
        68,
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

    let runtime_06_status = concat!(
        include_str!(
            "../../../../../docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md"
        ),
        include_str!(
            "../../../../../docs/plans/zircon_runtime/runtime/06/2026-07-09-plugin-surface-and-lifecycle-output-records.md"
        )
    );
    let runtime_index_status = concat!(
        include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md"),
        include_str!(
            "../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md"
        )
    );
    let mirror_docs = [
        ("Runtime 06 status", runtime_06_status),
        ("runtime index status", runtime_index_status),
        (
            "native plugin boundary",
            include_str!("../../../../../docs/engine-architecture/native-plugin-boundary.md"),
        ),
        (
            "interface convergence",
            include_str!(
                "../../../../../docs/engine-architecture/runtime-interface-convergence.md"
            ),
        ),
        (
            "M0 review",
            include_str!(
                "../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
            ),
        ),
    ];

    for (doc_name, doc_source) in mirror_docs {
        for required_anchor in [
            "plugin_surface_lifecycle_boundary",
            "expected_source_file_count = 20",
            "expected_doc_file_count = 5",
            "fallback lifecycle failure tests 4/4",
            "root_reexport_count = 0",
            "native_namespace_reexport_count = 68",
            "native root re-export 0/0",
            "native namespace re-export 68/68",
            "M4 gate `classified-and-clear`",
            "debt groups 0/0",
            "native namespace symbol groups 6/6",
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
            "native loader test files 4/4",
            "native test namespace import files 3/3",
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
