use super::super::super::data::TemplatePaneNodeData;
use super::super::super::paint_theme::PALETTE;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_field_text_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if node.disabled {
        PALETTE.text_disabled
    } else if matches!(node.validation_level.as_str(), "error" | "danger") {
        PALETTE.error
    } else if node.value_color.a > 0 {
        [
            node.value_color.r,
            node.value_color.g,
            node.value_color.b,
            node.value_color.a,
        ]
    } else {
        PALETTE.text
    }
}
