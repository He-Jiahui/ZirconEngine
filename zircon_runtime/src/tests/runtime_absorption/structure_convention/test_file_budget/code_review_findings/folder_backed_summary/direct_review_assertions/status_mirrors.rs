use super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_code_review_findings_direct_assertions_guard_folder_backed_status_is_current() {
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
                DIRECT_REVIEW_ASSERTIONS_GUARD_SPLIT_NAME,
                DIRECT_REVIEW_ASSERTIONS_GUARD_SPLIT_ID,
                DIRECT_REVIEW_ASSERTIONS_CHILD,
                DIRECT_REVIEW_ASSERTIONS_DELEGATION_CHILD,
                DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_CHILD,
                DIRECT_REVIEW_ASSERTIONS_STATUS_MIRRORS_CHILD,
                F12_DIRECT_ASSERTIONS_CHILD,
                F8_DIRECT_ASSERTIONS_CHILD,
                P0_DIRECT_ASSERTIONS_CHILD,
                RENDER_DIRECT_ASSERTIONS_CHILD,
                ROOT_PARENT_DIRECT_ASSERTIONS_CHILD,
                "runtime_15_code_review_findings_direct_assertions_are_child_owner",
                "runtime_15_code_review_findings_direct_assertions_children_are_child_owned",
                "runtime_15_code_review_findings_direct_assertions_guard_folder_backed_status_is_current",
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "M3 review status map records direct assertions folder-backed split",
        &status_map,
        &[
            DIRECT_REVIEW_ASSERTIONS_GUARD_SPLIT_NAME,
            DIRECT_REVIEW_ASSERTIONS_GUARD_SPLIT_ID,
        ],
    );
    assert_contains_all(
        "M3 review date map records direct assertions folder-backed split",
        &date_map,
        &[DIRECT_REVIEW_ASSERTIONS_GUARD_SPLIT_NAME, "2026-07-02"],
    );

    let parent = read_runtime_src(DIRECT_REVIEW_ASSERTIONS_CHILD);
    for (path, source) in [(DIRECT_REVIEW_ASSERTIONS_CHILD, parent)]
        .into_iter()
        .chain(direct_review_assertion_child_sources())
    {
        let line_count = source.lines().count();
        assert!(
            line_count < CODE_REVIEW_FINDINGS_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
