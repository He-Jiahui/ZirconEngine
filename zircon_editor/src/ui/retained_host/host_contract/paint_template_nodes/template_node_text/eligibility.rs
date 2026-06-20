use super::super::super::data::TemplatePaneNodeData;
use super::super::template_node_images::is_icon_only_node;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn should_skip_template_text(
    node: &TemplatePaneNodeData,
    label: &str,
    property_row_text_painted: bool,
    table_row_text_painted: bool,
) -> bool {
    property_row_text_painted
        || table_row_text_painted
        || ((label.is_empty() || is_icon_only_node(node))
            && !matches!(node.role.as_str(), "Label" | "Button"))
}
