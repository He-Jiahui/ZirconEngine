use super::ExpectedStatusOutputSlice;

#[path = "m3/asset_budget_tests.rs"]
mod asset_budget_tests;
#[path = "m3/foundation_guards.rs"]
mod foundation_guards;
#[path = "m3/production_guard_support.rs"]
mod production_guard_support;
#[path = "m3/scene_script_tests.rs"]
mod scene_script_tests;
#[path = "m3/status_support.rs"]
mod status_support;
#[path = "m3/ui_tests_first.rs"]
mod ui_tests_first;
#[path = "m3/ui_tests_second.rs"]
mod ui_tests_second;

pub(super) const FOUNDATION_GUARD_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    foundation_guards::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const UI_TESTS_FIRST_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    ui_tests_first::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ASSET_BUDGET_TESTS_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    asset_budget_tests::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const SCENE_SCRIPT_TESTS_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    scene_script_tests::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const STATUS_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    status_support::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const UI_TESTS_SECOND_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    ui_tests_second::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const PRODUCTION_GUARD_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = production_guard_support::EXPECTED_STATUS_OUTPUT_SLICES;
