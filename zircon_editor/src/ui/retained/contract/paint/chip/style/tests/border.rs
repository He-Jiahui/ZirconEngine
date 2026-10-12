use super::*;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

#[test]
fn outlined_chip_border_projects_from_host_palette() {
    let mut palette = PALETTE;
    palette.accent = [10, 11, 12, 255];
    palette.border = [20, 21, 22, 255];
    let mut node = TemplatePaneNodeData::default();
    node.component_variant = "outlined colorPrimary".into();

    assert_eq!(
        chip_border_color_from_host(&node, palette),
        Some([10, 11, 12, 255])
    );

    node.component_variant = "outlined".into();
    assert_eq!(
        chip_border_color_from_host(&node, palette),
        Some([20, 21, 22, 255])
    );
}

#[test]
fn explicit_chip_border_projects_from_host_palette() {
    let mut palette = PALETTE;
    palette.border = [10, 11, 12, 255];
    let mut node = TemplatePaneNodeData::default();

    assert_eq!(chip_border_color_from_host(&node, palette), None);

    node.border_width = 1.0;
    assert_eq!(
        chip_border_color_from_host(&node, palette),
        Some([10, 11, 12, 255])
    );
}

#[test]
fn declared_chip_border_overrides_palette_when_available() {
    let palette = PALETTE;
    let mut node = TemplatePaneNodeData::default();
    node.component_variant = "outlined colorPrimary".into();
    node.button_style.element.border_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(30, 31, 32, 255)));

    assert_eq!(
        chip_border_color_from_host(&node, palette),
        Some([30, 31, 32, 255])
    );
}
