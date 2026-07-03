use super::runtime_crates::IMPORTER_RUNTIME_CRATES;

#[test]
fn review_d13_importer_runtime_exports_use_sdk_macro() {
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
    let session_note = include_str!(
        "../../../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"
    );
    let sdk_exports =
        include_str!("../../../../../../../zircon_plugins/plugin_sdk/src/runtime_exports.rs");

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
