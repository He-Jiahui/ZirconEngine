use super::*;

#[test]
fn component_state_handles_reference_actions_and_drop_rejection_feedback() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();

    for component_id in ["AssetField", "InstanceField", "ObjectField"] {
        let descriptor = registry.descriptor(component_id).unwrap();
        assert_has_event(descriptor, UiComponentEventKind::ClearReference);
        assert_has_event(descriptor, UiComponentEventKind::LocateReference);
        assert_has_event(descriptor, UiComponentEventKind::OpenReference);
    }

    let asset = registry.descriptor("AssetField").unwrap();
    let mut asset_state = UiComponentState::new().with_value(
        "value",
        UiValue::AssetRef("res://materials/demo.mat".to_string()),
    );
    asset_state
        .apply_event(
            asset,
            UiComponentEvent::ClearReference {
                property: "value".to_string(),
            },
        )
        .unwrap();
    assert_eq!(asset_state.value("value"), Some(&UiValue::Null));

    let error = asset_state
        .apply_event(
            asset,
            UiComponentEvent::OpenReference {
                property: "value".to_string(),
            },
        )
        .unwrap_err();
    assert!(error.to_string().contains("value"));
    assert_eq!(asset_state.validation.level, UiValidationLevel::Error);

    let rejected = asset_state
        .apply_event(
            asset,
            UiComponentEvent::DropReference {
                property: "value".to_string(),
                payload: UiDragPayload::new(UiDragPayloadKind::SceneInstance, "scene://Root"),
            },
        )
        .unwrap_err();
    assert!(rejected.to_string().contains("scene-instance"));
    assert_eq!(asset_state.validation.level, UiValidationLevel::Error);
    assert!(asset_state
        .validation
        .message
        .as_deref()
        .is_some_and(|message| message.contains("rejected drop")));

    let instance = registry.descriptor("InstanceField").unwrap();
    let mut instance_state = UiComponentState::new();
    instance_state
        .apply_event(
            instance,
            UiComponentEvent::DropReference {
                property: "value".to_string(),
                payload: UiDragPayload::new(UiDragPayloadKind::SceneInstance, "scene://Root/Light"),
            },
        )
        .unwrap();
    assert_eq!(
        instance_state.value("value"),
        Some(&UiValue::InstanceRef("scene://Root/Light".to_string()))
    );

    let object = registry.descriptor("ObjectField").unwrap();
    let mut object_state = UiComponentState::new().with_value(
        "value",
        UiValue::InstanceRef("object://Selection/MainCamera".to_string()),
    );
    object_state
        .apply_event(
            object,
            UiComponentEvent::LocateReference {
                property: "value".to_string(),
            },
        )
        .unwrap();
    object_state
        .apply_event(
            object,
            UiComponentEvent::OpenReference {
                property: "value".to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        object_state.value("value"),
        Some(&UiValue::InstanceRef(
            "object://Selection/MainCamera".to_string()
        ))
    );
}

#[test]
fn component_state_retains_reference_drop_source_metadata() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let asset = registry.descriptor("AssetField").unwrap();
    let source = test_asset_source();
    let mut state = UiComponentState::new();

    state
        .apply_event(
            asset,
            UiComponentEvent::DropReference {
                property: "value".to_string(),
                payload: UiDragPayload::new(
                    UiDragPayloadKind::Asset,
                    "res://textures/grid.albedo.png",
                )
                .with_source(source.clone()),
            },
        )
        .unwrap();

    assert_eq!(
        state.value("value"),
        Some(&UiValue::AssetRef(
            "res://textures/grid.albedo.png".to_string()
        ))
    );
    assert_eq!(state.reference_source("value"), Some(&source));

    state
        .apply_event(
            asset,
            UiComponentEvent::ClearReference {
                property: "value".to_string(),
            },
        )
        .unwrap();

    assert_eq!(state.value("value"), Some(&UiValue::Null));
    assert_eq!(state.reference_source("value"), None);
}

#[test]
fn component_state_serializes_reference_sources_compatibly() {
    let legacy_state = serde_json::json!({
        "values": {
            "value": { "AssetRef": "res://textures/grid.albedo.png" }
        },
        "validation": {
            "level": "Normal",
            "message": null
        },
        "flags": {
            "focused": false,
            "dragging": false,
            "popup_open": false,
            "expanded": false,
            "selected": false,
            "checked": false,
            "disabled": false
        }
    });

    let decoded: UiComponentState = serde_json::from_value(legacy_state).unwrap();
    assert_eq!(decoded.reference_source("value"), None);

    let empty_json = serde_json::to_value(UiComponentState::new()).unwrap();
    assert!(empty_json.get("reference_sources").is_none());

    let no_source_json = serde_json::to_value(
        UiComponentState::new()
            .with_value("value", UiValue::AssetRef("res://legacy.mat".to_string())),
    )
    .unwrap();
    assert!(no_source_json.get("reference_sources").is_none());
}

#[test]
fn component_state_sourced_drop_reference_survives_serde_roundtrip() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let asset = registry.descriptor("AssetField").unwrap();
    let source = test_asset_source();
    let mut state = UiComponentState::new();

    state
        .apply_event(
            asset,
            UiComponentEvent::DropReference {
                property: "value".to_string(),
                payload: UiDragPayload::new(
                    UiDragPayloadKind::Asset,
                    "res://textures/grid.albedo.png",
                )
                .with_source(source.clone()),
            },
        )
        .unwrap();

    let encoded = serde_json::to_string(&state).unwrap();
    let decoded: UiComponentState = serde_json::from_str(&encoded).unwrap();

    assert!(
        serde_json::from_str::<serde_json::Value>(&encoded)
            .unwrap()
            .get("reference_sources")
            .is_some(),
        "non-empty retained reference sources should be serialized"
    );
    assert_eq!(decoded.reference_source("value"), Some(&source));
    assert_eq!(
        decoded.value("value"),
        Some(&UiValue::AssetRef(
            "res://textures/grid.albedo.png".to_string()
        ))
    );
}

#[test]
fn component_state_clears_reference_source_on_sourceless_accepted_drop() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let asset = registry.descriptor("AssetField").unwrap();
    let source = test_asset_source();
    let mut state = UiComponentState::new();

    state
        .apply_event(
            asset,
            UiComponentEvent::DropReference {
                property: "value".to_string(),
                payload: UiDragPayload::new(
                    UiDragPayloadKind::Asset,
                    "res://textures/grid.albedo.png",
                )
                .with_source(source),
            },
        )
        .unwrap();
    state
        .apply_event(
            asset,
            UiComponentEvent::DropReference {
                property: "value".to_string(),
                payload: UiDragPayload::new(
                    UiDragPayloadKind::Asset,
                    "res://textures/grid.normal.png",
                ),
            },
        )
        .unwrap();

    assert_eq!(
        state.value("value"),
        Some(&UiValue::AssetRef(
            "res://textures/grid.normal.png".to_string()
        ))
    );
    assert_eq!(state.reference_source("value"), None);
}

#[test]
fn component_state_preserves_reference_source_on_rejected_drop() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let asset = registry.descriptor("AssetField").unwrap();
    let source = test_asset_source();
    let mut state = UiComponentState::new();

    state
        .apply_event(
            asset,
            UiComponentEvent::DropReference {
                property: "value".to_string(),
                payload: UiDragPayload::new(
                    UiDragPayloadKind::Asset,
                    "res://textures/grid.albedo.png",
                )
                .with_source(source.clone()),
            },
        )
        .unwrap();
    let error = state
        .apply_event(
            asset,
            UiComponentEvent::DropReference {
                property: "value".to_string(),
                payload: UiDragPayload::new(UiDragPayloadKind::SceneInstance, "scene://Root"),
            },
        )
        .unwrap_err();

    assert!(error.to_string().contains("scene-instance"));
    assert_eq!(
        state.value("value"),
        Some(&UiValue::AssetRef(
            "res://textures/grid.albedo.png".to_string()
        ))
    );
    assert_eq!(state.reference_source("value"), Some(&source));
}
