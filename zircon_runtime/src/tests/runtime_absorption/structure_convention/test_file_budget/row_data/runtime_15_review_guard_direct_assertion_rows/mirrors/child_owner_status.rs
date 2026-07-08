use super::*;

#[test]
fn runtime_15_review_guard_direct_assertion_child_owner_status_is_current() {
    let status_support_rows = read_runtime_src(STATUS_SUPPORT_ROWS_PATH);
    let review_expected_status_map = read_runtime_src(REVIEW_GUARD_STATUS_MAP_PATH);
    let review_expected_date_map = read_runtime_src(REVIEW_GUARD_DATE_MAP_PATH);
    let status_support_expected_status_map = read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH);
    let status_support_expected_date_map = read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH);

    assert_contains_all(
        "review expected-slice maps retain direct-assertion row statuses",
        &review_expected_status_map,
        &[
            "Runtime 15 M3 code review findings direct assertions child-owner split",
            "runtime_15_code_review_findings_direct_assertions_child_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 code review findings F12 direct assertions child-owner split",
            "runtime_15_code_review_findings_f12_direct_assertions_child_owner_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "review expected-slice date maps retain direct-assertion dates",
        &review_expected_date_map,
        &[
            "Runtime 15 M3 code review findings F12 direct assertions child-owner split",
            "2026-07-01",
            "Runtime 15 M3 code review findings render direct assertions child-owner split",
        ],
    );
    assert_contains_all(
        "status-support row data records direct-assertion row-data split",
        &status_support_rows,
        &[
            CHILD_OWNER_STATUS_NAME,
            CHILD_OWNER_STATUS_ID,
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/direct_assertion_rows.rs",
            CHILD_OWNER_GUARD_NAME,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "status-support expected-slice maps record direct-assertion row-data split",
        &status_support_expected_status_map,
        &[CHILD_OWNER_STATUS_NAME, CHILD_OWNER_STATUS_ID],
    );
    assert_contains_all(
        "status-support expected-slice dates record direct-assertion row-data split",
        &status_support_expected_date_map,
        &[CHILD_OWNER_STATUS_NAME, "2026-07-01"],
    );

    let child_owner_status_anchors = [
        CHILD_OWNER_STATUS_NAME,
        CHILD_OWNER_STATUS_ID,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/direct_assertion_rows.rs",
        CHILD_OWNER_GUARD_NAME,
    ];
    for (label, path) in [
        (
            "Runtime 15 plan",
            "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
        ),
        (
            "Runtime index",
            "docs/plans/zircon_runtime/runtime/index.md",
        ),
        (
            "review findings",
            "docs/plans/engine-code-review-findings-2026-06.md",
        ),
        (
            "structure convention",
            "docs/plans/engine-code-structure-convention.md",
        ),
        (
            "module convention doc",
            "docs/zircon_runtime/structure/module-convention.md",
        ),
    ] {
        let source = read_repo(path);
        assert_contains_all(label, &source, &child_owner_status_anchors);
    }
}
