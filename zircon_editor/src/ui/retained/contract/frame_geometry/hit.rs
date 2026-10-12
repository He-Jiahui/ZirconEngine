use super::super::data::FrameRect;

pub(in crate::ui::retained_host::host_contract) fn contains_point(
    frame: &FrameRect,
    x: f32,
    y: f32,
) -> bool {
    x >= frame.x && x <= frame.x + frame.width && y >= frame.y && y <= frame.y + frame.height
}
