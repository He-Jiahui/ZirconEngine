use super::super::super::*;

#[path = "typed_error/budgets.rs"]
mod budgets;
#[path = "typed_error/delegation.rs"]
mod delegation;
#[path = "typed_error/structure_assertions.rs"]
mod structure_assertions;
#[path = "typed_error/top_level.rs"]
mod top_level;

const STRUCTURE_GUARD_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children.rs";
const STRUCTURE_GUARD_TYPED_ERROR_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/typed_error.rs";
const STRUCTURE_GUARD_TYPED_ERROR_DELEGATION_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/typed_error/delegation.rs";
const STRUCTURE_GUARD_TYPED_ERROR_TOP_LEVEL_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/typed_error/top_level.rs";
const STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/typed_error/structure_assertions.rs";
const STRUCTURE_GUARD_TYPED_ERROR_BUDGETS_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/typed_error/budgets.rs";

const TYPED_ERROR_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_child_owners.rs";
const TYPED_ERROR_TOP_LEVEL_DELEGATION_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/delegation.rs";
const TYPED_ERROR_TOP_LEVEL_CHILD_OWNERSHIP_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/child_ownership.rs";
const TYPED_ERROR_TOP_LEVEL_BUDGETS_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/budgets.rs";
const TYPED_ERROR_SOURCE_INVENTORY_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/source_inventory.rs";
const TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure_assertions.rs";
const TYPED_ERROR_STRUCTURE_CONVERGENCE_MOUNTS_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/convergence_mounts.rs";
const TYPED_ERROR_STRUCTURE_DELEGATION_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/delegation.rs";
const TYPED_ERROR_STRUCTURE_CHILD_OWNERSHIP_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/child_ownership.rs";
const TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/moved_guard_absence.rs";
const TYPED_ERROR_NATIVE_STRUCTURE_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/structure/native_plugin_loader.rs";

const TYPED_ERROR_FOLDER_BACKED_SLICE: &str =
    "Runtime 15 M3 typed-error structure guard folder-backed split";
const TYPED_ERROR_FOLDER_BACKED_STATUS: &str =
    "runtime_15_typed_error_structure_guard_folder_backed_static_passed_cargo_deferred";
const STRUCTURE_GUARD_TYPED_ERROR_FOLDER_BACKED_SLICE: &str =
    "Runtime 15 M3 code review findings structure guard typed-error folder-backed split";
const STRUCTURE_GUARD_TYPED_ERROR_FOLDER_BACKED_STATUS: &str = "runtime_15_code_review_findings_structure_guard_typed_error_folder_backed_static_passed_cargo_deferred";

const STRUCTURE_GUARD_TYPED_ERROR_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        STRUCTURE_GUARD_TYPED_ERROR_DELEGATION_CHILD_OWNER,
        "runtime_15_code_review_findings_structure_guard_typed_error_is_child_owner",
    ),
    (
        "top_level",
        STRUCTURE_GUARD_TYPED_ERROR_TOP_LEVEL_CHILD_OWNER,
        "runtime_15_code_review_findings_structure_guard_typed_error_top_level_checks_are_child_owned",
    ),
    (
        "structure_assertions",
        STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD_OWNER,
        "runtime_15_code_review_findings_structure_guard_typed_error_structure_assertions_are_child_owned",
    ),
    (
        "budgets",
        STRUCTURE_GUARD_TYPED_ERROR_BUDGETS_CHILD_OWNER,
        "runtime_15_code_review_findings_structure_guard_typed_error_children_line_budgets_are_current",
    ),
];

pub(super) fn assert_typed_error_structure_children_are_mounted() {
    delegation::assert_typed_error_structure_guard_delegation_is_current();
    top_level::assert_typed_error_top_level_checks_are_current();
    structure_assertions::assert_typed_error_structure_assertion_checks_are_current();
    budgets::assert_typed_error_structure_guard_line_budgets();
}

fn typed_error_structure_guard_child_sources() -> Vec<(&'static str, String)> {
    STRUCTURE_GUARD_TYPED_ERROR_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn typed_error_structure_guard_child_source_blob() -> String {
    let mut blob = String::new();
    blob.push_str(&read_runtime_src(STRUCTURE_GUARD_TYPED_ERROR_CHILD_OWNER));
    blob.push('\n');
    for (_, source) in typed_error_structure_guard_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob.push_str(&super::super::typed_error_child_owners::folder_backed_child_source_blob());
    blob.push_str(&read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/typed_error_owners/root_statuses.rs",
    ));
    blob.push('\n');
    blob
}

#[test]
fn runtime_15_code_review_findings_structure_guard_typed_error_is_child_owner() {
    assert_typed_error_structure_children_are_mounted();
}
