use super::super::super::super::*;
use super::super::*;
use super::*;

fn assert_structure_guard_children_status_mirror(
    slice_name: &str,
    slice_id: &str,
    date: &str,
    required_anchors: &[&str],
) {
    let status_map = structure_guard_status_map_source();
    let date_map = structure_guard_date_map_source();
    let structure_guard_rows = review_guard_status_rows_source();
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let mut required = Vec::from([slice_name, slice_id, "Cargo gate deferred"]);
    required.extend_from_slice(required_anchors);

    assert_contains_all(
        "review-guard status map",
        &status_map,
        &[slice_name, slice_id],
    );
    for (label, source) in [
        ("structure guard row data", structure_guard_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
    ] {
        assert_contains_all(label, source, &required);
    }
    assert_contains_all("review-guard date map", &date_map, &[slice_name, date]);
}

#[test]
fn runtime_15_code_review_findings_structure_guard_children_folder_backed_status_is_current() {
    assert_structure_guard_children_status_mirror(
        STRUCTURE_GUARD_FOLDER_BACKED_SPLIT_NAME,
        STRUCTURE_GUARD_FOLDER_BACKED_SPLIT_ID,
        "2026-07-02",
        &[
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/delegation.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/review_guard_groups.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/plugin_importer.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/status_docs.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/structure_guard_children/budgets.rs",
            "runtime_15_code_review_findings_structure_guard_children_are_mounted",
        ],
    );
}

#[test]
fn runtime_15_code_review_findings_structure_guard_children_budget_status_child_split_status_is_current(
) {
    assert_structure_guard_children_status_mirror(
        STRUCTURE_GUARD_CHILDREN_BUDGET_STATUS_SPLIT_NAME,
        STRUCTURE_GUARD_CHILDREN_BUDGET_STATUS_SPLIT_ID,
        "2026-07-04",
        &[
            STRUCTURE_GUARD_CHILDREN_BUDGETS_CHILD_OWNER,
            STRUCTURE_GUARD_CHILDREN_LINE_COUNTS_CHILD_OWNER,
            STRUCTURE_GUARD_CHILDREN_STATUS_MIRRORS_CHILD_OWNER,
            "runtime_15_code_review_findings_structure_guard_children_budget_status_is_child_owned",
            "runtime_15_code_review_findings_structure_guard_children_line_budgets_are_child_owned",
            "runtime_15_code_review_findings_structure_guard_children_budget_status_child_split_status_is_current",
        ],
    );
}
