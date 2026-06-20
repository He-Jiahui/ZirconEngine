use super::super::super::template_style_color::resolved_style_color;
use super::model::WorkbenchAlertStyle;
use super::state::is_unavailable_alert_state;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::primitives::Color;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn apply_declared_alert_colors(
    node: &TemplatePaneNodeData,
    style: &mut WorkbenchAlertStyle,
) {
    if is_unavailable_alert_state(style.state) {
        return;
    }

    if let Some(surface) = resolved_style_color(node.button_style.element.background_color.as_ref())
    {
        style.surface = surface;
    }
    if let Some(border) = resolved_style_color(node.button_style.element.border_color.as_ref()) {
        style.border = border;
    }
    if let Some(text) = resolved_style_color(node.button_style.element.foreground_color.as_ref()) {
        style.text = text;
    }
    if let Some(mark) = declared_color(node.icon_color).or_else(|| declared_color(node.label_color))
    {
        style.mark = mark;
    }
}

fn declared_color(color: Color) -> Option<[u8; 4]> {
    (color.a > 0).then_some([color.r, color.g, color.b, color.a])
}
