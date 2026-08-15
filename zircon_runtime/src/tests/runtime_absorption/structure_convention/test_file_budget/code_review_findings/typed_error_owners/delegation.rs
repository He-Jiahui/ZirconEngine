use super::*;

#[test]
fn runtime_15_typed_error_structure_guard_is_folder_backed() {
    let parent = read_runtime_src(TYPED_ERROR_STRUCTURE_CHILD);
    let child_inventory = read_runtime_src(TYPED_ERROR_ROOT_CHILD_ROWS_CHILD);
    let child_blob = folder_backed_child_source_blob();

    assert_contains_all(
        "typed-error structure guard parent mounts folder-backed children",
        &parent,
        &[
            "#[path = \"typed_error_owners/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"typed_error_owners/child_ownership.rs\"]",
            "mod child_ownership;",
            "#[path = \"typed_error_owners/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"typed_error_owners/source_inventory.rs\"]",
            "mod source_inventory;",
            "#[path = \"typed_error_owners/structure_assertions.rs\"]",
            "mod structure_assertions;",
            "#[path = \"typed_error_owners/root_paths.rs\"]",
            "mod root_paths;",
            "#[path = \"typed_error_owners/root_child_rows.rs\"]",
            "mod root_child_rows;",
            "#[path = \"typed_error_owners/root_sources.rs\"]",
            "mod root_sources;",
            "#[path = \"typed_error_owners/root_inventory.rs\"]",
            "mod root_inventory;",
        ],
    );
    for (_, child_path, child_guard) in FOLDER_BACKED_CHILDREN {
        assert!(
            child_inventory.contains(child_path),
            "typed-error root child inventory should include child path {child_path}"
        );
        assert!(
            child_blob.contains(child_guard),
            "typed-error child source blob should contain child guard {child_guard}"
        );
    }
    assert!(
        !parent.contains(
            "fn runtime_15_code_review_findings_typed_error_structure_guard_is_child_owner"
        ),
        "historical typed-error structure guard should live in child_ownership child"
    );
}
