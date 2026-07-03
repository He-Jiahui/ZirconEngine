use super::*;

#[test]
fn runtime_15_typed_error_structure_guard_is_folder_backed() {
    let parent = read_runtime_src(TYPED_ERROR_STRUCTURE_CHILD);
    let child_blob = folder_backed_child_source_blob();

    assert_contains_all(
        "typed-error structure guard parent mounts folder-backed children",
        &parent,
        &[
            "#[path = \"typed_error_child_owners/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"typed_error_child_owners/child_ownership.rs\"]",
            "mod child_ownership;",
            "#[path = \"typed_error_child_owners/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"typed_error_child_owners/source_inventory.rs\"]",
            "mod source_inventory;",
            "#[path = \"typed_error_child_owners/status_docs.rs\"]",
            "mod status_docs;",
            "#[path = \"typed_error_child_owners/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "#[path = \"typed_error_child_owners/structure_assertions.rs\"]",
            "mod structure_assertions;",
            FOLDER_BACKED_SLICE,
            FOLDER_BACKED_STATUS,
        ],
    );
    for (_, child_path, child_guard) in FOLDER_BACKED_CHILDREN {
        assert!(
            parent.contains(child_path),
            "typed-error structure guard parent should inventory child path {child_path}"
        );
        assert!(
            child_blob.contains(child_guard),
            "typed-error child source blob should contain child guard {child_guard}"
        );
    }
    assert!(
        !parent.contains(&format!("fn {GUARD}")),
        "historical typed-error structure guard should live in child_ownership child"
    );
    assert!(
        !parent.contains(&format!("fn {FOLDER_BACKED_STATUS_GUARD}")),
        "typed-error status mirror guard should live in status_mirrors child"
    );
    assert!(
        !parent.contains(&format!("fn {BUDGET_GUARD}")),
        "typed-error budget guard should live in budgets child"
    );
}
