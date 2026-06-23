use super::*;

#[test]
fn material_keyboard_action_activates_buttons_and_toggles_checked_controls() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();

    let button = registry.descriptor("Button").expect("Button descriptor");
    assert!(button.supports_event(UiComponentEventKind::KeyboardAction));
    let mut button_state = UiComponentState::new();
    button_state
        .apply_event(
            button,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Activate,
            },
        )
        .unwrap();
    assert_eq!(
        button_state.value("activated"),
        Some(&UiValue::Bool(true)),
        "semantic activate should deliver a retained activation value"
    );

    let checkbox = registry
        .descriptor("Checkbox")
        .expect("Checkbox descriptor");
    assert!(checkbox.supports_event(UiComponentEventKind::KeyboardAction));
    let mut checkbox_state = UiComponentState::new()
        .with_value("checked", UiValue::Bool(false))
        .with_value("indeterminate", UiValue::Bool(true));
    checkbox_state
        .apply_event(
            checkbox,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Activate,
            },
        )
        .unwrap();
    assert_eq!(checkbox_state.value("checked"), Some(&UiValue::Bool(true)));
    assert_eq!(
        checkbox_state.value("indeterminate"),
        Some(&UiValue::Bool(false))
    );
    assert!(checkbox_state.flags.checked);

    checkbox_state
        .apply_event(
            checkbox,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Activate,
            },
        )
        .unwrap();
    assert_eq!(checkbox_state.value("checked"), Some(&UiValue::Bool(false)));
    assert!(!checkbox_state.flags.checked);
}

#[test]
fn material_keyboard_action_moves_tabs_with_selection_following_focus() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let tabs = registry.descriptor("Tabs").expect("Tabs descriptor");
    assert!(tabs.supports_event(UiComponentEventKind::KeyboardAction));
    assert!(tabs.prop("selected_index").is_some());
    assert!(tabs.prop("focused_index").is_some());

    let mut state = UiComponentState::new()
        .with_value(
            "options",
            UiValue::Array(vec![
                UiValue::String("overview".to_string()),
                UiValue::String("details".to_string()),
                UiValue::String("stats".to_string()),
            ]),
        )
        .with_value("value", UiValue::String("overview".to_string()))
        .with_value("selected_index", UiValue::Int(0))
        .with_value("selection_follows_focus", UiValue::Bool(true));

    state
        .apply_event(
            tabs,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Next,
            },
        )
        .unwrap();
    assert_eq!(state.value("focused_index"), Some(&UiValue::Int(1)));
    assert_eq!(state.value("selected_index"), Some(&UiValue::Int(1)));
    assert_eq!(
        state.value("value"),
        Some(&UiValue::String("details".to_string()))
    );
    assert!(state.flags.focused);

    state
        .apply_event(
            tabs,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Last,
            },
        )
        .unwrap();
    assert_eq!(state.value("focused_index"), Some(&UiValue::Int(2)));
    assert_eq!(state.value("selected_index"), Some(&UiValue::Int(2)));
    assert_eq!(
        state.value("value"),
        Some(&UiValue::String("stats".to_string()))
    );

    state
        .apply_event(
            tabs,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Previous,
            },
        )
        .unwrap();
    assert_eq!(state.value("focused_index"), Some(&UiValue::Int(1)));
    assert_eq!(state.value("selected_index"), Some(&UiValue::Int(1)));
    assert_eq!(
        state.value("value"),
        Some(&UiValue::String("details".to_string()))
    );
}

#[test]
fn material_keyboard_action_moves_grouped_selection_controls() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();

    let radio_group = registry
        .descriptor("RadioGroup")
        .expect("RadioGroup descriptor");
    assert!(radio_group.supports_event(UiComponentEventKind::KeyboardAction));
    assert!(radio_group.prop("selected_index").is_some());
    assert!(radio_group.prop("focused_index").is_some());

    let mut radio_state = UiComponentState::new()
        .with_value(
            "options",
            UiValue::Array(vec![
                UiValue::String("editor".to_string()),
                UiValue::String("runtime".to_string()),
                UiValue::String("qa".to_string()),
            ]),
        )
        .with_value("value", UiValue::String("editor".to_string()))
        .with_value("value_text", UiValue::String("editor".to_string()))
        .with_value("group_value", UiValue::String("editor".to_string()))
        .with_value("selected_index", UiValue::Int(0));

    radio_state
        .apply_event(
            radio_group,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Next,
            },
        )
        .unwrap();
    assert_eq!(radio_state.value("focused_index"), Some(&UiValue::Int(1)));
    assert_eq!(radio_state.value("selected_index"), Some(&UiValue::Int(1)));
    assert_eq!(
        radio_state.value("value"),
        Some(&UiValue::String("runtime".to_string()))
    );
    assert_eq!(
        radio_state.value("value_text"),
        Some(&UiValue::String("runtime".to_string()))
    );
    assert_eq!(
        radio_state.value("group_value"),
        Some(&UiValue::String("runtime".to_string()))
    );
    assert!(radio_state.flags.focused);
    assert!(radio_state.flags.selected);

    let toggle_group = registry
        .descriptor("ToggleButtonGroup")
        .expect("ToggleButtonGroup descriptor");
    assert!(toggle_group.supports_event(UiComponentEventKind::KeyboardAction));
    assert!(toggle_group.prop("selected_index").is_some());
    assert!(toggle_group.prop("focused_index").is_some());

    let mut toggle_state = UiComponentState::new()
        .with_value(
            "options",
            UiValue::Array(vec![
                UiValue::String("translate".to_string()),
                UiValue::String("rotate".to_string()),
                UiValue::String("scale".to_string()),
            ]),
        )
        .with_value("value", UiValue::String("translate".to_string()))
        .with_value("selected_index", UiValue::Int(0))
        .with_value("selection_state", UiValue::Enum("exclusive".to_string()))
        .with_value("selection_follows_focus", UiValue::Bool(true));

    toggle_state
        .apply_event(
            toggle_group,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Last,
            },
        )
        .unwrap();
    assert_eq!(toggle_state.value("focused_index"), Some(&UiValue::Int(2)));
    assert_eq!(toggle_state.value("selected_index"), Some(&UiValue::Int(2)));
    assert_eq!(
        toggle_state.value("value"),
        Some(&UiValue::String("scale".to_string()))
    );
    assert_eq!(
        toggle_state.value("value_text"),
        Some(&UiValue::String("scale".to_string()))
    );
    assert!(toggle_state.flags.focused);
    assert!(toggle_state.flags.selected);
}

#[test]
fn material_keyboard_action_skips_disabled_grouped_selection_options() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let radio_group = registry
        .descriptor("RadioGroup")
        .expect("RadioGroup descriptor");

    let mut radio_state = UiComponentState::new()
        .with_value(
            "options",
            UiValue::Array(vec![
                UiValue::String("editor".to_string()),
                UiValue::String("runtime".to_string()),
                UiValue::String("qa".to_string()),
                UiValue::String("shipping".to_string()),
            ]),
        )
        .with_value(
            "disabled_options",
            UiValue::Array(vec![
                UiValue::String("runtime".to_string()),
                UiValue::String("shipping".to_string()),
            ]),
        )
        .with_value("value", UiValue::String("editor".to_string()))
        .with_value("value_text", UiValue::String("editor".to_string()))
        .with_value("group_value", UiValue::String("editor".to_string()))
        .with_value("selected_index", UiValue::Int(0));

    radio_state
        .apply_event(
            radio_group,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Next,
            },
        )
        .unwrap();
    assert_eq!(radio_state.value("focused_index"), Some(&UiValue::Int(2)));
    assert_eq!(radio_state.value("selected_index"), Some(&UiValue::Int(2)));
    assert_eq!(
        radio_state.value("value"),
        Some(&UiValue::String("qa".to_string()))
    );

    radio_state
        .apply_event(
            radio_group,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Last,
            },
        )
        .unwrap();
    assert_eq!(
        radio_state.value("focused_index"),
        Some(&UiValue::Int(2)),
        "Last should stop on the last enabled option"
    );
    assert_eq!(radio_state.value("selected_index"), Some(&UiValue::Int(2)));

    radio_state
        .apply_event(
            radio_group,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Previous,
            },
        )
        .unwrap();
    assert_eq!(radio_state.value("focused_index"), Some(&UiValue::Int(0)));
    assert_eq!(radio_state.value("selected_index"), Some(&UiValue::Int(0)));
    assert_eq!(
        radio_state.value("group_value"),
        Some(&UiValue::String("editor".to_string()))
    );
}

#[test]
fn material_keyboard_action_toggles_multiple_toggle_button_group_focused_option() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let toggle_group = registry
        .descriptor("ToggleButtonGroup")
        .expect("ToggleButtonGroup descriptor");
    assert!(toggle_group.supports_event(UiComponentEventKind::KeyboardAction));

    let mut state = UiComponentState::new()
        .with_value(
            "options",
            UiValue::Array(vec![
                UiValue::String("translate".to_string()),
                UiValue::String("rotate".to_string()),
                UiValue::String("scale".to_string()),
            ]),
        )
        .with_value(
            "value",
            UiValue::Array(vec![UiValue::Enum("translate".to_string())]),
        )
        .with_value("selected_index", UiValue::Int(0))
        .with_value("focused_index", UiValue::Int(1))
        .with_value("selection_state", UiValue::Enum("multiple".to_string()));

    state
        .apply_event(
            toggle_group,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Activate,
            },
        )
        .unwrap();
    assert_eq!(
        state.value("value"),
        Some(&UiValue::Array(vec![
            UiValue::Enum("translate".to_string()),
            UiValue::Enum("rotate".to_string())
        ]))
    );
    assert_eq!(state.value("focused_index"), Some(&UiValue::Int(1)));
    assert_eq!(state.value("selected_index"), Some(&UiValue::Int(0)));
    assert!(state.flags.focused);
    assert!(state.flags.selected);

    state
        .apply_event(
            toggle_group,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Activate,
            },
        )
        .unwrap();
    assert_eq!(
        state.value("value"),
        Some(&UiValue::Array(vec![UiValue::Enum(
            "translate".to_string()
        )]))
    );
    assert_eq!(state.value("selected_index"), Some(&UiValue::Int(0)));
    assert!(!state.flags.selected);

    state = state
        .with_value("focused_index", UiValue::Int(2))
        .with_value(
            "disabled_options",
            UiValue::Array(vec![UiValue::String("scale".to_string())]),
        );
    state
        .apply_event(
            toggle_group,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Activate,
            },
        )
        .unwrap();
    assert_eq!(
        state.value("value"),
        Some(&UiValue::Array(vec![UiValue::Enum(
            "translate".to_string()
        )])),
        "disabled focused options must not mutate the multiple selection"
    );
    assert_eq!(state.value("selected_index"), Some(&UiValue::Int(0)));
}
