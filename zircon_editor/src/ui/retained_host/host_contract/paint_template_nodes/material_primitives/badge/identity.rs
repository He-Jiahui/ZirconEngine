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
    component_variant_contains(node, "muiBadgeSlot")
        || component_variant_contains(node, "BadgeSlot")
        || component_variant_contains(node, "badgeSlot")
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
