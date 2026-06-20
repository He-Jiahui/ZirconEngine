use super::super::super::super::data::FrameRect;
use super::super::template_alert_glyphs::ALERT_ICON_SIZE;
use super::common::centered_rect;
use super::metrics::{ALERT_ICON_LEFT, ALERT_LINE_HEIGHT, ALERT_TEXT_GAP, ALERT_TEXT_RIGHT_INSET};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_icon_rect(
    rect: &FrameRect,
) -> FrameRect {
    centered_rect(rect, ALERT_ICON_LEFT, ALERT_ICON_SIZE, ALERT_ICON_SIZE)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_text_rect(
    rect: &FrameRect,
    icon: &FrameRect,
) -> Option<FrameRect> {
    let text_left = icon.x + icon.width + ALERT_TEXT_GAP;
    let text_right = rect.x + rect.width - ALERT_TEXT_RIGHT_INSET;
    (text_right > text_left).then(|| FrameRect {
        x: text_left,
        y: rect.y + (rect.height - ALERT_LINE_HEIGHT).max(0.0) * 0.5,
        width: text_right - text_left,
        height: ALERT_LINE_HEIGHT,
    })
}
