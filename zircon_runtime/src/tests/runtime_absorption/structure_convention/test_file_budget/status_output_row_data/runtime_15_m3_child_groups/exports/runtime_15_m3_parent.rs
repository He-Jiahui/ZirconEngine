use super::*;

#[test]
fn runtime_15_m3_child_groups_exports_runtime_15_m3_parent_is_child_owned() {
    let runtime_15_m3 = read_runtime_src(RUNTIME_15_M3_EXPECTED_STATUS_ROW_DATA_PATH);

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
            "pub(super) const STATUS_SUPPORT_ROW_DATA_AND_BUDGET_TEST_FILE_BUDGET_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const STATUS_SUPPORT_ROW_DATA_AND_BUDGET_RUNTIME_ROW_DATA_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const STATUS_SUPPORT_ROW_DATA_AND_BUDGET_HUB_EDITOR_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const STATUS_SUPPORT_ROW_DATA_AND_BUDGET_RENDER_SHADER_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const STATUS_SUPPORT_ROW_DATA_AND_BUDGET_M3_M4_ROW_DATA_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const PRODUCTION_GUARD_SUPPORT_CORE_AND_EVIDENCE_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const PRODUCTION_GUARD_SUPPORT_MODULE_LAYOUT_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const PRODUCTION_GUARD_SUPPORT_REVIEW_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const PRODUCTION_GUARD_SUPPORT_RUNTIME_ROW_DATA_FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const PRODUCTION_GUARD_SUPPORT_RUNTIME_ROW_DATA_LOCK_POISON_SCENE_SCRIPT_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const PRODUCTION_GUARD_SUPPORT_RUNTIME_ROW_DATA_STATUS_SUPPORT_PRIORITY_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const PRODUCTION_GUARD_SUPPORT_RUNTIME_ROW_DATA_ASSET_BUDGET_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const PRODUCTION_GUARD_SUPPORT_STATUS_DOCS_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const PRODUCTION_GUARD_SUPPORT_EXPECTED_SLICE_GUARDS_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
}
