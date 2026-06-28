const IMPORTER_RUNTIME_CRATES: &[(&str, &str, &str)] = &[
    (
        "asset_importers/audio",
        include_str!(
            "../../../../../../zircon_plugins/asset_importers/audio/runtime/src/plugin.rs"
        ),
        include_str!("../../../../../../zircon_plugins/asset_importers/audio/runtime/Cargo.toml"),
    ),
    (
        "asset_importers/data",
        include_str!("../../../../../../zircon_plugins/asset_importers/data/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/asset_importers/data/runtime/Cargo.toml"),
    ),
    (
        "asset_importers/model",
        include_str!(
            "../../../../../../zircon_plugins/asset_importers/model/runtime/src/plugin.rs"
        ),
        include_str!("../../../../../../zircon_plugins/asset_importers/model/runtime/Cargo.toml"),
    ),
    (
        "asset_importers/shader",
        include_str!(
            "../../../../../../zircon_plugins/asset_importers/shader/runtime/src/plugin.rs"
        ),
        include_str!("../../../../../../zircon_plugins/asset_importers/shader/runtime/Cargo.toml"),
    ),
    (
        "asset_importers/texture",
        include_str!(
            "../../../../../../zircon_plugins/asset_importers/texture/runtime/src/plugin.rs"
        ),
        include_str!("../../../../../../zircon_plugins/asset_importers/texture/runtime/Cargo.toml"),
    ),
    (
        "audio_importer",
        include_str!("../../../../../../zircon_plugins/audio_importer/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/audio_importer/runtime/Cargo.toml"),
    ),
    (
        "gltf_importer",
        include_str!("../../../../../../zircon_plugins/gltf_importer/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/gltf_importer/runtime/Cargo.toml"),
    ),
    (
        "obj_importer",
        include_str!("../../../../../../zircon_plugins/obj_importer/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/obj_importer/runtime/Cargo.toml"),
    ),
    (
        "opus_importer",
        include_str!("../../../../../../zircon_plugins/opus_importer/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/opus_importer/runtime/Cargo.toml"),
    ),
    (
        "shader_wgsl_importer",
        include_str!("../../../../../../zircon_plugins/shader_wgsl_importer/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/shader_wgsl_importer/runtime/Cargo.toml"),
    ),
    (
        "texture_importer",
        include_str!("../../../../../../zircon_plugins/texture_importer/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/texture_importer/runtime/Cargo.toml"),
    ),
    (
        "ui_document_importer",
        include_str!("../../../../../../zircon_plugins/ui_document_importer/runtime/src/plugin.rs"),
        include_str!("../../../../../../zircon_plugins/ui_document_importer/runtime/Cargo.toml"),
    ),
];

#[test]
fn review_d13_importer_runtime_exports_use_sdk_macro() {
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let plugins_12 = include_str!(
        "../../../../../../docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md"
    );
    let plugin_sdk_doc = include_str!("../../../../../../docs/zircon_plugins/plugin-sdk.md");
    let importer_doc =
        include_str!("../../../../../../docs/zircon_plugins/asset_importers/runtime-skeletons.md");
    let session_note = include_str!(
        "../../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"
    );
    let sdk_exports =
        include_str!("../../../../../../zircon_plugins/plugin_sdk/src/runtime_exports.rs");

    assert_eq!(
        IMPORTER_RUNTIME_CRATES.len(),
        12,
        "D13 importer export guard should cover every first-party importer runtime owner"
    );

    for (label, source, cargo_toml) in IMPORTER_RUNTIME_CRATES {
        assert!(
            source.contains("impl RuntimePlugin for"),
            "{label} should keep RuntimePlugin as the runtime descriptor owner"
        );
        assert!(
            source.contains("fn package_manifest(&self) -> PluginPackageManifest"),
            "{label} should keep importer-specific package manifest projection in RuntimePlugin"
        );
        assert!(
            source.contains("zircon_plugin_sdk::runtime_plugin_exports!("),
            "{label} should export runtime helpers through the plugin SDK macro"
        );
        assert!(
            cargo_toml
                .contains("zircon_plugin_sdk = { workspace = true, features = [\"runtime\"] }"),
            "{label} should inherit the SDK runtime dependency from the plugin workspace"
        );

        for stale in [
            "pub fn runtime_plugin()",
            "pub fn package_manifest()",
            "pub fn runtime_selection()",
            "pub fn plugin_registration()",
            "ProjectPluginSelection",
            "RuntimePluginRegistrationReport",
            "RuntimePlugin::project_selection(&runtime_plugin())",
            "RuntimePluginRegistrationReport::from_plugin(&runtime_plugin())",
        ] {
            assert!(
                !source.contains(stale),
                "{label} should not keep hand-written importer runtime export helper `{stale}`"
            );
        }
    }

    for required in [
        "macro_rules! runtime_plugin_exports",
        "pub fn runtime_selection() -> zircon_runtime::plugin::ProjectPluginSelection",
        "pub fn plugin_registration() -> zircon_runtime::plugin::RuntimePluginRegistrationReport",
        "zircon_runtime::plugin::RuntimePlugin::project_selection(&runtime_plugin())",
        "zircon_runtime::plugin::RuntimePluginRegistrationReport::from_plugin(&runtime_plugin())",
    ] {
        assert!(
            sdk_exports.contains(required),
            "plugin SDK runtime export macro should own generated helper `{required}`"
        );
    }

    let d13_row = review_findings
        .lines()
        .find(|line| line.starts_with("| D13 |"))
        .expect("D13 review finding row should exist");
    for required in [
        "selection/helper 导出已由 plugin SDK macro 收敛",
        "12/12 importer runtime plugin.rs owner",
        "zircon_plugin_sdk::runtime_plugin_exports!",
        "d13_importer_runtime_export_macro_convergence_static_passed_cargo_deferred",
    ] {
        assert!(
            d13_row.contains(required),
            "D13 row should record importer runtime export macro convergence anchor `{required}`"
        );
    }
    for stale in [
        "`runtime_selection` 手填 8 字段",
        "asset_importers/model/runtime/src/registration.rs",
    ] {
        assert!(
            !d13_row.contains(stale),
            "D13 row should not keep stale importer export evidence `{stale}`"
        );
    }

    for (doc_label, doc) in [
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("Plugins 12 plan", plugins_12),
        ("plugin SDK doc", plugin_sdk_doc),
        ("asset importer doc", importer_doc),
        ("session note", session_note),
    ] {
        for required in [
            "D13 importer runtime export macro convergence",
            "d13_importer_runtime_export_macro_convergence_static_passed_cargo_deferred",
            "review_d13_importer_runtime_exports_use_sdk_macro",
        ] {
            assert!(
                doc.contains(required),
                "{doc_label} should record D13 importer export convergence anchor `{required}`"
            );
        }
    }
}

#[test]
fn review_d13_importer_runtime_manifests_use_sdk_builder() {
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let plugins_12 = include_str!(
        "../../../../../../docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md"
    );
    let plugin_sdk_doc = include_str!("../../../../../../docs/zircon_plugins/plugin-sdk.md");
    let importer_doc =
        include_str!("../../../../../../docs/zircon_plugins/asset_importers/runtime-skeletons.md");
    let runtime_15 = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let module_convention =
        include_str!("../../../../../../docs/zircon_runtime/structure/module-convention.md");
    let session_note = include_str!(
        "../../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"
    );
    let sdk_importer_manifest = include_str!(
        "../../../../../../zircon_plugins/plugin_sdk/src/manifest/importer_runtime.rs"
    );

    assert_eq!(
        IMPORTER_RUNTIME_CRATES.len(),
        12,
        "D13 importer manifest builder guard should cover every first-party importer runtime owner"
    );

    for required in [
        "pub struct ImporterRuntimeManifestBuilder",
        "pub fn importer_runtime_supported_targets() -> [RuntimeTargetMode; 2]",
        "pub fn importer_runtime_supported_platforms() -> [ExportTargetPlatform; 3]",
        "pub const NATIVE_DESCRIPTOR_SYMBOL_V3",
        "pub const NATIVE_ABI_VERSION_V3",
        "PluginModuleManifest::runtime",
        "PluginModuleManifest::native",
        "PluginDistributionManifest",
        "build_package_manifest",
    ] {
        assert!(
            sdk_importer_manifest.contains(required),
            "plugin SDK importer manifest helper should own `{required}`"
        );
    }

    for (label, source, _) in IMPORTER_RUNTIME_CRATES {
        for required in [
            "ImporterRuntimeManifestBuilder",
            "importer_runtime_supported_targets()",
            "importer_runtime_supported_platforms()",
            "importer_manifest_builder().runtime_module_manifest()",
            "importer_manifest_builder().dist_module_manifest()",
            ".build_package_manifest(descriptor)",
        ] {
            assert!(
                source.contains(required),
                "{label} should route importer manifest boilerplate through SDK builder `{required}`"
            );
        }

        for stale in [
            "PluginDistributionManifest",
            "ExportPackagingStrategy",
            "NATIVE_DESCRIPTOR_SYMBOL_V3",
            "NATIVE_ABI_VERSION_V3",
            "DIST_ENGINE_COMPAT",
            "PluginModuleManifest::runtime",
            "PluginModuleManifest::native",
            "with_distribution(",
            "default_packaging.push",
        ] {
            assert!(
                !source.contains(stale),
                "{label} should not keep hand-written importer manifest boilerplate `{stale}`"
            );
        }
    }

    let d13_row = review_findings
        .lines()
        .find(|line| line.starts_with("| D13 |"))
        .expect("D13 review finding row should exist");
    for required in [
        "selection/helper 导出与 targets/platforms/module/dist-module manifest 样板已由 plugin SDK 收敛",
        "ImporterRuntimeManifestBuilder",
        "d13_importer_runtime_manifest_builder_convergence_static_passed_cargo_deferred",
        "review_d13_importer_runtime_manifests_use_sdk_builder",
    ] {
        assert!(
            d13_row.contains(required),
            "D13 row should record importer runtime manifest builder convergence anchor `{required}`"
        );
    }
    assert!(
        !d13_row.contains("targets/platforms/module/dist-module 样板仍是后续 builder/parity"),
        "D13 row should not keep stale importer manifest builder follow-up text"
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
        ("session note", session_note),
    ] {
        for required in [
            "D13 importer runtime manifest builder convergence",
            "d13_importer_runtime_manifest_builder_convergence_static_passed_cargo_deferred",
            "review_d13_importer_runtime_manifests_use_sdk_builder",
            "ImporterRuntimeManifestBuilder",
        ] {
            assert!(
                doc.contains(required),
                "{doc_label} should record D13 importer manifest builder convergence anchor `{required}`"
            );
        }
    }
}

#[test]
fn review_d13_importer_manifest_parity_guard_lives_in_sdk_builder() {
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let plugins_12 = include_str!(
        "../../../../../../docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md"
    );
    let plugin_sdk_doc = include_str!("../../../../../../docs/zircon_plugins/plugin-sdk.md");
    let importer_doc =
        include_str!("../../../../../../docs/zircon_plugins/asset_importers/runtime-skeletons.md");
    let runtime_15 = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let module_convention =
        include_str!("../../../../../../docs/zircon_runtime/structure/module-convention.md");
    let status_rows = include_str!(
        "../../plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs"
    );
    let session_note = include_str!(
        "../../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"
    );
    let sdk_importer_manifest = include_str!(
        "../../../../../../zircon_plugins/plugin_sdk/src/manifest/importer_runtime.rs"
    );
    let sdk_manifest_tests =
        include_str!("../../../../../../zircon_plugins/plugin_sdk/src/manifest/tests.rs");
    let sdk_lib = include_str!("../../../../../../zircon_plugins/plugin_sdk/src/lib.rs");
    let sdk_prelude = include_str!("../../../../../../zircon_plugins/plugin_sdk/src/prelude.rs");

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
