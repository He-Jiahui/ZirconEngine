pub(super) type Slice = super::ExpectedStatusOutputSlice;

#[path = "naming_graphics_misc/graphics_asset_rows.rs"]
mod graphics_asset_rows;
#[path = "naming_graphics_misc/plugin_banned_rows.rs"]
mod plugin_banned_rows;
#[path = "naming_graphics_misc/root_route_rows.rs"]
mod root_route_rows;
#[path = "naming_graphics_misc/row_data_owner.rs"]
mod row_data_owner;
#[path = "naming_graphics_misc/scene_platform_rows.rs"]
mod scene_platform_rows;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    root_route_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const GRAPHICS_ASSET_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    graphics_asset_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const SCENE_PLATFORM_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    scene_platform_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const PLUGIN_BANNED_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    plugin_banned_rows::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES;
