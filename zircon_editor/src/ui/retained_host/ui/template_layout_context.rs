use crate::ui::retained_host as host_contract;
use crate::ui::workbench::autolayout::{
    workbench_layout_tier_for_logical_width, WorkbenchLayoutTier,
};

pub(in crate::ui::retained_host::ui) const TABLE_LAYOUT_NARROW_VARIANT: &str = "layoutNarrow";
pub(in crate::ui::retained_host::ui) const TABLE_LAYOUT_REGULAR_VARIANT: &str = "layoutRegular";
pub(in crate::ui::retained_host::ui) const TABLE_LAYOUT_WIDE_VARIANT: &str = "layoutWide";

pub(in crate::ui::retained_host::ui) fn apply_table_layout_context_variant(
    mut node: host_contract::TemplatePaneNodeData,
    context_width: f32,
) -> host_contract::TemplatePaneNodeData {
    if is_table_node(&node) && context_width > 0.0 {
        node.component_variant = append_component_variant_token(
            node.component_variant.as_str(),
            table_layout_context_variant_for_width(context_width),
        )
        .into();
    }
    node
}

pub(in crate::ui::retained_host::ui) fn table_layout_context_variant_for_width(
    context_width: f32,
) -> &'static str {
    match workbench_layout_tier_for_logical_width(context_width) {
        WorkbenchLayoutTier::Ultra | WorkbenchLayoutTier::Narrow => TABLE_LAYOUT_NARROW_VARIANT,
        WorkbenchLayoutTier::Regular => TABLE_LAYOUT_REGULAR_VARIANT,
        WorkbenchLayoutTier::Wide => TABLE_LAYOUT_WIDE_VARIANT,
    }
}

fn is_table_node(node: &host_contract::TemplatePaneNodeData) -> bool {
    node.role.as_str() == "Table" || node.component_role.as_str() == "table"
}

fn append_component_variant_token(variant: &str, token: &str) -> String {
    if token.is_empty() || component_variant_has_token(variant, token) {
        return variant.to_string();
    }
    if variant.trim().is_empty() {
        token.to_string()
    } else {
        format!("{} {}", variant.trim(), token)
    }
}

fn component_variant_has_token(variant: &str, token: &str) -> bool {
    variant
        .split_whitespace()
        .any(|candidate| candidate == token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_nodes_receive_context_tier_variant() {
        let node = host_contract::TemplatePaneNodeData {
            role: "Table".into(),
            component_role: "table".into(),
            component_variant: "asset-table".into(),
            ..host_contract::TemplatePaneNodeData::default()
        };

        let node = apply_table_layout_context_variant(node, 640.0);

        assert!(node
            .component_variant
            .as_str()
            .split_whitespace()
            .any(|token| token == TABLE_LAYOUT_NARROW_VARIANT));
    }

    #[test]
    fn non_table_nodes_keep_variant_without_context_tier() {
        let node = host_contract::TemplatePaneNodeData {
            role: "Button".into(),
            component_role: "button".into(),
            component_variant: "outlined".into(),
            ..host_contract::TemplatePaneNodeData::default()
        };

        let node = apply_table_layout_context_variant(node, 640.0);

        assert_eq!(node.component_variant.as_str(), "outlined");
    }
}
