use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_template_nodes::material_primitives::resolved_style_color;

const MUI_PAPER_DARK_BACKGROUND: [u8; 4] = [18, 18, 18, 255];
const MUI_PAPER_DARK_DIVIDER: [u8; 4] = [255, 255, 255, 31];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn paper_background_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    resolved_style_color(node.button_style.element.background_color.as_ref())
        .unwrap_or(MUI_PAPER_DARK_BACKGROUND)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn paper_border_color(
    node: &TemplatePaneNodeData,
    outlined: bool,
) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.border_color.as_ref())
        .or_else(|| (paper_border_width(node, outlined) > 0.0).then_some(MUI_PAPER_DARK_DIVIDER))
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn paper_border_width(
    node: &TemplatePaneNodeData,
    outlined: bool,
) -> f32 {
    let configured = node
        .border_width
        .max(node.button_style.element.border_width)
        .max(0.0);
    if outlined {
        configured.max(1.0)
    } else {
        configured
    }
}
