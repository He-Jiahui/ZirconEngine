use super::super::super::*;

#[path = "source_inventory/budgets.rs"]
mod budgets;
#[path = "source_inventory/delegation.rs"]
mod delegation;
#[path = "source_inventory/model.rs"]
mod model;
#[path = "source_inventory/reads.rs"]
mod reads;

pub(super) use model::CodeReviewFindingsSources;

pub(super) const FOLDER_BACKED_SUMMARY_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary.rs";
pub(super) const SOURCE_INVENTORY_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/source_inventory.rs";
pub(super) const REVIEW_GUARD_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps.rs";
pub(super) const REVIEW_GUARD_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps.rs";
pub(super) const CODE_REVIEW_FINDINGS_LINE_BUDGET: usize = 800;

pub(super) const SOURCE_INVENTORY_FOLDER_BACKED_SPLIT_NAME: &str =
    "Runtime 15 M3 code review findings source inventory folder-backed split";
pub(super) const SOURCE_INVENTORY_FOLDER_BACKED_SPLIT_ID: &str =
    "runtime_15_code_review_findings_source_inventory_folder_backed_static_passed_cargo_deferred";

pub(super) const SOURCE_INVENTORY_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "model",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/source_inventory/model.rs",
        "struct CodeReviewFindingsSources",
    ),
    (
        "reads",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/source_inventory/reads.rs",
        "pub(super) fn code_review_findings_sources",
    ),
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/source_inventory/budgets.rs",
        "pub(super) fn assert_code_review_findings_line_budgets",
    ),
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/source_inventory/delegation.rs",
        "runtime_15_code_review_findings_source_inventory_is_child_owner",
    ),
];

pub(super) fn code_review_findings_sources() -> CodeReviewFindingsSources {
    reads::code_review_findings_sources()
}

pub(super) fn assert_code_review_findings_line_budgets(sources: &CodeReviewFindingsSources) {
    budgets::assert_code_review_findings_line_budgets(sources);
}

pub(super) fn source_inventory_status_rows_source() -> String {
    super::review_guard_status_rows_source()
}

pub(super) fn source_inventory_child_sources() -> Vec<(&'static str, String)> {
    SOURCE_INVENTORY_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn source_inventory_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in source_inventory_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
