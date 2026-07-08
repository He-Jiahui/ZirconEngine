use super::*;

#[test]
fn runtime_15_review_guard_row_data_status_doc_review_guard_status_is_current() {
    let review_guard_status_support_rows = review_guard_status_support_source_blob();

    assert_contains_all(
        "Runtime 15 M3 review-guard row-data child records the original split status",
        &review_guard_status_support_rows,
        &[
            "Runtime 15 M3 review guard status row-data child-owner split",
            "runtime_15_review_guard_status_row_data_child_owner_split_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits.rs",
            REVIEW_GUARD_CHILD_OWNER_GUARD_NAME,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "Runtime 15 M3 review-guard row-data support child records topic split",
        &review_guard_status_support_rows,
        &[
            TOPIC_CHILD_OWNER_STATUS_NAME,
            TOPIC_CHILD_OWNER_STATUS_ID,
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/typed_error_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows.rs",
            REVIEW_GUARD_CHILD_OWNER_GUARD_NAME,
            "Cargo gate deferred",
        ],
    );

    let historical_status_anchors = [
        REVIEW_GUARD_CHILD_OWNER_STATUS_NAME,
        REVIEW_GUARD_CHILD_OWNER_STATUS_ID,
        "structure_convention/test_file_budget/row_data/runtime_15_row_data.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data.rs",
        REVIEW_GUARD_CHILD_OWNER_GUARD_NAME,
    ];
    let topic_status_anchors = [
        TOPIC_CHILD_OWNER_STATUS_NAME,
        TOPIC_CHILD_OWNER_STATUS_ID,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/typed_error_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows.rs",
        REVIEW_GUARD_CHILD_OWNER_GUARD_NAME,
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
        assert_contains_all(label, &source, &historical_status_anchors);
        assert_contains_all(label, &source, &topic_status_anchors);
    }
}
