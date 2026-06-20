use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::{select_workbench_alert_style, WorkbenchAlertTone as AlertTone};
use super::super::template_alert_glyphs::push_alert_mark;
use super::super::template_node_labels::template_node_label;
use super::layout::{
    alert_icon_rect, alert_text_rect, ALERT_BORDER_WIDTH, ALERT_FONT_SIZE, ALERT_LINE_HEIGHT,
    ALERT_RADIUS,
};
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
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(style.surface),
        Some(style.border),
        ALERT_BORDER_WIDTH,
        ALERT_RADIUS,
        opacity,
    ));

    let icon = alert_icon_rect(rect);
    push_alert_mark(commands, &icon, clip, order + 1, tone, style.mark, opacity);

    let label = template_node_label(node, None);
    if label.trim().is_empty() {
        return;
    }
    let Some(text_rect) = alert_text_rect(rect, &icon) else {
        return;
    };
    commands.push(HostPaintCommand::text(
        text_rect,
        Some(clip.clone()),
        order + 2,
        label,
        style.text,
        ALERT_FONT_SIZE,
        ALERT_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
