use super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_code_review_findings_structure_guard_folder_backed_summary_guard_folder_backed_status_is_current(
) {
    let status_rows = review_guard_status_rows_source();
    let status_map = super::super::structure_guard_status_map_source();
    let date_map = super::super::structure_guard_date_map_source();
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    for (label, source) in [
        ("structure guard row data", status_rows.as_str()),
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
                FOLDER_BACKED_SUMMARY_STRUCTURE_GUARD_SPLIT_NAME,
                FOLDER_BACKED_SUMMARY_STRUCTURE_GUARD_SPLIT_ID,
                FOLDER_BACKED_SUMMARY_STRUCTURE_CHILD_OWNER,
                FOLDER_BACKED_SUMMARY_STRUCTURE_DELEGATION_CHILD_OWNER,
                FOLDER_BACKED_SUMMARY_STRUCTURE_DIRECT_ASSERTIONS_CHILD_OWNER,
                FOLDER_BACKED_SUMMARY_STRUCTURE_SOURCE_INVENTORY_CHILD_OWNER,
                FOLDER_BACKED_SUMMARY_STRUCTURE_BUDGETS_CHILD_OWNER,
                FOLDER_BACKED_SUMMARY_STRUCTURE_STATUS_MIRRORS_CHILD_OWNER,
                "runtime_15_code_review_findings_structure_guard_folder_backed_summary_is_child_owner",
                "runtime_15_code_review_findings_structure_guard_folder_backed_summary_direct_assertions_are_child_owned",
                "runtime_15_code_review_findings_structure_guard_folder_backed_summary_source_inventory_is_child_owned",
                "runtime_15_code_review_findings_structure_guard_folder_backed_summary_children_line_budgets_are_current",
                "runtime_15_code_review_findings_structure_guard_folder_backed_summary_guard_folder_backed_status_is_current",
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "M3 review status map records folder-backed summary structure guard split",
        &status_map,
        &[
            FOLDER_BACKED_SUMMARY_STRUCTURE_GUARD_SPLIT_NAME,
            FOLDER_BACKED_SUMMARY_STRUCTURE_GUARD_SPLIT_ID,
        ],
    );
    assert_contains_all(
        "M3 review date map records folder-backed summary structure guard split",
        &date_map,
        &[
            FOLDER_BACKED_SUMMARY_STRUCTURE_GUARD_SPLIT_NAME,
            "2026-07-02",
        ],
    );

    budgets::assert_folder_backed_summary_structure_line_budgets();
}
