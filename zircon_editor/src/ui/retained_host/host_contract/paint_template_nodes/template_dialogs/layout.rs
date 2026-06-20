use super::super::super::data::FrameRect;

const DIALOG_PADDING_X: f32 = 20.0;
const DIALOG_TITLE_TOP: f32 = 18.0;
const DIALOG_BODY_TOP: f32 = 48.0;
const DIALOG_TITLE_LINE_HEIGHT: f32 = 18.0;
const DIALOG_BODY_LINE_HEIGHT: f32 = 16.0;
const CONFIRM_SEVERITY_MARK_WIDTH: f32 = 4.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn pixel_aligned_rect(
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x.round(),
        y: rect.y.round(),
        width: rect.width.round().max(1.0),
        height: rect.height.round().max(1.0),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dialog_has_visible_area(
    rect: &FrameRect,
) -> bool {
    rect.width > 1.0 && rect.height > 1.0
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn title_rect(
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: content_left(rect),
        y: rect.y + DIALOG_TITLE_TOP,
        width: content_width(rect),
        height: DIALOG_TITLE_LINE_HEIGHT,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn body_rect(
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: content_left(rect),
        y: rect.y + DIALOG_BODY_TOP,
        width: content_width(rect),
        height: DIALOG_BODY_LINE_HEIGHT,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn severity_mark_rect(
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x,
        y: rect.y,
        width: CONFIRM_SEVERITY_MARK_WIDTH,
        height: rect.height,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn action_right(
    rect: &FrameRect,
) -> f32 {
    rect.x + rect.width - DIALOG_PADDING_X
}

fn content_left(rect: &FrameRect) -> f32 {
    rect.x + DIALOG_PADDING_X
}

fn content_width(rect: &FrameRect) -> f32 {
    (rect.width - DIALOG_PADDING_X * 2.0).max(1.0)
}
