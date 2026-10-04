use super::super::super::super::data::TemplatePaneNodeData;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn text_field_variant(
    node: &TemplatePaneNodeData,
) -> TextFieldVariant {
    text_field_variant_from_component(&node.component_variant)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum TextFieldVariant {
    Outlined,
    Filled,
    Standard,
}

// 模板可携带多个外观词元；filled 优先于 standard，未声明时由 outlined 保持默认表面。
fn text_field_variant_from_component(component_variant: &str) -> TextFieldVariant {
    let mut variant = TextFieldVariant::Outlined;
    for part in component_variant.split(|character: char| {
        character.is_ascii_whitespace() || matches!(character, ',' | '/' | '|' | ':' | ';')
    }) {
        if part.eq_ignore_ascii_case("filled") {
            return TextFieldVariant::Filled;
        }
        if part.eq_ignore_ascii_case("standard") {
            variant = TextFieldVariant::Standard;
        }
    }
    variant
}

#[cfg(test)]
#[path = "variant/tests/single_scan_variant_tests.rs"]
mod single_scan_variant_tests;
