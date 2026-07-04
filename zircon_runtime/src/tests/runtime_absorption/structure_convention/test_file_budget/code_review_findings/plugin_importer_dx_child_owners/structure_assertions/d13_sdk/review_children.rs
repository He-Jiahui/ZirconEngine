use super::super::super::super::super::*;
use super::*;

pub(super) fn assert_plugin_importer_d13_sdk_review_children_are_child_owned(
    sources: &PluginImporterD13SdkStructureSources,
) {
    assert_contains_all(
        "plugin importer DX D13 runtime-crates child owns importer runtime source inventory",
        &sources.plugin_importer_dx_d13_runtime_crates,
        &[
            "pub(super) const IMPORTER_RUNTIME_CRATES",
            "asset_importers/audio",
            "asset_importers/data",
            "asset_importers/model",
            "asset_importers/shader",
            "asset_importers/texture",
            "ui_document_importer",
        ],
    );
    assert_contains_all(
        "plugin importer DX D13 runtime-exports child owns export macro review guard",
        &sources.plugin_importer_dx_d13_runtime_exports,
        &[
            "fn review_d13_importer_runtime_exports_use_sdk_macro",
            "d13_importer_runtime_export_macro_convergence_static_passed_cargo_deferred",
            "zircon_plugin_sdk::runtime_plugin_exports!",
        ],
    );
    assert_contains_all(
        "plugin importer DX D13 runtime-manifests child owns manifest builder review guard",
        &sources.plugin_importer_dx_d13_runtime_manifests,
        &[
            "fn review_d13_importer_runtime_manifests_use_sdk_builder",
            "d13_importer_runtime_manifest_builder_convergence_static_passed_cargo_deferred",
            "ImporterRuntimeManifestBuilder",
        ],
    );
    assert_contains_all(
        "plugin importer DX D13 manifest-parity child owns parity review guard",
        &sources.plugin_importer_dx_d13_manifest_parity,
        &[
            "fn review_d13_importer_manifest_parity_guard_lives_in_sdk_builder",
            "d13_importer_manifest_parity_guard_static_passed_cargo_deferred",
            "d13_importer_top_row_closed_status_static_passed_cargo_deferred",
            "importer_runtime_manifest_builder_keeps_targets_platforms_modules_and_distribution_in_parity",
            "NATIVE_ABI_VERSION_V3",
            "NATIVE_DESCRIPTOR_SYMBOL_V3",
        ],
    );
}
