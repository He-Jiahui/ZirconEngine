use super::*;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

#[test]
fn chip_foreground_projects_from_host_palette() {
    let mut palette = PALETTE;
    palette.text = [10, 11, 12, 255];
    palette.text_disabled = [20, 21, 22, 255];
    palette.shell_background = [30, 31, 32, 255];
    palette.accent = [40, 41, 42, 255];
    let mut node = TemplatePaneNodeData::default();

    assert_eq!(
        chip_foreground_color_from_host(&node, palette),
        [10, 11, 12, 255]
    );

    node.component_variant = "primary".into();
    assert_eq!(
        chip_foreground_color_from_host(&node, palette),
        [30, 31, 32, 255]
    );

    node.component_variant = "primary outlined".into();
    assert_eq!(
        chip_foreground_color_from_host(&node, palette),
        [40, 41, 42, 255]
    );

    node.disabled = true;
    assert_eq!(
        chip_foreground_color_from_host(&node, palette),
        [20, 21, 22, 255]
    );
}

#[test]
fn chip_foreground_declared_override_stays_explicit() {
    let mut palette = PALETTE;
    palette.text = [10, 11, 12, 255];
    let mut node = TemplatePaneNodeData::default();
    node.button_style.element.foreground_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(70, 71, 72, 255)));

    assert_eq!(
        chip_foreground_color_from_host(&node, palette),
        [70, 71, 72, 255]
    );
}
