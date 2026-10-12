use super::*;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

#[test]
fn skeleton_fill_border_and_wave_project_from_host_palette() {
    let mut palette = PALETTE;
    palette.surface_hover = [10, 11, 12, 255];
    palette.separator_soft = [20, 21, 22, 128];
    let mut node = TemplatePaneNodeData::default();

    assert_eq!(skeleton_color_from_host(&node, palette), [10, 11, 12, 255]);
    assert_eq!(skeleton_wave_color_from_host(palette), [20, 21, 22, 128]);
    assert_eq!(skeleton_border_color_from_host(&node, palette), None);

    node.border_width = 1.0;
    assert_eq!(
        skeleton_border_color_from_host(&node, palette),
        Some([10, 11, 12, 255])
    );
}

#[test]
fn skeleton_declared_fill_and_border_override_palette_when_available() {
    let mut palette = PALETTE;
    palette.surface_hover = [10, 11, 12, 255];
    let mut node = TemplatePaneNodeData::default();
    node.border_width = 1.0;
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(30, 31, 32, 255)));
    node.button_style.element.border_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(40, 41, 42, 255)));

    assert_eq!(skeleton_color_from_host(&node, palette), [30, 31, 32, 255]);
    assert_eq!(
        skeleton_border_color_from_host(&node, palette),
        Some([40, 41, 42, 255])
    );
}
