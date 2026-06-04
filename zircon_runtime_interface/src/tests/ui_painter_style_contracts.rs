use crate::ui::style::{
    ButtonInteractionState, UiPainterFamily, UiPainterResolvedState, UiPainterState,
};

#[test]
fn ui_painter_state_resolves_family_specific_priority() {
    let state = UiPainterState {
        hovered: true,
        focused: true,
        pressed: false,
        checked: true,
        selected: false,
        disabled: false,
        open: true,
        dragging: false,
        drop_hovered: false,
        loading: false,
    };

    assert_eq!(
        state.resolved_state_for_family(UiPainterFamily::Button),
        UiPainterResolvedState::Focused
    );
    assert_eq!(
        state.resolved_state_for_family(UiPainterFamily::Checkbox),
        UiPainterResolvedState::Focused
    );
    assert_eq!(
        state.resolved_state_for_family(UiPainterFamily::Slider),
        UiPainterResolvedState::Focused
    );
    assert_eq!(
        state.resolved_state_for_family(UiPainterFamily::Dropdown),
        UiPainterResolvedState::Focused
    );
    assert_eq!(
        state.resolved_state_for_family(UiPainterFamily::Alert),
        UiPainterResolvedState::Focused
    );
}

#[test]
fn ui_painter_state_keeps_disabled_and_loading_priorities_explicit() {
    let disabled_pressed = UiPainterState {
        disabled: true,
        loading: true,
        pressed: true,
        hovered: true,
        ..UiPainterState::normal()
    };
    assert_eq!(
        disabled_pressed.resolved_state_for_family(UiPainterFamily::Slider),
        UiPainterResolvedState::Disabled
    );
    assert_eq!(
        disabled_pressed.button_interaction_state(),
        ButtonInteractionState::Disabled
    );

    let loading_button = UiPainterState {
        loading: true,
        pressed: true,
        hovered: true,
        ..UiPainterState::normal()
    };
    assert_eq!(
        loading_button.resolved_state_for_family(UiPainterFamily::Button),
        UiPainterResolvedState::Loading
    );
    assert_eq!(
        loading_button.button_interaction_state(),
        ButtonInteractionState::Loading
    );
    assert_eq!(
        loading_button.resolved_state_for_family(UiPainterFamily::Slider),
        UiPainterResolvedState::Pressed
    );
}
