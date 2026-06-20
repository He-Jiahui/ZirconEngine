use super::super::super::super::data::TemplatePaneNodeData;
use super::super::component_variant_contains;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn text_field_variant(
    node: &TemplatePaneNodeData,
) -> TextFieldVariant {
    if component_variant_contains(node, "filled") {
        TextFieldVariant::Filled
    } else if component_variant_contains(node, "standard") {
        TextFieldVariant::Standard
    } else {
        TextFieldVariant::Outlined
    }
}

#[derive(Clone, Copy)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum TextFieldVariant {
    Outlined,
    Filled,
    Standard,
}
