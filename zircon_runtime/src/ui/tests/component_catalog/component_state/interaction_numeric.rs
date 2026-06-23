use super::*;

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

#[test]
fn component_state_clamps_range_slider_thumbs_against_each_other() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let range_slider = registry.descriptor("RangeSlider").unwrap();
    assert_has_event(range_slider, UiComponentEventKind::LargeDragDelta);

    let mut state = UiComponentState::new()
        .with_value("range_min", UiValue::Float(30.0))
        .with_value("value", UiValue::Float(70.0))
        .with_value("min", UiValue::Float(0.0))
        .with_value("max", UiValue::Float(100.0))
        .with_value("step", UiValue::Float(5.0))
        .with_value("large_step", UiValue::Float(20.0));

    state
        .apply_event(
            range_slider,
            UiComponentEvent::DragDelta {
                property: "range_min".to_string(),
                delta: 20.0,
            },
        )
        .unwrap();
    assert_eq!(state.value("range_min"), Some(&UiValue::Float(70.0)));

    state
        .apply_event(
            range_slider,
            UiComponentEvent::LargeDragDelta {
                property: "value".to_string(),
                delta: -4.0,
            },
        )
        .unwrap();
    assert_eq!(state.value("value"), Some(&UiValue::Float(70.0)));

    state
        .apply_event(
            range_slider,
            UiComponentEvent::Commit {
                property: "range_min".to_string(),
                value: UiValue::Float(120.0),
            },
        )
        .unwrap();
    assert_eq!(state.value("range_min"), Some(&UiValue::Float(70.0)));
}
