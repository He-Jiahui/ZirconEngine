use super::*;

pub(super) fn assert_runtime_row_data_status_mirrors_are_current() {
    let status_rows = read_runtime_src(PRODUCTION_GUARD_SUPPORT_EXPECTED_SLICE_GUARDS_ROWS_PATH);
    let status_map = [
        read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH),
        read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PLAN_DOC_EXPECTED_SLICE_SUPPORT_PATH),
    ]
    .join("\n");
    let date_map = [
        read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH),
        read_runtime_src(STATUS_SUPPORT_DATE_MAP_PLAN_DOC_EXPECTED_SLICE_SUPPORT_PATH),
    ]
    .join("\n");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    let mut status_paths = vec![
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/runtime_row_data.rs",
    ];
    status_paths.extend(
        super::child_rows::RUNTIME_ROW_DATA_CHILD_ROWS
            .iter()
            .map(|(_, _, status_path, _, _)| *status_path),
    );

    let mut status_anchors = vec![
        PRODUCTION_GUARD_RUNTIME_ROW_DATA_CHILD_SPLIT_STATUS_NAME,
        PRODUCTION_GUARD_RUNTIME_ROW_DATA_CHILD_SPLIT_STATUS_ID,
        PRODUCTION_GUARD_RUNTIME_ROW_DATA_CHILD_SPLIT_GUARD_NAME,
        "Cargo gate deferred",
    ];
    status_anchors.extend(status_paths);
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
        ("production guard expected-slice rows", status_rows.as_str()),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "status map",
        &status_map,
        &[
            PRODUCTION_GUARD_RUNTIME_ROW_DATA_CHILD_SPLIT_STATUS_NAME,
            PRODUCTION_GUARD_RUNTIME_ROW_DATA_CHILD_SPLIT_STATUS_ID,
        ],
    );
    assert_contains_all(
        "date map",
        &date_map,
        &[
            PRODUCTION_GUARD_RUNTIME_ROW_DATA_CHILD_SPLIT_STATUS_NAME,
            "2026-07-04",
        ],
    );
}
