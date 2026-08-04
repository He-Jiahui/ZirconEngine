#[path = "code_review_findings/f8_child_owners.rs"]
mod f8_child_owners;
#[path = "code_review_findings/folder_backed_summary.rs"]
mod folder_backed_summary;
#[path = "code_review_findings/late_api_cleanup_child_owners.rs"]
mod late_api_cleanup_child_owners;
#[path = "code_review_findings/p0_child_owners.rs"]
mod p0_child_owners;
#[path = "code_review_findings/p0_native_fixture_leaf_owners.rs"]
mod p0_native_fixture_leaf_owners;
#[path = "code_review_findings/plugin_importer_dx_child_owners.rs"]
mod plugin_importer_dx_child_owners;
#[path = "code_review_findings/structure_guard_children.rs"]
mod structure_guard_children;
#[path = "code_review_findings/typed_error_child_owners.rs"]
mod typed_error_child_owners;

#[test]
fn runtime_15_code_review_findings_tests_are_folder_backed() {
    folder_backed_summary::assert_code_review_findings_tests_are_folder_backed();
}
