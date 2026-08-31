mod common;
mod inline;
mod metrics;
mod toast;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use common::{
    frame_is_within, has_paintable_alert_extent, paint_rect,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use inline::{
    alert_icon_rect, alert_text_rect,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::{
    alert_metrics, toast_metrics, WorkbenchAlertMetrics, WorkbenchToastMetrics,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use toast::{
    toast_action_rect, toast_close_rect, toast_has_action, toast_icon_rect, toast_text_rect,
};
