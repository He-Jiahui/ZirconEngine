use super::super::*;
use super::*;

pub(super) fn assert_plugin_importer_dx_structure_assertions_are_mounted() {
    let plugin_importer_dx_structure_assertions_child =
        read_runtime_src(PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD_OWNER);
    let plugin_importer_dx_review_mounts_child =
        read_runtime_src(PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_CHILD_OWNER);
    let plugin_importer_dx_delegation_child =
        read_runtime_src(PLUGIN_IMPORTER_DX_STRUCTURE_DELEGATION_CHILD_OWNER);
    let plugin_importer_dx_child_ownership_child =
        read_runtime_src(PLUGIN_IMPORTER_DX_STRUCTURE_CHILD_OWNERSHIP_CHILD_OWNER);
    let plugin_importer_d13_structure_assertions_child =
        read_runtime_src(PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_CHILD_OWNER);

    assert_contains_all(
        "plugin-importer DX structure assertions parent keeps focused guard mounts",
        &plugin_importer_dx_structure_assertions_child,
        &[
            "#[path = \"structure/review_mounts.rs\"]",
            "mod review_mounts;",
            "#[path = \"structure/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"structure/child_ownership.rs\"]",
            "mod child_ownership;",
            "#[path = \"structure/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "#[path = \"structure/d13_sdk.rs\"]",
            "mod d13_sdk;",
            "plugin_importer_dx_structure_assertion_child_sources",
            "plugin_importer_dx_structure_assertion_child_source_blob",
        ],
    );
}
