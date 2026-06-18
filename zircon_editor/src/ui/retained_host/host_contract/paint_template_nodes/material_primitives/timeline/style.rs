use super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::paint_theme::PALETTE;
use super::super::{component_variant_contains, resolved_style_color};

const TIMELINE_DOT_BORDER_WIDTH: f32 = 2.0;
const MUI_GREY_400: [u8; 4] = [189, 189, 189, 255];
const MUI_SECONDARY_MAIN: [u8; 4] = [156, 39, 176, 255];

pub(super) fn timeline_dot_is_outlined(node: &TemplatePaneNodeData) -> bool {
    component_variant_contains(node, "outlined")
}

pub(super) fn timeline_dot_background_color(
    node: &TemplatePaneNodeData,
    outlined: bool,
    tone: [u8; 4],
) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.background_color.as_ref()).or_else(|| {
        if outlined {
            None
        } else if timeline_dot_color_token(node) == "grey" {
            Some(MUI_GREY_400)
        } else {
            Some(tone)
        }
    })
}

pub(super) fn timeline_dot_border_color(
    node: &TemplatePaneNodeData,
    outlined: bool,
    tone: [u8; 4],
) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.border_color.as_ref()).or_else(|| {
        if outlined {
            Some(tone)
        } else {
            None
        }
    })
}

pub(super) fn timeline_dot_border_width(
    node: &TemplatePaneNodeData,
    outlined: bool,
    has_border: bool,
) -> f32 {
    if !has_border {
        return 0.0;
    }
    let style_width = node.button_style.element.border_width;
    if style_width.is_finite() && style_width > 0.0 {
        style_width
    } else if node.border_width.is_finite() && node.border_width > 0.0 {
        node.border_width.max(if outlined {
            TIMELINE_DOT_BORDER_WIDTH
        } else {
            1.0
        })
    } else if outlined {
        TIMELINE_DOT_BORDER_WIDTH
    } else {
        1.0
    }
}

pub(super) fn timeline_connector_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    resolved_style_color(node.button_style.element.background_color.as_ref())
        .or_else(|| resolved_style_color(node.button_style.element.foreground_color.as_ref()))
        .or_else(|| resolved_style_color(node.button_style.element.border_color.as_ref()))
        .unwrap_or(MUI_GREY_400)
}

pub(super) fn timeline_dot_tone_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    match timeline_dot_color_token(node) {
        "secondary" => MUI_SECONDARY_MAIN,
        "grey" => MUI_GREY_400,
        "inherit" | "muted" | "subtle" => PALETTE.text_muted,
        "warning" => PALETTE.warning,
        "error" | "danger" => PALETTE.error,
        "success" => PALETTE.success,
        "info" => PALETTE.info,
        "primary" | "accent" | "default" => PALETTE.accent,
        _ => PALETTE.accent,
    }
}

fn timeline_dot_color_token(node: &TemplatePaneNodeData) -> &str {
    for token in [
        "secondary",
        "primary",
        "grey",
        "inherit",
        "warning",
        "error",
        "danger",
        "success",
        "info",
    ] {
        if component_variant_contains(node, token) {
            return token;
        }
    }
    match node.text_tone.as_str() {
        "" => "grey",
        "inverse" | "on-dark" => "inherit",
        other => other,
    }
}
