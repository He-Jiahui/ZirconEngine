use super::super::super::data::TemplatePaneNodeData;
use super::super::template_node_images::is_icon_only_node;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn should_skip_template_text(
    node: &TemplatePaneNodeData,
    label: &str,
    property_row_text_painted: bool,
    table_row_text_painted: bool,
) -> bool {
    should_skip_template_text_before_label(node, property_row_text_painted, table_row_text_painted)
        || (label.is_empty() && !fallback_text_role_allows_empty_label(node))
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn should_skip_template_text_before_label(
    node: &TemplatePaneNodeData,
    property_row_text_painted: bool,
    table_row_text_painted: bool,
) -> bool {
    property_row_text_painted
        || table_row_text_painted
        || (is_icon_only_node(node) && !fallback_text_role_allows_empty_label(node))
}

fn fallback_text_role_allows_empty_label(node: &TemplatePaneNodeData) -> bool {
    matches!(node.role.as_str(), "Label" | "Button")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_only_node_skips_before_label_materialization() {
        let node = TemplatePaneNodeData {
            role: "IconButton".into(),
            text: "unused fallback text".into(),
            ..TemplatePaneNodeData::default()
        };

        assert!(should_skip_template_text_before_label(&node, false, false));
        assert!(should_skip_template_text(&node, &node.text, false, false));
    }
}
