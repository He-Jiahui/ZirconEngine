use super::super::super::*;

#[path = "structure/child_ownership.rs"]
mod child_ownership;
#[path = "structure/convergence_mounts.rs"]
mod convergence_mounts;
#[path = "structure/delegation.rs"]
mod delegation;
#[path = "structure/moved_guard_absence.rs"]
mod moved_guard_absence;
#[path = "structure/native_plugin_loader.rs"]
mod native_plugin_loader;

pub(super) const TYPED_ERROR_STRUCTURE_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners.rs";
pub(super) const TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure_assertions.rs";
pub(super) const TYPED_ERROR_STRUCTURE_CONVERGENCE_MOUNTS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/convergence_mounts.rs";
pub(super) const TYPED_ERROR_STRUCTURE_DELEGATION_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/delegation.rs";
pub(super) const TYPED_ERROR_STRUCTURE_CHILD_OWNERSHIP_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/child_ownership.rs";
pub(super) const TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/moved_guard_absence.rs";
pub(super) const TYPED_ERROR_NATIVE_STRUCTURE_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/native_plugin_loader.rs";

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
