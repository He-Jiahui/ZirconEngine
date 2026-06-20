use crate::ui::retained_host::host_contract::data::FrameRect;

use super::metrics::{ALERT_ICON_EDGE, ALERT_ICON_MARK_EDGE, ALERT_PADDING_X};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_icon_frame(
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x + ALERT_PADDING_X,
        y: rect.y + (rect.height - ALERT_ICON_EDGE).max(0.0) * 0.5,
        width: ALERT_ICON_EDGE,
        height: ALERT_ICON_EDGE,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_icon_mark_frame(
    frame: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: frame.x + (frame.width - ALERT_ICON_MARK_EDGE) * 0.5,
        y: frame.y + (frame.height - ALERT_ICON_MARK_EDGE) * 0.5,
        width: ALERT_ICON_MARK_EDGE,
        height: ALERT_ICON_MARK_EDGE,
    }
}
