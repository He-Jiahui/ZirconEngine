use super::model::WorkbenchPopupRowState;
use super::palette::workbench_popup_row_palette_from_host;
use super::selection::select_workbench_popup_row_style;
use crate::ui::retained_host::host_contract::paint_theme::{current_host_palette, PALETTE};
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

#[test]
fn popup_row_palette_projects_from_host_palette() {
    let host_palette = current_host_palette();
    let palette = workbench_popup_row_palette_from_host(host_palette);

    assert_eq!(palette.marked_background, host_palette.surface_pressed);
    assert_eq!(palette.hot_background, host_palette.surface_hover);
    assert_eq!(palette.text, host_palette.text);
    assert_eq!(palette.text_muted, host_palette.text_muted);
    assert_eq!(palette.text_disabled, host_palette.text_disabled);
    assert_eq!(palette.danger_text, host_palette.error);
}

#[test]
fn popup_row_selector_projects_full_semantic_state() {
    let open = select_workbench_popup_row_style(WorkbenchPopupRowState {
        open: true,
        ..WorkbenchPopupRowState::default()
    });
    assert_eq!(open.state, UiPainterResolvedState::Open);
    assert_eq!(open.background, Some(PALETTE.surface_hover));
    assert_eq!(open.text, PALETTE.text);

    let dragging = select_workbench_popup_row_style(WorkbenchPopupRowState {
        dragging: true,
        ..WorkbenchPopupRowState::default()
    });
    assert_eq!(dragging.state, UiPainterResolvedState::Dragging);
    assert_eq!(dragging.background, Some(PALETTE.surface_hover));
    assert_eq!(dragging.adornment, PALETTE.text);

    let drop_hovered = select_workbench_popup_row_style(WorkbenchPopupRowState {
        drop_hovered: true,
        ..WorkbenchPopupRowState::default()
    });
    assert_eq!(drop_hovered.state, UiPainterResolvedState::DropHovered);
    assert_eq!(drop_hovered.background, Some(PALETTE.surface_hover));
    assert_eq!(drop_hovered.shortcut, PALETTE.text_muted);
}

#[test]
fn popup_row_loading_state_uses_unavailable_visuals() {
    let loading_selected = select_workbench_popup_row_style(WorkbenchPopupRowState {
        loading: true,
        hovered: true,
        selected: true,
        danger: true,
        ..WorkbenchPopupRowState::default()
    });

    assert_eq!(loading_selected.state, UiPainterResolvedState::Loading);
    assert_eq!(loading_selected.background, None);
    assert_eq!(loading_selected.outline, None);
    assert_eq!(loading_selected.text, PALETTE.text_disabled);
    assert_eq!(loading_selected.shortcut, PALETTE.text_disabled);
    assert_eq!(loading_selected.adornment, PALETTE.text_disabled);
}

#[test]
fn popup_row_focused_only_keeps_normal_background_with_focus_outline() {
    let focused = select_workbench_popup_row_style(WorkbenchPopupRowState {
        focused: true,
        ..WorkbenchPopupRowState::default()
    });

    assert_eq!(focused.state, UiPainterResolvedState::Focused);
    assert_eq!(focused.background, None);
    assert_eq!(focused.outline, Some(PALETTE.accent));
    assert_eq!(focused.text, PALETTE.text);
    assert_eq!(focused.adornment, PALETTE.text_muted);
}

#[test]
fn popup_row_hovered_while_focused_still_uses_hover_fill() {
    let hovered = select_workbench_popup_row_style(WorkbenchPopupRowState {
        focused: true,
        hovered: true,
        ..WorkbenchPopupRowState::default()
    });

    assert_eq!(hovered.state, UiPainterResolvedState::Focused);
    assert_eq!(hovered.background, Some(PALETTE.surface_hover));
    assert_eq!(hovered.outline, Some(PALETTE.accent));
    assert_eq!(hovered.adornment, PALETTE.text);
}

#[test]
fn popup_row_pressed_uses_pressed_fill_without_selected_outline() {
    let pressed = select_workbench_popup_row_style(WorkbenchPopupRowState {
        pressed: true,
        ..WorkbenchPopupRowState::default()
    });

    assert_eq!(pressed.state, UiPainterResolvedState::Pressed);
    assert_eq!(pressed.background, Some(PALETTE.surface_pressed));
    assert_eq!(pressed.outline, None);
    assert_eq!(pressed.text, PALETTE.text);
    assert_eq!(pressed.adornment, PALETTE.text);
}

#[test]
fn popup_row_selected_or_checked_uses_muted_selected_fill_and_teal_outline() {
    let selected = select_workbench_popup_row_style(WorkbenchPopupRowState {
        selected: true,
        focused: true,
        ..WorkbenchPopupRowState::default()
    });

    assert_eq!(selected.state, UiPainterResolvedState::Focused);
    assert_eq!(selected.background, Some(PALETTE.surface_pressed));
    assert_ne!(selected.background, Some(PALETTE.surface_selected));
    assert_eq!(selected.outline, Some(PALETTE.accent));
    assert_ne!(selected.outline, Some(PALETTE.border));

    let checked_pressed = select_workbench_popup_row_style(WorkbenchPopupRowState {
        checked: true,
        pressed: true,
        ..WorkbenchPopupRowState::default()
    });

    assert_eq!(checked_pressed.state, UiPainterResolvedState::Pressed);
    assert_eq!(checked_pressed.background, Some(PALETTE.surface_pressed));
    assert_ne!(checked_pressed.background, Some(PALETTE.surface_selected));
    assert_eq!(checked_pressed.outline, Some(PALETTE.accent));
}
