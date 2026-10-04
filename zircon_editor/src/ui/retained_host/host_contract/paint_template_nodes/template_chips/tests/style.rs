use super::*;
use crate::ui::retained_host::host_contract::paint_theme::project_host_palette;
use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;
use zircon_runtime_interface::ui::style::UiRgbaColor;

#[test]
fn workbench_chip_palette_projects_from_host_palette() {
    let mut tokens = EditorDesignTokens::workbench_dark();
    tokens.palette.surface_hover = UiRgbaColor::from_u8(42, 53, 60, 255);
    tokens.palette.focus_ring = UiRgbaColor::from_u8(12, 140, 180, 255);
    tokens.palette.text_primary = UiRgbaColor::from_u8(220, 226, 230, 255);

    let palette = workbench_chip_palette_from_host(project_host_palette(&tokens));

    assert_eq!(palette.hover_surface, [42, 53, 60, 255]);
    assert_eq!(palette.focus_ring, [12, 140, 180, 255]);
    assert_eq!(palette.text, [220, 226, 230, 255]);
}

#[test]
fn focused_chip_keeps_normal_surface_and_glyph_with_focus_border() {
    let mut node = TemplatePaneNodeData::default();
    node.focused = true;

    let palette = workbench_chip_palette();

    assert_eq!(chip_surface(&node), palette.surface);
    assert_eq!(chip_border(&node), palette.focus_ring);
    assert_eq!(chip_glyph_color(&node), palette.text_muted);
}

#[test]
fn hovered_chip_still_uses_hover_surface() {
    let mut node = TemplatePaneNodeData::default();
    node.hovered = true;

    assert_eq!(chip_surface(&node), workbench_chip_palette().hover_surface);
}

#[test]
fn normal_chip_uses_the_standard_control_border() {
    let node = TemplatePaneNodeData::default();
    let palette = workbench_chip_palette();

    assert_eq!(chip_border(&node), palette.border);
    assert_ne!(chip_border(&node), chip_surface(&node));
}

#[test]
fn pressed_chip_uses_pressed_surface_and_accent_glyph_without_focus_border() {
    let mut node = TemplatePaneNodeData::default();
    node.pressed = true;

    let palette = workbench_chip_palette();

    assert_eq!(chip_surface(&node), palette.pressed_surface);
    assert_eq!(chip_border(&node), palette.border);
    assert_eq!(chip_glyph_color(&node), palette.accent);
    assert_ne!(chip_border(&node), palette.focus_ring);
}

#[test]
fn selected_chip_uses_a_low_emphasis_selection_surface_without_focus_ring() {
    let mut node = TemplatePaneNodeData::default();
    node.selected = true;

    let palette = workbench_chip_palette();

    assert_eq!(chip_surface(&node), palette.selected_surface);
    assert_eq!(chip_border(&node), palette.selected_border);
    assert_ne!(chip_border(&node), palette.focus_ring);
}
