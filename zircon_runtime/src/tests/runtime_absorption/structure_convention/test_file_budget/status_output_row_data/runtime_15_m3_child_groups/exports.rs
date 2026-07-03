use super::*;

#[test]
fn runtime_15_status_output_m3_row_data_child_owner_split() {
    let parent = read_runtime_src(TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15 = read_runtime_src(RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15_m3 = read_runtime_src(RUNTIME_15_M3_EXPECTED_STATUS_ROW_DATA_PATH);

    assert_contains_all(
        "top-level status rows include every Runtime 15 M3 child group",
        &parent,
        &[
            "runtime_15::RUNTIME_15_M3_FOUNDATION_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_LOCK_POISON_STATUS_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_MODULE_CONVENTION_STATUS_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_REVIEW_STATUS_SYNC_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_REVIEW_GUARD_CODE_REVIEW_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_REVIEW_GUARD_STATUS_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_REVIEW_GUARD_TYPED_ERROR_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_UI_TESTS_FIRST_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_ASSET_BUDGET_TESTS_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_SCENE_SCRIPT_TESTS_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_STATUS_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_UI_TESTS_SECOND_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_PRODUCTION_GUARD_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "Runtime 15 status row parent exposes M3 child groups",
        &runtime_15,
        &[
            "pub(super) const RUNTIME_15_M3_FOUNDATION_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
            "m3::FOUNDATION_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_M3_LOCK_POISON_STATUS_EXPECTED_STATUS_OUTPUT_SLICES",
            "m3::LOCK_POISON_STATUS_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_M3_MODULE_CONVENTION_STATUS_EXPECTED_STATUS_OUTPUT_SLICES",
            "m3::MODULE_CONVENTION_STATUS_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_M3_REVIEW_STATUS_SYNC_EXPECTED_STATUS_OUTPUT_SLICES",
            "m3::REVIEW_STATUS_SYNC_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_M3_REVIEW_GUARD_CODE_REVIEW_EXPECTED_STATUS_OUTPUT_SLICES",
            "m3::REVIEW_GUARD_CODE_REVIEW_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_M3_REVIEW_GUARD_STATUS_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
            "m3::REVIEW_GUARD_STATUS_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_M3_REVIEW_GUARD_TYPED_ERROR_EXPECTED_STATUS_OUTPUT_SLICES",
            "m3::REVIEW_GUARD_TYPED_ERROR_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_M3_STATUS_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
            "m3::STATUS_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_M3_PRODUCTION_GUARD_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
            "m3::PRODUCTION_GUARD_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "Runtime 15 M3 status row parent is a child-group aggregator",
        &runtime_15_m3,
        &[
            "#[path = \"m3/foundation_guards.rs\"]",
            "#[path = \"m3/lock_poison_status.rs\"]",
            "#[path = \"m3/module_convention_status.rs\"]",
            "#[path = \"m3/review_guard_splits.rs\"]",
            "#[path = \"m3/review_status_sync.rs\"]",
            "#[path = \"m3/status_support.rs\"]",
            "#[path = \"m3/ui_tests_second.rs\"]",
            "#[path = \"m3/production_guard_support.rs\"]",
            "pub(super) const FOUNDATION_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const LOCK_POISON_STATUS_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const MODULE_CONVENTION_STATUS_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const REVIEW_GUARD_CODE_REVIEW_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const REVIEW_GUARD_STATUS_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const REVIEW_GUARD_TYPED_ERROR_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const REVIEW_STATUS_SYNC_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const STATUS_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const PRODUCTION_GUARD_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
}
