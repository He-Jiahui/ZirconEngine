use super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_workbench_axis_value_field(
    node: &TemplatePaneNodeData,
) -> bool {
    if !is_text_input_node(node) {
        return false;
    }
    let control_id = node.control_id.as_str();
    control_id == "WorkbenchAxisValueFieldRoot"
        || transform_axis_value_id(control_id).is_some()
        || node.component_role.as_str() == "axis-value-field"
}

fn transform_axis_value_id(control_id: &str) -> Option<TransformAxisValueId> {
    let field = control_id.strip_prefix("WorkbenchTransform")?;
    let axis = if field.ends_with('X') {
        "X"
    } else if field.ends_with('Y') {
        "Y"
    } else if field.ends_with('Z') {
        "Z"
    } else {
        return None;
    };
    if field
        .strip_suffix(axis)
        .is_some_and(|kind| matches!(kind, "Position" | "Rotation" | "Scale"))
    {
        Some(TransformAxisValueId)
    } else {
        None
    }
}

#[derive(Clone, Copy)]
struct TransformAxisValueId;

fn is_text_input_node(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.role.as_str(),
        "InputField" | "LineEdit" | "TextField" | "MuiTextField"
    ) || matches!(
        node.component_role.as_str(),
        "input-field" | "number-field" | "text-field"
    )
}
