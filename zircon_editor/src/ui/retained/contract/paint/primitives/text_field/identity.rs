use super::super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_text_field_node(
    node: &TemplatePaneNodeData,
) -> bool {
    matches!(
        node.component_role.as_str(),
        "input-field" | "number-field" | "text-field" | "mui-text-field" | "MuiTextField"
    ) || matches!(
        node.role.as_str(),
        "InputField" | "TextField" | "MuiTextField"
    )
}
