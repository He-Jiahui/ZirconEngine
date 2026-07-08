use super::palette::{alert_container_color, alert_filled_text_color, alert_main_color};
use super::variants::{alert_is_filled, alert_is_outlined};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::current_host_palette;

use super::super::super::resolved_style_color;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_text_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if node.disabled {
        return current_host_palette().text_disabled;
    }
    resolved_style_color(node.button_style.element.foreground_color.as_ref()).unwrap_or_else(|| {
        if alert_is_filled(node) {
            alert_filled_text_color(node)
        } else {
            alert_main_color(node)
        }
    })
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_icon_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if node.disabled {
        return current_host_palette().text_disabled;
    }
    if alert_is_filled(node) {
        alert_filled_text_color(node)
    } else {
        alert_main_color(node)
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_action_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    alert_text_color(node)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_icon_cutout_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    if alert_is_filled(node) {
        alert_main_color(node)
    } else if alert_is_outlined(node) {
        [0, 0, 0, 0]
    } else {
        alert_container_color(node)
    }
}
