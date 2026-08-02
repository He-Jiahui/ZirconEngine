mod common;
mod metrics;
mod panel;
mod row;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use common::pixel_aligned_rect;
pub(super) use metrics::{NotificationCenterMetrics, notification_center_metrics};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use panel::{
    empty_text_rect, header_rect,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use row::{
    mark_rect, message_rect, row_rect, row_text_width, title_rect,
};
