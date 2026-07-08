use super::super::super::super::super::*;
use super::super::{
    direct_review_date_map_source, direct_review_status_map_source,
    direct_review_status_rows_source,
};
use super::*;

#[test]
fn runtime_15_code_review_findings_p0_direct_assertions_guard_folder_backed_status_is_current() {
    let status_rows = direct_review_status_rows_source();
    let status_map = direct_review_status_map_source();
    let date_map = direct_review_date_map_source();
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
                P0_DIRECT_ASSERTIONS_FOLDER_BACKED_SLICE,
                P0_DIRECT_ASSERTIONS_FOLDER_BACKED_STATUS,
                P0_DIRECT_ASSERTIONS_CHILD,
                P0_DIRECT_ASSERTIONS_DELEGATION_CHILD,
                P0_DIRECT_ASSERTIONS_PARENT_MOUNTS_CHILD,
                P0_DIRECT_ASSERTIONS_REVIEW_CHILDREN_CHILD,
                P0_DIRECT_ASSERTIONS_BUDGETS_CHILD,
                P0_DIRECT_ASSERTIONS_STATUS_MIRRORS_CHILD,
                "assert_p0_direct_sources_are_folder_backed",
                P0_DIRECT_ASSERTIONS_FOLDER_BACKED_GUARD,
                P0_DIRECT_ASSERTIONS_STATUS_GUARD,
                P0_DIRECT_ASSERTIONS_BUDGET_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "M3 review status map records P0 direct assertions folder-backed split",
        &status_map,
        &[
            P0_DIRECT_ASSERTIONS_FOLDER_BACKED_SLICE,
            P0_DIRECT_ASSERTIONS_FOLDER_BACKED_STATUS,
        ],
    );
    assert_contains_all(
        "M3 review date map records P0 direct assertions folder-backed split",
        &date_map,
        &[
            P0_DIRECT_ASSERTIONS_FOLDER_BACKED_SLICE,
            P0_DIRECT_ASSERTIONS_FOLDER_BACKED_DATE,
        ],
    );
}
