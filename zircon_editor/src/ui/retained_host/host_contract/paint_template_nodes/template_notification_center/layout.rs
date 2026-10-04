//! 面板、标题与通知行共享的几何出口；同一metrics决定预取计算、行位置及行内文本预算。

mod common;
mod metrics;
mod panel;
mod row;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use common::paint_rect;
pub(super) use metrics::{notification_center_metrics, NotificationCenterMetrics};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use panel::{
    empty_text_rect, header_rect,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use row::{
    mark_rect, message_rect, row_rect, row_text_width, title_rect,
};
