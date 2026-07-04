use super::super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_code_review_findings_f12_direct_assertions_guard_folder_backed_status_is_current() {
    let status_rows = read_runtime_src(REVIEW_GUARD_STATUS_ROWS_PATH);
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
        ("direct assertion row data", status_rows.as_str()),
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
                F12_DIRECT_ASSERTIONS_FOLDER_BACKED_SLICE,
                F12_DIRECT_ASSERTIONS_FOLDER_BACKED_STATUS,
                DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_CHILD,
                F12_DIRECT_ASSERTIONS_CHILD,
                F12_DIRECT_ASSERTIONS_DELEGATION_CHILD,
                F12_DIRECT_ASSERTIONS_REVIEW_GUARD_CHILD,
                F12_DIRECT_ASSERTIONS_BUDGETS_CHILD,
                F12_DIRECT_ASSERTIONS_STATUS_MIRRORS_CHILD,
                "assert_f12_direct_sources_are_folder_backed",
                F12_DIRECT_ASSERTIONS_FOLDER_BACKED_GUARD,
                F12_DIRECT_ASSERTIONS_STATUS_GUARD,
                F12_DIRECT_ASSERTIONS_BUDGET_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "M3 review status map records F12 direct assertions folder-backed split",
        &status_map,
        &[
            F12_DIRECT_ASSERTIONS_FOLDER_BACKED_SLICE,
            F12_DIRECT_ASSERTIONS_FOLDER_BACKED_STATUS,
        ],
    );
    assert_contains_all(
        "M3 review date map records F12 direct assertions folder-backed split",
        &date_map,
        &[
            F12_DIRECT_ASSERTIONS_FOLDER_BACKED_SLICE,
            F12_DIRECT_ASSERTIONS_FOLDER_BACKED_DATE,
        ],
    );
}
