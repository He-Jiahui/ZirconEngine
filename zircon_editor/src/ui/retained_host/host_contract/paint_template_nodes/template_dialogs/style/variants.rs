use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn variant_contains_any(
    node: &TemplatePaneNodeData,
    expected: &[&str],
) -> bool {
    [
        node.component_variant.as_str(),
        node.surface_variant.as_str(),
        node.validation_level.as_str(),
        node.text_tone.as_str(),
        node.button_variant.as_str(),
    ]
    .iter()
    .flat_map(|value| value.split_whitespace())
    .any(|part| {
        expected
            .iter()
            .any(|expected| part.eq_ignore_ascii_case(expected))
    })
}
