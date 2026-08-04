use super::super::data::FrameRect;
use super::bounds::valid_bounds;
use super::metrics::{TEMPLATE_POPUP_ANCHOR_GAP, dropdown_option_row_height};
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

    let bounds_bottom = bounds.y + bounds.height;
    let below_y = control_frame.y + control_frame.height + TEMPLATE_POPUP_ANCHOR_GAP;
    let below_start = below_y.max(bounds.y);
    let below_space = (bounds_bottom - below_start).max(0.0);
    let above_space = (control_frame.y - TEMPLATE_POPUP_ANCHOR_GAP - bounds.y).max(0.0);
    let opens_above = below_space < popup.height && above_space > below_space;
    let available_height = if opens_above {
        above_space
    } else {
        below_space
    }
    .min(bounds.height);
    if available_height <= 0.0 {
        return None;
    }
    popup.height = popup.height.min(available_height);
    let popup_y = if opens_above {
        control_frame.y - TEMPLATE_POPUP_ANCHOR_GAP - popup.height
    } else {
        below_start
    };
    popup.y = popup_y.clamp(bounds.y, bounds_bottom - popup.height);

    let popup_width = popup.width.min(bounds.width);
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
    let row_y = popup.y + row as f32 * row_height;
    if row_y + row_height > popup.y + popup.height {
        return None;
    }
    Some(FrameRect {
        x: popup.x,
        y: row_y,
        width: popup.width,
        height: row_height,
    })
}
