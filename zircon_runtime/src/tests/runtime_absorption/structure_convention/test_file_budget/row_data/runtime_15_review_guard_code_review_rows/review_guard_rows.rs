use super::*;

const REVIEW_GUARD_ROW_GROUPS: &[(&str, &str)] = &[
    ("core_rows", "EXPECTED_STATUS_OUTPUT_SLICES"),
    ("p0_rows", "P0_EXPECTED_STATUS_OUTPUT_SLICES"),
    ("f8_rows", "F8_EXPECTED_STATUS_OUTPUT_SLICES"),
    ("late_api_rows", "LATE_API_EXPECTED_STATUS_OUTPUT_SLICES"),
    (
        "row_data_owner",
        "ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
    ),
];

#[test]
fn runtime_15_review_guard_rows_row_data_owner_is_child_backed() {
    let parent = read_runtime_src(REVIEW_GUARD_ROWS_PATH);
    let code_review_rows = read_runtime_src(CODE_REVIEW_ROWS_PATH);
    for (module, export) in REVIEW_GUARD_ROW_GROUPS {
        let mount = format!("#[path = \"review_guard_rows/{module}.rs\"]");
        let export = format!("review_guard_rows::{export}");
        assert_contains_all(
            "review-guard row parent mounts child rows",
            &parent,
            &[mount.as_str()],
        );
        assert_contains_all(
            "code-review row parent exports review-guard groups",
            &code_review_rows,
            &[export.as_str()],
        );
    }
    assert!(
        !parent.contains("Runtime 15 M3 P0 robustness review guard child-owner split"),
        "review_guard_rows.rs should route row groups instead of owning status row tuples",
    );

    assert_contains_all(
        "review-guard row children own representative rows",
        &review_guard_rows_source_blob(),
        &[
            "Runtime 15 M3 code review findings test folder split",
            "Runtime 15 M3 P0 robustness review guard child-owner split",
            "Runtime 15 M3 F8 API convergence review guard child-owner split",
            "Runtime 15 M3 late API cleanup review guard child-owner split",
            REVIEW_GUARD_ROWS_ROW_DATA_STATUS_ID,
            REVIEW_GUARD_ROWS_ROW_DATA_GUARD_NAME,
        ],
    );
    assert_review_guard_rows_row_data_status_is_current();
}

fn assert_review_guard_rows_row_data_status_is_current() {
    let row_data_owner = read_runtime_src(REVIEW_GUARD_ROW_DATA_OWNER_PATH);
    let status_map = read_runtime_src(REVIEW_GUARD_STATUS_MAP_PATH);
    let date_map = read_runtime_src(REVIEW_GUARD_DATE_MAP_PATH);
    let row_data_anchors = [
        REVIEW_GUARD_ROWS_ROW_DATA_STATUS_NAME,
        REVIEW_GUARD_ROWS_ROW_DATA_STATUS_ID,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/review_guard_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/review_guard_rows/core_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/review_guard_rows/p0_rows.rs",
        REVIEW_GUARD_ROWS_ROW_DATA_GUARD_NAME,
        "Cargo gate deferred",
    ];
    assert_contains_all(
        "review-guard row-data owner",
        &row_data_owner,
        &row_data_anchors,
    );

    let doc_anchors = [
        REVIEW_GUARD_ROWS_ROW_DATA_STATUS_NAME,
        REVIEW_GUARD_ROWS_ROW_DATA_STATUS_ID,
        REVIEW_GUARD_ROWS_PATH,
        REVIEW_GUARD_ROWS_ROW_DATA_GUARD_NAME,
        "Cargo gate deferred",
    ];
    for path in [
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
        "docs/plans/zircon_runtime/runtime/index.md",
        "docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md",
        "docs/plans/engine-code-review-findings-2026-06.md",
        "docs/plans/engine-code-structure-convention.md",
        "docs/zircon_runtime/structure/module-convention.md",
        ".codex/sessions/20260612-0847-runtime-architecture-implementation.md",
    ] {
        assert_contains_all(path, &read_repo(path), &doc_anchors);
    }
    assert_contains_all(
        "review guard status map records review-guard row-data split",
        &status_map,
        &[
            REVIEW_GUARD_ROWS_ROW_DATA_STATUS_NAME,
            REVIEW_GUARD_ROWS_ROW_DATA_STATUS_ID,
        ],
    );
    assert_contains_all(
        "review guard date map records review-guard row-data split",
        &date_map,
        &[REVIEW_GUARD_ROWS_ROW_DATA_STATUS_NAME, "2026-07-07"],
    );
}
