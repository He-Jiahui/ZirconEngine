use super::*;

#[test]
fn runtime_15_plugin_importer_dx_structure_guard_is_folder_backed() {
    let parent = read_runtime_src(PLUGIN_IMPORTER_DX_STRUCTURE_CHILD);
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
            FOLDER_BACKED_SLICE,
            FOLDER_BACKED_STATUS,
        ],
    );
    for (_, child_path, child_guard) in FOLDER_BACKED_CHILDREN {
        assert!(
            parent.contains(child_path),
            "plugin-importer DX structure guard parent should inventory child path {child_path}"
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
