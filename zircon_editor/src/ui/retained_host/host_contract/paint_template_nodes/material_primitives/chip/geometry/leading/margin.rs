use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

use super::super::super::identity::{chip_is_outlined, chip_is_small};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_leading_margin(
    node: &TemplatePaneNodeData,
) -> f32 {
    if chip_is_small(node) {
        if chip_is_outlined(node) { 2.0 } else { 4.0 }
    } else if chip_is_outlined(node) {
        4.0
    } else {
        5.0
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_negative_slot_margin(
    node: &TemplatePaneNodeData,
) -> f32 {
    if chip_is_small(node) { 4.0 } else { 6.0 }
}
