use super::super::super::data::TemplatePaneNodeData;

pub(super) fn is_text_input_node(node: &TemplatePaneNodeData) -> bool {
    has_text_input_identity(node) || (has_edit_binding(node) && !is_command_control(node))
}

fn has_text_input_identity(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.component_role.as_str(),
        "input-field" | "number-field" | "text-field" | "search-field"
    ) || matches!(
        node.role.as_str(),
        "InputField" | "LineEdit" | "TextField" | "NumberField"
    )
}

fn has_edit_binding(node: &TemplatePaneNodeData) -> bool {
    !node.edit_action_id.is_empty() || !node.commit_action_id.is_empty()
}

fn is_command_control(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.component_role.as_str(),
        "button"
            | "icon-button"
            | "toggle"
            | "toggle-button"
            | "tab"
            | "tabs"
            | "tab-list"
            | "segmented-control"
            | "checkbox"
            | "radio"
            | "radio-field"
            | "combo-box"
            | "dropdown"
            | "enum-field"
            | "flags-field"
            | "search-select"
    ) || matches!(
        node.role.as_str(),
        "Button" | "IconButton" | "Toggle" | "Checkbox" | "Radio" | "Dropdown" | "ComboBox"
    )
}
