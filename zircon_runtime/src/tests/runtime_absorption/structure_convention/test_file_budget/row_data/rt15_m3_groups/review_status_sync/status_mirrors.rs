use super::super::*;

const STATUS_ROW_DATA_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/status_row_data_support_maps.rs";
const STATUS_ROW_DATA_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/status_row_data_support_maps.rs";

pub(super) fn assert_review_status_sync_status_mirrors_are_current() {
    let status_map = read_runtime_src(STATUS_ROW_DATA_STATUS_MAP_PATH);
    let date_map = read_runtime_src(STATUS_ROW_DATA_DATE_MAP_PATH);
    assert_contains_all(
        "M3 status map records review status-sync row-data child split",
        &status_map,
        &[
            REVIEW_STATUS_SYNC_CHILD_SPLIT_STATUS_NAME,
            REVIEW_STATUS_SYNC_CHILD_SPLIT_STATUS_ID,
        ],
    );
    assert_contains_all(
        "M3 date map records review status-sync row-data child split",
        &date_map,
        &[REVIEW_STATUS_SYNC_CHILD_SPLIT_STATUS_NAME, "2026-07-07"],
    );

    for (label, source) in [
        (
            "Runtime 15 plan",
            read_repo(
                "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
            ),
        ),
        (
            "Runtime index",
            read_repo("docs/plans/zircon_runtime/runtime/index.md"),
        ),
        (
            "review findings",
            read_repo("docs/plans/engine-code-review-findings-2026-06.md"),
        ),
        (
            "structure convention",
            read_repo("docs/plans/engine-code-structure-convention.md"),
        ),
        (
            "module convention doc",
            read_repo("docs/zircon_runtime/structure/module-convention.md"),
        ),
        (
            "session note",
            read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md"),
        ),
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                REVIEW_STATUS_SYNC_CHILD_SPLIT_STATUS_NAME,
                REVIEW_STATUS_SYNC_CHILD_SPLIT_STATUS_ID,
                REVIEW_STATUS_SYNC_CHILD_SPLIT_GUARD_NAME,
            ],
        );
    }
}
