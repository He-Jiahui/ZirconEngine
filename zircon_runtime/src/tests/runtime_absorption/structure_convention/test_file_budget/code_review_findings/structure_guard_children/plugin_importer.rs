use super::super::super::*;
use super::*;

pub(super) fn assert_plugin_importer_dx_children_are_mounted() {
    let plugin_importer_dx_child = read_runtime_src(PLUGIN_IMPORTER_DX_CHILD_OWNER);
    let plugin_importer_dx_top_level_delegation_child =
        read_runtime_src(PLUGIN_IMPORTER_DX_TOP_LEVEL_DELEGATION_CHILD_OWNER);
    let plugin_importer_dx_top_level_child_ownership_child =
        read_runtime_src(PLUGIN_IMPORTER_DX_TOP_LEVEL_CHILD_OWNERSHIP_CHILD_OWNER);
    let plugin_importer_dx_top_level_status_mirrors_child =
        read_runtime_src(PLUGIN_IMPORTER_DX_TOP_LEVEL_STATUS_MIRRORS_CHILD_OWNER);
    let plugin_importer_dx_top_level_budgets_child =
        read_runtime_src(PLUGIN_IMPORTER_DX_TOP_LEVEL_BUDGETS_CHILD_OWNER);
    let plugin_importer_dx_source_inventory_child =
        read_runtime_src(PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_CHILD_OWNER);
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
    let plugin_importer_dx_status_docs_child =
        read_runtime_src(PLUGIN_IMPORTER_DX_STATUS_DOCS_CHILD_OWNER);
    let plugin_importer_dx_child_tree = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        plugin_importer_dx_child,
        plugin_importer_dx_top_level_delegation_child,
        plugin_importer_dx_top_level_child_ownership_child,
        plugin_importer_dx_top_level_status_mirrors_child,
        plugin_importer_dx_top_level_budgets_child,
        plugin_importer_dx_source_inventory_child
    );
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
        "plugin-importer DX structure child owner keeps plugin DX review guard ownership checks",
        &plugin_importer_dx_child_tree,
        &[
            "Runtime 15 M3 plugin-importer DX structure guard folder-backed split",
            "runtime_15_plugin_importer_dx_structure_guard_folder_backed_static_passed_cargo_deferred",
            "runtime_15_plugin_importer_dx_structure_guard_is_folder_backed",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/delegation.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/child_ownership.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/status_mirrors.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/budgets.rs",
            "#[path = \"plugin_importer_dx_child_owners/source_inventory.rs\"]",
            "mod source_inventory;",
            "#[path = \"plugin_importer_dx_child_owners/structure_assertions.rs\"]",
            "mod structure_assertions;",
            "#[path = \"plugin_importer_dx_child_owners/status_docs.rs\"]",
            "mod status_docs;",
            "fn runtime_15_code_review_findings_plugin_importer_dx_structure_guard_is_child_owner",
            "structure_assertions::assert_plugin_importer_dx_child_owners_are_folder_backed",
            "source_inventory::plugin_importer_dx_review_guard_count",
            "status_docs::assert_plugin_importer_dx_status_docs_are_synced",
        ],
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

#[test]
fn runtime_15_code_review_findings_structure_guard_plugin_importer_is_child_owned() {
    assert_plugin_importer_dx_children_are_mounted();
}
