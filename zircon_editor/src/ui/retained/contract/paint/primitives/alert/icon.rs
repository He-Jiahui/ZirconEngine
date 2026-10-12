use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::style_selector::WorkbenchAlertTone;
use super::super::super::template_alert_glyphs::push_alert_mark;
use super::geometry::alert_icon_frame;
use super::style::{alert_color_token, alert_icon_color};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_alert_icon(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let frame = alert_icon_frame(rect);
    let color = alert_icon_color(node);
    let tone = match alert_color_token(node) {
        "success" => WorkbenchAlertTone::Success,
        "warning" => WorkbenchAlertTone::Warning,
        "error" | "danger" => WorkbenchAlertTone::Error,
        _ => WorkbenchAlertTone::Info,
    };
    push_alert_mark(commands, &frame, clip, order, tone, color, opacity);
}
