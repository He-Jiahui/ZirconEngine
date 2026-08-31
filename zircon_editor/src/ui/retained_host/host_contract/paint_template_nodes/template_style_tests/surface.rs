use super::super::super::super::paint_theme::PALETTE;
use super::super::colors::{border_color, surface_color, text_color};
use super::super::dimensions::template_border_width;
use super::super::state::button_interaction_state;
use super::support::{button_node, panel_node, resolved_background};
use zircon_runtime_interface::ui::style::ButtonInteractionState;

#[test]
fn native_template_button_state_resolves_shared_surface_priority() {
    let mut node = button_node();
    node.focused = true;
    node.pressed = true;

    assert_eq!(
        button_interaction_state(&node),
        ButtonInteractionState::Pressed
    );
    assert_eq!(surface_color(&node), PALETTE.surface_pressed);
    assert_eq!(border_color(&node), PALETTE.border);
    assert_ne!(border_color(&node), PALETTE.focus_ring);

    node.button_style.loading = true;
    node.disabled = true;
    assert_eq!(surface_color(&node), PALETTE.surface_disabled);
}

#[test]
fn native_template_hover_keeps_neutral_border_while_focus_uses_focus_ring() {
    let mut node = button_node();
    node.hovered = true;

    assert_eq!(border_color(&node), PALETTE.border);
    assert_ne!(border_color(&node), PALETTE.focus_ring);

    node.hovered = false;
    node.focused = true;
    assert_eq!(border_color(&node), PALETTE.focus_ring);
}

#[test]
fn native_template_button_style_keeps_declared_colors_after_state_resolution() {
    let mut node = button_node();
    node.hovered = true;
    node.button_style = resolved_background([11, 22, 33, 255]);

    assert_eq!(
        button_interaction_state(&node),
        ButtonInteractionState::Hover
    );
    assert_eq!(surface_color(&node), PALETTE.surface_hover);

    node.hovered = false;
    assert_eq!(surface_color(&node), [11, 22, 33, 255]);
}

#[test]
fn transparent_surface_is_clear_at_idle_and_keeps_interaction_feedback() {
    let mut node = panel_node("transparent");
    node.corner_radius = 6.0;

    assert_eq!(surface_color(&node), [0, 0, 0, 0]);

    node.hovered = true;
    assert_eq!(surface_color(&node), PALETTE.surface_hover);

    node.hovered = false;
    node.pressed = true;
    assert_eq!(surface_color(&node), PALETTE.surface_pressed);
}

#[test]
fn asset_placeholder_surface_uses_low_emphasis_inset_without_border() {
    let mut node = panel_node("asset-placeholder");
    node.corner_radius = 6.0;
    node.border_width = 0.0;

    assert_eq!(surface_color(&node), PALETTE.surface_inset);
    assert_eq!(template_border_width(&node), 0.0);

    node.surface_variant = "asset-placeholder-visual".into();
    assert_eq!(surface_color(&node), PALETTE.surface_inset);
    assert_eq!(template_border_width(&node), 0.0);
}

#[test]
fn component_panel_surface_uses_slate_panel_surface_with_thin_border() {
    let mut node = panel_node("component-panel");
    node.corner_radius = 4.0;
    node.border_width = 1.0;

    assert_eq!(surface_color(&node), PALETTE.surface);
    assert_eq!(template_border_width(&node), 1.0);
    assert_eq!(border_color(&node), PALETTE.border);
}

#[test]
fn asset_type_badge_surface_uses_low_emphasis_hover_layer_without_border() {
    let mut node = panel_node("asset-type-badge");
    node.corner_radius = 3.0;
    node.border_width = 0.0;

    assert_eq!(surface_color(&node), PALETTE.surface_hover);
    assert_eq!(template_border_width(&node), 0.0);
}

#[test]
fn asset_thumbnail_card_and_name_area_use_content_browser_layers() {
    let mut card = panel_node("asset-thumbnail-card");
    card.selected = true;
    card.focused = true;
    card.corner_radius = 4.0;
    card.border_width = 1.0;

    assert_eq!(surface_color(&card), [0, 0, 0, 0]);
    assert_eq!(template_border_width(&card), 1.0);
    assert_eq!(border_color(&card), PALETTE.border);
    assert_ne!(border_color(&card), PALETTE.accent);
    assert_ne!(border_color(&card), PALETTE.focus_ring);

    let mut name_area = panel_node("asset-thumbnail-name-area");
    name_area.corner_radius = 4.0;
    assert_eq!(surface_color(&name_area), PALETTE.surface);

    name_area.hovered = true;
    assert_eq!(surface_color(&name_area), PALETTE.surface_hover);

    name_area.hovered = false;
    name_area.selected = true;
    assert_eq!(surface_color(&name_area), PALETTE.surface_selected);
    assert_ne!(surface_color(&name_area), PALETTE.surface_pressed);
    assert_eq!(template_border_width(&name_area), 0.0);
    assert_eq!(border_color(&name_area), PALETTE.border);
}

#[test]
fn asset_thumbnail_name_area_text_uses_selected_palette_without_accent_noise() {
    let mut title = panel_node("");
    title.role = "Label".into();
    title.component_role = "asset-thumbnail-name-area-text".into();
    title.selected = true;

    assert_eq!(text_color(&title), PALETTE.text);

    title.text_tone = "accent".into();
    assert_eq!(text_color(&title), PALETTE.text);
    assert_ne!(text_color(&title), PALETTE.focus_ring);

    title.text_tone = "muted".into();
    assert_eq!(text_color(&title), PALETTE.text_muted);

    title.selected = false;
    title.text_tone = "accent".into();
    assert_eq!(text_color(&title), PALETTE.accent);
    assert_ne!(text_color(&title), PALETTE.focus_ring);
}

#[test]
fn content_panel_surface_uses_recessed_content_layer_without_focus_emphasis() {
    let mut node = panel_node("content-panel");
    node.corner_radius = 4.0;
    node.border_width = 1.0;
    node.focused = true;
    node.hovered = true;
    node.selected = true;

    assert_eq!(surface_color(&node), PALETTE.surface_inset);
    assert_eq!(template_border_width(&node), 1.0);
    assert_eq!(border_color(&node), PALETTE.border);
    assert_ne!(border_color(&node), PALETTE.focus_ring);

    node.surface_variant = "asset-content".into();
    assert_eq!(surface_color(&node), PALETTE.surface_inset);
    assert_eq!(template_border_width(&node), 1.0);
    assert_eq!(border_color(&node), PALETTE.border);
}

#[test]
fn asset_preview_selected_surface_uses_slate_outline_emphasis() {
    let mut node = panel_node("asset-preview");
    node.selected = true;
    node.corner_radius = 6.0;
    node.border_width = 1.0;

    assert_eq!(surface_color(&node), PALETTE.surface_pressed);
    assert_ne!(surface_color(&node), PALETTE.surface_selected);
    assert_eq!(template_border_width(&node), 1.0);
    assert_eq!(border_color(&node), PALETTE.border);
    assert_ne!(border_color(&node), PALETTE.focus_ring);

    node.surface_variant = "asset-preview-visual".into();
    assert_eq!(surface_color(&node), PALETTE.surface_pressed);
    assert_ne!(surface_color(&node), PALETTE.surface_selected);
    assert_eq!(border_color(&node), PALETTE.border);

    node.selected = false;
    node.focused = true;
    assert_eq!(border_color(&node), PALETTE.border);

    node.validation_level = "error".into();
    assert_eq!(border_color(&node), PALETTE.error);
}
