use super::super::super::*;
use super::*;

#[test]
fn runtime_15_code_review_findings_folder_backed_summary_is_child_owner() {
    let parent = read_runtime_src(STRUCTURE_GUARD_PARENT);
    let child = read_runtime_src(FOLDER_BACKED_SUMMARY_CHILD);
    let child_tree = folder_backed_summary_guard_child_source_blob();

    assert_contains_all(
        "code review findings structure guard delegates folder-backed summary",
        &parent,
        &[
            "#[path = \"code_review_findings/folder_backed_summary.rs\"]",
            "mod folder_backed_summary;",
            "folder_backed_summary::assert_code_review_findings_tests_are_folder_backed",
        ],
    );
    for backflow_guard in [
        concat!("let ", "f8_api_convergence ="),
        concat!("let ", "p0_robustness ="),
        concat!("let ", "render_structure ="),
        concat!("let ", "child_test_total ="),
        "code review findings children should preserve all 78 review guards",
    ] {
        assert!(
            !parent.contains(backflow_guard),
            "folder-backed summary guard `{backflow_guard}` should stay in {FOLDER_BACKED_SUMMARY_CHILD}"
        );
    }
    assert_contains_all(
        "folder-backed summary parent delegates focused guard children",
        &child,
        &[
            "#[path = \"folder_backed_summary/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"folder_backed_summary/child_ownership.rs\"]",
            "mod child_ownership;",
            "#[path = \"folder_backed_summary/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "#[path = \"folder_backed_summary/direct_review_assertions.rs\"]",
            "mod direct_review_assertions;",
            "#[path = \"folder_backed_summary/source_inventory.rs\"]",
            "mod source_inventory;",
            "pub(super) fn assert_code_review_findings_tests_are_folder_backed",
            "direct_review_assertions::assert_code_review_direct_sources_are_folder_backed",
            "source_inventory::code_review_findings_sources",
            "source_inventory::assert_code_review_findings_line_budgets",
            "super::plugin_importer_dx_child_owners::assert_plugin_importer_dx_child_owners_are_folder_backed",
            "super::late_api_cleanup_child_owners::assert_late_api_cleanup_child_owners_are_folder_backed",
            "super::typed_error_child_owners::assert_typed_error_child_owners_are_folder_backed",
            "code review findings children should preserve all 78 review guards",
        ],
    );
    for (_, child_path, anchor) in FOLDER_BACKED_SUMMARY_GUARD_CHILDREN {
        assert!(
            child.contains(child_path),
            "folder-backed summary parent should inventory child path {child_path}"
        );
        assert!(
            child_tree.contains(anchor),
            "folder-backed summary child {child_path} should own anchor {anchor}"
        );
    }

    assert_code_review_findings_tests_are_folder_backed();
}
