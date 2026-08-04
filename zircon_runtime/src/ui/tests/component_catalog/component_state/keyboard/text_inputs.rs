use super::*;

#[test]
fn material_keyboard_text_appends_text_input_values_without_full_editing_policy() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();

    let search = registry
        .descriptor("SearchField")
        .expect("SearchField descriptor");
    assert!(search.supports_event(UiComponentEventKind::KeyboardText));
    let mut search_state =
        UiComponentState::new().with_value("query", UiValue::String("sc".to_string()));
    search_state
        .apply_event(
            search,
            UiComponentEvent::KeyboardText {
                text: "ene".to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        search_state.value("query"),
        Some(&UiValue::String("scene".to_string()))
    );
    assert!(search_state.flags.focused);

    let text_field = registry
        .descriptor("TextField")
        .expect("TextField descriptor");
    assert!(text_field.supports_event(UiComponentEventKind::KeyboardText));
    let mut text_state =
        UiComponentState::new().with_value("value_text", UiValue::String("Mat".to_string()));
    text_state
        .apply_event(
            text_field,
            UiComponentEvent::KeyboardText {
                text: "erial".to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        text_state.value("value_text"),
        Some(&UiValue::String("Material".to_string()))
    );

    let input = registry.descriptor("Input").expect("Input descriptor");
    assert!(input.supports_event(UiComponentEventKind::KeyboardText));
    let mut input_state =
        UiComponentState::new().with_value("value_text", UiValue::String("UI".to_string()));
    input_state
        .apply_event(
            input,
            UiComponentEvent::KeyboardText {
                text: " Kit".to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        input_state.value("value_text"),
        Some(&UiValue::String("UI Kit".to_string()))
    );
    assert_eq!(
        input_state.value("value"),
        Some(&UiValue::String("UI Kit".to_string())),
        "MUI text inputs keep value_text and value mirrored for render and schema consumers"
    );

    let textarea = registry
        .descriptor("TextareaAutosize")
        .expect("TextareaAutosize descriptor");
    let mut textarea_state =
        UiComponentState::new().with_value("value_text", UiValue::String("line".to_string()));
    textarea_state
        .apply_event(
            textarea,
            UiComponentEvent::KeyboardText {
                text: "\n 2".to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        textarea_state.value("value_text"),
        Some(&UiValue::String("line 2".to_string())),
        "control characters are dropped, but printable spacing is preserved"
    );

    let source_editor = registry
        .descriptor("SourceEditor")
        .expect("SourceEditor descriptor");
    let mut source_state =
        UiComponentState::new().with_value("text", UiValue::String("let ".to_string()));
    source_state
        .apply_event(
            source_editor,
            UiComponentEvent::KeyboardText {
                text: "x".to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        source_state.value("text"),
        Some(&UiValue::String("let x".to_string()))
    );

    source_state
        .apply_event(
            source_editor,
            UiComponentEvent::KeyboardText {
                text: "\t".to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        source_state.value("text"),
        Some(&UiValue::String("let x".to_string())),
        "whitespace-only text payloads are not treated as editor text before the full plan-03 editing chain"
    );

    let mut readonly_state = UiComponentState::new()
        .with_value("value_text", UiValue::String("locked".to_string()))
        .with_value("readOnly", UiValue::Bool(true));
    readonly_state
        .apply_event(
            text_field,
            UiComponentEvent::KeyboardText {
                text: "!".to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        readonly_state.value("value_text"),
        Some(&UiValue::String("locked".to_string()))
    );
}

#[test]
fn material_keyboard_text_replaces_text_input_selection_and_updates_caret_state() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let text_field = registry
        .descriptor("TextField")
        .expect("TextField descriptor");
    assert!(text_field.supports_event(UiComponentEventKind::KeyboardText));
    assert!(text_field.prop("caret_offset").is_some());
    assert_eq!(
        text_field
            .prop("caret_affinity")
            .and_then(|prop| prop.default_value.as_ref()),
        Some(&UiValue::String("downstream".to_string()))
    );
    assert!(text_field.prop("selection_anchor").is_some());
    assert!(text_field.prop("selection_focus").is_some());

    let mut state = UiComponentState::new()
        .with_value("value_text", UiValue::String("abcd".to_string()))
        .with_value("caret_offset", UiValue::Int(3))
        .with_value("selection_anchor", UiValue::Int(1))
        .with_value("selection_focus", UiValue::Int(3));
    state
        .apply_event(
            text_field,
            UiComponentEvent::KeyboardText {
                text: "X".to_string(),
            },
        )
        .unwrap();

    assert_eq!(
        state.value("value_text"),
        Some(&UiValue::String("aXd".to_string()))
    );
    assert_eq!(state.value("caret_offset"), Some(&UiValue::Int(2)));
    assert_eq!(state.value("selection_anchor"), Some(&UiValue::Int(2)));
    assert_eq!(state.value("selection_focus"), Some(&UiValue::Int(2)));
    assert!(state.flags.focused);
}
