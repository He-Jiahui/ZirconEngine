use super::*;

#[test]
fn component_state_applies_retained_number_dropdown_collection_and_drop_events() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let number = registry.descriptor("NumberField").unwrap();
    let mut number_state = UiComponentState::new().with_value("value", UiValue::Float(98.0));

    number_state
        .apply_event(
            number,
            UiComponentEvent::DragDelta {
                property: "value".to_string(),
                delta: 8.0,
            },
        )
        .unwrap();
    assert_eq!(number_state.value("value"), Some(&UiValue::Float(100.0)));

    let error = number_state
        .apply_event(
            number,
            UiComponentEvent::Commit {
                property: "value".to_string(),
                value: UiValue::String("not-a-number".to_string()),
            },
        )
        .unwrap_err();
    assert!(error.to_string().contains("not-a-number"));
    assert_eq!(
        number_state.validation.level,
        UiValidationLevel::Error,
        "invalid numeric commits should leave validation state on the retained control"
    );

    let dropdown = registry.descriptor("Dropdown").unwrap();
    let mut dropdown_state = UiComponentState::new();
    dropdown_state
        .apply_event(
            dropdown,
            UiComponentEvent::SelectOption {
                property: "value".to_string(),
                option_id: "runtime".to_string(),
                selected: true,
            },
        )
        .unwrap();
    assert_eq!(
        dropdown_state.value("value"),
        Some(&UiValue::Enum("runtime".to_string()))
    );

    let flags = registry.descriptor("FlagsField").unwrap();
    let mut flags_state = UiComponentState::new();
    flags_state
        .apply_event(
            flags,
            UiComponentEvent::SelectOption {
                property: "value".to_string(),
                option_id: "runtime".to_string(),
                selected: true,
            },
        )
        .unwrap();
    assert_eq!(
        flags_state.value("value"),
        Some(&UiValue::Flags(vec!["runtime".to_string()]))
    );

    let array = registry.descriptor("ArrayField").unwrap();
    let mut array_state = UiComponentState::new().with_value("items", UiValue::Array(Vec::new()));
    array_state
        .apply_event(
            array,
            UiComponentEvent::AddElement {
                property: "items".to_string(),
                value: UiValue::String("Label".to_string()),
            },
        )
        .unwrap();
    assert_eq!(
        array_state.value("items"),
        Some(&UiValue::Array(vec![UiValue::String("Label".to_string())]))
    );

    let map = registry.descriptor("MapField").unwrap();
    let mut map_state = UiComponentState::new();
    map_state
        .apply_event(
            map,
            UiComponentEvent::AddMapEntry {
                property: "entries".to_string(),
                key: "speed".to_string(),
                value: UiValue::Float(1.0),
            },
        )
        .unwrap();
    assert!(
        map_state
            .apply_event(
                map,
                UiComponentEvent::AddMapEntry {
                    property: "entries".to_string(),
                    key: "speed".to_string(),
                    value: UiValue::Float(2.0),
                },
            )
            .is_err(),
        "MapField must reject duplicate keys"
    );

    let group = registry.descriptor("Group").unwrap();
    let mut group_state = UiComponentState::new();
    group_state
        .apply_event(group, UiComponentEvent::ToggleExpanded { expanded: false })
        .unwrap();
    assert_eq!(group_state.value("expanded"), Some(&UiValue::Bool(false)));
    assert!(!group_state.flags.expanded);

    let asset = registry.descriptor("AssetField").unwrap();
    let mut asset_state = UiComponentState::new();
    asset_state
        .apply_event(
            asset,
            UiComponentEvent::DropReference {
                property: "value".to_string(),
                payload: UiDragPayload::new(
                    UiDragPayloadKind::Asset,
                    "res://textures/grid.albedo.png",
                ),
            },
        )
        .unwrap();
    assert_eq!(
        asset_state.value("value"),
        Some(&UiValue::AssetRef(
            "res://textures/grid.albedo.png".to_string()
        ))
    );
    assert!(asset_state
        .apply_event(
            asset,
            UiComponentEvent::DropReference {
                property: "value".to_string(),
                payload: UiDragPayload::new(UiDragPayloadKind::SceneInstance, "scene://Root"),
            },
        )
        .is_err());
}

#[test]
fn drag_payload_source_metadata_roundtrips_and_summarizes() {
    let source = UiDragSourceMetadata::asset(
        "browser",
        "AssetBrowserContentPanel",
        "asset-uuid-1",
        "res://textures/grid.albedo.png",
        "Grid Albedo",
        "Texture",
        "png",
    );
    let payload = UiDragPayload::new(UiDragPayloadKind::Asset, "res://textures/grid.albedo.png")
        .with_source(source.clone());

    assert_eq!(payload.source.as_ref(), Some(&source));
    assert_eq!(
        payload.source_summary().as_deref(),
        Some("Texture: Grid Albedo")
    );

    let encoded = serde_json::to_string(&payload).unwrap();
    let decoded: UiDragPayload = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded, payload);

    let legacy = UiDragPayload::new(UiDragPayloadKind::Asset, "res://legacy.mat");
    assert!(legacy.source.is_none());
    assert!(legacy.source_summary().is_none());

    let decoded_legacy: UiDragPayload =
        serde_json::from_str(r#"{"kind":"Asset","reference":"res://legacy.mat"}"#).unwrap();
    assert!(decoded_legacy.source.is_none());
    assert!(decoded_legacy.source_summary().is_none());

    let legacy_json: serde_json::Value = serde_json::to_value(&legacy).unwrap();
    assert!(legacy_json.get("source").is_none());
}

#[test]
fn component_state_rejects_disabled_selection_options_with_validation_reason() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let dropdown = registry.descriptor("Dropdown").unwrap();
    let mut state =
        UiComponentState::new().with_value("value", UiValue::Enum("primary".to_string()));

    let error = state
        .apply_event(
            dropdown,
            UiComponentEvent::SelectOption {
                property: "value".to_string(),
                option_id: "secondary".to_string(),
                selected: true,
            },
        )
        .unwrap_err();

    assert!(error.to_string().contains("secondary"));
    assert_eq!(
        state.value("value"),
        Some(&UiValue::Enum("primary".to_string()))
    );
    assert_eq!(state.validation.level, UiValidationLevel::Error);
    assert!(state
        .validation
        .message
        .as_deref()
        .is_some_and(|message| message.contains("disabled option")));
}

#[test]
fn component_state_opens_context_action_menu_at_pointer_anchor() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let menu = registry.descriptor("ContextActionMenu").unwrap();
    assert_has_event(menu, UiComponentEventKind::OpenPopupAt);

    let mut state = UiComponentState::new();
    state
        .apply_event(menu, UiComponentEvent::OpenPopupAt { x: 212.0, y: 96.0 })
        .unwrap();

    assert!(state.flags.popup_open);
    assert_eq!(state.value("popup_anchor_x"), Some(&UiValue::Float(212.0)));
    assert_eq!(state.value("popup_anchor_y"), Some(&UiValue::Float(96.0)));
}
