use super::super::super::super::data::{FrameRect, HostWindowPresentationData};
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_template_nodes::{draw_template_nodes, has_template_nodes};
use super::super::super::root_frames::zero_origin;

pub(in super::super) fn draw_profiled_root_template_overlay(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    zircon_runtime::profile_scope!("editor", "host_painter", "painter_root_template_overlay");
    draw_root_template_overlay(frame, presentation);
}

pub(super) fn draw_root_template_overlay(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    if !has_template_nodes(&presentation.root_template_nodes) {
        return;
    }

    let frame_bounds = frame_bounds(frame);
    draw_template_nodes(
        frame,
        &presentation.root_template_nodes,
        &zero_origin(),
        &frame_bounds,
        None,
    );
}

pub(super) fn frame_bounds(frame: &HostRgbaFrame) -> FrameRect {
    FrameRect {
        x: 0.0,
        y: 0.0,
        width: frame.width() as f32,
        height: frame.height() as f32,
    }
}
