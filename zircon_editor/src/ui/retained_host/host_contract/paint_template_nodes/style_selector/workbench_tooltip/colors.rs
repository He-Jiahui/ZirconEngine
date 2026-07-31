use super::super::super::template_style_color::resolved_style_color;
use super::model::WorkbenchTooltipStyle;
use super::state::is_unavailable_tooltip_state;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn apply_declared_tooltip_colors(
    node: &TemplatePaneNodeData,
    style: &mut WorkbenchTooltipStyle,
) {
    if is_unavailable_tooltip_state(style.state) {
        return;
    }

    if accepts_normal_tooltip_override(style.state) {
        if let Some(surface) =
            resolved_style_color(node.button_style.element.background_color.as_ref())
        {
            style.surface = surface;
            if declared_color(node.value_color).is_none() {
                style.arrow = surface;
            }
        }
        if let Some(border) = resolved_style_color(node.button_style.element.border_color.as_ref())
        {
            style.border = border;
        }
        if let Some(title) =
            resolved_style_color(node.button_style.element.foreground_color.as_ref())
        {
            style.title = title;
        }
    }
    if let Some(body) = declared_color(node.label_color) {
        style.body = body;
    }
    if let Some(icon) = declared_color(node.icon_color) {
        style.icon = icon;
    }
    if let Some(arrow) = declared_color(node.value_color) {
        style.arrow = arrow;
    }
}

fn declared_color(color: Color) -> Option<[u8; 4]> {
    (color.a > 0).then_some([color.r, color.g, color.b, color.a])
}

fn accepts_normal_tooltip_override(state: UiPainterResolvedState) -> bool {
    state == UiPainterResolvedState::Normal
}
