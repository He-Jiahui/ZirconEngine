use super::super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};

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
    if let Some(token) = timeline_dot_color_variant(&node.component_variant) {
        return token;
    }
    match node.text_tone.as_str() {
        "" => "grey",
        "inverse" | "on-dark" => "inherit",
        other => other,
    }
}

fn timeline_dot_color_variant(component_variant: &str) -> Option<&'static str> {
    let mut best = u8::MAX;
    for part in component_variant.split(|character: char| {
        character.is_ascii_whitespace() || matches!(character, ',' | '/' | '|' | ':' | ';')
    }) {
        let candidate = if part.eq_ignore_ascii_case("secondary") {
            1
        } else if part.eq_ignore_ascii_case("primary") {
            2
        } else if part.eq_ignore_ascii_case("grey") {
            3
        } else if part.eq_ignore_ascii_case("inherit") {
            4
        } else if part.eq_ignore_ascii_case("warning") {
            5
        } else if part.eq_ignore_ascii_case("error") {
            6
        } else if part.eq_ignore_ascii_case("danger") {
            7
        } else if part.eq_ignore_ascii_case("success") {
            8
        } else if part.eq_ignore_ascii_case("info") {
            9
        } else {
            continue;
        };
        best = best.min(candidate);
        if best == 1 {
            break;
        }
    }
    match best {
        1 => Some("secondary"),
        2 => Some("primary"),
        3 => Some("grey"),
        4 => Some("inherit"),
        5 => Some("warning"),
        6 => Some("error"),
        7 => Some("danger"),
        8 => Some("success"),
        9 => Some("info"),
        _ => None,
    }
}

#[cfg(test)]
#[path = "tokens/single_scan_color_tests.rs"]
mod single_scan_color_tests;

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
