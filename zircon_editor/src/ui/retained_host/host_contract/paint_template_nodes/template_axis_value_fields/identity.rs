//! 轴值字段必须同时具备文本输入角色和变换轴 ID 或 axis-value-field 组件角色；X/Y/Z ID 仅接受 Position/Rotation/Scale。
//! 此边界防止轴标签和普通工作台输入抢占同一个专用绘制入口。

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
    let bytes = field.as_bytes();
    let Some((&axis, kind_bytes)) = bytes.split_last() else {
        return None;
    };
    if !matches!(axis, b'X' | b'Y' | b'Z') {
        return None;
    }
    let kind = std::str::from_utf8(kind_bytes).ok()?;
    if matches!(kind, "Position" | "Rotation" | "Scale") {
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

#[cfg(test)]
#[path = "tests/identity.rs"]
mod tests;
