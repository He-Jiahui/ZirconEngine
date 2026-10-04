use super::super::super::super::data::TemplatePaneNodeData;

// 材质分发器只把 Avatar 角色交给此绘制器；其他角色交给后续原语或通用模板。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_avatar_node(
    node: &TemplatePaneNodeData,
) -> bool {
    matches!(
        node.component_role.as_str(),
        "avatar" | "Avatar" | "mui-avatar" | "MuiAvatar"
    ) || matches!(node.role.as_str(), "Avatar" | "MuiAvatar")
}
