use crate::ui::style::{
    ButtonInteractionState, UiPainterFamily, UiPainterResolvedState, UiPainterState,
    UiPainterStyleSelector,
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

#[test]
fn ui_painter_style_selector_is_canonical_for_all_workbench_families() {
    let disabled_over_everything = UiPainterState {
        disabled: true,
        loading: true,
        pressed: true,
        focused: true,
        hovered: true,
        selected: true,
        checked: true,
        open: true,
        dragging: true,
        drop_hovered: true,
    };
    for family in all_painter_families() {
        assert_selector(
            disabled_over_everything,
            family,
            UiPainterResolvedState::Disabled,
        );
    }

    let interactive_busy = UiPainterState {
        loading: true,
        pressed: true,
        focused: true,
        hovered: true,
        selected: true,
        checked: true,
        open: true,
        dragging: true,
        drop_hovered: true,
        ..UiPainterState::normal()
    };
    for family in [
        UiPainterFamily::Generic,
        UiPainterFamily::IconButton,
        UiPainterFamily::Dropdown,
        UiPainterFamily::PopupRow,
        UiPainterFamily::Alert,
        UiPainterFamily::Tooltip,
        UiPainterFamily::TextField,
        UiPainterFamily::ListRow,
        UiPainterFamily::TreeRow,
        UiPainterFamily::TableRow,
        UiPainterFamily::Tab,
        UiPainterFamily::Toast,
    ] {
        assert_selector(interactive_busy, family, UiPainterResolvedState::Loading);
    }
    assert_selector(
        interactive_busy,
        UiPainterFamily::Button,
        UiPainterResolvedState::Loading,
    );

    for family in [
        UiPainterFamily::Checkbox,
        UiPainterFamily::Radio,
        UiPainterFamily::Toggle,
    ] {
        assert_selector(interactive_busy, family, UiPainterResolvedState::Pressed);
    }
    assert_selector(
        interactive_busy,
        UiPainterFamily::Slider,
        UiPainterResolvedState::Pressed,
    );

    let button_hot_without_focus = UiPainterState {
        open: true,
        dragging: true,
        drop_hovered: true,
        ..UiPainterState::normal()
    };
    assert_selector(
        button_hot_without_focus,
        UiPainterFamily::Button,
        UiPainterResolvedState::Hovered,
    );

    let button_selected_is_focus_visible = UiPainterState {
        selected: true,
        ..UiPainterState::normal()
    };
    assert_selector(
        button_selected_is_focus_visible,
        UiPainterFamily::Button,
        UiPainterResolvedState::Focused,
    );
}

fn all_painter_families() -> [UiPainterFamily; 17] {
    [
        UiPainterFamily::Generic,
        UiPainterFamily::Button,
        UiPainterFamily::IconButton,
        UiPainterFamily::Toggle,
        UiPainterFamily::Checkbox,
        UiPainterFamily::Radio,
        UiPainterFamily::Slider,
        UiPainterFamily::Dropdown,
        UiPainterFamily::PopupRow,
        UiPainterFamily::Alert,
        UiPainterFamily::Tooltip,
        UiPainterFamily::TextField,
        UiPainterFamily::ListRow,
        UiPainterFamily::TreeRow,
        UiPainterFamily::TableRow,
        UiPainterFamily::Tab,
        UiPainterFamily::Toast,
    ]
}

fn assert_selector(
    state: UiPainterState,
    family: UiPainterFamily,
    expected: UiPainterResolvedState,
) {
    assert_eq!(
        UiPainterStyleSelector::resolved_state_for_family(state, family),
        expected,
        "{family:?} selector priority drifted"
    );
    assert_eq!(
        state.resolved_state_for_family(family),
        expected,
        "{family:?} UiPainterState helper drifted from selector"
    );
}
