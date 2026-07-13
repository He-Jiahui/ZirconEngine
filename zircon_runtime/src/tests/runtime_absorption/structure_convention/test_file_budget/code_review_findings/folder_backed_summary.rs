use super::super::*;

#[path = "folder_backed_summary/child_ownership.rs"]
mod child_ownership;
#[path = "folder_backed_summary/delegation.rs"]
mod delegation;
#[path = "folder_backed_summary/direct_review_assertions.rs"]
mod direct_review_assertions;
#[path = "folder_backed_summary/source_inventory.rs"]
mod source_inventory;
#[path = "folder_backed_summary/status_mirrors.rs"]
mod status_mirrors;

pub(super) const STRUCTURE_GUARD_PARENT: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings.rs";
pub(super) const FOLDER_BACKED_SUMMARY_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary.rs";
pub(super) const DIRECT_REVIEW_ASSERTIONS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions.rs";
pub(super) const F12_DIRECT_ASSERTIONS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f12.rs";
pub(super) const F8_DIRECT_ASSERTIONS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f8.rs";
pub(super) const P0_DIRECT_ASSERTIONS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/p0.rs";
pub(super) const RENDER_DIRECT_ASSERTIONS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/render.rs";
pub(super) const ROOT_PARENT_DIRECT_ASSERTIONS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/root_parent.rs";
pub(super) const SOURCE_INVENTORY_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/source_inventory.rs";
pub(super) const REVIEW_GUARD_STATUS_MAP_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps/code_review_guard_maps/folder_backed_summary_rows.rs";
pub(super) const REVIEW_GUARD_DATE_MAP_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps/code_review_guard_maps/folder_backed_summary_rows.rs";

pub(super) const CODE_REVIEW_FINDINGS_LINE_BUDGET: usize = 800;
pub(super) const FOLDER_BACKED_SUMMARY_GUARD_SPLIT_NAME: &str =
    "Runtime 15 M3 code review findings folder-backed summary guard folder-backed split";
pub(super) const FOLDER_BACKED_SUMMARY_GUARD_SPLIT_ID: &str = "runtime_15_code_review_findings_folder_backed_summary_guard_folder_backed_static_passed_cargo_deferred";

pub(super) const FOLDER_BACKED_SUMMARY_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/delegation.rs",
        "runtime_15_code_review_findings_folder_backed_summary_is_child_owner",
    ),
    (
        "child_ownership",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/child_ownership.rs",
        "runtime_15_code_review_findings_folder_backed_summary_children_are_child_owned",
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/status_mirrors.rs",
        "runtime_15_code_review_findings_folder_backed_summary_guard_folder_backed_status_is_current",
    ),
];

pub(super) fn assert_code_review_findings_tests_are_folder_backed() {
    let sources = source_inventory::code_review_findings_sources();

    super::plugin_importer_dx_child_owners::assert_plugin_importer_dx_child_owners_are_folder_backed();
    super::plugin_importer_dx_child_owners::assert_plugin_importer_dx_line_budgets();
    super::late_api_cleanup_child_owners::assert_late_api_cleanup_child_owners_are_folder_backed();
    super::typed_error_child_owners::assert_typed_error_child_owners_are_folder_backed();
    super::typed_error_child_owners::assert_typed_error_line_budgets();

    direct_review_assertions::assert_code_review_direct_sources_are_folder_backed(&sources);
    let child_test_total =
        super::plugin_importer_dx_child_owners::plugin_importer_dx_review_guard_count()
            + super::late_api_cleanup_child_owners::late_api_cleanup_review_guard_count()
            + super::typed_error_child_owners::typed_error_review_guard_count()
            + sources.direct_review_guard_count();
    assert_eq!(
        child_test_total, 78,
        "code review findings children should preserve all 78 review guards"
    );

    source_inventory::assert_code_review_findings_line_budgets(&sources);
    super::status_docs::assert_code_review_findings_status_docs_are_synced();
}

pub(super) fn review_guard_status_rows_source() -> String {
    super::status_docs::review_guard_status_rows_source()
}

pub(super) fn folder_backed_summary_guard_child_sources() -> Vec<(&'static str, String)> {
    FOLDER_BACKED_SUMMARY_GUARD_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn folder_backed_summary_guard_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in folder_backed_summary_guard_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
