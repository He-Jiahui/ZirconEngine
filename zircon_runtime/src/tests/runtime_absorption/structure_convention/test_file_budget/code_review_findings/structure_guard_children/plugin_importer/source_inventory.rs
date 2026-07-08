use super::super::*;
use super::*;

pub(super) fn assert_plugin_importer_dx_source_inventory_is_mounted() {
    let plugin_importer_dx_source_inventory_child = format!(
        "{}\n{}",
        read_runtime_src(PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_CHILD_OWNER),
        super::super::super::plugin_importer_dx_child_owners::plugin_importer_dx_source_inventory_child_source_blob()
    );

    assert_contains_all(
        "plugin-importer DX source inventory child keeps DX source-path checks",
        &plugin_importer_dx_source_inventory_child,
        &[
            "fn runtime_15_plugin_importer_dx_source_inventory_is_child_owner",
            "const PLUGIN_IMPORTER_DX_SOURCE_PATHS",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx.rs",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d10_bridge_call.rs",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk.rs",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk/manifest_parity.rs",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk/runtime_crates.rs",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk/runtime_exports.rs",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk/runtime_manifests.rs",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d5_editor_authoring.rs",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d8_registration_builder.rs",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d9_editor_runtime_mirror.rs",
            "plugin_importer_dx_review_guard_count",
        ],
    );
}
