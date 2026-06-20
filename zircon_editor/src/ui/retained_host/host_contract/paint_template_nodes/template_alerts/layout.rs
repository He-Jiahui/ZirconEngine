mod common;
mod inline;
mod metrics;
mod toast;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use common::pixel_aligned_rect;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use inline::{
    alert_icon_rect, alert_text_rect,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::{
    ALERT_BORDER_WIDTH, ALERT_FONT_SIZE, ALERT_LINE_HEIGHT, ALERT_RADIUS, TOAST_FONT_SIZE,
    TOAST_ICON_SIZE, TOAST_LINE_HEIGHT, TOAST_RADIUS,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use toast::{
    toast_action_rect, toast_close_rect, toast_has_action, toast_icon_rect, toast_text_rect,
};
