use super::palette::{alert_container_color, alert_main_color};
use super::variants::{alert_is_filled, alert_is_outlined};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

use super::super::super::resolved_style_color;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_background_color(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.background_color.as_ref()).or_else(|| {
        if alert_is_outlined(node) {
            None
        } else if alert_is_filled(node) {
            Some(alert_main_color(node))
        } else {
            Some(alert_container_color(node))
        }
    })
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_border_color(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.border_color.as_ref()).or_else(|| {
        (alert_border_width(node) > 0.0).then(|| {
            if alert_is_outlined(node) {
                alert_main_color(node)
            } else {
                PALETTE.border
            }
        })
    })
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_border_width(
    node: &TemplatePaneNodeData,
) -> f32 {
    let configured = node
        .border_width
        .max(node.button_style.element.border_width)
        .max(0.0);
    if alert_is_outlined(node) {
        configured.max(1.0)
    } else {
        configured
    }
}
