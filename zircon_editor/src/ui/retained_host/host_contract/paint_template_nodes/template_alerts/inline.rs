use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::{WorkbenchAlertTone as AlertTone, select_workbench_alert_style};
use super::super::template_alert_glyphs::push_alert_mark;
use super::super::template_node_labels::template_node_label;
use super::layout::{alert_icon_rect, alert_metrics, alert_text_rect, frame_is_within};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_inline_alert(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    tone: AlertTone,
    opacity: f32,
) {
    let style = select_workbench_alert_style(node, tone);
    let metrics = alert_metrics();
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(style.surface),
        Some(style.border),
        metrics.border_width,
        metrics.radius,
        opacity,
    ));

    let icon = alert_icon_rect(rect, metrics);
    let has_icon = !node.icon_name.is_empty();
    if has_icon
        && frame_is_within(&icon, rect)
        && icon.width >= metrics.icon_size
        && icon.height >= metrics.icon_size
    {
        push_alert_mark(commands, &icon, clip, order + 1, tone, style.mark, opacity);
    }

    let label = template_node_label(node, None);
    if label.trim().is_empty() {
        return;
    }
    let icon = if has_icon { Some(&icon) } else { None };
    let Some(text_rect) = alert_text_rect(rect, icon, metrics) else {
        return;
    };
    if !frame_is_within(&text_rect, rect) || text_rect.height < metrics.line_height {
        return;
    }
    commands.push(HostPaintCommand::text(
        text_rect,
        Some(clip.clone()),
        order + 2,
        label,
        style.text,
        metrics.font_size,
        metrics.line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
