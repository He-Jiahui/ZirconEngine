use super::*;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

#[test]
fn badge_root_border_projects_from_host_palette() {
    let mut palette = PALETTE;
    palette.border = [10, 11, 12, 255];
    let node = TemplatePaneNodeData::default();

    assert_eq!(
        badge_root_border_color_from_host(&node, 1.0, palette),
        Some([10, 11, 12, 255])
    );
    assert_eq!(badge_root_border_color_from_host(&node, 0.0, palette), None);
}

#[test]
fn badge_root_declared_border_overrides_palette_when_available() {
    let mut palette = PALETTE;
    palette.border = [10, 11, 12, 255];
    let mut node = TemplatePaneNodeData::default();
    node.button_style.element.border_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(20, 21, 22, 255)));

    assert_eq!(
        badge_root_border_color_from_host(&node, 0.0, palette),
        Some([20, 21, 22, 255])
    );
}

#[test]
fn badge_root_text_projects_from_host_palette() {
    let mut palette = PALETTE;
    palette.text = [30, 31, 32, 255];
    palette.text_disabled = [40, 41, 42, 255];
    let mut node = TemplatePaneNodeData::default();

    assert_eq!(
        badge_root_text_color_from_host(&node, palette),
        [30, 31, 32, 255]
    );

    node.disabled = true;
    assert_eq!(
        badge_root_text_color_from_host(&node, palette),
        [40, 41, 42, 255]
    );
}

#[test]
fn badge_root_declared_text_overrides_palette_when_available() {
    let mut palette = PALETTE;
    palette.text = [30, 31, 32, 255];
    let mut node = TemplatePaneNodeData::default();
    node.button_style.element.foreground_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(50, 51, 52, 255)));

    assert_eq!(
        badge_root_text_color_from_host(&node, palette),
        [50, 51, 52, 255]
    );
}
