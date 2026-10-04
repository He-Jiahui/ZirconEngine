use super::*;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

#[test]
fn text_field_stroke_projects_from_host_palette() {
    let mut palette = PALETTE;
    palette.border = [10, 11, 12, 255];
    palette.border_disabled = [20, 21, 22, 255];
    palette.error = [30, 31, 32, 255];
    palette.focus_ring = [40, 41, 42, 255];
    let mut node = TemplatePaneNodeData::default();

    assert_eq!(
        field_stroke_color_from_host(&node, palette),
        [10, 11, 12, 255]
    );

    node.focused = true;
    assert_eq!(
        field_stroke_color_from_host(&node, palette),
        [40, 41, 42, 255]
    );

    node.focused = false;
    node.validation_level = "error".into();
    assert_eq!(
        field_stroke_color_from_host(&node, palette),
        [30, 31, 32, 255]
    );

    node.validation_level.clear();
    node.disabled = true;
    assert_eq!(
        field_stroke_color_from_host(&node, palette),
        [20, 21, 22, 255]
    );
}

#[test]
fn text_field_declared_stroke_overrides_palette_when_available() {
    let palette = PALETTE;
    let mut node = TemplatePaneNodeData::default();
    node.button_style.element.border_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(50, 51, 52, 255)));

    assert_eq!(
        field_stroke_color_from_host(&node, palette),
        [50, 51, 52, 255]
    );
}
