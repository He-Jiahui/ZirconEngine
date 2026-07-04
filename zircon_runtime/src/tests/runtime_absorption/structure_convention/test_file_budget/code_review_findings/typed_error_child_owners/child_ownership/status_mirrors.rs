use super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_typed_error_child_ownership_guard_folder_backed_status_is_current() {
    let status_row_parent = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows.rs",
    );
    let status_row_child = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/child_ownership.rs",
    );
    let status_rows = format!("{status_row_parent}\n{status_row_child}");
    let status_map = read_runtime_src(REVIEW_GUARD_STATUS_MAP);
    let date_map = read_runtime_src(REVIEW_GUARD_DATE_MAP);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    for (label, source) in [
        ("typed-error row data", status_rows.as_str()),
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
                TYPED_ERROR_CHILD_OWNERSHIP_FOLDER_BACKED_SLICE,
                TYPED_ERROR_CHILD_OWNERSHIP_FOLDER_BACKED_STATUS,
                TYPED_ERROR_CHILD_OWNERSHIP_CHILD,
                TYPED_ERROR_CHILD_OWNERSHIP_BUDGETS_CHILD,
                TYPED_ERROR_CHILD_OWNERSHIP_DELEGATION_CHILD,
                TYPED_ERROR_CHILD_OWNERSHIP_REVIEW_GUARDS_CHILD,
                TYPED_ERROR_CHILD_OWNERSHIP_STATUS_MIRRORS_CHILD,
                TYPED_ERROR_CHILD_OWNERSHIP_STRUCTURE_SUBTREE_CHILD,
                GUARD,
                TYPED_ERROR_CHILD_OWNERSHIP_FOLDER_BACKED_GUARD,
                TYPED_ERROR_CHILD_OWNERSHIP_STATUS_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "M3 review status map records typed-error child-ownership split",
        &status_map,
        &[
            TYPED_ERROR_CHILD_OWNERSHIP_FOLDER_BACKED_SLICE,
            TYPED_ERROR_CHILD_OWNERSHIP_FOLDER_BACKED_STATUS,
        ],
    );
    assert_contains_all(
        "M3 review date map records typed-error child-ownership split",
        &date_map,
        &[
            TYPED_ERROR_CHILD_OWNERSHIP_FOLDER_BACKED_SLICE,
            TYPED_ERROR_CHILD_OWNERSHIP_FOLDER_BACKED_DATE,
        ],
    );
}
