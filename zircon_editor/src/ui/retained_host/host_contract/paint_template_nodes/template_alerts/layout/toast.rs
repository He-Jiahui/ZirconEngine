use super::super::super::super::data::FrameRect;
use super::common::centered_rect;
use super::metrics::{
    TOAST_ACTION_GAP, TOAST_ACTION_MIN_WIDTH, TOAST_ACTION_WIDTH, TOAST_CLOSE_SIZE,
    TOAST_ICON_LEFT, TOAST_LINE_HEIGHT, TOAST_TEXT_GAP, TOAST_TRAILING_INSET,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn toast_icon_rect(
    rect: &FrameRect,
    icon_size: f32,
) -> FrameRect {
    centered_rect(rect, TOAST_ICON_LEFT, icon_size, icon_size)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn toast_close_rect(
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x + rect.width - TOAST_TRAILING_INSET - TOAST_CLOSE_SIZE,
        y: rect.y + (rect.height - TOAST_CLOSE_SIZE).max(0.0) * 0.5,
        width: TOAST_CLOSE_SIZE,
        height: TOAST_CLOSE_SIZE,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn toast_has_action(
    rect: &FrameRect,
) -> bool {
    rect.width >= TOAST_ACTION_MIN_WIDTH
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn toast_action_rect(
    rect: &FrameRect,
    close: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: close.x - TOAST_ACTION_WIDTH,
        y: rect.y + (rect.height - TOAST_LINE_HEIGHT).max(0.0) * 0.5,
        width: TOAST_ACTION_WIDTH,
        height: TOAST_LINE_HEIGHT,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn toast_text_rect(
    rect: &FrameRect,
    icon: &FrameRect,
    close: &FrameRect,
    has_action: bool,
) -> Option<FrameRect> {
    let action_left = close.x - TOAST_ACTION_WIDTH;
    let text_right = if has_action {
        action_left - TOAST_ACTION_GAP
    } else {
        rect.x + rect.width - TOAST_TRAILING_INSET
    };
    let text_left = icon.x + icon.width + TOAST_TEXT_GAP;
    (text_right > text_left).then(|| FrameRect {
        x: text_left,
        y: rect.y + (rect.height - TOAST_LINE_HEIGHT).max(0.0) * 0.5,
        width: text_right - text_left,
        height: TOAST_LINE_HEIGHT,
    })
}
