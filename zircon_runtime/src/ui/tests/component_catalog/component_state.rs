use std::collections::BTreeMap;

use crate::ui::component::UiComponentStateRuntimeExt;
use zircon_runtime_interface::ui::component::UiComponentEventError;

use super::*;

mod selection;
mod value_validation;

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

#[test]
fn component_state_applies_transient_interaction_flags() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let list_row = registry.descriptor("ListRow").unwrap();
    assert_has_event(list_row, UiComponentEventKind::Hover);
    assert_has_event(list_row, UiComponentEventKind::Press);

    let mut row_state = UiComponentState::new();
    row_state
        .apply_event(list_row, UiComponentEvent::Hover { hovered: true })
        .unwrap();
    row_state
        .apply_event(list_row, UiComponentEvent::Press { pressed: true })
        .unwrap();

    assert!(row_state.flags.hovered);
    assert!(row_state.flags.pressed);

    row_state
        .apply_event(list_row, UiComponentEvent::Hover { hovered: false })
        .unwrap();
    row_state
        .apply_event(list_row, UiComponentEvent::Press { pressed: false })
        .unwrap();

    assert!(!row_state.flags.hovered);
    assert!(!row_state.flags.pressed);

    let asset = registry.descriptor("AssetField").unwrap();
    assert_has_event(asset, UiComponentEventKind::DropHover);
    assert_has_event(asset, UiComponentEventKind::ActiveDragTarget);

    let mut asset_state = UiComponentState::new();
    asset_state
        .apply_event(asset, UiComponentEvent::DropHover { hovered: true })
        .unwrap();
    asset_state
        .apply_event(asset, UiComponentEvent::ActiveDragTarget { active: true })
        .unwrap();

    assert!(asset_state.flags.drop_hovered);
    assert!(asset_state.flags.active_drag_target);
}

#[test]
fn component_state_clears_reference_source_on_non_drop_value_replacement() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let asset = registry.descriptor("AssetField").unwrap();
    let input = registry.descriptor("InputField").unwrap();
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
    state = state.with_value(
        "value",
        UiValue::AssetRef("res://textures/overridden.png".to_string()),
    );
    assert_eq!(state.reference_source("value"), None);

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
            input,
            UiComponentEvent::ValueChanged {
                property: "value".to_string(),
                value: UiValue::String("manual override".to_string()),
            },
        )
        .unwrap();

    assert_eq!(
        state.value("value"),
        Some(&UiValue::String("manual override".to_string()))
    );
    assert_eq!(state.reference_source("value"), None);
}

#[test]
fn component_state_updates_existing_map_entries_without_creating_keys() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let map = registry.descriptor("MapField").unwrap();
    assert_has_event(map, UiComponentEventKind::SetMapEntry);

    let mut entries = std::collections::BTreeMap::new();
    entries.insert("speed".to_string(), UiValue::Float(1.0));
    entries.insert("visible".to_string(), UiValue::Bool(true));
    let mut state = UiComponentState::new().with_value("entries", UiValue::Map(entries));

    state
        .apply_event(
            map,
            UiComponentEvent::SetMapEntry {
                property: "entries".to_string(),
                key: "speed".to_string(),
                value: UiValue::Float(2.5),
            },
        )
        .unwrap();

    assert!(state
        .apply_event(
            map,
            UiComponentEvent::SetMapEntry {
                property: "entries".to_string(),
                key: "missing".to_string(),
                value: UiValue::String("value".to_string()),
            },
        )
        .is_err());

    let Some(UiValue::Map(entries)) = state.value("entries") else {
        panic!("expected retained map entries");
    };
    assert_eq!(entries.get("speed"), Some(&UiValue::Float(2.5)));
    assert!(!entries.contains_key("missing"));
}

#[test]
fn component_state_applies_numeric_state_step_large_step_and_clamp_settings() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let number = registry.descriptor("NumberField").unwrap();
    assert_has_event(number, UiComponentEventKind::LargeDragDelta);

    let mut state = UiComponentState::new()
        .with_value("value", UiValue::Float(50.0))
        .with_value("min", UiValue::Float(10.0))
        .with_value("max", UiValue::Float(60.0))
        .with_value("step", UiValue::Float(0.5))
        .with_value("large_step", UiValue::Float(5.0));

    state
        .apply_event(
            number,
            UiComponentEvent::DragDelta {
                property: "value".to_string(),
                delta: 4.0,
            },
        )
        .unwrap();
    assert_eq!(state.value("value"), Some(&UiValue::Float(52.0)));

    state
        .apply_event(
            number,
            UiComponentEvent::LargeDragDelta {
                property: "value".to_string(),
                delta: 2.0,
            },
        )
        .unwrap();
    assert_eq!(state.value("value"), Some(&UiValue::Float(60.0)));

    state
        .apply_event(
            number,
            UiComponentEvent::Commit {
                property: "value".to_string(),
                value: UiValue::Float(3.0),
            },
        )
        .unwrap();
    assert_eq!(state.value("value"), Some(&UiValue::Float(10.0)));
}
