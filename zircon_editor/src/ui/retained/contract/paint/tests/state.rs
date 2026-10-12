use super::super::super::super::data::TemplatePaneNodeData;
use super::super::{
    checkbox_background, checkbox_border_color, control_accent_color, control_border_color,
    radio_background, radio_border_color, selection_mark_label_color, selection_text_color,
    selection_visual_state, selection_visual_unavailable, toggle_thumb_color, toggle_track_color,
    CHECKBOX_CHECKED_FILL, PALETTE,
};
use super::support::{
    node_with_role, resolved_background_foreground_and_border, SELECTION_MARK_IDLE_FILL,
};
use zircon_runtime_interface::ui::style::{ResolvedButtonStyle, UiPainterResolvedState};

#[test]
fn selection_control_uses_shared_selector_for_pressed_checked_border() {
    let node = TemplatePaneNodeData {
        checked: true,
        pressed: true,
        ..node_with_role("Checkbox", "checkbox", "WorkbenchCheckboxOn")
    };

    assert_eq!(
        selection_visual_state(&node),
        UiPainterResolvedState::Pressed
    );
    assert_eq!(checkbox_background(&node), CHECKBOX_CHECKED_FILL);
    assert_eq!(checkbox_border_color(&node), PALETTE.accent);
    assert_ne!(checkbox_border_color(&node), PALETTE.focus_ring);
}

#[test]
fn selection_control_loading_state_mutes_active_checked_visuals() {
    let node = TemplatePaneNodeData {
        checked: true,
        selected: true,
        pressed: true,
        hovered: true,
        drop_hovered: true,
        label_color: crate::ui::retained_host::primitives::Color::from_rgb_u8(131, 141, 148),
        value_color: crate::ui::retained_host::primitives::Color::from_rgb_u8(67, 216, 226),
        button_style: ResolvedButtonStyle {
            loading: true,
            ..resolved_background_foreground_and_border(
                [32, 159, 168, 255],
                [255, 255, 255, 255],
                [53, 199, 208, 255],
            )
        },
        ..node_with_role("Checkbox", "checkbox", "WorkbenchCheckboxOn")
    };

    assert_eq!(
        selection_visual_state(&node),
        UiPainterResolvedState::Loading
    );
    assert!(selection_visual_unavailable(&node));
    assert_eq!(checkbox_background(&node), PALETTE.surface_disabled);
    assert_eq!(checkbox_border_color(&node), PALETTE.border_disabled);
    assert_eq!(radio_background(&node), PALETTE.surface_disabled);
    assert_eq!(radio_border_color(&node), PALETTE.border_disabled);
    assert_eq!(toggle_track_color(&node), PALETTE.surface_disabled);
    assert_eq!(toggle_thumb_color(&node), PALETTE.text_disabled);
    assert_eq!(control_border_color(&node), PALETTE.border_disabled);
    assert_eq!(control_accent_color(&node), PALETTE.text_disabled);
    assert_eq!(selection_text_color(&node), PALETTE.text_disabled);
    assert_eq!(selection_mark_label_color(&node), PALETTE.text_disabled);
}

#[test]
fn focused_unchecked_selection_controls_keep_idle_surfaces_with_focus_border() {
    let toggle = TemplatePaneNodeData {
        focused: true,
        ..node_with_role("Toggle", "toggle", "WorkbenchToggleOff")
    };
    let checkbox = TemplatePaneNodeData {
        focused: true,
        ..node_with_role("Checkbox", "checkbox", "WorkbenchCheckboxOff")
    };

    assert_eq!(toggle_track_color(&toggle), PALETTE.track);
    assert_eq!(control_border_color(&toggle), PALETTE.focus_ring);
    assert_eq!(checkbox_background(&checkbox), SELECTION_MARK_IDLE_FILL);
    assert_eq!(checkbox_border_color(&checkbox), PALETTE.focus_ring);
}

#[test]
fn focused_hovered_unchecked_toggle_keeps_hover_track_with_focus_border() {
    let toggle = TemplatePaneNodeData {
        focused: true,
        hovered: true,
        ..node_with_role("Toggle", "toggle", "WorkbenchToggleOff")
    };

    assert_eq!(toggle_track_color(&toggle), PALETTE.surface_hover);
    assert_eq!(control_border_color(&toggle), PALETTE.focus_ring);
}
