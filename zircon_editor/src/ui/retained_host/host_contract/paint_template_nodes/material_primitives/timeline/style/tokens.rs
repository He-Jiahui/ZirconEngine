use super::super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::super::super::component_variant_contains;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn timeline_dot_tone_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
    timeline_dot_tone_color_from_host(node, current_host_palette())
}

pub(super) fn timeline_neutral_color_from_host(palette: HostMaterialPalette) -> [u8; 4] {
    palette.separator_strong
}

fn timeline_dot_tone_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> [u8; 4] {
    match timeline_dot_color_token(node) {
        "secondary" => palette.accent_soft,
        "grey" => timeline_neutral_color_from_host(palette),
        "inherit" | "muted" | "subtle" => palette.text_muted,
        "warning" => palette.warning,
        "error" | "danger" => palette.error,
        "success" => palette.success,
        "info" => palette.info,
        "primary" | "accent" | "default" => palette.accent,
        _ => palette.accent,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

    #[test]
    fn timeline_dot_tone_projects_from_host_palette() {
        let mut palette = PALETTE;
        palette.separator_strong = [10, 11, 12, 255];
        palette.accent_soft = [20, 21, 22, 255];
        palette.warning = [30, 31, 32, 255];
        palette.error = [40, 41, 42, 255];
        palette.accent = [50, 51, 52, 255];
        let mut node = TemplatePaneNodeData::default();

        assert_eq!(
            timeline_dot_tone_color_from_host(&node, palette),
            [10, 11, 12, 255]
        );

        node.component_variant = "secondary".into();
        assert_eq!(
            timeline_dot_tone_color_from_host(&node, palette),
            [20, 21, 22, 255]
        );

        node.component_variant = "warning".into();
        assert_eq!(
            timeline_dot_tone_color_from_host(&node, palette),
            [30, 31, 32, 255]
        );

        node.component_variant.clear();
        node.text_tone = "danger".into();
        assert_eq!(
            timeline_dot_tone_color_from_host(&node, palette),
            [40, 41, 42, 255]
        );

        node.text_tone.clear();
        node.component_variant = "primary".into();
        assert_eq!(
            timeline_dot_tone_color_from_host(&node, palette),
            [50, 51, 52, 255]
        );
    }
}
