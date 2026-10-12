//! 反馈身份与呈现模式分开解析；circular 仅改变形状，indeterminate 才使用固定示意进度。

use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::material_primitives::component_variant_contains;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn progress_is_circular(
    node: &TemplatePaneNodeData,
) -> bool {
    matches!(
        node.component_role.as_str(),
        "circular-progress" | "spinner"
    ) || matches!(node.role.as_str(), "CircularProgress" | "Spinner")
        || component_variant_contains(node, "circular")
}

// TODO: [CR-EDITOR-PAINT-CONTROLSTYLE-0002] 确认仅声明 role=Spinner 的模板是否必须自动使用不确定进度；
// 当前身份和环形解析接受该别名，此处只接受语义 spinner 或变体，需补真实模板投影的别名测试。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn progress_is_indeterminate(
    node: &TemplatePaneNodeData,
) -> bool {
    matches!(node.component_role.as_str(), "spinner")
        || component_variant_contains(node, "indeterminate")
}
