use crate::ui::design_tokens::EditorDesignTokens;
use crate::ui::style::{
    ButtonInteractionState, UiPainterFamily, UiPainterResolvedState, UiPainterState,
    UiPainterStyleSelector, UiPainterVisualState,
};

#[test]
fn ui_painter_primary_state_uses_one_priority_for_every_family() {
    let cases = [
        (
            UiPainterState {
                disabled: true,
                loading: true,
                drop_hovered: true,
                ..UiPainterState::normal()
            },
            UiPainterResolvedState::Disabled,
        ),
        (
            UiPainterState {
                loading: true,
                drop_hovered: true,
                dragging: true,
                ..UiPainterState::normal()
            },
            UiPainterResolvedState::Loading,
        ),
        (
            UiPainterState {
                drop_hovered: true,
                dragging: true,
                pressed: true,
                ..UiPainterState::normal()
            },
            UiPainterResolvedState::DropHovered,
        ),
        (
            UiPainterState {
                dragging: true,
                pressed: true,
                open: true,
                ..UiPainterState::normal()
            },
            UiPainterResolvedState::Dragging,
        ),
        (
            UiPainterState {
                pressed: true,
                open: true,
                checked: true,
                ..UiPainterState::normal()
            },
            UiPainterResolvedState::Pressed,
        ),
        (
            UiPainterState {
                open: true,
                checked: true,
                selected: true,
                ..UiPainterState::normal()
            },
            UiPainterResolvedState::Open,
        ),
        (
            UiPainterState {
                checked: true,
                selected: true,
                hovered: true,
                ..UiPainterState::normal()
            },
            UiPainterResolvedState::Checked,
        ),
        (
            UiPainterState {
                selected: true,
                hovered: true,
                ..UiPainterState::normal()
            },
            UiPainterResolvedState::Selected,
        ),
        (
            UiPainterState {
                hovered: true,
                ..UiPainterState::normal()
            },
            UiPainterResolvedState::Hovered,
        ),
        (UiPainterState::normal(), UiPainterResolvedState::Normal),
    ];

    for family in all_painter_families() {
        for (state, expected) in cases {
            assert_visual_state(state, family, expected);
        }
    }
}

#[test]
fn focus_visible_is_an_overlay_and_never_replaces_primary_identity() {
    let selected_and_focused = UiPainterState {
        focused: true,
        focus_visible: true,
        selected: true,
        hovered: true,
        ..UiPainterState::normal()
    };

    for family in all_painter_families() {
        let visual = UiPainterStyleSelector::visual_state_for_family(selected_and_focused, family);
        assert_eq!(visual.primary, UiPainterResolvedState::Selected);
        assert!(visual.focus_visible);
        assert!(!visual.drop_indicator);
    }

    let pointer_focus = UiPainterState {
        focused: true,
        ..UiPainterState::normal()
    };
    assert_eq!(
        pointer_focus.visual_state_for_family(UiPainterFamily::Button),
        UiPainterVisualState {
            primary: UiPainterResolvedState::Normal,
            focus_visible: false,
            drop_indicator: false,
        }
    );
}

#[test]
fn legacy_scalar_projection_keeps_native_focus_consumers_rendering_during_cutover() {
    let selected_and_focused = UiPainterState {
        selected: true,
        focused: true,
        focus_visible: true,
        ..UiPainterState::normal()
    };

    for family in all_painter_families() {
        let composite = selected_and_focused.visual_state_for_family(family);
        assert_eq!(composite.primary, UiPainterResolvedState::Selected);
        assert!(composite.focus_visible);
        assert_eq!(
            selected_and_focused.resolved_state_for_family(family),
            UiPainterResolvedState::Focused,
            "{family:?} legacy scalar projection must remain visible until M4 migration"
        );
    }
}

#[test]
fn painter_state_defaults_focus_visible_for_older_payloads_and_round_trips_new_intent() {
    let older: UiPainterState =
        serde_json::from_str(r#"{"focused":true}"#).expect("older painter state must deserialize");
    assert!(older.focused);
    assert!(!older.focus_visible);

    let keyboard_focus = UiPainterState {
        focused: true,
        focus_visible: true,
        ..UiPainterState::normal()
    };
    let encoded = serde_json::to_string(&keyboard_focus).expect("painter state must serialize");
    let decoded: UiPainterState =
        serde_json::from_str(&encoded).expect("painter state must round-trip");
    assert_eq!(decoded, keyboard_focus);
}

#[test]
fn drop_indicator_is_composed_without_losing_primary_priority() {
    let disabled_drop_target = UiPainterState {
        disabled: true,
        drop_hovered: true,
        focus_visible: true,
        ..UiPainterState::normal()
    };

    let visual = disabled_drop_target.visual_state_for_family(UiPainterFamily::TreeRow);
    assert_eq!(visual.primary, UiPainterResolvedState::Disabled);
    assert!(visual.focus_visible);
    assert!(visual.drop_indicator);

    let active_drop_target = UiPainterState {
        drop_hovered: true,
        dragging: true,
        pressed: true,
        ..UiPainterState::normal()
    };
    assert_eq!(
        active_drop_target
            .visual_state_for_family(UiPainterFamily::Button)
            .primary,
        UiPainterResolvedState::DropHovered
    );
}

#[test]
fn button_interaction_keeps_legacy_focus_without_changing_primary_state() {
    let keyboard_focus = UiPainterState {
        focused: true,
        focus_visible: true,
        ..UiPainterState::normal()
    };

    assert_eq!(
        keyboard_focus.resolved_state_for_family(UiPainterFamily::Button),
        UiPainterResolvedState::Normal
    );
    assert_eq!(
        keyboard_focus.button_interaction_state(),
        ButtonInteractionState::Focused
    );

    let disabled_focus = UiPainterState {
        disabled: true,
        focus_visible: true,
        ..UiPainterState::normal()
    };
    assert_eq!(
        disabled_focus.button_interaction_state(),
        ButtonInteractionState::Disabled
    );

    let drop_target_focus = UiPainterState {
        drop_hovered: true,
        focus_visible: true,
        ..UiPainterState::normal()
    };
    assert_eq!(
        drop_target_focus.button_interaction_state(),
        ButtonInteractionState::Hover,
        "higher-priority drop target feedback must not collapse into focus"
    );
}

#[test]
fn editor_painter_tokens_compose_focus_and_drop_overlays_over_primary_fill() {
    let tokens = EditorDesignTokens::workbench_dark();
    let selected_focus = tokens.resolve_painter_style(
        UiPainterState {
            selected: true,
            focused: true,
            focus_visible: true,
            ..UiPainterState::normal()
        },
        UiPainterFamily::Button,
    );

    assert_eq!(selected_focus.state, UiPainterResolvedState::Selected);
    assert_eq!(
        selected_focus.background_color,
        tokens.palette.surface_selected
    );
    assert_eq!(selected_focus.border_color, tokens.palette.accent);
    assert_eq!(
        selected_focus.focus_outline_color,
        Some(tokens.palette.focus_ring)
    );
    assert_eq!(selected_focus.drop_indicator_color, None);

    let drop_target = tokens.resolve_painter_style(
        UiPainterState {
            drop_hovered: true,
            selected: true,
            ..UiPainterState::normal()
        },
        UiPainterFamily::TreeRow,
    );
    assert_eq!(drop_target.state, UiPainterResolvedState::DropHovered);
    assert_eq!(drop_target.background_color, tokens.palette.surface[2]);
    assert_eq!(
        drop_target.drop_indicator_color,
        Some(tokens.palette.accent)
    );
}

fn all_painter_families() -> [UiPainterFamily; 18] {
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
        UiPainterFamily::Chrome,
    ]
}

fn assert_visual_state(
    state: UiPainterState,
    family: UiPainterFamily,
    expected: UiPainterResolvedState,
) {
    assert_eq!(
        UiPainterStyleSelector::visual_state_for_family(state, family).primary,
        expected,
        "{family:?} selector priority drifted"
    );
    assert_eq!(
        state.visual_state_for_family(family).primary,
        expected,
        "{family:?} UiPainterState helper drifted from selector"
    );
    assert_eq!(
        state.resolved_state_for_family(family),
        expected,
        "{family:?} legacy primary helper drifted from composite output"
    );
}
