use super::ExpectedStatusOutputSlice;

#[path = "foundation/core_rows.rs"]
mod core_rows;
#[path = "foundation/typed_error_plugin_rows.rs"]
mod typed_error_plugin_rows;
#[path = "foundation/typed_error_runtime_rows.rs"]
mod typed_error_runtime_rows;
#[path = "foundation/typed_error_scene_asset_rows.rs"]
mod typed_error_scene_asset_rows;

pub(super) const FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    core_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const FOUNDATION_TYPED_ERROR_RUNTIME_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = typed_error_runtime_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const FOUNDATION_TYPED_ERROR_PLUGIN_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = typed_error_plugin_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const FOUNDATION_TYPED_ERROR_SCENE_ASSET_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = typed_error_scene_asset_rows::EXPECTED_STATUS_OUTPUT_SLICES;
