use super::super::super::render_commands::HostPaintCommand;
use super::super::super::style_selector::WorkbenchAlertTone as AlertTone;
use super::super::super::template_alert_glyphs::push_alert_mark;
use super::super::layout::{WorkbenchToastMetrics, toast_metrics};
use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};

pub(super) fn push_toast_status_mark(
    commands: &mut Vec<HostPaintCommand>,
    icon: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_alert_mark(
        commands,
        icon,
        clip,
        order,
        AlertTone::Success,
        color,
        opacity,
    );
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn toast_status_mark_size(
    node: &TemplatePaneNodeData,
) -> f32 {
    toast_status_mark_size_for_metrics(node, toast_metrics())
}

pub(super) fn toast_status_mark_size_for_metrics(
    node: &TemplatePaneNodeData,
    metrics: WorkbenchToastMetrics,
) -> f32 {
    if node.value_number > 0.0 {
        node.value_number
    } else {
        metrics.icon_size
    }
}
