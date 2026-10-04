use super::*;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

#[test]
fn typed_button_background_projects_from_host_palette() {
    let mut palette = PALETTE;
    palette.warning_container = [10, 11, 12, 255];
    palette.surface_inset = [20, 21, 22, 255];
    palette.text_muted = [30, 31, 32, 255];
    let mut node = TemplatePaneNodeData::default();
    node.role = "Button".into();

    node.button_style.variant = ButtonVariant::Contained;
    node.button_style.color = ButtonColor::Warning;
    assert_eq!(
        typed_button_variant_background_from_host(&node, palette),
        Some([10, 11, 12, 255])
    );

    node.button_style.variant = ButtonVariant::Outlined;
    assert_eq!(
        typed_button_variant_background_from_host(&node, palette),
        Some([20, 21, 22, 255])
    );

    node.button_style.variant = ButtonVariant::Contained;
    node.button_style.color = ButtonColor::Style("muted".into());
    assert_eq!(
        typed_button_variant_background_from_host(&node, palette),
        Some([30, 31, 32, 255])
    );

    node.role = "Text".into();
    assert_eq!(
        typed_button_variant_background_from_host(&node, palette),
        None
    );
}

#[test]
fn typed_button_tone_projects_from_host_palette() {
    let mut palette = PALETTE;
    palette.warning = [10, 11, 12, 255];
    palette.text_muted = [20, 21, 22, 255];
    palette.accent = [30, 31, 32, 255];
    palette.shell_background = [40, 41, 42, 255];
    let mut node = TemplatePaneNodeData::default();
    node.role = "IconButton".into();

    node.button_style.color = ButtonColor::Warning;
    assert_eq!(
        typed_button_tone_color_from_host(&node, palette),
        Some([10, 11, 12, 255])
    );

    node.button_style.color = ButtonColor::Style("muted".into());
    assert_eq!(
        typed_button_tone_color_from_host(&node, palette),
        Some([20, 21, 22, 255])
    );

    node.button_style.variant = ButtonVariant::Contained;
    node.button_style.color = ButtonColor::Default;
    assert_eq!(
        typed_button_tone_color_from_host(&node, palette),
        Some([40, 41, 42, 255])
    );

    node.button_style.variant = ButtonVariant::Outlined;
    assert_eq!(
        typed_button_tone_color_from_host(&node, palette),
        Some([30, 31, 32, 255])
    );

    node.button_style.color = ButtonColor::Secondary;
    assert_eq!(typed_button_tone_color_from_host(&node, palette), None);
}

#[test]
fn typed_primary_border_uses_accent_without_borrowing_focus_ring() {
    let mut palette = PALETTE;
    palette.accent = [10, 11, 12, 255];
    palette.focus_ring = [20, 21, 22, 255];
    let mut node = TemplatePaneNodeData::default();
    node.role = "Button".into();
    node.button_style.variant = ButtonVariant::Contained;
    node.button_style.color = ButtonColor::Primary;

    assert_eq!(
        typed_button_border_color_from_host(&node, palette),
        Some(palette.accent)
    );
    assert_ne!(
        typed_button_border_color_from_host(&node, palette),
        Some(palette.focus_ring)
    );
}
