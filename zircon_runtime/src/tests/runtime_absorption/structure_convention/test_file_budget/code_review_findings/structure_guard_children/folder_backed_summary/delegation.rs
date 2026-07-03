use super::super::super::super::*;
use super::*;

pub(super) fn assert_folder_backed_summary_structure_delegation_is_current() {
    let structure_child = read_runtime_src(STRUCTURE_GUARD_CHILD_OWNER);
    let parent = read_runtime_src(FOLDER_BACKED_SUMMARY_STRUCTURE_CHILD_OWNER);
    let child_tree = folder_backed_summary_structure_child_source_blob();

    assert_contains_all(
        "code-review structure guard delegates folder-backed summary child checks",
        &structure_child,
        &[
            "#[path = \"structure_guard_children/folder_backed_summary.rs\"]",
            "mod folder_backed_summary;",
            "folder_backed_summary::assert_folder_backed_summary_structure_children_are_mounted",
        ],
    );
    for backflow_guard in [
        "folder-backed summary child owner keeps code-review aggregate ownership checks",
        "folder-backed summary direct-assertions child keeps focused direct source checks",
        "folder-backed summary source inventory child keeps source-path and count checks",
    ] {
        assert!(
            !structure_child.contains(backflow_guard),
            "folder-backed summary structure guard `{backflow_guard}` should stay in {FOLDER_BACKED_SUMMARY_STRUCTURE_CHILD_OWNER}"
        );
    }
    assert_contains_all(
        "folder-backed summary structure guard parent mounts focused children",
        &parent,
        &[
            "#[path = \"folder_backed_summary/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"folder_backed_summary/direct_assertions.rs\"]",
            "mod direct_assertions;",
            "#[path = \"folder_backed_summary/source_inventory.rs\"]",
            "mod source_inventory;",
            "#[path = \"folder_backed_summary/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"folder_backed_summary/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "pub(super) fn assert_folder_backed_summary_structure_children_are_mounted",
            "folder_backed_summary_structure_child_sources",
            "folder_backed_summary_structure_child_source_blob",
        ],
    );
    assert_contains_all(
        "folder-backed summary structure guard children own delegated checks",
        &child_tree,
        &[
            "runtime_15_code_review_findings_structure_guard_folder_backed_summary_is_child_owner",
            "runtime_15_code_review_findings_structure_guard_folder_backed_summary_direct_assertions_are_child_owned",
            "runtime_15_code_review_findings_structure_guard_folder_backed_summary_source_inventory_is_child_owned",
            "runtime_15_code_review_findings_structure_guard_folder_backed_summary_children_line_budgets_are_current",
            "runtime_15_code_review_findings_structure_guard_folder_backed_summary_guard_folder_backed_status_is_current",
        ],
    );
    for (_, child_path, anchor) in FOLDER_BACKED_SUMMARY_STRUCTURE_GUARD_CHILDREN {
        assert!(
            parent.contains(child_path),
            "folder-backed summary structure parent should inventory child path {child_path}"
        );
        assert!(
            child_tree.contains(anchor),
            "folder-backed summary structure child {child_path} should own anchor {anchor}"
        );
    }
}

#[test]
fn runtime_15_code_review_findings_structure_guard_folder_backed_summary_is_child_owner() {
    assert_folder_backed_summary_structure_children_are_mounted();
}
