use super::*;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::{UiPainterResolvedState, UiRgbaColor, UiStyleColor};

#[test]
fn tooltip_palette_projects_from_host_palette() {
    let mut host = PALETTE;
    host.popup = [10, 11, 12, 255];
    host.border = [20, 21, 22, 255];
    host.text = [30, 31, 32, 255];
    host.text_muted = [40, 41, 42, 255];
    host.focus_ring = [50, 51, 52, 255];
    host.shadow = [1, 2, 3, 90];
    host.surface_disabled = [60, 61, 62, 255];
    host.border_disabled = [70, 71, 72, 255];
    host.text_disabled = [80, 81, 82, 255];
    host.accent = [90, 91, 92, 255];

    let palette = palette::tooltip_palette_from_host(host);

    assert_eq!(palette.surface, [10, 11, 12, 255]);
    assert_eq!(palette.border, [20, 21, 22, 255]);
    assert_eq!(palette.title, [30, 31, 32, 255]);
    assert_eq!(palette.body, [40, 41, 42, 255]);
    assert_eq!(palette.icon, [50, 51, 52, 255]);
    assert_eq!(palette.shadow, [1, 2, 3, 90]);
    assert_eq!(palette.disabled_surface, [60, 61, 62, 255]);
    assert_eq!(palette.disabled_border, [70, 71, 72, 255]);
    assert_eq!(palette.disabled_text, [80, 81, 82, 255]);
    assert_eq!(palette.disabled_shadow, [1, 2, 3, 48]);
    assert_eq!(palette.focused_border, [50, 51, 52, 255]);
    assert_eq!(palette.hover_icon, [90, 91, 92, 255]);
}

#[test]
fn tooltip_state_style_projects_state_roles_from_host_palette() {
    let mut host = PALETTE;
    host.popup = [10, 11, 12, 255];
    host.border = [20, 21, 22, 255];
    host.text = [30, 31, 32, 255];
    host.surface_disabled = [40, 41, 42, 255];
    host.border_disabled = [50, 51, 52, 255];
    host.text_disabled = [60, 61, 62, 255];
    host.shadow = [1, 2, 3, 90];
    host.focus_ring = [70, 71, 72, 255];
    host.accent = [80, 81, 82, 255];

    let tooltip_palette = palette::tooltip_palette_from_host(host);
    let loading =
        state::tooltip_state_style_from_palette(UiPainterResolvedState::Loading, tooltip_palette);
    let pressed =
        state::tooltip_state_style_from_palette(UiPainterResolvedState::Pressed, tooltip_palette);
    let hovered =
        state::tooltip_state_style_from_palette(UiPainterResolvedState::Hovered, tooltip_palette);

    assert_eq!(loading.surface, [40, 41, 42, 255]);
    assert_eq!(loading.border, [50, 51, 52, 255]);
    assert_eq!(loading.title, [60, 61, 62, 255]);
    assert_eq!(loading.body, [60, 61, 62, 255]);
    assert_eq!(loading.arrow, [40, 41, 42, 255]);
    assert_eq!(loading.icon, [60, 61, 62, 255]);
    assert_eq!(loading.shadow, [1, 2, 3, 48]);
    assert_eq!(pressed.surface, [10, 11, 12, 255]);
    assert_eq!(pressed.border, [70, 71, 72, 255]);
    assert_eq!(pressed.icon, [70, 71, 72, 255]);
    assert_eq!(pressed.title, [30, 31, 32, 255]);
    assert_eq!(hovered.border, [20, 21, 22, 255]);
    assert_eq!(hovered.icon, [80, 81, 82, 255]);
}

#[test]
fn tooltip_loading_state_uses_unavailable_visuals() {
    let mut node = TemplatePaneNodeData::default();
    node.hovered = true;
    node.focused = true;
    node.pressed = true;
    node.button_style.loading = true;
    node.value_color = Color::from_rgb_u8(23, 28, 32);
    node.label_color = Color::from_rgb_u8(168, 179, 184);
    node.icon_color = Color::from_rgb_u8(37, 156, 167);
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(23, 28, 32, 255)));
    node.button_style.element.border_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(37, 45, 50, 255)));
    node.button_style.element.foreground_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(208, 217, 221, 255)));

    let style = select_workbench_tooltip_style(&node);

    assert_eq!(style.state, UiPainterResolvedState::Loading);
    assert_eq!(style.surface, PALETTE.surface_disabled);
    assert_eq!(style.border, PALETTE.border_disabled);
    assert_eq!(style.title, PALETTE.text_disabled);
    assert_eq!(style.body, PALETTE.text_disabled);
    assert_eq!(style.arrow, PALETTE.surface_disabled);
    assert_eq!(style.icon, PALETTE.text_disabled);
}

#[test]
fn focused_tooltip_keeps_neutral_bubble_border() {
    let mut node = TemplatePaneNodeData::default();
    node.focused = true;

    let style = select_workbench_tooltip_style(&node);

    assert_eq!(style.state, UiPainterResolvedState::Focused);
    assert_eq!(style.border, PALETTE.border);
    assert_ne!(style.border, PALETTE.focus_ring);
}

#[test]
fn pressed_tooltip_uses_active_bubble_border() {
    let mut node = TemplatePaneNodeData::default();
    node.focused = true;
    node.pressed = true;

    let style = select_workbench_tooltip_style(&node);

    assert_eq!(style.state, UiPainterResolvedState::Pressed);
    assert_eq!(style.border, PALETTE.focus_ring);
}
