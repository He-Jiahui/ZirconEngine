use super::*;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

#[test]
fn chip_delete_icon_projects_from_host_palette() {
    let mut palette = PALETTE;
    palette.text_muted = [10, 11, 12, 255];
    palette.accent = [20, 21, 22, 255];
    palette.shell_background = [30, 31, 32, 255];
    palette.text = [40, 41, 42, 255];
    let mut node = TemplatePaneNodeData::default();

    assert_eq!(
        chip_delete_icon_color_from_host(&node, palette),
        [40, 41, 42, 255]
    );

    node.component_variant = "primary".into();
    assert_eq!(
        chip_delete_icon_color_from_host(&node, palette),
        [30, 31, 32, 255]
    );

    node.component_variant = "outlined".into();
    assert_eq!(
        chip_delete_icon_color_from_host(&node, palette),
        [10, 11, 12, 255]
    );

    node.component_variant = "primary outlined".into();
    assert_eq!(
        chip_delete_icon_color_from_host(&node, palette),
        [20, 21, 22, 255]
    );
}
