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
