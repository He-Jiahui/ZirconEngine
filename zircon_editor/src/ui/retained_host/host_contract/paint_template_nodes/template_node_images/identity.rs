use super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_icon_node(
    node: &TemplatePaneNodeData,
) -> bool {
    matches!(node.role.as_str(), "Icon" | "IconButton" | "SvgIcon") || !node.icon_name.is_empty()
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_icon_only_node(
    node: &TemplatePaneNodeData,
) -> bool {
    matches!(node.role.as_str(), "Icon" | "IconButton" | "SvgIcon")
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn template_node_has_image_source(
    node: &TemplatePaneNodeData,
) -> bool {
    node.has_preview_image || !node.media_source.is_empty() || !node.icon_name.is_empty()
}
