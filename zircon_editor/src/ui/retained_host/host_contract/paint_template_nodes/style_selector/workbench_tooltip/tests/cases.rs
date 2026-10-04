use super::*;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::{
    ButtonInteractionState, UiPainterResolvedState, UiRgbaColor, UiStyleColor,
};

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
    assert_eq!(palette.icon, [90, 91, 92, 255]);
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
    let focused =
        state::tooltip_state_style_from_palette(UiPainterResolvedState::Focused, tooltip_palette);

    assert_eq!(loading.surface, [40, 41, 42, 255]);
    assert_eq!(loading.border, [50, 51, 52, 255]);
    assert_eq!(loading.title, [60, 61, 62, 255]);
    assert_eq!(loading.body, [60, 61, 62, 255]);
    assert_eq!(loading.arrow, [40, 41, 42, 255]);
    assert_eq!(loading.icon, [60, 61, 62, 255]);
    assert_eq!(loading.shadow, [1, 2, 3, 48]);
    assert_eq!(pressed.surface, [10, 11, 12, 255]);
    assert_eq!(pressed.border, [20, 21, 22, 255]);
    assert_eq!(pressed.icon, [80, 81, 82, 255]);
    assert_eq!(pressed.title, [30, 31, 32, 255]);
    assert_eq!(hovered.border, [20, 21, 22, 255]);
    assert_eq!(hovered.icon, [80, 81, 82, 255]);
    assert_eq!(focused.surface, [10, 11, 12, 255]);
    assert_eq!(focused.border, [70, 71, 72, 255]);
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
fn focused_tooltip_keeps_normal_surface_with_focus_border() {
    let mut node = TemplatePaneNodeData::default();
    node.focused = true;

    let style = select_workbench_tooltip_style(&node);

    assert_eq!(style.state, UiPainterResolvedState::Focused);
    assert_eq!(style.surface, PALETTE.popup);
    assert_eq!(style.border, PALETTE.focus_ring);
}

#[test]
fn pressed_tooltip_uses_accent_content_without_a_focus_outline() {
    let mut node = TemplatePaneNodeData::default();
    node.focused = true;
    node.pressed = true;

    let style = select_workbench_tooltip_style(&node);

    assert_eq!(style.state, UiPainterResolvedState::Pressed);
    assert_eq!(style.border, PALETTE.border);
    assert_eq!(style.icon, PALETTE.accent);
    assert_ne!(style.border, PALETTE.focus_ring);
}

#[test]
fn tooltip_dynamic_states_ignore_normal_declared_chrome() {
    let normal = declared_tooltip();
    let normal_style = select_workbench_tooltip_style(&normal);

    assert_eq!(normal_style.surface, [81, 88, 94, 255]);
    assert_eq!(normal_style.border, [109, 116, 122, 255]);
    assert_eq!(normal_style.title, [221, 226, 230, 255]);
    assert_eq!(normal_style.arrow, [81, 88, 94, 255]);

    let mut hovered = declared_tooltip();
    hovered.hovered = true;
    assert_tooltip_chrome_matches_central_style(&hovered);

    let mut focused = declared_tooltip();
    focused.focused = true;
    assert_tooltip_chrome_matches_central_style(&focused);

    let mut pressed = declared_tooltip();
    pressed.pressed = true;
    assert_tooltip_chrome_matches_central_style(&pressed);

    let mut enter_pressed = declared_tooltip();
    enter_pressed.enter_pressed = true;
    assert_tooltip_chrome_matches_central_style(&enter_pressed);

    let mut selected = declared_tooltip();
    selected.selected = true;
    assert_tooltip_chrome_matches_central_style(&selected);

    let mut checked = declared_tooltip();
    checked.checked = true;
    assert_tooltip_chrome_matches_central_style(&checked);

    let mut open = declared_tooltip();
    open.popup_open = true;
    assert_tooltip_chrome_matches_central_style(&open);

    let mut dragging = declared_tooltip();
    dragging.dragging = true;
    assert_tooltip_chrome_matches_central_style(&dragging);

    let mut drop_hovered = declared_tooltip();
    drop_hovered.drop_hovered = true;
    assert_tooltip_chrome_matches_central_style(&drop_hovered);

    let mut active_drag_target = declared_tooltip();
    active_drag_target.active_drag_target = true;
    assert_eq!(
        select_workbench_tooltip_style(&active_drag_target).state,
        UiPainterResolvedState::DropHovered
    );
    assert_tooltip_chrome_matches_central_style(&active_drag_target);

    let mut interaction_hovered = declared_tooltip();
    interaction_hovered.button_style.interaction_state = ButtonInteractionState::Hover;
    assert_eq!(
        select_workbench_tooltip_style(&interaction_hovered).state,
        UiPainterResolvedState::Hovered
    );
    assert_tooltip_chrome_matches_central_style(&interaction_hovered);

    let mut interaction_focused = declared_tooltip();
    interaction_focused.button_style.interaction_state = ButtonInteractionState::Focused;
    assert_eq!(
        select_workbench_tooltip_style(&interaction_focused).state,
        UiPainterResolvedState::Focused
    );
    assert_tooltip_chrome_matches_central_style(&interaction_focused);

    let mut interaction_pressed = declared_tooltip();
    interaction_pressed.button_style.interaction_state = ButtonInteractionState::Pressed;
    assert_eq!(
        select_workbench_tooltip_style(&interaction_pressed).state,
        UiPainterResolvedState::Pressed
    );
    assert_tooltip_chrome_matches_central_style(&interaction_pressed);

    let mut interaction_disabled = declared_tooltip();
    interaction_disabled.button_style.interaction_state = ButtonInteractionState::Disabled;
    assert_eq!(
        select_workbench_tooltip_style(&interaction_disabled).state,
        UiPainterResolvedState::Disabled
    );
    assert_tooltip_chrome_matches_central_style(&interaction_disabled);

    let mut interaction_loading = declared_tooltip();
    interaction_loading.button_style.interaction_state = ButtonInteractionState::Loading;
    assert_eq!(
        select_workbench_tooltip_style(&interaction_loading).state,
        UiPainterResolvedState::Loading
    );
    assert_tooltip_chrome_matches_central_style(&interaction_loading);
}

#[test]
fn tooltip_dynamic_states_ignore_normal_chrome_but_keep_declared_semantic_colors() {
    let mut hovered = declared_semantic_tooltip();
    hovered.hovered = true;
    assert_tooltip_semantic_colors(&hovered);

    let mut focused = declared_semantic_tooltip();
    focused.focused = true;
    assert_tooltip_semantic_colors(&focused);

    let mut pressed = declared_semantic_tooltip();
    pressed.pressed = true;
    assert_tooltip_semantic_colors(&pressed);

    let mut selected = declared_semantic_tooltip();
    selected.selected = true;
    assert_tooltip_semantic_colors(&selected);

    let mut checked = declared_semantic_tooltip();
    checked.checked = true;
    assert_tooltip_semantic_colors(&checked);
}

fn declared_tooltip() -> TemplatePaneNodeData {
    let mut node = TemplatePaneNodeData::default();
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(81, 88, 94, 255)));
    node.button_style.element.border_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(109, 116, 122, 255)));
    node.button_style.element.foreground_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(221, 226, 230, 255)));
    node
}

fn declared_semantic_tooltip() -> TemplatePaneNodeData {
    let mut node = declared_tooltip();
    node.label_color = Color::from_rgb_u8(130, 138, 144);
    node.icon_color = Color::from_rgb_u8(38, 191, 203);
    node.value_color = Color::from_rgb_u8(47, 157, 170);
    node
}

fn assert_tooltip_chrome_matches_central_style(node: &TemplatePaneNodeData) {
    let actual = select_workbench_tooltip_style(node);
    let mut central_node = node.clone();
    central_node.button_style.element.background_color = None;
    central_node.button_style.element.border_color = None;
    central_node.button_style.element.foreground_color = None;
    let expected = select_workbench_tooltip_style(&central_node);

    assert_eq!(actual.state, expected.state);
    assert_eq!(actual.surface, expected.surface);
    assert_eq!(actual.border, expected.border);
    assert_eq!(actual.title, expected.title);
    assert_eq!(actual.arrow, expected.arrow);
}

fn assert_tooltip_semantic_colors(node: &TemplatePaneNodeData) {
    let actual = select_workbench_tooltip_style(node);

    assert_eq!(actual.body, [130, 138, 144, 255]);
    assert_eq!(actual.icon, [38, 191, 203, 255]);
    assert_eq!(actual.arrow, [47, 157, 170, 255]);
}
