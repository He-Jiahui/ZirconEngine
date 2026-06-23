use super::*;

#[test]
fn material_keyboard_action_steps_numeric_controls_and_closes_popup_controls() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();

    let number = registry
        .descriptor("NumberField")
        .expect("NumberField descriptor");
    assert!(number.supports_event(UiComponentEventKind::KeyboardAction));
    let mut number_state = UiComponentState::new()
        .with_value("value", UiValue::Float(10.0))
        .with_value("step", UiValue::Float(2.0))
        .with_value("large_step", UiValue::Float(5.0));
    number_state
        .apply_event(
            number,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Increment,
            },
        )
        .unwrap();
    assert_eq!(number_state.value("value"), Some(&UiValue::Float(12.0)));
    number_state
        .apply_event(
            number,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::LargeDecrement,
            },
        )
        .unwrap();
    assert_eq!(number_state.value("value"), Some(&UiValue::Float(7.0)));

    let range_slider = registry
        .descriptor("RangeSlider")
        .expect("RangeSlider descriptor");
    assert!(range_slider.supports_event(UiComponentEventKind::KeyboardAction));
    let mut range_state = UiComponentState::new()
        .with_value("range_min", UiValue::Float(20.0))
        .with_value("value", UiValue::Float(25.0))
        .with_value("step", UiValue::Float(10.0))
        .with_value("focused_thumb", UiValue::Enum("upper".to_string()));
    range_state
        .apply_event(
            range_slider,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Decrement,
            },
        )
        .unwrap();
    assert_eq!(range_state.value("value"), Some(&UiValue::Float(20.0)));

    let select = registry.descriptor("Select").expect("Select descriptor");
    assert!(select.supports_event(UiComponentEventKind::KeyboardAction));
    let mut select_state = UiComponentState::new()
        .with_value("popup_open", UiValue::Bool(true))
        .with_value("open", UiValue::Bool(true));
    select_state.flags.popup_open = true;
    select_state
        .apply_event(
            select,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Cancel,
            },
        )
        .unwrap();
    assert!(!select_state.flags.popup_open);
    assert_eq!(
        select_state.value("popup_open"),
        Some(&UiValue::Bool(false))
    );
    assert_eq!(select_state.value("open"), Some(&UiValue::Bool(false)));
}

#[test]
fn material_keyboard_action_targets_range_slider_focused_thumb() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let range_slider = registry
        .descriptor("RangeSlider")
        .expect("RangeSlider descriptor");

    let mut lower_range_state = UiComponentState::new()
        .with_value("range_min", UiValue::Float(20.0))
        .with_value("value", UiValue::Float(80.0))
        .with_value("step", UiValue::Float(10.0))
        .with_value("large_step", UiValue::Float(100.0))
        .with_value("focused_thumb", UiValue::Enum("lower".to_string()));
    lower_range_state
        .apply_event(
            range_slider,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Decrement,
            },
        )
        .unwrap();
    assert_eq!(
        lower_range_state.value("range_min"),
        Some(&UiValue::Float(10.0))
    );
    assert_eq!(
        lower_range_state.value("value"),
        Some(&UiValue::Float(80.0))
    );
    lower_range_state
        .apply_event(
            range_slider,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::LargeIncrement,
            },
        )
        .unwrap();
    assert_eq!(
        lower_range_state.value("range_min"),
        Some(&UiValue::Float(80.0))
    );

    let mut upper_range_state = UiComponentState::new()
        .with_value("range_min", UiValue::Float(20.0))
        .with_value("value", UiValue::Float(80.0))
        .with_value("step", UiValue::Float(10.0))
        .with_value("focused_thumb", UiValue::Enum("upper".to_string()));
    upper_range_state
        .apply_event(
            range_slider,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Increment,
            },
        )
        .unwrap();
    assert_eq!(
        upper_range_state.value("range_min"),
        Some(&UiValue::Float(20.0))
    );
    assert_eq!(
        upper_range_state.value("value"),
        Some(&UiValue::Float(90.0))
    );
}
