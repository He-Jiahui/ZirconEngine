use super::super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::super::paint_theme::PALETTE;
use super::super::super::component_variant_contains;

pub(super) const MUI_GREY_400: [u8; 4] = [189, 189, 189, 255];
const MUI_SECONDARY_MAIN: [u8; 4] = [156, 39, 176, 255];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn timeline_dot_tone_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
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

pub(super) fn timeline_dot_color_token(node: &TemplatePaneNodeData) -> &str {
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
