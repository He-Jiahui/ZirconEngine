use crate::ui::retained_host::host_contract::data::HostResizeLayerData;

use super::super::super::{ChromePointerRoute, geometry::contains};

pub(super) fn route_resize_splitters(
    resize_layer: &HostResizeLayerData,
    x: f32,
    y: f32,
) -> Option<ChromePointerRoute> {
    for splitter in [
        &resize_layer.left_splitter_frame,
        &resize_layer.right_splitter_frame,
        &resize_layer.bottom_splitter_frame,
    ] {
        if contains(splitter, x, y) {
            return Some(ChromePointerRoute::Resize);
        }
    }
    None
}
