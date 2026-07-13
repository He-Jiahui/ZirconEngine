use super::super::super::super::super::*;
use super::super::{
    direct_review_date_map_source, direct_review_status_map_source,
    direct_review_status_rows_source,
};
use super::*;

#[test]
fn runtime_15_code_review_findings_direct_assertions_child_ownership_guard_folder_backed_status_is_current(
) {
    let status_rows = direct_review_status_rows_source();
    let status_map = direct_review_status_map_source();
    let date_map = direct_review_date_map_source();
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    for (label, source) in [
        ("direct assertion row data", status_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_SLICE,
                DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_STATUS,
                DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_CHILD,
                DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_DELEGATION_CHILD,
                DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_PARENT_ABSENCE_CHILD,
                DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_ENTRY_POINTS_CHILD,
                DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_BUDGETS_CHILD,
                DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_STATUS_MIRRORS_CHILD,
                "runtime_15_code_review_findings_direct_assertions_children_are_child_owned",
                DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_GUARD,
                DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_STATUS_GUARD,
                DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_BUDGET_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "M3 review status map records direct assertions child-ownership folder-backed split",
        &status_map,
        &[
            DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_SLICE,
            DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_STATUS,
        ],
    );
    assert_contains_all(
        "M3 review date map records direct assertions child-ownership folder-backed split",
        &date_map,
        &[
            DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_SLICE,
            DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_DATE,
        ],
    );
}
