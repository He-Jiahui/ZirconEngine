use crate::ui::component::{UiComponentDescriptorRegistry, UiComponentStateRuntimeExt};
use zircon_runtime_interface::ui::component::{
    UiComponentEvent, UiComponentEventKind, UiComponentState, UiValidationLevel, UiValue,
};

#[test]
fn text_input_commit_timing_defers_validation_until_commit() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let text_field = registry
        .descriptor("TextField")
        .expect("TextField descriptor");
    assert!(text_field.supports_event(UiComponentEventKind::ValueChanged));
    assert!(text_field.supports_event(UiComponentEventKind::Commit));

    let mut state = UiComponentState::new()
        .with_value("required", UiValue::Bool(true))
        .with_value("min_length", UiValue::Int(3))
        .with_value("validation_timing", UiValue::Enum("commit".to_string()));

    state
        .apply_event(
            text_field,
            UiComponentEvent::ValueChanged {
                property: "value_text".to_string(),
                value: UiValue::String("a".to_string()),
            },
        )
        .unwrap();
    assert_eq!(
        state.value("value_text"),
        Some(&UiValue::String("a".to_string()))
    );
    assert_eq!(
        state.validation.level,
        UiValidationLevel::Normal,
        "typing should not show validation errors before commit"
    );
    assert_eq!(state.value("validation_dirty"), Some(&UiValue::Bool(true)));

    state
        .apply_event(
            text_field,
            UiComponentEvent::Commit {
                property: "value_text".to_string(),
                value: UiValue::String("a".to_string()),
            },
        )
        .unwrap();
    assert_eq!(state.validation.level, UiValidationLevel::Error);
    assert!(
        state
            .validation
            .message
            .as_deref()
            .is_some_and(|message| message.contains("at least 3"))
    );
    assert_eq!(
        state.value("validation_level"),
        Some(&UiValue::Enum("error".to_string()))
    );
    assert_eq!(state.value("validation_dirty"), Some(&UiValue::Bool(false)));
    assert_eq!(
        state.value("validation_touched"),
        Some(&UiValue::Bool(true))
    );

    state
        .apply_event(
            text_field,
            UiComponentEvent::ValueChanged {
                property: "value_text".to_string(),
                value: UiValue::String("abc".to_string()),
            },
        )
        .unwrap();
    assert_eq!(
        state.value("value_text"),
        Some(&UiValue::String("abc".to_string()))
    );
    assert_eq!(
        state.validation.level,
        UiValidationLevel::Normal,
        "editing after a failed commit should clear stale error feedback"
    );

    state
        .apply_event(
            text_field,
            UiComponentEvent::Commit {
                property: "value_text".to_string(),
                value: UiValue::String("abc".to_string()),
            },
        )
        .unwrap();
    assert_eq!(state.validation.level, UiValidationLevel::Normal);
    assert_eq!(
        state.value("validation_level"),
        Some(&UiValue::Enum("normal".to_string()))
    );
    assert_eq!(
        state.value("validation_message"),
        Some(&UiValue::String(String::new()))
    );
}

#[test]
fn text_input_blur_timing_validates_on_focus_loss() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let textarea = registry
        .descriptor("TextareaAutosize")
        .expect("TextareaAutosize descriptor");
    assert!(textarea.supports_event(UiComponentEventKind::Focus));
    assert!(textarea.supports_event(UiComponentEventKind::ValueChanged));

    let mut state = UiComponentState::new()
        .with_value("required", UiValue::Bool(true))
        .with_value("validation_timing", UiValue::Enum("blur".to_string()))
        .with_value(
            "validation_message",
            UiValue::String("name is required".to_string()),
        );

    state
        .apply_event(
            textarea,
            UiComponentEvent::ValueChanged {
                property: "value_text".to_string(),
                value: UiValue::String(String::new()),
            },
        )
        .unwrap();
    assert_eq!(
        state.validation.level,
        UiValidationLevel::Normal,
        "blur timing should defer required validation while the user edits"
    );
    assert_eq!(state.value("validation_dirty"), Some(&UiValue::Bool(true)));

    state
        .apply_event(textarea, UiComponentEvent::Focus { focused: false })
        .unwrap();
    assert_eq!(state.validation.level, UiValidationLevel::Error);
    assert_eq!(
        state.validation.message.as_deref(),
        Some("name is required")
    );
    assert_eq!(state.value("validation_dirty"), Some(&UiValue::Bool(false)));
    assert_eq!(
        state.value("validation_touched"),
        Some(&UiValue::Bool(true))
    );

    state
        .apply_event(
            textarea,
            UiComponentEvent::ValueChanged {
                property: "value_text".to_string(),
                value: UiValue::String("Scene".to_string()),
            },
        )
        .unwrap();
    assert_eq!(state.validation.level, UiValidationLevel::Normal);

    state
        .apply_event(textarea, UiComponentEvent::Focus { focused: false })
        .unwrap();
    assert_eq!(state.validation.level, UiValidationLevel::Normal);
    assert_eq!(
        state.value("validation_level"),
        Some(&UiValue::Enum("normal".to_string()))
    );
}

#[test]
fn text_input_change_timing_validates_max_length_live() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let search = registry
        .descriptor("SearchField")
        .expect("SearchField descriptor");

    let mut state = UiComponentState::new()
        .with_value("max_length", UiValue::Int(4))
        .with_value("validation_timing", UiValue::Enum("change".to_string()));

    state
        .apply_event(
            search,
            UiComponentEvent::ValueChanged {
                property: "query".to_string(),
                value: UiValue::String("camera".to_string()),
            },
        )
        .unwrap();
    assert_eq!(state.validation.level, UiValidationLevel::Error);
    assert!(
        state
            .validation
            .message
            .as_deref()
            .is_some_and(|message| message.contains("at most 4"))
    );
    assert_eq!(
        state.value("validation_level"),
        Some(&UiValue::Enum("error".to_string()))
    );

    state
        .apply_event(
            search,
            UiComponentEvent::ValueChanged {
                property: "query".to_string(),
                value: UiValue::String("cam".to_string()),
            },
        )
        .unwrap();
    assert_eq!(state.validation.level, UiValidationLevel::Normal);
}
