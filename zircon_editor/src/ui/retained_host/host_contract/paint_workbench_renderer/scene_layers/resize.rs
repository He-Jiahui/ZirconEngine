use super::super::super::data::HostWindowPresentationData;
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::is_visible_frame;
use super::super::super::paint_primitives::draw_rect;

pub(super) fn draw_resize_layer(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    zircon_runtime::profile_scope!("editor", "host_painter", "painter_resize_layer");
    let resize = &presentation.host_scene_data.resize_layer;
    for splitter in [
        &resize.left_splitter_frame,
        &resize.right_splitter_frame,
        &resize.bottom_splitter_frame,
    ] {
        if is_visible_frame(splitter) {
            draw_rect(frame, splitter.clone(), [79, 92, 112, 255]);
        }
    }
}
