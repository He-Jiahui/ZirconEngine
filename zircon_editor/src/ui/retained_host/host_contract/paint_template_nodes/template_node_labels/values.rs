use super::super::super::data::TemplatePaneNodeData;
use super::input_kind::is_text_input_node;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn fallback_node_label(
    node: &TemplatePaneNodeData,
) -> String {
    let values = if is_text_input_node(node) {
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
    };
    first_non_empty(&values).to_string()
}

fn first_non_empty<'a>(values: &[&'a str]) -> &'a str {
    values
        .iter()
        .copied()
        .find(|value| !value.trim().is_empty())
        .unwrap_or("")
}
