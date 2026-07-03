use super::super::data::FrameRect;
use super::bounds::valid_bounds;
use super::metrics::{dropdown_option_row_height, TEMPLATE_POPUP_ANCHOR_GAP};
use crate::ui::retained_host::popup_anchor_metrics::clamp_popup_x_to_bounds;

pub(crate) fn dropdown_option_popup_frame(
    control_frame: &FrameRect,
    row_count: usize,
) -> Option<FrameRect> {
    if row_count == 0 {
        return None;
    }
    let row_height = dropdown_option_row_height(control_frame);
    Some(FrameRect {
        x: control_frame.x,
        y: control_frame.y + control_frame.height + TEMPLATE_POPUP_ANCHOR_GAP,
        width: control_frame.width.max(1.0),
        height: row_height * row_count as f32,
    })
}

pub(crate) fn dropdown_option_popup_frame_within(
    control_frame: &FrameRect,
    row_count: usize,
    bounds: &FrameRect,
) -> Option<FrameRect> {
    let mut popup = dropdown_option_popup_frame(control_frame, row_count)?;
    if !valid_bounds(bounds) {
        return Some(popup);
    }

    let below_y = control_frame.y + control_frame.height + TEMPLATE_POPUP_ANCHOR_GAP;
    let above_y = control_frame.y - TEMPLATE_POPUP_ANCHOR_GAP - popup.height;
    let bounds_bottom = bounds.y + bounds.height;
    if below_y + popup.height > bounds_bottom && above_y >= bounds.y {
        popup.y = above_y;
    }

    let popup_width = popup.width.min(bounds.width.max(1.0)).max(1.0);
    popup.x = clamp_popup_x_to_bounds(popup.x, bounds.x, bounds.width, popup_width);
    popup.width = popup_width;
    Some(popup)
}

pub(crate) fn dropdown_option_row_frame(control_frame: &FrameRect, row: usize) -> FrameRect {
    let row_height = dropdown_option_row_height(control_frame);
    FrameRect {
        x: control_frame.x,
        y: control_frame.y
            + control_frame.height
            + TEMPLATE_POPUP_ANCHOR_GAP
            + row as f32 * row_height,
        width: control_frame.width.max(1.0),
        height: row_height,
    }
}

pub(crate) fn dropdown_option_row_frame_within(
    control_frame: &FrameRect,
    row_count: usize,
    row: usize,
    bounds: &FrameRect,
) -> Option<FrameRect> {
    if row >= row_count {
        return None;
    }
    let popup = dropdown_option_popup_frame_within(control_frame, row_count, bounds)?;
    let row_height = dropdown_option_row_height(control_frame);
    Some(FrameRect {
        x: popup.x,
        y: popup.y + row as f32 * row_height,
        width: popup.width,
        height: row_height,
    })
}
