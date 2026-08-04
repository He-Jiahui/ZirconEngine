use super::super::super::super::*;
use super::*;

#[path = "structure/budgets.rs"]
mod budgets;
#[path = "structure/current_checks.rs"]
mod current_checks;
#[path = "structure/folder_backed.rs"]
mod folder_backed;
#[path = "structure/source_trees.rs"]
mod source_trees;

const STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_SOURCE_TREES_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/typed_error/structure/source_trees.rs";
const STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_CURRENT_CHECKS_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/typed_error/structure/current_checks.rs";
const STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_FOLDER_BACKED_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/typed_error/structure/folder_backed.rs";
const STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_BUDGETS_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/typed_error/structure/budgets.rs";

const STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_FOLDER_BACKED_SLICE: &str = "Runtime 15 M3 code review findings structure guard typed-error structure assertions folder-backed split";
const STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_FOLDER_BACKED_STATUS: &str = "runtime_15_code_review_findings_structure_guard_typed_error_structure_assertions_folder_backed_static_passed_cargo_deferred";
const STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_FRAMEWORKS_STATUS: &str = "frameworks_02_m3_code_review_findings_structure_guard_typed_error_structure_assertions_folder_backed_static_passed_cargo_deferred";
const STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_FOLDER_BACKED_GUARD: &str = "runtime_15_code_review_findings_structure_guard_typed_error_structure_assertions_guard_is_folder_backed";

const STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "source_trees",
        STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_SOURCE_TREES_CHILD_OWNER,
        "typed_error_structure_assertions_child_tree",
    ),
    (
        "current_checks",
        STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_CURRENT_CHECKS_CHILD_OWNER,
        "runtime_15_code_review_findings_structure_guard_typed_error_structure_assertions_are_child_owned",
    ),
    (
        "folder_backed",
        STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_FOLDER_BACKED_CHILD_OWNER,
        STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_FOLDER_BACKED_GUARD,
    ),
    (
        "budgets",
        STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_BUDGETS_CHILD_OWNER,
        "runtime_15_code_review_findings_structure_guard_typed_error_structure_assertions_children_stay_budgeted",
    ),
];

pub(super) fn assert_typed_error_structure_assertion_checks_are_current() {
    current_checks::assert_typed_error_structure_assertion_checks_are_current();
}

fn structure_assertion_guard_child_sources() -> Vec<(&'static str, String)> {
    STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

fn structure_assertion_guard_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in structure_assertion_guard_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
