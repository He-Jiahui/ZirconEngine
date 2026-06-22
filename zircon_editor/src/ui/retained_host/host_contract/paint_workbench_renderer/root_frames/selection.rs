use super::super::super::data::{HostWindowLayoutData, HostWindowPresentationData};
use super::super::super::paint_geometry::is_visible_frame;

pub(super) fn selected_root_layout(
    presentation: &HostWindowPresentationData,
) -> &HostWindowLayoutData {
    let scene_layout = &presentation.host_scene_data.layout;
    if has_visible_root_frame(scene_layout) {
        scene_layout
    } else {
        &presentation.host_layout
    }
}

fn has_visible_root_frame(layout: &HostWindowLayoutData) -> bool {
    is_visible_frame(&layout.center_band_frame)
        || is_visible_frame(&layout.status_bar_frame)
        || is_visible_frame(&layout.document_region_frame)
        || is_visible_frame(&layout.viewport_content_frame)
}
