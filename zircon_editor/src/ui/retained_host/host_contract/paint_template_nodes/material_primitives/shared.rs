use super::super::super::data::TemplatePaneNodeData;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use crate::ui::retained_host::host_contract::paint_geometry::{
    bounded_extent,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn component_variant_contains(
    node: &TemplatePaneNodeData,
    expected: &str,
) -> bool {
    node.component_variant
        .as_str()
        .split(|character: char| {
            character.is_ascii_whitespace() || matches!(character, ',' | '/' | '|' | ':' | ';')
        })
        .any(|part| part.eq_ignore_ascii_case(expected))
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn first_non_empty<'a>(
    values: &[&'a str],
) -> &'a str {
    values
        .iter()
        .copied()
        .find(|value| !value.is_empty())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::bounded_extent;

    #[test]
    fn bounded_extent_rejects_negative_and_non_finite_values() {
        assert_eq!(bounded_extent(12.5), 12.5);
        assert_eq!(bounded_extent(-1.0), 0.0);
        assert_eq!(bounded_extent(f32::NAN), 0.0);
        assert_eq!(bounded_extent(f32::INFINITY), 0.0);
    }
}
