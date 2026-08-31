use super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn property_row_label(
    node: &TemplatePaneNodeData,
) -> Option<String> {
    if !is_property_row(node) {
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

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn property_row_has_label(
    node: &TemplatePaneNodeData,
) -> Option<bool> {
    is_property_row(node)
        .then(|| !node.text.trim().is_empty() || !node.value_text.trim().is_empty())
}

fn is_property_row(node: &TemplatePaneNodeData) -> bool {
    node.component_role.as_str() == "property-row" || node.role.as_str() == "PropertyRow"
}
