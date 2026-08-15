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
    let d13_sdk_child = read_runtime_src(PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_CHILD);

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
            "#[path = \"plugin_importer_dx_owners/structure_assertions.rs\"]",
            "mod structure_assertions;",
            "#[path = \"plugin_importer_dx_owners/source_inventory.rs\"]",
            "mod source_inventory;",
            "structure_assertions::assert_plugin_importer_dx_child_owners_are_folder_backed",
            "source_inventory::assert_plugin_importer_dx_line_budgets",
            "source_inventory::plugin_importer_dx_review_guard_count",
        ],
    );
    assert_contains_all(
        "plugin-importer DX structure assertions parent mounts focused guard children",
        &structure_assertions_child,
        &[
            "#[path = \"structure/review_mounts.rs\"]",
            "mod review_mounts;",
            "#[path = \"structure/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"structure/child_ownership.rs\"]",
            "mod child_ownership;",
            "#[path = \"structure/d13_sdk.rs\"]",
            "mod d13_sdk;",
            "review_mounts::assert_plugin_importer_dx_review_mounts_are_folder_backed",
            "d13_sdk::assert_plugin_importer_d13_sdk_child_owners_are_folder_backed",
        ],
    );

    assert_plugin_importer_dx_child_owners_are_folder_backed();
    assert_plugin_importer_dx_line_budgets();
    assert_eq!(
        plugin_importer_dx_review_guard_count(),
        11,
        "plugin-importer DX child owners should preserve all current D1/D5/D6/D8/D9/D10/D11/D12/D13 review guards"
    );
}
