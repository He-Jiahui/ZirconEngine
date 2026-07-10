use super::*;

#[test]
fn runtime_15_status_support_expected_slice_row_data_status_mirrors_are_current() {
    let production_runtime_rows =
        read_runtime_src(PRODUCTION_GUARD_SUPPORT_STATUS_SUPPORT_PRIORITY_ROWS_PATH);
    let row_status_map = read_runtime_src(STATUS_SUPPORT_EXPECTED_SLICE_STATUS_MAP_PATH);
    let row_date_map = read_runtime_src(STATUS_SUPPORT_EXPECTED_SLICE_DATE_MAP_PATH);
    let guard_status_map = read_runtime_src(STATUS_SUPPORT_ROW_DATA_STATUS_MAP_PATH);
    let guard_date_map = read_runtime_src(STATUS_SUPPORT_ROW_DATA_DATE_MAP_PATH);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let frameworks_plan = read_repo(
        "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
    );
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    for (label, source) in [
        ("status rows", production_runtime_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("runtime index", runtime_index.as_str()),
        ("frameworks plan", frameworks_plan.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            child_sources::EXPECTED_SLICE_ROW_DATA_OWNER_STATUS_ANCHORS,
        );
        assert_contains_all(
            label,
            source,
            child_sources::EXPECTED_SLICE_ROW_DATA_GUARD_STATUS_ANCHORS,
        );
    }
    assert_contains_all(
        "status map records expected-slice row-data owner split",
        &row_status_map,
        &[
            EXPECTED_SLICE_ROW_DATA_OWNER_STATUS_NAME,
            EXPECTED_SLICE_ROW_DATA_OWNER_STATUS_ID,
        ],
    );
    assert_contains_all(
        "date map records expected-slice row-data owner split",
        &row_date_map,
        &[EXPECTED_SLICE_ROW_DATA_OWNER_STATUS_NAME, "2026-07-05"],
    );
    assert_contains_all(
        "status map records expected-slice row-data guard split",
        &guard_status_map,
        &[
            EXPECTED_SLICE_ROW_DATA_GUARD_STATUS_NAME,
            EXPECTED_SLICE_ROW_DATA_GUARD_STATUS_ID,
        ],
    );
    assert_contains_all(
        "date map records expected-slice row-data guard split",
        &guard_date_map,
        &[EXPECTED_SLICE_ROW_DATA_GUARD_STATUS_NAME, "2026-07-07"],
    );
}
