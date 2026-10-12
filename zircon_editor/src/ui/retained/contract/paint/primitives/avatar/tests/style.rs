use super::*;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

#[test]
fn avatar_default_background_projects_from_host_palette() {
    let mut palette = PALETTE;
    palette.surface_hover = [10, 11, 12, 255];
    palette.surface_selected = [20, 21, 22, 255];
    palette.surface_disabled = [30, 31, 32, 255];
    let mut node = TemplatePaneNodeData::default();

    assert_eq!(
        avatar_background_color_from_host(&node, true, palette),
        [10, 11, 12, 255]
    );
    assert_eq!(
        avatar_background_color_from_host(&node, false, palette),
        [20, 21, 22, 255]
    );

    node.disabled = true;
    assert_eq!(
        avatar_background_color_from_host(&node, true, palette),
        [30, 31, 32, 255]
    );
}

#[test]
fn avatar_declared_background_overrides_palette_when_available() {
    let mut palette = PALETTE;
    palette.surface_disabled = [30, 31, 32, 255];
    let mut node = TemplatePaneNodeData::default();
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(40, 41, 42, 255)));

    assert_eq!(
        avatar_background_color_from_host(&node, true, palette),
        [40, 41, 42, 255]
    );

    node.disabled = true;
    assert_eq!(
        avatar_background_color_from_host(&node, true, palette),
        [30, 31, 32, 255]
    );
}

#[test]
fn avatar_foreground_projects_from_host_palette() {
    let mut palette = PALETTE;
    palette.text = [10, 11, 12, 255];
    palette.text_disabled = [20, 21, 22, 255];
    palette.accent = [30, 31, 32, 255];
    palette.text_muted = [40, 41, 42, 255];
    palette.warning = [50, 51, 52, 255];
    palette.error = [60, 61, 62, 255];
    palette.success = [70, 71, 72, 255];
    palette.info = [80, 81, 82, 255];
    let mut node = TemplatePaneNodeData::default();

    assert_eq!(
        avatar_foreground_color_from_host(&node, palette),
        [10, 11, 12, 255]
    );

    node.text_tone = "primary".into();
    assert_eq!(
        avatar_foreground_color_from_host(&node, palette),
        [30, 31, 32, 255]
    );

    node.text_tone = "secondary".into();
    assert_eq!(
        avatar_foreground_color_from_host(&node, palette),
        [40, 41, 42, 255]
    );

    node.text_tone.clear();
    node.validation_level = "warning".into();
    assert_eq!(
        avatar_foreground_color_from_host(&node, palette),
        [50, 51, 52, 255]
    );

    node.validation_level = "error".into();
    assert_eq!(
        avatar_foreground_color_from_host(&node, palette),
        [60, 61, 62, 255]
    );

    node.validation_level = "success".into();
    assert_eq!(
        avatar_foreground_color_from_host(&node, palette),
        [70, 71, 72, 255]
    );

    node.validation_level = "info".into();
    assert_eq!(
        avatar_foreground_color_from_host(&node, palette),
        [80, 81, 82, 255]
    );

    node.disabled = true;
    assert_eq!(
        avatar_foreground_color_from_host(&node, palette),
        [20, 21, 22, 255]
    );
}

#[test]
fn avatar_declared_foreground_overrides_palette_when_available() {
    let palette = PALETTE;
    let mut node = TemplatePaneNodeData::default();
    node.text_tone = "primary".into();
    node.button_style.element.foreground_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(90, 91, 92, 255)));

    assert_eq!(
        avatar_foreground_color_from_host(&node, palette),
        [90, 91, 92, 255]
    );
}

#[test]
fn avatar_border_projects_from_host_palette() {
    let mut palette = PALETTE;
    palette.border = [10, 11, 12, 255];
    let mut node = TemplatePaneNodeData::default();

    assert_eq!(avatar_border_color_from_host(&node, palette), None);

    node.border_width = 1.0;
    assert_eq!(
        avatar_border_color_from_host(&node, palette),
        Some([10, 11, 12, 255])
    );

    node.button_style.element.border_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(20, 21, 22, 255)));
    assert_eq!(
        avatar_border_color_from_host(&node, palette),
        Some([20, 21, 22, 255])
    );
}
