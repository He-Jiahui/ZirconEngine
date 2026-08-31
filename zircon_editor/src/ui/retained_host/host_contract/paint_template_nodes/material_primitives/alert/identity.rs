use super::super::super::super::data::TemplatePaneNodeData;
use super::super::component_variant_contains;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_alert_root_node(
    node: &TemplatePaneNodeData,
) -> bool {
    matches!(
        node.component_role.as_str(),
        "alert" | "Alert" | "mui-alert" | "MuiAlert"
    ) || matches!(node.role.as_str(), "Alert" | "MuiAlert")
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_alert_slot_node(
    node: &TemplatePaneNodeData,
) -> bool {
    alert_slot_variant(&node.component_variant)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_has_icon(
    node: &TemplatePaneNodeData,
) -> bool {
    component_variant_contains(node, "hasIcon") || component_variant_contains(node, "alertSlotIcon")
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_has_action(
    node: &TemplatePaneNodeData,
) -> bool {
    alert_has_close_action(node)
        || component_variant_contains(node, "hasAction")
        || component_variant_contains(node, "alertSlotAction")
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_has_close_action(
    node: &TemplatePaneNodeData,
) -> bool {
    component_variant_contains(node, "hasCloseAction")
        || component_variant_contains(node, "alertSlotCloseButton")
        || component_variant_contains(node, "alertSlotCloseIcon")
}

fn alert_slot_variant(component_variant: &str) -> bool {
    component_variant
        .split(|character: char| {
            character.is_ascii_whitespace() || matches!(character, ',' | '/' | '|' | ':' | ';')
        })
        .any(|part| {
            part.eq_ignore_ascii_case("muiAlertSlot")
                || part
                    .get(.."alertSlot".len())
                    .is_some_and(|prefix| prefix.eq_ignore_ascii_case("alertSlot"))
        })
}

#[cfg(test)]
#[path = "identity/single_scan_slot_tests.rs"]
mod single_scan_slot_tests;
