use super::super::super::data::{FrameRect, HostWindowPresentationData};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::{intersect, is_visible_frame};
use super::super::super::paint_primitives::draw_rect;
use super::super::super::paint_theme::current_host_palette;

pub(super) fn draw_resize_layer(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    zircon_runtime::profile_scope!("editor", "host_painter", "painter_resize_layer");
    let resize = &presentation.host_scene_data.resize_layer;
    let splitter_color = current_host_palette().separator_strong;
    for splitter in [
        &resize.left_splitter_frame,
        &resize.right_splitter_frame,
        &resize.bottom_splitter_frame,
    ] {
        if is_visible_frame(splitter)
            && frame
                .paint_clip()
                .is_none_or(|damage| intersect(splitter, damage).is_some())
        {
            draw_rect(frame, splitter_visual_frame(splitter), splitter_color);
        }
    }
}

fn splitter_visual_frame(hit_frame: &FrameRect) -> FrameRect {
    if hit_frame.width <= hit_frame.height {
        FrameRect {
            x: (hit_frame.x + (hit_frame.width - 1.0) * 0.5).floor(),
            y: hit_frame.y,
            width: 1.0,
            height: hit_frame.height,
        }
    } else {
        FrameRect {
            x: hit_frame.x,
            y: (hit_frame.y + (hit_frame.height - 1.0) * 0.5).floor(),
            width: hit_frame.width,
            height: 1.0,
        }
    }
}

#[cfg(test)]
#[path = "tests/resize.rs"]
mod tests;
