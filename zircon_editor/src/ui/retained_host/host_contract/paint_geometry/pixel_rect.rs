use super::super::data::FrameRect;
use super::frame::is_visible_frame;
use super::rect_ops::intersect;

pub(in crate::ui::retained_host::host_contract) struct PixelRect {
    pub(in crate::ui::retained_host::host_contract) x0: u32,
    pub(in crate::ui::retained_host::host_contract) y0: u32,
    pub(in crate::ui::retained_host::host_contract) x1: u32,
    pub(in crate::ui::retained_host::host_contract) y1: u32,
}

impl PixelRect {
    pub(in crate::ui::retained_host::host_contract) fn from_frame(
        frame: &FrameRect,
        clip: Option<&FrameRect>,
        width: u32,
        height: u32,
    ) -> Option<Self> {
        if width == 0 || height == 0 {
            return None;
        }
        let frame = match clip {
            Some(clip) => intersect(frame, clip)?,
            None if is_visible_frame(frame) => frame.clone(),
            None => return None,
        };

        let x0 = frame.x.floor().max(0.0).min(width as f32) as u32;
        let y0 = frame.y.floor().max(0.0).min(height as f32) as u32;
        let x1 = (frame.x + frame.width).ceil().max(0.0).min(width as f32) as u32;
        let y1 = (frame.y + frame.height).ceil().max(0.0).min(height as f32) as u32;
        (x0 < x1 && y0 < y1).then_some(Self { x0, y0, x1, y1 })
    }
}
