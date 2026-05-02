use zircon_runtime_interface::ui::layout::UiFrame;

use super::layout::AssetFolderTreePointerLayout;

const HEADER_HEIGHT: f32 = 48.0;
const VIEWPORT_Y: f32 = HEADER_HEIGHT + 1.0;
pub(super) const ROW_X: f32 = 8.0;
pub(super) const ROW_Y: f32 = 8.0;
pub(super) const ROW_HEIGHT: f32 = 28.0;
pub(super) const ROW_GAP: f32 = 4.0;
const ROW_WIDTH_INSET: f32 = 16.0;

pub(super) fn viewport_frame(layout: &AssetFolderTreePointerLayout) -> UiFrame {
    UiFrame::new(
        0.0,
        VIEWPORT_Y,
        layout.pane_size.width.max(0.0),
        (layout.pane_size.height - VIEWPORT_Y).max(0.0),
    )
}

pub(super) fn content_height(item_count: usize) -> f32 {
    if item_count == 0 {
        0.0
    } else {
        ROW_Y * 2.0 + item_count as f32 * ROW_HEIGHT + (item_count as f32 - 1.0) * ROW_GAP
    }
}

pub(super) fn row_width(layout: &AssetFolderTreePointerLayout) -> f32 {
    (layout.pane_size.width - ROW_WIDTH_INSET).max(0.0)
}

pub(super) fn viewport_y() -> f32 {
    VIEWPORT_Y
}
