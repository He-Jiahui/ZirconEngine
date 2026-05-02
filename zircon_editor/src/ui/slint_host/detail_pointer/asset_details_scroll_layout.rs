use zircon_runtime_interface::ui::layout::UiSize;

use crate::ui::workbench::snapshot::AssetSelectionSnapshot;

use super::asset_details_constants::ASSET_DETAILS_VIEWPORT_Y;
use super::asset_details_content_extent::asset_details_content_extent;
use super::scroll_surface_pointer_layout::ScrollSurfacePointerLayout;

pub(crate) fn asset_details_scroll_layout(
    pane_size: UiSize,
    selection: &AssetSelectionSnapshot,
) -> ScrollSurfacePointerLayout {
    ScrollSurfacePointerLayout {
        pane_size,
        viewport_origin_y: ASSET_DETAILS_VIEWPORT_Y,
        content_extent: asset_details_content_extent(selection),
    }
}
