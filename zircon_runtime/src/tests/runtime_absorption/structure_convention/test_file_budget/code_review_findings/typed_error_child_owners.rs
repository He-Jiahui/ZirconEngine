use super::super::*;

#[path = "typed_error_child_owners/budgets.rs"]
mod budgets;
#[path = "typed_error_child_owners/child_ownership.rs"]
mod child_ownership;
#[path = "typed_error_child_owners/delegation.rs"]
mod delegation;
#[path = "typed_error_child_owners/source_inventory.rs"]
mod source_inventory;
#[path = "typed_error_child_owners/status_docs.rs"]
mod status_docs;
#[path = "typed_error_child_owners/status_mirrors.rs"]
mod status_mirrors;
#[path = "typed_error_child_owners/structure_assertions.rs"]
mod structure_assertions;

pub(super) const STRUCTURE_GUARD_PARENT: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings.rs";
pub(super) const TYPED_ERROR_STRUCTURE_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners.rs";
pub(super) const TYPED_ERROR_TOP_LEVEL_DELEGATION_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/delegation.rs";
pub(super) const TYPED_ERROR_TOP_LEVEL_CHILD_OWNERSHIP_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/child_ownership.rs";
pub(super) const TYPED_ERROR_TOP_LEVEL_STATUS_MIRRORS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_mirrors.rs";
pub(super) const TYPED_ERROR_TOP_LEVEL_BUDGETS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/budgets.rs";
pub(super) const TYPED_ERROR_SOURCE_INVENTORY_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/source_inventory.rs";
pub(super) const TYPED_ERROR_STATUS_DOCS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/status_docs.rs";
pub(super) const TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions.rs";
pub(super) const TYPED_ERROR_STRUCTURE_CONVERGENCE_MOUNTS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/convergence_mounts.rs";
pub(super) const TYPED_ERROR_STRUCTURE_DELEGATION_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/delegation.rs";
pub(super) const TYPED_ERROR_STRUCTURE_CHILD_OWNERSHIP_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/child_ownership.rs";
pub(super) const TYPED_ERROR_STRUCTURE_STATUS_MIRRORS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/status_mirrors.rs";
pub(super) const TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/moved_guard_absence.rs";
pub(super) const TYPED_ERROR_NATIVE_STRUCTURE_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/native_plugin_loader.rs";
pub(super) const STRUCTURE_GUARD_TYPED_ERROR_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/typed_error.rs";
pub(super) const STRUCTURE_GUARD_ROWS: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/folder_backed.rs";
pub(super) const REVIEW_GUARD_STATUS_MAP: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps.rs";
pub(super) const REVIEW_GUARD_DATE_MAP: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps.rs";

pub(super) const SLICE: &str =
    "Runtime 15 M3 code review findings typed-error structure guard child-owner split";
pub(super) const STATUS: &str =
    "runtime_15_code_review_findings_typed_error_structure_guard_child_owner_split_static_passed_cargo_deferred";
pub(super) const DATE: &str = "2026-06-30";
pub(super) const GUARD: &str =
    "runtime_15_code_review_findings_typed_error_structure_guard_is_child_owner";
pub(super) const FOLDER_BACKED_SLICE: &str =
    "Runtime 15 M3 typed-error structure guard folder-backed split";
pub(super) const FOLDER_BACKED_STATUS: &str =
    "runtime_15_typed_error_structure_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const FOLDER_BACKED_DATE: &str = "2026-07-03";
pub(super) const FOLDER_BACKED_GUARD: &str =
    "runtime_15_typed_error_structure_guard_is_folder_backed";
pub(super) const FOLDER_BACKED_STATUS_GUARD: &str =
    "runtime_15_typed_error_structure_guard_folder_backed_status_is_current";
pub(super) const BUDGET_GUARD: &str = "runtime_15_typed_error_structure_guard_budgets_are_focused";
pub(super) const TYPED_ERROR_CHILD_OWNER_LINE_BUDGET: usize = 800;

pub(super) const FOLDER_BACKED_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        TYPED_ERROR_TOP_LEVEL_DELEGATION_CHILD,
        FOLDER_BACKED_GUARD,
    ),
    (
        "child_ownership",
        TYPED_ERROR_TOP_LEVEL_CHILD_OWNERSHIP_CHILD,
        GUARD,
    ),
    (
        "source_inventory",
        TYPED_ERROR_SOURCE_INVENTORY_CHILD,
        "runtime_15_typed_error_source_inventory_is_child_owner",
    ),
    (
        "status_docs",
        TYPED_ERROR_STATUS_DOCS_CHILD,
        "runtime_15_typed_error_status_docs_are_folder_backed",
    ),
    (
        "structure_assertions",
        TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD,
        "pub(super) fn assert_typed_error_child_owners_are_folder_backed",
    ),
    (
        "status_mirrors",
        TYPED_ERROR_TOP_LEVEL_STATUS_MIRRORS_CHILD,
        FOLDER_BACKED_STATUS_GUARD,
    ),
    ("budgets", TYPED_ERROR_TOP_LEVEL_BUDGETS_CHILD, BUDGET_GUARD),
];

pub(super) fn assert_typed_error_child_owners_are_folder_backed() {
    structure_assertions::assert_typed_error_child_owners_are_folder_backed();
}

pub(super) fn typed_error_children_source() -> String {
    source_inventory::typed_error_children_source()
}

pub(super) fn assert_typed_error_line_budgets() {
    source_inventory::assert_typed_error_line_budgets();
}

pub(super) fn typed_error_review_guard_count() -> usize {
    source_inventory::typed_error_review_guard_count()
}

pub(super) fn assert_typed_error_status_docs_are_synced() {
    status_docs::assert_typed_error_status_docs_are_synced();
}

pub(super) fn folder_backed_child_sources() -> Vec<(&'static str, String)> {
    FOLDER_BACKED_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn folder_backed_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, child_source) in folder_backed_child_sources() {
        blob.push_str(&child_source);
        blob.push('\n');
    }
    blob
}
