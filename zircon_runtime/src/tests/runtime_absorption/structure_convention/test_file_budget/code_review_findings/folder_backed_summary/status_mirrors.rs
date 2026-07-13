use super::super::super::*;
use super::*;

#[test]
fn runtime_15_code_review_findings_folder_backed_summary_guard_folder_backed_status_is_current() {
    let status_rows = review_guard_status_rows_source();
    let status_map = read_runtime_src(REVIEW_GUARD_STATUS_MAP_PATH);
    let date_map = read_runtime_src(REVIEW_GUARD_DATE_MAP_PATH);
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
                FOLDER_BACKED_SUMMARY_GUARD_SPLIT_NAME,
                FOLDER_BACKED_SUMMARY_GUARD_SPLIT_ID,
                FOLDER_BACKED_SUMMARY_CHILD,
                "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/delegation.rs",
                "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/child_ownership.rs",
                "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/status_mirrors.rs",
                "runtime_15_code_review_findings_folder_backed_summary_is_child_owner",
                "runtime_15_code_review_findings_folder_backed_summary_children_are_child_owned",
                "runtime_15_code_review_findings_folder_backed_summary_guard_folder_backed_status_is_current",
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "M3 review status map records folder-backed summary guard split",
        &status_map,
        &[
            FOLDER_BACKED_SUMMARY_GUARD_SPLIT_NAME,
            FOLDER_BACKED_SUMMARY_GUARD_SPLIT_ID,
        ],
    );
    assert_contains_all(
        "M3 review date map records folder-backed summary guard split",
        &date_map,
        &[FOLDER_BACKED_SUMMARY_GUARD_SPLIT_NAME, "2026-07-02"],
    );

    let parent = read_runtime_src(FOLDER_BACKED_SUMMARY_CHILD);
    for (path, source) in folder_backed_summary_guard_child_sources()
        .into_iter()
        .chain(direct_review_assertions::direct_review_assertion_child_sources())
        .chain([
            (FOLDER_BACKED_SUMMARY_CHILD, parent),
            (
                DIRECT_REVIEW_ASSERTIONS_CHILD,
                read_runtime_src(DIRECT_REVIEW_ASSERTIONS_CHILD),
            ),
            (
                SOURCE_INVENTORY_CHILD,
                read_runtime_src(SOURCE_INVENTORY_CHILD),
            ),
        ])
    {
        let line_count = source.lines().count();
        assert!(
            line_count < CODE_REVIEW_FINDINGS_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
