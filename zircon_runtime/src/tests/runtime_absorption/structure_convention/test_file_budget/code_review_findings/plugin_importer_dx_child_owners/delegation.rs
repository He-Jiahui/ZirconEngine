use super::*;

#[test]
fn runtime_15_plugin_importer_dx_structure_guard_is_folder_backed() {
    let parent = read_runtime_src(PLUGIN_IMPORTER_DX_STRUCTURE_CHILD);
    let child_inventory = read_runtime_src(PLUGIN_IMPORTER_DX_ROOT_CHILD_ROWS_CHILD);
    let status_inventory = read_runtime_src(PLUGIN_IMPORTER_DX_ROOT_STATUSES_CHILD);
    let child_blob = folder_backed_child_source_blob();

    assert_contains_all(
        "plugin-importer DX structure guard parent mounts folder-backed children",
        &parent,
        &[
            "#[path = \"plugin_importer_dx_child_owners/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"plugin_importer_dx_child_owners/child_ownership.rs\"]",
            "mod child_ownership;",
            "#[path = \"plugin_importer_dx_child_owners/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"plugin_importer_dx_child_owners/source_inventory.rs\"]",
            "mod source_inventory;",
            "#[path = \"plugin_importer_dx_child_owners/status_docs.rs\"]",
            "mod status_docs;",
            "#[path = \"plugin_importer_dx_child_owners/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "#[path = \"plugin_importer_dx_child_owners/structure_assertions.rs\"]",
            "mod structure_assertions;",
            "#[path = \"plugin_importer_dx_child_owners/root_paths.rs\"]",
            "mod root_paths;",
            "#[path = \"plugin_importer_dx_child_owners/root_statuses.rs\"]",
            "mod root_statuses;",
            "#[path = \"plugin_importer_dx_child_owners/root_child_rows.rs\"]",
            "mod root_child_rows;",
            "#[path = \"plugin_importer_dx_child_owners/root_sources.rs\"]",
            "mod root_sources;",
            "#[path = \"plugin_importer_dx_child_owners/root_inventory.rs\"]",
            "mod root_inventory;",
        ],
    );
    assert_contains_all(
        "plugin-importer DX root status child preserves folder-backed status anchors",
        &status_inventory,
        &[FOLDER_BACKED_SLICE, FOLDER_BACKED_STATUS],
    );
    for (_, child_path, child_guard) in FOLDER_BACKED_CHILDREN {
        assert!(
            child_inventory.contains(child_path),
            "plugin-importer DX root child inventory should include child path {child_path}"
        );
        assert!(
            child_blob.contains(child_guard),
            "plugin-importer DX child source blob should contain child guard {child_guard}"
        );
    }
    assert!(
        !parent.contains(&format!("fn {GUARD}")),
        "historical plugin-importer DX structure guard should live in child_ownership child"
    );
    assert!(
        !parent.contains(&format!("fn {FOLDER_BACKED_STATUS_GUARD}")),
        "plugin-importer DX status mirror guard should live in status_mirrors child"
    );
    assert!(
        !parent.contains(&format!("fn {BUDGET_GUARD}")),
        "plugin-importer DX budget guard should live in budgets child"
    );
}
