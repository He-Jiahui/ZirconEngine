use super::super::super::super::template_style_color::resolved_style_color;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn declared_icon_button_background(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.background_color.as_ref())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn declared_icon_button_border(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.border_color.as_ref())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn declared_icon_color(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
    (node.icon_color.a > 0).then_some([
        node.icon_color.r,
        node.icon_color.g,
        node.icon_color.b,
        node.icon_color.a,
    ])
}
