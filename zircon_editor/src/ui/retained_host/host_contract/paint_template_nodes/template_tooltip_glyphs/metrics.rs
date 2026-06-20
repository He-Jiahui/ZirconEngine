use super::super::super::data::TemplatePaneNodeData;

const TOOLTIP_ARROW_SIZE: f32 = 8.0;
const TOOLTIP_ICON_SIZE: f32 = 18.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tooltip_arrow_size(
    node: &TemplatePaneNodeData,
) -> f32 {
    let size = if node.value_number > 0.0 {
        node.value_number
    } else {
        TOOLTIP_ARROW_SIZE
    };
    size.clamp(4.0, 14.0)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tooltip_icon_size(
    node: &TemplatePaneNodeData,
) -> f32 {
    let size = if node.layout_icon_size > 0.0 {
        node.layout_icon_size
    } else {
        TOOLTIP_ICON_SIZE
    };
    size.clamp(10.0, 24.0)
}
