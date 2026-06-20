use crate::ui::retained_host::host_contract::data::FrameRect;

use super::metrics::AVATAR_FALLBACK_SCALE;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn avatar_fallback_child_frame(
    rect: &FrameRect,
) -> FrameRect {
    centered_child_rect(rect, AVATAR_FALLBACK_SCALE)
}

fn centered_child_rect(rect: &FrameRect, scale: f32) -> FrameRect {
    let size = (rect.width.min(rect.height) * scale.clamp(0.0, 1.0)).max(1.0);
    FrameRect {
        x: rect.x + (rect.width - size) * 0.5,
        y: rect.y + (rect.height - size) * 0.5,
        width: size,
        height: size,
    }
}
