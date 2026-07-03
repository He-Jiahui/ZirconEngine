use super::super::super::*;

#[path = "structure_assertions/child_ownership.rs"]
mod child_ownership;
#[path = "structure_assertions/convergence_mounts.rs"]
mod convergence_mounts;
#[path = "structure_assertions/delegation.rs"]
mod delegation;
#[path = "structure_assertions/moved_guard_absence.rs"]
mod moved_guard_absence;
#[path = "structure_assertions/native_plugin_loader.rs"]
mod native_plugin_loader;
#[path = "structure_assertions/status_mirrors.rs"]
mod status_mirrors;

pub(super) const TYPED_ERROR_STRUCTURE_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners.rs";
pub(super) const TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions.rs";
pub(super) const TYPED_ERROR_STRUCTURE_CONVERGENCE_MOUNTS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/convergence_mounts.rs";
pub(super) const TYPED_ERROR_STRUCTURE_DELEGATION_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/delegation.rs";
pub(super) const TYPED_ERROR_STRUCTURE_CHILD_OWNERSHIP_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/child_ownership.rs";
pub(super) const TYPED_ERROR_STRUCTURE_STATUS_MIRRORS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/status_mirrors.rs";
pub(super) const TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/moved_guard_absence.rs";
pub(super) const TYPED_ERROR_NATIVE_STRUCTURE_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners/structure_assertions/native_plugin_loader.rs";
pub(super) const REVIEW_GUARD_STATUS_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/structure_assertions.rs";
pub(super) const REVIEW_GUARD_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps.rs";
pub(super) const REVIEW_GUARD_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps.rs";

pub(super) const TYPED_ERROR_CHILD_OWNER_LINE_BUDGET: usize = 800;
pub(super) const TYPED_ERROR_STRUCTURE_ASSERTIONS_GUARD_SPLIT_NAME: &str =
    "Runtime 15 M3 typed-error structure assertions guard folder-backed split";
pub(super) const TYPED_ERROR_STRUCTURE_ASSERTIONS_GUARD_SPLIT_ID: &str =
    "runtime_15_typed_error_structure_assertions_guard_folder_backed_static_passed_cargo_deferred";

pub(super) const TYPED_ERROR_STRUCTURE_ASSERTION_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        TYPED_ERROR_STRUCTURE_DELEGATION_CHILD,
        "runtime_15_typed_error_structure_assertions_are_child_owner",
    ),
    (
        "convergence_mounts",
        TYPED_ERROR_STRUCTURE_CONVERGENCE_MOUNTS_CHILD,
        "pub(super) fn assert_typed_error_convergence_parents_are_folder_backed",
    ),
    (
        "child_ownership",
        TYPED_ERROR_STRUCTURE_CHILD_OWNERSHIP_CHILD,
        "runtime_15_typed_error_structure_assertions_children_are_child_owned",
    ),
    (
        "native_plugin_loader",
        TYPED_ERROR_NATIVE_STRUCTURE_CHILD,
        "runtime_15_typed_error_native_plugin_loader_structure_is_child_owner",
    ),
    (
        "moved_guard_absence",
        TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD,
        "runtime_15_typed_error_structure_moved_guard_absence_is_child_owner",
    ),
    (
        "status_mirrors",
        TYPED_ERROR_STRUCTURE_STATUS_MIRRORS_CHILD,
        "runtime_15_typed_error_structure_assertions_guard_folder_backed_status_is_current",
    ),
];

pub(super) fn assert_typed_error_child_owners_are_folder_backed() {
    convergence_mounts::assert_typed_error_convergence_parents_are_folder_backed();
    native_plugin_loader::assert_typed_error_native_plugin_loader_children_are_folder_backed();
    moved_guard_absence::assert_typed_error_moved_guards_stay_child_owned();
}

pub(super) fn structure_assertion_guard_child_sources() -> Vec<(&'static str, String)> {
    TYPED_ERROR_STRUCTURE_ASSERTION_GUARD_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn structure_assertion_guard_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in structure_assertion_guard_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
