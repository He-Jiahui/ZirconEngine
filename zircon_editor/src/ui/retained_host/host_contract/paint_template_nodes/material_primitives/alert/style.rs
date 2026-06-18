use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_theme::PALETTE;
use super::super::{component_variant_contains, first_non_empty, resolved_style_color};

const ALERT_DEFAULT_RADIUS: f32 = 4.0;

const MUI_SUCCESS_MAIN: [u8; 4] = [46, 125, 50, 255];
const MUI_INFO_MAIN: [u8; 4] = [2, 136, 209, 255];
const MUI_WARNING_MAIN: [u8; 4] = [237, 108, 2, 255];
const MUI_ERROR_MAIN: [u8; 4] = [211, 47, 47, 255];
const MUI_ON_DARK: [u8; 4] = [255, 255, 255, 255];
const MUI_ON_WARNING: [u8; 4] = [0, 0, 0, 222];

pub(super) fn alert_background_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
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

pub(super) fn alert_border_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
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

pub(super) fn alert_border_width(node: &TemplatePaneNodeData) -> f32 {
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

pub(super) fn alert_corner_radius(node: &TemplatePaneNodeData, rect: &FrameRect) -> f32 {
    let configured = node
        .corner_radius
        .max(node.button_style.element.corner_radius)
        .max(0.0);
    let radius = if configured > 0.0 {
        configured
    } else {
        ALERT_DEFAULT_RADIUS
    };
    radius.min(rect.width.min(rect.height) * 0.5)
}

pub(super) fn alert_text_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        return PALETTE.text_disabled;
    }
    resolved_style_color(node.button_style.element.foreground_color.as_ref()).unwrap_or_else(|| {
        if alert_is_filled(node) {
            alert_filled_text_color(node)
        } else {
            alert_main_color(node)
        }
    })
}

pub(super) fn alert_icon_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        return PALETTE.text_disabled;
    }
    if alert_is_filled(node) {
        alert_filled_text_color(node)
    } else {
        alert_main_color(node)
    }
}

pub(super) fn alert_action_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    alert_text_color(node)
}

pub(super) fn alert_icon_cutout_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if alert_is_filled(node) {
        alert_main_color(node)
    } else if alert_is_outlined(node) {
        [0, 0, 0, 0]
    } else {
        alert_container_color(node)
    }
}

fn alert_filled_text_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if alert_color_token(node) == "warning" {
        MUI_ON_WARNING
    } else {
        MUI_ON_DARK
    }
}

fn alert_main_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    match alert_color_token(node) {
        "success" => MUI_SUCCESS_MAIN,
        "info" => MUI_INFO_MAIN,
        "error" | "danger" => MUI_ERROR_MAIN,
        "warning" => MUI_WARNING_MAIN,
        _ => PALETTE.info,
    }
}

fn alert_container_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    match alert_color_token(node) {
        "success" => PALETTE.success_container,
        "info" => PALETTE.info_container,
        "error" | "danger" => PALETTE.error_container,
        "warning" => PALETTE.warning_container,
        _ => PALETTE.info_container,
    }
}

fn alert_color_token(node: &TemplatePaneNodeData) -> &str {
    for token in ["success", "info", "warning", "error", "danger"] {
        if component_variant_contains(node, token)
            || component_variant_contains(node, &format!("color{}", pascal_case(token)))
        {
            return token;
        }
    }
    match first_non_empty(&[node.validation_level.as_str(), node.text_tone.as_str()]) {
        "success" => "success",
        "info" => "info",
        "warning" => "warning",
        "error" | "danger" => "error",
        _ => "success",
    }
}

fn alert_is_filled(node: &TemplatePaneNodeData) -> bool {
    component_variant_contains(node, "filled")
}

fn alert_is_outlined(node: &TemplatePaneNodeData) -> bool {
    component_variant_contains(node, "outlined")
}

fn pascal_case(value: &str) -> String {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return String::new();
    };
    first.to_ascii_uppercase().to_string() + characters.as_str()
}
