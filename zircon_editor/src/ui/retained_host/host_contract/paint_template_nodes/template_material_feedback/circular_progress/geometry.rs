use crate::ui::retained_host::host_contract::data::FrameRect;

pub(super) fn circular_progress_rect(rect: &FrameRect) -> FrameRect {
    let size = rect.width.min(rect.height).max(1.0);
    FrameRect {
        x: rect.x + (rect.width - size) * 0.5,
        y: rect.y + (rect.height - size) * 0.5,
        width: size,
        height: size,
    }
}
