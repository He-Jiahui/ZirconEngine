use super::super::super::data::TemplatePaneNodeData;
use super::input_kind::is_text_input_node;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn fallback_node_label(
    node: &TemplatePaneNodeData,
) -> String {
    first_non_empty(&fallback_values(node)).to_string()
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn fallback_node_has_label(
    node: &TemplatePaneNodeData,
) -> bool {
    fallback_values(node)
        .iter()
        .any(|value| !value.trim().is_empty())
}

fn fallback_values(node: &TemplatePaneNodeData) -> [&str; 3] {
    if is_text_input_node(node) {
        [
            node.value_text.as_str(),
            node.text.as_str(),
            node.options_text.as_str(),
        ]
    } else {
        [
            node.text.as_str(),
            node.value_text.as_str(),
            node.options_text.as_str(),
        ]
    }
}

fn first_non_empty<'a>(values: &[&'a str]) -> &'a str {
    values
        .iter()
        .copied()
        .find(|value| !value.trim().is_empty())
        .unwrap_or("")
}
