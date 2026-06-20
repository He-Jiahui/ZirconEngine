use super::ExpectedStatusOutputSlice;

#[path = "runtime_07/asset_render.rs"]
mod asset_render;
#[path = "runtime_07/owner_budget.rs"]
mod owner_budget;
#[path = "runtime_07/performance.rs"]
mod performance;
#[path = "runtime_07/scene_asset.rs"]
mod scene_asset;

pub(super) const RUNTIME_07_PERFORMANCE_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = performance::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_07_ASSET_RENDER_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = asset_render::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_07_SCENE_ASSET_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = scene_asset::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_07_OWNER_BUDGET_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = owner_budget::EXPECTED_STATUS_OUTPUT_SLICES;
