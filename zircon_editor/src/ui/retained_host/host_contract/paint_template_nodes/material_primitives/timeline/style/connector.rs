use super::super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::resolved_style_color;
use super::tokens::MUI_GREY_400;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn timeline_connector_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    resolved_style_color(node.button_style.element.background_color.as_ref())
        .or_else(|| resolved_style_color(node.button_style.element.foreground_color.as_ref()))
        .or_else(|| resolved_style_color(node.button_style.element.border_color.as_ref()))
        .unwrap_or(MUI_GREY_400)
}
