use zircon_ui::UiFrame;

use super::layout::AssetContentListPointerLayout;
use crate::ui::slint_host::asset_pointer::AssetListViewMode;

const HEADER_HEIGHT: f32 = 52.0;
const VIEWPORT_Y: f32 = HEADER_HEIGHT + 1.0;
pub(super) const ROW_X: f32 = 8.0;
pub(super) const ROW_Y: f32 = 8.0;
pub(super) const ROW_GAP: f32 = 8.0;
const ROW_WIDTH_INSET: f32 = 16.0;

pub(super) fn viewport_frame(layout: &AssetContentListPointerLayout) -> UiFrame {
    UiFrame::new(
        0.0,
        VIEWPORT_Y,
        layout.pane_size.width.max(0.0),
        (layout.pane_size.height - VIEWPORT_Y).max(0.0),
    )
}

pub(super) fn list_height(layout: &AssetContentListPointerLayout) -> f32 {
    let row_count = layout.folder_ids.len() + layout.item_ids.len();
    if row_count == 0 {
        return 0.0;
    }

    let folder_height = folder_height(layout.view_mode);
    let item_height = item_height(layout.view_mode);
    ROW_Y * 2.0
        + layout.folder_ids.len() as f32 * folder_height
        + layout.item_ids.len() as f32 * item_height
        + (row_count as f32 - 1.0) * ROW_GAP
}

pub(super) fn folder_height(view_mode: AssetListViewMode) -> f32 {
    match view_mode {
        AssetListViewMode::List => 32.0,
        AssetListViewMode::Thumbnail => 60.0,
    }
}

pub(super) fn item_height(view_mode: AssetListViewMode) -> f32 {
    match view_mode {
        AssetListViewMode::List => 38.0,
        AssetListViewMode::Thumbnail => 88.0,
    }
}

pub(super) fn row_width(layout: &AssetContentListPointerLayout) -> f32 {
    (layout.pane_size.width - ROW_WIDTH_INSET).max(0.0)
}

pub(super) fn viewport_y() -> f32 {
    VIEWPORT_Y
}
