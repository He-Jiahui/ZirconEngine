use super::super::super::*;

#[path = "folder_backed_summary/budgets.rs"]
mod budgets;
#[path = "folder_backed_summary/delegation.rs"]
mod delegation;
#[path = "folder_backed_summary/direct_assertions.rs"]
mod direct_assertions;
#[path = "folder_backed_summary/source_inventory.rs"]
mod source_inventory;
#[path = "folder_backed_summary/status_mirrors.rs"]
mod status_mirrors;

pub(super) const STRUCTURE_GUARD_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children.rs";
pub(super) const FOLDER_BACKED_SUMMARY_STRUCTURE_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/folder_backed_summary.rs";
pub(super) const FOLDER_BACKED_SUMMARY_STRUCTURE_DELEGATION_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/folder_backed_summary/delegation.rs";
pub(super) const FOLDER_BACKED_SUMMARY_STRUCTURE_DIRECT_ASSERTIONS_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/folder_backed_summary/direct_assertions.rs";
pub(super) const FOLDER_BACKED_SUMMARY_STRUCTURE_SOURCE_INVENTORY_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/folder_backed_summary/source_inventory.rs";
pub(super) const FOLDER_BACKED_SUMMARY_STRUCTURE_BUDGETS_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/folder_backed_summary/budgets.rs";
pub(super) const FOLDER_BACKED_SUMMARY_STRUCTURE_STATUS_MIRRORS_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/folder_backed_summary/status_mirrors.rs";

pub(super) const FOLDER_BACKED_SUMMARY_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary.rs";
pub(super) const FOLDER_BACKED_SUMMARY_DELEGATION_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/delegation.rs";
pub(super) const FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/child_ownership.rs";
pub(super) const FOLDER_BACKED_SUMMARY_STATUS_MIRRORS_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/status_mirrors.rs";
pub(super) const FOLDER_BACKED_SUMMARY_DIRECT_ASSERTIONS_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions.rs";
pub(super) const FOLDER_BACKED_SUMMARY_DIRECT_ASSERTIONS_DELEGATION_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/delegation.rs";
pub(super) const FOLDER_BACKED_SUMMARY_DIRECT_ASSERTIONS_CHILD_OWNERSHIP_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/child_ownership.rs";
pub(super) const FOLDER_BACKED_SUMMARY_DIRECT_ASSERTIONS_STATUS_MIRRORS_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/status_mirrors.rs";
pub(super) const FOLDER_BACKED_SUMMARY_F12_DIRECT_ASSERTIONS_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f12.rs";
pub(super) const FOLDER_BACKED_SUMMARY_F8_DIRECT_ASSERTIONS_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f8.rs";
pub(super) const FOLDER_BACKED_SUMMARY_P0_DIRECT_ASSERTIONS_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/p0.rs";
pub(super) const FOLDER_BACKED_SUMMARY_RENDER_DIRECT_ASSERTIONS_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/render.rs";
pub(super) const FOLDER_BACKED_SUMMARY_ROOT_PARENT_DIRECT_ASSERTIONS_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/root_parent.rs";
pub(super) const FOLDER_BACKED_SUMMARY_SOURCE_INVENTORY_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/source_inventory.rs";
pub(super) const FOLDER_BACKED_SUMMARY_SOURCE_MODEL_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/source_inventory/model.rs";
pub(super) const FOLDER_BACKED_SUMMARY_SOURCE_READS_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/source_inventory/reads.rs";
pub(super) const FOLDER_BACKED_SUMMARY_SOURCE_BUDGETS_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/source_inventory/budgets.rs";
pub(super) const FOLDER_BACKED_SUMMARY_SOURCE_DELEGATION_CHILD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/source_inventory/delegation.rs";

pub(super) const REVIEW_GUARD_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps.rs";
pub(super) const REVIEW_GUARD_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps.rs";

pub(super) const FOLDER_BACKED_SUMMARY_STRUCTURE_GUARD_SPLIT_NAME: &str =
    "Runtime 15 M3 code review findings structure guard folder-backed summary guard folder-backed split";
pub(super) const FOLDER_BACKED_SUMMARY_STRUCTURE_GUARD_SPLIT_ID: &str =
    "runtime_15_code_review_findings_structure_guard_folder_backed_summary_guard_folder_backed_static_passed_cargo_deferred";

pub(super) const FOLDER_BACKED_SUMMARY_STRUCTURE_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        FOLDER_BACKED_SUMMARY_STRUCTURE_DELEGATION_CHILD_OWNER,
        "runtime_15_code_review_findings_structure_guard_folder_backed_summary_is_child_owner",
    ),
    (
        "direct_assertions",
        FOLDER_BACKED_SUMMARY_STRUCTURE_DIRECT_ASSERTIONS_CHILD_OWNER,
        "runtime_15_code_review_findings_structure_guard_folder_backed_summary_direct_assertions_are_child_owned",
    ),
    (
        "source_inventory",
        FOLDER_BACKED_SUMMARY_STRUCTURE_SOURCE_INVENTORY_CHILD_OWNER,
        "runtime_15_code_review_findings_structure_guard_folder_backed_summary_source_inventory_is_child_owned",
    ),
    (
        "budgets",
        FOLDER_BACKED_SUMMARY_STRUCTURE_BUDGETS_CHILD_OWNER,
        "runtime_15_code_review_findings_structure_guard_folder_backed_summary_children_line_budgets_are_current",
    ),
    (
        "status_mirrors",
        FOLDER_BACKED_SUMMARY_STRUCTURE_STATUS_MIRRORS_CHILD_OWNER,
        "runtime_15_code_review_findings_structure_guard_folder_backed_summary_guard_folder_backed_status_is_current",
    ),
];

pub(super) fn assert_folder_backed_summary_structure_children_are_mounted() {
    delegation::assert_folder_backed_summary_structure_delegation_is_current();
    direct_assertions::assert_folder_backed_summary_direct_assertions_are_current();
    source_inventory::assert_folder_backed_summary_source_inventory_is_current();
    budgets::assert_folder_backed_summary_structure_line_budgets();
}

pub(super) fn folder_backed_summary_structure_child_sources() -> Vec<(&'static str, String)> {
    FOLDER_BACKED_SUMMARY_STRUCTURE_GUARD_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn folder_backed_summary_structure_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in folder_backed_summary_structure_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}

pub(super) fn review_guard_status_rows_source() -> String {
    super::review_guard_status_rows_source()
}
