use super::super::super::super::data::FrameRect;

pub(in crate::ui::retained_host::host_contract) fn contains(
    frame: &FrameRect,
    x: f32,
    y: f32,
) -> bool {
    frame.width > 0.0
        && frame.height > 0.0
        && x >= frame.x
        && y >= frame.y
        && x < frame.x + frame.width
        && y < frame.y + frame.height
}
