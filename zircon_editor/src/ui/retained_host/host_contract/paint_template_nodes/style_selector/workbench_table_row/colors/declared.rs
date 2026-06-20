use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_style_color::resolved_style_color;

pub(super) fn declared_background_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.background_color.as_ref())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn declared_value_color(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
    (node.value_color.a > 0).then_some([
        node.value_color.r,
        node.value_color.g,
        node.value_color.b,
        node.value_color.a,
    ])
}
