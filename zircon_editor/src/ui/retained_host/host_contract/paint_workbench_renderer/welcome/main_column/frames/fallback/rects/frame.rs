use crate::ui::retained_host::host_contract::data::FrameRect;

pub(super) fn fallback(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}
