use super::super::data::{HostTextInputFocusData, TemplatePaneNodeData};

pub(super) fn template_node_label(
    node: &TemplatePaneNodeData,
    text_input_focus: Option<&HostTextInputFocusData>,
) -> String {
    if let Some(focus) = focused_text_value(node, text_input_focus) {
        return focus.to_string();
    }
    if let Some(property_label) = property_row_label(node) {
        return property_label;
    }
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

fn focused_text_value<'a>(
    node: &TemplatePaneNodeData,
    text_input_focus: Option<&'a HostTextInputFocusData>,
) -> Option<&'a str> {
    let focus = text_input_focus?;
    (focus.is_active() && focus.control_id.as_str() == node.control_id.as_str())
        .then_some(focus.value_text.as_str())
}

fn property_row_label(node: &TemplatePaneNodeData) -> Option<String> {
    if node.component_role.as_str() != "property-row" && node.role.as_str() != "PropertyRow" {
        return None;
    }
    let text = node.text.trim();
    let value = node.value_text.trim();
    match (text.is_empty(), value.is_empty()) {
        (true, true) => Some(String::new()),
        (false, true) => Some(text.to_string()),
        (true, false) => Some(value.to_string()),
        (false, false) => Some(format!("{text}    {value}")),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn property_row_label_keeps_label_and_value_visible() {
        let node = TemplatePaneNodeData {
            role: "Mount".into(),
            component_role: "property-row".into(),
            text: "Position".into(),
            value_text: "X 12.0   Y 3.5   Z -8.0".into(),
            ..TemplatePaneNodeData::default()
        };

        assert_eq!(
            template_node_label(&node, None),
            "Position    X 12.0   Y 3.5   Z -8.0"
        );
    }
}
