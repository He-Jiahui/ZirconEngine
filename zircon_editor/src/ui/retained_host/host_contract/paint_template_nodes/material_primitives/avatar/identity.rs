use super::super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_avatar_node(
    node: &TemplatePaneNodeData,
) -> bool {
    matches!(
        node.component_role.as_str(),
        "avatar" | "Avatar" | "mui-avatar" | "MuiAvatar"
    ) || matches!(node.role.as_str(), "Avatar" | "MuiAvatar")
}
