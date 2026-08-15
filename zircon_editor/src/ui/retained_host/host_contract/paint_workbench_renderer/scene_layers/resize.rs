use super::super::super::data::{FrameRect, HostWindowPresentationData};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::{intersect, is_visible_frame};
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
        if is_visible_frame(splitter)
            && frame
                .paint_clip()
                .is_none_or(|damage| intersect(splitter, damage).is_some())
        {
            draw_rect(frame, splitter_visual_frame(splitter), [42, 50, 56, 255]);
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
mod tests {
    use super::*;

    #[test]
    fn splitter_visual_keeps_large_hit_target_but_draws_one_pixel_rule() {
        let vertical = splitter_visual_frame(&FrameRect {
            x: 260.0,
            y: 72.0,
            width: 8.0,
            height: 378.0,
        });
        assert_eq!((vertical.x, vertical.y), (263.0, 72.0));
        assert_eq!((vertical.width, vertical.height), (1.0, 378.0));

        let horizontal = splitter_visual_frame(&FrameRect {
            x: 0.0,
            y: 450.0,
            width: 900.0,
            height: 8.0,
        });
        assert_eq!((horizontal.x, horizontal.y), (0.0, 453.0));
        assert_eq!((horizontal.width, horizontal.height), (900.0, 1.0));
    }
}
