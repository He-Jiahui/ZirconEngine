mod capture_frame;

pub(in crate::graphics::runtime::render_framework) use capture_frame::{
    capture_frame, capture_frame_if_newer, capture_scene_color_hdr, poll_captured_frame_if_newer,
};
