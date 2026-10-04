use super::*;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

#[test]
fn fallback_text_color_projects_all_generic_tones_from_the_host_palette() {
    let mut palette = PALETTE;
    palette.text = [11, 12, 13, 255];
    palette.text_muted = [21, 22, 23, 255];
    palette.accent = [31, 32, 33, 255];
    palette.warning = [41, 42, 43, 255];
    palette.error = [51, 52, 53, 255];
    palette.success = [61, 62, 63, 255];
    palette.info = [71, 72, 73, 255];

    let mut node = TemplatePaneNodeData::default();
    assert_eq!(text_color_from_palette(&node, palette), [11, 12, 13, 255]);
    node.text_tone = "muted".into();
    assert_eq!(text_color_from_palette(&node, palette), [21, 22, 23, 255]);
    node.text_tone = "primary".into();
    assert_eq!(text_color_from_palette(&node, palette), [31, 32, 33, 255]);
    node.text_tone = "warning".into();
    assert_eq!(text_color_from_palette(&node, palette), [41, 42, 43, 255]);
    node.text_tone = "danger".into();
    assert_eq!(text_color_from_palette(&node, palette), [51, 52, 53, 255]);
    node.text_tone = "success".into();
    assert_eq!(text_color_from_palette(&node, palette), [61, 62, 63, 255]);
    node.text_tone = "info".into();
    assert_eq!(text_color_from_palette(&node, palette), [71, 72, 73, 255]);
}
