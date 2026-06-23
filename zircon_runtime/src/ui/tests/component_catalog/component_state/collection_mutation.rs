use super::*;

#[test]
fn component_state_edits_and_reorders_array_elements() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let array = registry.descriptor("ArrayField").unwrap();
    let mut state = UiComponentState::new().with_value(
        "items",
        UiValue::Array(vec![
            UiValue::String("Position".to_string()),
            UiValue::String("Rotation".to_string()),
            UiValue::String("Scale".to_string()),
        ]),
    );

    state
        .apply_event(
            array,
            UiComponentEvent::SetElement {
                property: "items".to_string(),
                index: 1,
                value: UiValue::String("Orientation".to_string()),
            },
        )
        .unwrap();
    state
        .apply_event(
            array,
            UiComponentEvent::MoveElement {
                property: "items".to_string(),
                from: 2,
                to: 0,
            },
        )
        .unwrap();
    state
        .apply_event(
            array,
            UiComponentEvent::RemoveElement {
                property: "items".to_string(),
                index: 1,
            },
        )
        .unwrap();

    assert_eq!(
        state.value("items"),
        Some(&UiValue::Array(vec![
            UiValue::String("Scale".to_string()),
            UiValue::String("Orientation".to_string()),
        ]))
    );
}

#[test]
fn component_state_renames_map_keys_and_rejects_duplicate_targets() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let map = registry.descriptor("MapField").unwrap();
    assert_has_event(map, UiComponentEventKind::RenameMapKey);

    let mut entries = BTreeMap::new();
    entries.insert("speed".to_string(), UiValue::Float(1.0));
    entries.insert("visible".to_string(), UiValue::Bool(true));
    let mut state = UiComponentState::new().with_value("entries", UiValue::Map(entries));

    state
        .apply_event(
            map,
            UiComponentEvent::RenameMapKey {
                property: "entries".to_string(),
                from_key: "speed".to_string(),
                to_key: "velocity".to_string(),
            },
        )
        .unwrap();

    let Some(UiValue::Map(entries)) = state.value("entries") else {
        panic!("entries should stay a map");
    };
    assert!(!entries.contains_key("speed"));
    assert_eq!(entries.get("velocity"), Some(&UiValue::Float(1.0)));

    let error = state
        .apply_event(
            map,
            UiComponentEvent::RenameMapKey {
                property: "entries".to_string(),
                from_key: "velocity".to_string(),
                to_key: "visible".to_string(),
            },
        )
        .unwrap_err();
    assert!(matches!(
        error,
        UiComponentEventError::DuplicateMapKey { .. }
    ));
    assert_eq!(state.validation.level, UiValidationLevel::Error);
    assert!(state
        .validation
        .message
        .as_deref()
        .is_some_and(|message| message.contains("already exists")));

    let Some(UiValue::Map(entries)) = state.value("entries") else {
        panic!("entries should stay a map");
    };
    assert_eq!(entries.get("velocity"), Some(&UiValue::Float(1.0)));
}

#[test]
fn component_state_sets_collection_validation_on_row_errors() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let array = registry.descriptor("ArrayField").unwrap();
    let map = registry.descriptor("MapField").unwrap();

    let mut array_state = UiComponentState::new().with_value("items", UiValue::Array(Vec::new()));
    let array_error = array_state
        .apply_event(
            array,
            UiComponentEvent::RemoveElement {
                property: "items".to_string(),
                index: 2,
            },
        )
        .unwrap_err();
    assert!(matches!(
        array_error,
        UiComponentEventError::ArrayIndexOutOfBounds { .. }
    ));
    assert_eq!(array_state.validation.level, UiValidationLevel::Error);
    assert!(array_state
        .validation
        .message
        .as_deref()
        .is_some_and(|message| message.contains("index 2")));

    let mut entries = BTreeMap::new();
    entries.insert("speed".to_string(), UiValue::Float(1.0));
    let mut map_state = UiComponentState::new().with_value("entries", UiValue::Map(entries));
    let map_error = map_state
        .apply_event(
            map,
            UiComponentEvent::SetMapEntry {
                property: "entries".to_string(),
                key: "missing".to_string(),
                value: UiValue::Bool(true),
            },
        )
        .unwrap_err();
    assert!(matches!(
        map_error,
        UiComponentEventError::MissingMapKey { .. }
    ));
    assert_eq!(map_state.validation.level, UiValidationLevel::Error);
    assert!(map_state
        .validation
        .message
        .as_deref()
        .is_some_and(|message| message.contains("does not exist")));

    let remove_error = map_state
        .apply_event(
            map,
            UiComponentEvent::RemoveMapEntry {
                property: "entries".to_string(),
                key: "missing".to_string(),
            },
        )
        .unwrap_err();
    assert!(matches!(
        remove_error,
        UiComponentEventError::MissingMapKey { .. }
    ));
    let Some(UiValue::Map(entries)) = map_state.value("entries") else {
        panic!("entries should stay a map after rejected removal");
    };
    assert_eq!(entries.get("speed"), Some(&UiValue::Float(1.0)));
}
