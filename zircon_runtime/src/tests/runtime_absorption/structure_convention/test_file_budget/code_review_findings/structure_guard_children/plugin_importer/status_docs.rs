use super::super::*;
use super::*;

pub(super) fn assert_plugin_importer_dx_status_docs_are_mounted() {
    let plugin_importer_dx_status_docs_child =
        read_runtime_src(PLUGIN_IMPORTER_DX_STATUS_DOCS_CHILD_OWNER);

    assert_contains_all(
        "plugin-importer DX status-doc child keeps docs/status checks",
        &plugin_importer_dx_status_docs_child,
        &[
            "fn runtime_15_plugin_importer_dx_status_docs_are_child_owner",
            "Runtime 15 M3 plugin-importer DX status-doc guard child-owner split",
            "runtime_15_plugin_importer_dx_status_docs_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/status_docs.rs",
            "runtime_15_plugin_importer_dx_source_inventory_is_child_owner",
        ],
    );
}
