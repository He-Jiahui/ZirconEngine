use crate::ui::retained_host::host_contract::data::FrameRect;

pub(super) fn viewport_body_frame_below_toolbar(
    content: &FrameRect,
    toolbar_height: f32,
) -> FrameRect {
    let mut body = content.clone();
    body.y += toolbar_height;
    body.height = (body.height - toolbar_height).max(0.0);
    body
}
