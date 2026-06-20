use super::super::super::data::TemplatePaneNodeData;

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

fn is_text_input_node(node: &TemplatePaneNodeData) -> bool {
    matches!(node.component_role.as_str(), "input-field" | "number-field")
        || matches!(node.role.as_str(), "InputField" | "LineEdit")
        || !node.edit_action_id.is_empty()
        || !node.commit_action_id.is_empty()
}

fn first_non_empty<'a>(values: &[&'a str]) -> &'a str {
    values
        .iter()
        .copied()
        .find(|value| !value.trim().is_empty())
        .unwrap_or("")
}
