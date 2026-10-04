use super::*;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::{
    ButtonInteractionState, UiPainterResolvedState, UiRgbaColor, UiStyleColor,
};

#[test]
fn toast_loading_state_uses_unavailable_visuals() {
    let mut node = TemplatePaneNodeData::default();
    node.hovered = true;
    node.focused = true;
    node.pressed = true;
    node.button_style.loading = true;
    node.label_color = Color::from_rgb_u8(53, 199, 208);
    node.value_color = Color::from_rgb_u8(53, 199, 208);
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(21, 48, 53, 247)));
    node.button_style.element.border_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(53, 199, 208, 20)));
    node.button_style.element.foreground_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(206, 224, 226, 255)));

    let style = select_workbench_toast_style(&node);

    assert_eq!(style.state, UiPainterResolvedState::Loading);
    assert_eq!(style.surface, PALETTE.surface_disabled);
    assert_eq!(style.border, PALETTE.border_disabled);
    assert_eq!(style.text, PALETTE.text_disabled);
    assert_eq!(style.mark, PALETTE.text_disabled);
    assert_eq!(style.action, PALETTE.text_disabled);
    assert_eq!(style.close, PALETTE.text_disabled);
}

#[test]
fn focused_toast_keeps_neutral_surface_with_focus_border() {
    let mut node = TemplatePaneNodeData::default();
    node.focused = true;

    let style = select_workbench_toast_style(&node);

    assert_eq!(style.state, UiPainterResolvedState::Focused);
    assert_eq!(style.surface, super::palette::WORKBENCH_TOAST_SURFACE);
    assert_eq!(style.border, PALETTE.focus_ring);
    assert_ne!(style.border, super::palette::WORKBENCH_TOAST_BORDER);
}

#[test]
fn focused_open_toast_keeps_shared_focused_priority() {
    let mut node = TemplatePaneNodeData::default();
    node.focused = true;
    node.popup_open = true;

    let style = select_workbench_toast_style(&node);

    assert_eq!(style.state, UiPainterResolvedState::Focused);
    assert_eq!(style.surface, super::palette::WORKBENCH_TOAST_SURFACE);
    assert_eq!(style.border, PALETTE.focus_ring);
}

#[test]
fn toast_dynamic_states_ignore_normal_declared_chrome() {
    let normal = declared_toast();
    let normal_style = select_workbench_toast_style(&normal);

    assert_eq!(normal_style.surface, [81, 88, 94, 255]);
    assert_eq!(normal_style.border, [109, 116, 122, 255]);
    assert_eq!(normal_style.text, [221, 226, 230, 255]);

    let mut hovered = declared_toast();
    hovered.hovered = true;
    assert_toast_chrome_matches_central_style(&hovered);

    let mut focused = declared_toast();
    focused.focused = true;
    assert_toast_chrome_matches_central_style(&focused);

    let mut pressed = declared_toast();
    pressed.pressed = true;
    assert_toast_chrome_matches_central_style(&pressed);

    let mut enter_pressed = declared_toast();
    enter_pressed.enter_pressed = true;
    assert_toast_chrome_matches_central_style(&enter_pressed);

    let mut selected = declared_toast();
    selected.selected = true;
    assert_toast_chrome_matches_central_style(&selected);

    let mut checked = declared_toast();
    checked.checked = true;
    assert_toast_chrome_matches_central_style(&checked);

    let mut open = declared_toast();
    open.popup_open = true;
    assert_toast_chrome_matches_central_style(&open);

    let mut dragging = declared_toast();
    dragging.dragging = true;
    assert_toast_chrome_matches_central_style(&dragging);

    let mut drop_hovered = declared_toast();
    drop_hovered.drop_hovered = true;
    assert_toast_chrome_matches_central_style(&drop_hovered);

    let mut active_drag_target = declared_toast();
    active_drag_target.active_drag_target = true;
    assert_eq!(
        select_workbench_toast_style(&active_drag_target).state,
        UiPainterResolvedState::DropHovered
    );
    assert_toast_chrome_matches_central_style(&active_drag_target);

    let mut interaction_hovered = declared_toast();
    interaction_hovered.button_style.interaction_state = ButtonInteractionState::Hover;
    assert_eq!(
        select_workbench_toast_style(&interaction_hovered).state,
        UiPainterResolvedState::Hovered
    );
    assert_toast_chrome_matches_central_style(&interaction_hovered);

    let mut interaction_focused = declared_toast();
    interaction_focused.button_style.interaction_state = ButtonInteractionState::Focused;
    assert_eq!(
        select_workbench_toast_style(&interaction_focused).state,
        UiPainterResolvedState::Focused
    );
    assert_toast_chrome_matches_central_style(&interaction_focused);

    let mut interaction_pressed = declared_toast();
    interaction_pressed.button_style.interaction_state = ButtonInteractionState::Pressed;
    assert_eq!(
        select_workbench_toast_style(&interaction_pressed).state,
        UiPainterResolvedState::Pressed
    );
    assert_toast_chrome_matches_central_style(&interaction_pressed);

    let mut interaction_disabled = declared_toast();
    interaction_disabled.button_style.interaction_state = ButtonInteractionState::Disabled;
    assert_eq!(
        select_workbench_toast_style(&interaction_disabled).state,
        UiPainterResolvedState::Disabled
    );
    assert_toast_chrome_matches_central_style(&interaction_disabled);

    let mut interaction_loading = declared_toast();
    interaction_loading.button_style.interaction_state = ButtonInteractionState::Loading;
    assert_eq!(
        select_workbench_toast_style(&interaction_loading).state,
        UiPainterResolvedState::Loading
    );
    assert_toast_chrome_matches_central_style(&interaction_loading);
}

#[test]
fn toast_dynamic_states_ignore_normal_chrome_but_keep_declared_semantic_colors() {
    let mut hovered = declared_semantic_toast();
    hovered.hovered = true;
    assert_toast_semantic_colors(&hovered);

    let mut focused = declared_semantic_toast();
    focused.focused = true;
    assert_toast_semantic_colors(&focused);

    let mut pressed = declared_semantic_toast();
    pressed.pressed = true;
    assert_toast_semantic_colors(&pressed);

    let mut selected = declared_semantic_toast();
    selected.selected = true;
    assert_toast_semantic_colors(&selected);

    let mut checked = declared_semantic_toast();
    checked.checked = true;
    assert_toast_semantic_colors(&checked);
}

fn declared_toast() -> TemplatePaneNodeData {
    let mut node = TemplatePaneNodeData::default();
    node.button_style.element.background_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(81, 88, 94, 255)));
    node.button_style.element.border_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(109, 116, 122, 255)));
    node.button_style.element.foreground_color =
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(221, 226, 230, 255)));
    node
}

fn declared_semantic_toast() -> TemplatePaneNodeData {
    let mut node = declared_toast();
    node.label_color = Color::from_rgb_u8(53, 199, 208);
    node.value_color = Color::from_rgb_u8(197, 230, 232);
    node
}

fn assert_toast_chrome_matches_central_style(node: &TemplatePaneNodeData) {
    let actual = select_workbench_toast_style(node);
    let mut central_node = node.clone();
    central_node.button_style.element.background_color = None;
    central_node.button_style.element.border_color = None;
    central_node.button_style.element.foreground_color = None;
    let expected = select_workbench_toast_style(&central_node);

    assert_eq!(actual.state, expected.state);
    assert_eq!(actual.surface, expected.surface);
    assert_eq!(actual.border, expected.border);
    assert_eq!(actual.text, expected.text);
}

fn assert_toast_semantic_colors(node: &TemplatePaneNodeData) {
    let actual = select_workbench_toast_style(node);

    assert_eq!(actual.mark, [53, 199, 208, 255]);
    assert_eq!(actual.action, [197, 230, 232, 255]);
}
