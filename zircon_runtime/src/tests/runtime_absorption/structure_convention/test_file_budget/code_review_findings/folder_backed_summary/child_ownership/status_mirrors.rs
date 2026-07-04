use super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_code_review_findings_folder_backed_summary_child_ownership_guard_folder_backed_status_is_current(
) {
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/folder_backed_summary.rs",
    );
    let status_map = read_runtime_src(REVIEW_GUARD_STATUS_MAP_PATH);
    let date_map = read_runtime_src(REVIEW_GUARD_DATE_MAP_PATH);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    for (label, source) in [
        ("folder-backed summary row data", status_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("runtime architecture session note", session_note.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_SLICE,
                FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_STATUS,
                FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_CHILD,
                FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_DELEGATION_CHILD,
                FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_PARENT_ABSENCE_CHILD,
                FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_DIRECT_ASSERTIONS_CHILD,
                FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_SOURCE_INVENTORY_CHILD,
                FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_BUDGETS_CHILD,
                FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_STATUS_MIRRORS_CHILD,
                "runtime_15_code_review_findings_folder_backed_summary_children_are_child_owned",
                FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_GUARD,
                FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_STATUS_GUARD,
                FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_BUDGET_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "M3 review status map records folder-backed summary child-ownership split",
        &status_map,
        &[
            FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_SLICE,
            FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_STATUS,
        ],
    );
    assert_contains_all(
        "M3 review date map records folder-backed summary child-ownership split",
        &date_map,
        &[
            FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_SLICE,
            FOLDER_BACKED_SUMMARY_CHILD_OWNERSHIP_DATE,
        ],
    );
}
