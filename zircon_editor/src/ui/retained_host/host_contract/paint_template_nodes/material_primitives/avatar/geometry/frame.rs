use crate::ui::retained_host::host_contract::data::FrameRect;

use super::metrics::AVATAR_DEFAULT_EDGE;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn avatar_frame(
    rect: &FrameRect,
) -> FrameRect {
    let size = rect.width.min(rect.height).min(AVATAR_DEFAULT_EDGE).round();
    let size = size.max(1.0);
    FrameRect {
        x: rect.x.round(),
        y: (rect.y + (rect.height - size).max(0.0) * 0.5).round(),
        width: size,
        height: size,
    }
}
