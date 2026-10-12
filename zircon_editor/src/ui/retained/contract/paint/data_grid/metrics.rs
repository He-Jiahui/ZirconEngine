use super::super::super::super::data::FrameRect;

const MUI_X_HEADER_HEIGHT_FRACTION: f32 = 0.32;
const MUI_X_ROW_HEIGHT_FRACTION: f32 = 0.22;

pub(super) const MUI_X_DATA_GRID_ROW_COUNT: i32 = 2;

pub(super) fn data_grid_header_height(rect: &FrameRect) -> f32 {
    (rect.height * MUI_X_HEADER_HEIGHT_FRACTION).max(8.0)
}

pub(super) fn data_grid_row_height(rect: &FrameRect) -> f32 {
    (rect.height * MUI_X_ROW_HEIGHT_FRACTION).max(6.0)
}
