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
    let plugin_importer_dx_status_mirrors_child =
        read_runtime_src(PLUGIN_IMPORTER_DX_STRUCTURE_STATUS_MIRRORS_CHILD_OWNER);
    let plugin_importer_d13_structure_assertions_child =
        read_runtime_src(PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_CHILD_OWNER);
    let plugin_importer_dx_structure_assertions_child_tree = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        plugin_importer_dx_structure_assertions_child,
        plugin_importer_dx_review_mounts_child,
        plugin_importer_dx_delegation_child,
        plugin_importer_dx_child_ownership_child,
        plugin_importer_dx_status_mirrors_child,
        plugin_importer_d13_structure_assertions_child
    );

    assert_contains_all(
        "plugin-importer DX structure assertions parent keeps focused guard mounts",
        &plugin_importer_dx_structure_assertions_child,
        &[
            "#[path = \"structure_assertions/review_mounts.rs\"]",
            "mod review_mounts;",
            "#[path = \"structure_assertions/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"structure_assertions/child_ownership.rs\"]",
            "mod child_ownership;",
            "#[path = \"structure_assertions/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "#[path = \"structure_assertions/d13_sdk.rs\"]",
            "mod d13_sdk;",
            "plugin_importer_dx_structure_assertion_child_sources",
            "plugin_importer_dx_structure_assertion_child_source_blob",
        ],
    );
    assert_contains_all(
        "plugin-importer DX structure assertions subtree keeps DX review guard ownership checks",
        &plugin_importer_dx_structure_assertions_child_tree,
        &[
            "pub(super) fn assert_plugin_importer_dx_child_owners_are_folder_backed",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx.rs",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d10_bridge_call.rs",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk.rs",
            "review_d10_animation_physics_tests_use_sdk_bridge_call",
            "review_d8_runtime_registration_builder_original_evidence_paths_use_sdk_builder",
            "review_d9_editor_runtime_mirror_consumers_use_sdk_declaration",
            "d13_sdk::assert_plugin_importer_d13_sdk_child_owners_are_folder_backed",
            "runtime_15_plugin_importer_dx_structure_assertions_children_are_child_owned",
            "runtime_15_plugin_importer_dx_structure_assertions_guard_folder_backed_status_is_current",
        ],
    );
}
