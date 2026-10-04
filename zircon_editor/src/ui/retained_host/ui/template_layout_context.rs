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
        let variant = variant.trim();
        let mut combined = String::with_capacity(variant.len() + 1 + token.len());
        combined.push_str(variant);
        combined.push(' ');
        combined.push_str(token);
        combined
    }
}

fn component_variant_has_token(variant: &str, token: &str) -> bool {
    variant
        .split_whitespace()
        .any(|candidate| candidate == token)
}

#[cfg(test)]
#[path = "tests/template_layout_context.rs"]
mod tests;
