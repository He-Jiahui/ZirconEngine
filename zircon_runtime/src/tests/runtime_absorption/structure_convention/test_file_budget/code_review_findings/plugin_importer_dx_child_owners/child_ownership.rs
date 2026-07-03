use super::*;

#[test]
fn runtime_15_code_review_findings_plugin_importer_dx_structure_guard_is_child_owner() {
    let parent = read_runtime_src(STRUCTURE_GUARD_PARENT);
    let folder_backed_summary = read_runtime_src(FOLDER_BACKED_SUMMARY_CHILD);
    let child = read_runtime_src(PLUGIN_IMPORTER_DX_STRUCTURE_CHILD);
    let structure_assertions_child =
        read_runtime_src(PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD);
    let review_mounts_child = read_runtime_src(PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_CHILD);
    let delegation_child = read_runtime_src(PLUGIN_IMPORTER_DX_STRUCTURE_DELEGATION_CHILD);
    let child_ownership_child =
        read_runtime_src(PLUGIN_IMPORTER_DX_STRUCTURE_CHILD_OWNERSHIP_CHILD);
    let status_mirrors_child = read_runtime_src(PLUGIN_IMPORTER_DX_STRUCTURE_STATUS_MIRRORS_CHILD);
    let d13_sdk_child = read_runtime_src(PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_CHILD);
    let structure_assertions_child_tree = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        structure_assertions_child,
        review_mounts_child,
        delegation_child,
        child_ownership_child,
        status_mirrors_child,
        d13_sdk_child
    );

    assert_contains_all(
        "code review findings structure guard parent mounts plugin-importer DX child owner",
        &parent,
        &[
            "#[path = \"code_review_findings/plugin_importer_dx_child_owners.rs\"]",
            "mod plugin_importer_dx_child_owners;",
        ],
    );
    assert_contains_all(
        "folder-backed summary delegates plugin-importer DX aggregate checks",
        &folder_backed_summary,
        &[
            "plugin_importer_dx_child_owners::assert_plugin_importer_dx_child_owners_are_folder_backed",
            "plugin_importer_dx_child_owners::assert_plugin_importer_dx_line_budgets",
            "plugin_importer_dx_child_owners::plugin_importer_dx_review_guard_count",
        ],
    );
    assert_contains_all(
        "plugin-importer DX structure child delegates plugin DX guard structure checks",
        &child,
        &[
            "#[path = \"plugin_importer_dx_child_owners/structure_assertions.rs\"]",
            "mod structure_assertions;",
            "#[path = \"plugin_importer_dx_child_owners/source_inventory.rs\"]",
            "mod source_inventory;",
            "#[path = \"plugin_importer_dx_child_owners/status_docs.rs\"]",
            "mod status_docs;",
            "structure_assertions::assert_plugin_importer_dx_child_owners_are_folder_backed",
            "source_inventory::assert_plugin_importer_dx_line_budgets",
            "source_inventory::plugin_importer_dx_review_guard_count",
            "status_docs::assert_plugin_importer_dx_status_docs_are_synced",
        ],
    );
    assert_contains_all(
        "plugin-importer DX structure assertions parent mounts focused guard children",
        &structure_assertions_child,
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
            "review_mounts::assert_plugin_importer_dx_review_mounts_are_folder_backed",
            "d13_sdk::assert_plugin_importer_d13_sdk_child_owners_are_folder_backed",
        ],
    );
    assert_contains_all(
        "plugin-importer DX structure assertion subtree keeps review guard structure checks",
        &structure_assertions_child_tree,
        &[
            "fn runtime_15_plugin_importer_dx_structure_assertions_are_child_owner",
            "fn runtime_15_plugin_importer_dx_structure_assertions_children_are_child_owned",
            "fn runtime_15_plugin_importer_dx_structure_assertions_guard_folder_backed_status_is_current",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx.rs",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d10_bridge_call.rs",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk.rs",
            "review_d10_animation_physics_tests_use_sdk_bridge_call",
            "review_d8_runtime_registration_builder_original_evidence_paths_use_sdk_builder",
            "review_d5_editor_authoring_plugins_use_sdk_macro",
            "review_d9_editor_runtime_mirror_consumers_use_sdk_declaration",
            "runtime_15_plugin_importer_d13_sdk_structure_assertions_are_child_owner",
        ],
    );

    assert_plugin_importer_dx_child_owners_are_folder_backed();
    assert_plugin_importer_dx_line_budgets();
    assert_eq!(
        plugin_importer_dx_review_guard_count(),
        11,
        "plugin-importer DX child owners should preserve all current D1/D5/D6/D8/D9/D10/D11/D12/D13 review guards"
    );
    assert_plugin_importer_dx_status_docs_are_synced();
}
