use super::super::super::super::*;
use super::*;

fn typed_error_top_level_child_tree() -> String {
    [
        read_runtime_src(TYPED_ERROR_TOP_LEVEL_DELEGATION_CHILD_OWNER),
        read_runtime_src(TYPED_ERROR_TOP_LEVEL_CHILD_OWNERSHIP_CHILD_OWNER),
        read_runtime_src(TYPED_ERROR_SOURCE_INVENTORY_CHILD_OWNER),
        read_runtime_src(TYPED_ERROR_STATUS_DOCS_CHILD_OWNER),
        read_runtime_src(TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD_OWNER),
        read_runtime_src(TYPED_ERROR_TOP_LEVEL_STATUS_MIRRORS_CHILD_OWNER),
        read_runtime_src(TYPED_ERROR_TOP_LEVEL_BUDGETS_CHILD_OWNER),
    ]
    .join("\n")
}

pub(super) fn assert_typed_error_top_level_checks_are_current() {
    let parent = read_runtime_src(STRUCTURE_GUARD_TYPED_ERROR_CHILD_OWNER);
    let typed_error_child = read_runtime_src(TYPED_ERROR_CHILD_OWNER);
    let child_tree = typed_error_top_level_child_tree();

    assert_contains_all(
        "typed-error structure child owner keeps route inventory and helper delegation",
        &typed_error_child,
        &[
            "#[path = \"typed_error_child_owners/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"typed_error_child_owners/child_ownership.rs\"]",
            "mod child_ownership;",
            "#[path = \"typed_error_child_owners/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "#[path = \"typed_error_child_owners/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"typed_error_child_owners/source_inventory.rs\"]",
            "mod source_inventory;",
            "#[path = \"typed_error_child_owners/status_docs.rs\"]",
            "mod status_docs;",
            "#[path = \"typed_error_child_owners/structure_assertions.rs\"]",
            "mod structure_assertions;",
            "structure_assertions::assert_typed_error_child_owners_are_folder_backed",
            "source_inventory::typed_error_children_source",
            "source_inventory::assert_typed_error_line_budgets",
            "source_inventory::typed_error_review_guard_count",
            "status_docs::assert_typed_error_status_docs_are_synced",
            TYPED_ERROR_FOLDER_BACKED_SLICE,
            TYPED_ERROR_FOLDER_BACKED_STATUS,
        ],
    );
    assert!(
        !typed_error_child.contains(
            "fn runtime_15_code_review_findings_typed_error_structure_guard_is_child_owner"
        ),
        "typed-error historical structure guard should stay in child_ownership child"
    );
    assert_contains_all(
        "typed-error top-level folder-backed children own actual guard bodies",
        &child_tree,
        &[
            "fn runtime_15_typed_error_structure_guard_is_folder_backed",
            "fn runtime_15_code_review_findings_typed_error_structure_guard_is_child_owner",
            "fn runtime_15_typed_error_structure_guard_folder_backed_status_is_current",
            "fn runtime_15_typed_error_structure_guard_budgets_are_focused",
            "runtime_15_typed_error_source_inventory_is_child_owner",
            "runtime_15_typed_error_status_docs_are_folder_backed",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/mod.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader.rs",
            "review_f5_texture_loader_uses_typed_error",
            "review_f7_asset_artifact_errors_use_asset_import_error_sources",
        ],
    );
    assert!(
        !parent.contains("typed-error top-level folder-backed children own actual guard bodies"),
        "typed-error top-level guard details should stay in {STRUCTURE_GUARD_TYPED_ERROR_TOP_LEVEL_CHILD_OWNER}"
    );
}

#[test]
fn runtime_15_code_review_findings_structure_guard_typed_error_top_level_checks_are_child_owned() {
    assert_typed_error_top_level_checks_are_current();
}
