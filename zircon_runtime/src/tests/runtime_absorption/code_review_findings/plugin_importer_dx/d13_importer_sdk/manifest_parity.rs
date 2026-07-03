use super::runtime_crates::IMPORTER_RUNTIME_CRATES;

#[test]
fn review_d13_importer_manifest_parity_guard_lives_in_sdk_builder() {
    let review_findings =
        include_str!("../../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention =
        include_str!("../../../../../../../docs/plans/engine-code-structure-convention.md");
    let plugins_12 = include_str!(
        "../../../../../../../docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md"
    );
    let plugin_sdk_doc = include_str!("../../../../../../../docs/zircon_plugins/plugin-sdk.md");
    let importer_doc = include_str!(
        "../../../../../../../docs/zircon_plugins/asset_importers/runtime-skeletons.md"
    );
    let runtime_15 = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let module_convention =
        include_str!("../../../../../../../docs/zircon_runtime/structure/module-convention.md");
    let status_rows = include_str!(
        "../../../plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs"
    );
    let session_note = include_str!(
        "../../../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"
    );
    let sdk_importer_manifest = include_str!(
        "../../../../../../../zircon_plugins/plugin_sdk/src/manifest/importer_runtime.rs"
    );
    let sdk_manifest_tests =
        include_str!("../../../../../../../zircon_plugins/plugin_sdk/src/manifest/tests.rs");
    let sdk_lib = include_str!("../../../../../../../zircon_plugins/plugin_sdk/src/lib.rs");
    let sdk_prelude = include_str!("../../../../../../../zircon_plugins/plugin_sdk/src/prelude.rs");

    for required in [
        "importer_runtime_supported_targets",
        "importer_runtime_supported_platforms",
        "ImporterRuntimeManifestBuilder",
        "NATIVE_ABI_VERSION_V3",
        "NATIVE_DESCRIPTOR_SYMBOL_V3",
    ] {
        assert!(
            sdk_importer_manifest.contains(required)
                && sdk_lib.contains(required)
                && sdk_prelude.contains(required),
            "D13 importer parity helpers should be owned by the SDK and exported through lib/prelude `{required}`"
        );
    }

    for required in [
        "fn importer_runtime_manifest_builder_projects_dist_and_importer_manifest",
        "fn importer_runtime_manifest_builder_keeps_targets_platforms_modules_and_distribution_in_parity",
        "importer_runtime_supported_targets()",
        "importer_runtime_supported_platforms()",
        "runtime_module_manifest()",
        "dist_module_manifest()",
        "build_package_manifest",
        "NATIVE_ABI_VERSION_V3",
        "NATIVE_DESCRIPTOR_SYMBOL_V3",
    ] {
        assert!(
            sdk_manifest_tests.contains(required),
            "plugin SDK manifest tests should cover importer parity anchor `{required}`"
        );
    }

    for (label, source, _) in IMPORTER_RUNTIME_CRATES {
        assert!(
            source.contains("importer_manifest_builder().runtime_module_manifest()")
                && source.contains("importer_manifest_builder().dist_module_manifest()")
                && source.contains(".with_asset_importers(asset_importer_descriptors())")
                && source.contains(".build_package_manifest(descriptor)"),
            "{label} should keep runtime/dist/package manifest projection on ImporterRuntimeManifestBuilder"
        );
    }

    let d13_row = review_findings
        .lines()
        .find(|line| line.starts_with("| D13 |"))
        .expect("D13 review finding row should exist");
    for required in [
        "d13_importer_manifest_parity_guard_static_passed_cargo_deferred",
        "d13_importer_top_row_closed_status_static_passed_cargo_deferred",
        "review_d13_importer_manifest_parity_guard_lives_in_sdk_builder",
        "importer_runtime_manifest_builder_keeps_targets_platforms_modules_and_distribution_in_parity",
        "NATIVE_ABI_VERSION_V3",
        "NATIVE_DESCRIPTOR_SYMBOL_V3",
    ] {
        assert!(
            d13_row.contains(required),
            "D13 row should record importer parity guard anchor `{required}`"
        );
    }
    assert!(
        d13_row.ends_with("| M3 / closed |"),
        "D13 row should mark the current importer SDK convergence chain closed"
    );

    for (doc_label, doc) in [
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("Plugins 12 plan", plugins_12),
        ("plugin SDK doc", plugin_sdk_doc),
        ("asset importer doc", importer_doc),
        ("Runtime 15 plan", runtime_15),
        ("Runtime index", runtime_index),
        ("module convention", module_convention),
        ("status-output row data", status_rows),
        ("session note", session_note),
    ] {
        for required in [
            "D13 importer manifest parity guard",
            "d13_importer_manifest_parity_guard_static_passed_cargo_deferred",
            "review_d13_importer_manifest_parity_guard_lives_in_sdk_builder",
            "importer_runtime_manifest_builder_keeps_targets_platforms_modules_and_distribution_in_parity",
        ] {
            assert!(
                doc.contains(required),
                "{doc_label} should record D13 importer manifest parity guard anchor `{required}`"
            );
        }
    }
}
