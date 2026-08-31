use super::super::super::super::data::TemplatePaneNodeData;
use super::super::component_variant_contains;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_badge_root_node(
    node: &TemplatePaneNodeData,
) -> bool {
    matches!(
        node.component_role.as_str(),
        "badge" | "Badge" | "mui-badge" | "MuiBadge"
    ) || matches!(node.role.as_str(), "Badge" | "MuiBadge")
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_badge_slot_node(
    node: &TemplatePaneNodeData,
) -> bool {
    badge_slot_variant(&node.component_variant)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn badge_is_dot(
    node: &TemplatePaneNodeData,
) -> bool {
    component_variant_contains(node, "dot")
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn badge_is_invisible(
    node: &TemplatePaneNodeData,
) -> bool {
    node.disabled
        || component_variant_contains(node, "invisible")
        || component_variant_contains(node, "hidden")
}

fn badge_slot_variant(component_variant: &str) -> bool {
    component_variant
        .split(|character: char| {
            character.is_ascii_whitespace() || matches!(character, ',' | '/' | '|' | ':' | ';')
        })
        .any(|part| {
            part.eq_ignore_ascii_case("muiBadgeSlot") || part.eq_ignore_ascii_case("badgeSlot")
        })
}

#[cfg(test)]
#[path = "identity/single_scan_slot_tests.rs"]
mod single_scan_slot_tests;
