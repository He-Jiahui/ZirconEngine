use super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_code_review_findings_structure_guard_typed_error_folder_backed_status_is_current() {
    let status_rows = review_guard_status_rows_source();
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
        ("structure guard row data", status_rows.as_str()),
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
                STRUCTURE_GUARD_TYPED_ERROR_FOLDER_BACKED_SLICE,
                STRUCTURE_GUARD_TYPED_ERROR_FOLDER_BACKED_STATUS,
                STRUCTURE_GUARD_TYPED_ERROR_CHILD_OWNER,
                STRUCTURE_GUARD_TYPED_ERROR_DELEGATION_CHILD_OWNER,
                STRUCTURE_GUARD_TYPED_ERROR_TOP_LEVEL_CHILD_OWNER,
                STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD_OWNER,
                STRUCTURE_GUARD_TYPED_ERROR_BUDGETS_CHILD_OWNER,
                STRUCTURE_GUARD_TYPED_ERROR_STATUS_MIRRORS_CHILD_OWNER,
                "runtime_15_code_review_findings_structure_guard_typed_error_is_child_owner",
                "runtime_15_code_review_findings_structure_guard_typed_error_top_level_checks_are_child_owned",
                "runtime_15_code_review_findings_structure_guard_typed_error_structure_assertions_are_child_owned",
                "runtime_15_code_review_findings_structure_guard_typed_error_children_line_budgets_are_current",
                "runtime_15_code_review_findings_structure_guard_typed_error_folder_backed_status_is_current",
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "M3 review status map records typed-error structure guard folder-backed split",
        &status_map,
        &[
            STRUCTURE_GUARD_TYPED_ERROR_FOLDER_BACKED_SLICE,
            STRUCTURE_GUARD_TYPED_ERROR_FOLDER_BACKED_STATUS,
        ],
    );
    assert_contains_all(
        "M3 review date map records typed-error structure guard folder-backed split",
        &date_map,
        &[
            STRUCTURE_GUARD_TYPED_ERROR_FOLDER_BACKED_SLICE,
            "2026-07-03",
        ],
    );

    budgets::assert_typed_error_structure_guard_line_budgets();
}
