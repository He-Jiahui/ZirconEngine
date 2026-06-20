use super::super::super::super::data::TemplatePaneNodeData;
use super::super::component_variant_contains;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_chip_root_node(
    node: &TemplatePaneNodeData,
) -> bool {
    matches!(
        node.component_role.as_str(),
        "chip" | "Chip" | "mui-chip" | "MuiChip"
    ) || matches!(node.role.as_str(), "Chip" | "MuiChip")
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_chip_slot_node(
    node: &TemplatePaneNodeData,
) -> bool {
    component_variant_contains(node, "muiChipSlot")
        || component_variant_contains(node, "ChipSlot")
        || component_variant_contains(node, "chipSlot")
        || component_variant_token_starts_with(node, "chipSlot")
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_is_small(
    node: &TemplatePaneNodeData,
) -> bool {
    component_variant_contains(node, "small") || component_variant_contains(node, "sizeSmall")
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_is_outlined(
    node: &TemplatePaneNodeData,
) -> bool {
    component_variant_contains(node, "outlined")
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_is_deletable(
    node: &TemplatePaneNodeData,
) -> bool {
    component_variant_contains(node, "deletable")
        || component_variant_contains(node, "hasDeleteIcon")
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_has_avatar(
    node: &TemplatePaneNodeData,
) -> bool {
    component_variant_contains(node, "hasAvatar")
        || component_variant_contains(node, "chipSlotAvatar")
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_has_icon(
    node: &TemplatePaneNodeData,
) -> bool {
    component_variant_contains(node, "hasIcon") || component_variant_contains(node, "chipSlotIcon")
}

fn component_variant_token_starts_with(node: &TemplatePaneNodeData, expected_prefix: &str) -> bool {
    node.component_variant
        .as_str()
        .split(|character: char| {
            character.is_ascii_whitespace() || matches!(character, ',' | '/' | '|' | ':' | ';')
        })
        .any(|part| {
            part.get(..expected_prefix.len())
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case(expected_prefix))
        })
}
