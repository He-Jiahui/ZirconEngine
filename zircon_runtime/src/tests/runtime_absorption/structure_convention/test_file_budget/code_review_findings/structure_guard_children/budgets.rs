use super::super::super::*;
use super::*;

#[path = "budgets/line_counts.rs"]
mod line_counts;

pub(super) const STRUCTURE_GUARD_CHILDREN_BUDGETS_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/budgets.rs";
pub(super) const STRUCTURE_GUARD_CHILDREN_LINE_COUNTS_CHILD_OWNER: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/budgets/line_counts.rs";

const STRUCTURE_GUARD_CHILDREN_BUDGET_STATUS_SPLIT_NAME: &str =
    "Runtime 15 M3 code review findings structure guard children budget-status child split";
const STRUCTURE_GUARD_CHILDREN_BUDGET_STATUS_SPLIT_ID: &str = "runtime_15_code_review_findings_structure_guard_children_budget_status_child_split_static_passed_cargo_deferred";

const STRUCTURE_GUARD_CHILDREN_BUDGET_CHILDREN: &[(&str, &str, &str)] = &[(
    "line_counts",
    STRUCTURE_GUARD_CHILDREN_LINE_COUNTS_CHILD_OWNER,
    "runtime_15_code_review_findings_structure_guard_children_line_budgets_are_child_owned",
)];

pub(super) fn assert_structure_guard_children_line_budgets() {
    line_counts::assert_structure_guard_children_line_budgets();
}

fn structure_guard_children_budget_child_sources() -> Vec<(&'static str, String)> {
    STRUCTURE_GUARD_CHILDREN_BUDGET_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn structure_guard_children_budget_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in structure_guard_children_budget_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}

#[test]
fn runtime_15_code_review_findings_structure_guard_children_budget_status_is_child_owned() {
    let parent = read_runtime_src(STRUCTURE_GUARD_CHILDREN_BUDGETS_CHILD_OWNER);
    let child_sources = structure_guard_children_budget_child_source_blob();

    assert_contains_all(
        "structure guard children budgets parent mounts focused children",
        &parent,
        &[
            "#[path = \"budgets/line_counts.rs\"]",
            "mod line_counts;",
            "#[path = \"budgets/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "pub(super) fn assert_structure_guard_children_line_budgets",
            STRUCTURE_GUARD_CHILDREN_BUDGET_STATUS_SPLIT_NAME,
            STRUCTURE_GUARD_CHILDREN_BUDGET_STATUS_SPLIT_ID,
        ],
    );
    for (_, child_path, guard_name) in STRUCTURE_GUARD_CHILDREN_BUDGET_CHILDREN {
        assert!(
            parent.contains(child_path),
            "structure guard children budgets parent should inventory child path {child_path}"
        );
        assert!(
            child_sources.contains(guard_name),
            "structure guard children budgets child {child_path} should own anchor {guard_name}"
        );
    }

    line_counts::assert_structure_guard_children_line_budgets();
}
