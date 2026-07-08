use super::super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_typed_error_moved_guard_absence_child_owner_route_split_status_is_current() {
    let status_rows = review_guard_status_rows_source();
    let status_map = review_guard_status_map_source();
    let date_map = review_guard_date_map_source();
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    let status_anchors = [
        TYPED_ERROR_MOVED_GUARD_ABSENCE_CHILD_OWNER_ROUTE_SLICE,
        TYPED_ERROR_MOVED_GUARD_ABSENCE_CHILD_OWNER_ROUTE_STATUS,
        TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD,
        TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD_OWNERSHIP_CHILD,
        TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD_OWNERSHIP_STATUS_CHILD,
        TYPED_ERROR_MOVED_GUARD_ABSENCE_ROOT_PATHS_CHILD,
        TYPED_ERROR_MOVED_GUARD_ABSENCE_ROOT_STATUSES_CHILD,
        TYPED_ERROR_MOVED_GUARD_ABSENCE_ROOT_CHILD_ROWS_CHILD,
        TYPED_ERROR_MOVED_GUARD_ABSENCE_ROOT_SOURCES_CHILD,
        "runtime_15_typed_error_structure_moved_guard_absence_is_child_owner",
        TYPED_ERROR_MOVED_GUARD_ABSENCE_CHILD_OWNER_ROUTE_STATUS_GUARD,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("typed-error structure row data", status_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("runtime architecture session note", session_note.as_str()),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "M3 review status map records moved-guard absence child-owner route split",
        &status_map,
        &[
            TYPED_ERROR_MOVED_GUARD_ABSENCE_CHILD_OWNER_ROUTE_SLICE,
            TYPED_ERROR_MOVED_GUARD_ABSENCE_CHILD_OWNER_ROUTE_STATUS,
        ],
    );
    assert_contains_all(
        "M3 review date map records moved-guard absence child-owner route split",
        &date_map,
        &[
            TYPED_ERROR_MOVED_GUARD_ABSENCE_CHILD_OWNER_ROUTE_SLICE,
            TYPED_ERROR_MOVED_GUARD_ABSENCE_CHILD_OWNER_ROUTE_DATE,
        ],
    );

    budgets::assert_typed_error_moved_guard_absence_line_budgets();
}
