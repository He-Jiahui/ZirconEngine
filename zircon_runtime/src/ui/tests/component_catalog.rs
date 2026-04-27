use crate::ui::component::{
    UiComponentCategory, UiComponentDescriptor, UiComponentDescriptorRegistry, UiComponentEvent,
    UiComponentEventKind, UiComponentState, UiDragPayload, UiDragPayloadKind, UiValidationLevel,
    UiValue, UiValueKind,
};

#[test]
fn runtime_component_catalog_contains_showcase_v1_controls() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();

    for component_id in [
        "Label",
        "RichLabel",
        "Image",
        "Icon",
        "Separator",
        "ProgressBar",
        "Spinner",
        "Badge",
        "HelpRow",
        "Button",
        "IconButton",
        "ToggleButton",
        "Checkbox",
        "Radio",
        "SegmentedControl",
        "InputField",
        "TextField",
        "NumberField",
        "RangeField",
        "ColorField",
        "Vector2Field",
        "Vector3Field",
        "Vector4Field",
        "Dropdown",
        "ComboBox",
        "EnumField",
        "FlagsField",
        "SearchSelect",
        "AssetField",
        "InstanceField",
        "ObjectField",
        "Group",
        "Foldout",
        "PropertyRow",
        "InspectorSection",
        "ArrayField",
        "MapField",
        "ListRow",
        "TreeRow",
        "ContextActionMenu",
        "SvgIcon",
    ] {
        assert!(
            registry.descriptor(component_id).is_some(),
            "missing V1 component descriptor `{component_id}`"
        );
    }

    let number = registry.descriptor("NumberField").unwrap();
    assert_eq!(number.category, UiComponentCategory::Numeric);
    assert_eq!(number.role, "number-field");
    assert!(number.prop("value").is_some());
    assert!(number.prop("min").is_some());
    assert!(number.prop("max").is_some());
    assert!(number.prop("large_step").is_some());
    assert_has_prop(number, "validation_level");
    assert!(number.supports_event(UiComponentEventKind::DragDelta));
    assert!(number.supports_event(UiComponentEventKind::Commit));
    assert_has_prop(number, "large_step");

    let dropdown = registry.descriptor("Dropdown").unwrap();
    assert_eq!(dropdown.category, UiComponentCategory::Selection);
    assert!(dropdown.prop("options").is_some());
    assert!(dropdown.prop("multiple").is_some());
    assert!(dropdown.supports_event(UiComponentEventKind::SelectOption));
    assert_has_prop(dropdown, "validation_level");
    assert!(
        dropdown
            .prop("options")
            .unwrap()
            .options
            .iter()
            .any(|option| option.disabled),
        "selection controls must describe disabled choices for showcase validation"
    );
    assert_has_state(dropdown, "focused");
    assert_has_state(dropdown, "popup_open");
    assert_has_state(dropdown, "selected");

    let flags = registry.descriptor("FlagsField").unwrap();
    assert_has_prop(flags, "validation_level");
    assert_eq!(flags.prop("value").unwrap().value_kind, UiValueKind::Flags);
    assert_has_state(flags, "focused");
    assert_has_state(flags, "selected");

    let combo_box = registry.descriptor("ComboBox").unwrap();
    assert_has_prop(combo_box, "validation_level");

    let enum_field = registry.descriptor("EnumField").unwrap();
    assert_has_prop(enum_field, "validation_level");

    let asset = registry.descriptor("AssetField").unwrap();
    assert!(asset.accepts_drag_payload(UiDragPayloadKind::Asset));
    assert!(!asset.accepts_drag_payload(UiDragPayloadKind::SceneInstance));
    assert_has_state(asset, "focused");
    assert_has_state(asset, "dragging");
    assert_has_prop(asset, "validation_level");

    let number = registry.descriptor("NumberField").unwrap();
    assert_has_state(number, "focused");
    assert_has_state(number, "dragging");

    let icon = registry.descriptor("SvgIcon").unwrap();
    assert_has_state(icon, "source");

    let help_row = registry.descriptor("HelpRow").unwrap();
    assert_has_prop(help_row, "validation_level");
    assert_has_prop(help_row, "validation_message");

    let button = registry.descriptor("Button").unwrap();
    assert_has_prop(button, "validation_level");

    let progress = registry.descriptor("ProgressBar").unwrap();
    assert_has_prop(progress, "validation_level");

    let text_field = registry.descriptor("TextField").unwrap();
    assert_has_prop(text_field, "validation_level");

    let input_field = registry.descriptor("InputField").unwrap();
    assert_has_prop(input_field, "validation_level");

    let range_field = registry.descriptor("RangeField").unwrap();
    assert_has_prop(range_field, "validation_level");

    let color_field = registry.descriptor("ColorField").unwrap();
    assert_has_prop(color_field, "validation_level");

    let vector2 = registry.descriptor("Vector2Field").unwrap();
    assert_has_prop(vector2, "validation_level");

    let vector3 = registry.descriptor("Vector3Field").unwrap();
    assert_has_prop(vector3, "validation_level");

    let vector4 = registry.descriptor("Vector4Field").unwrap();
    assert_has_prop(vector4, "validation_level");

    let search = registry.descriptor("SearchSelect").unwrap();
    assert_has_state(search, "query");
    assert_has_prop(search, "query");
    assert_has_prop(search, "validation_level");

    let group = registry.descriptor("Group").unwrap();
    assert_has_state(group, "expanded");
    assert_has_prop(group, "text");

    let menu = registry.descriptor("ContextActionMenu").unwrap();
    assert_has_prop(menu, "value");
    assert_has_state(menu, "popup_open");

    let prop_row = registry.descriptor("PropertyRow").unwrap();
    assert_has_prop(prop_row, "text");
    assert_has_prop(prop_row, "value");

    let inspector_section = registry.descriptor("InspectorSection").unwrap();
    assert_has_prop(inspector_section, "text");

    let list_row = registry.descriptor("ListRow").unwrap();
    assert_has_prop(list_row, "value");

    let icon_button = registry.descriptor("IconButton").unwrap();
    assert_has_prop(icon_button, "text");

    let separator = registry.descriptor("Separator").unwrap();
    assert_has_prop(separator, "text");

    let spinner = registry.descriptor("Spinner").unwrap();
    assert_has_prop(spinner, "text");

    let segmented = registry.descriptor("SegmentedControl").unwrap();
    assert_has_prop(segmented, "value");
}

#[test]
fn runtime_component_catalog_selection_and_state_coverage() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();

    for (component_id, property, expected_states) in [
        (
            "Dropdown",
            "options",
            &["focused", "popup_open", "selected"][..],
        ),
        (
            "ComboBox",
            "options",
            &["focused", "popup_open", "selected"][..],
        ),
        (
            "EnumField",
            "options",
            &["focused", "popup_open", "selected"][..],
        ),
        (
            "FlagsField",
            "options",
            &["focused", "popup_open", "selected"][..],
        ),
        (
            "SearchSelect",
            "query",
            &["focused", "popup_open", "selected", "query"][..],
        ),
        (
            "SegmentedControl",
            "options",
            &["focused", "selected", "disabled"][..],
        ),
        (
            "AssetField",
            "value",
            &["focused", "dragging", "disabled"][..],
        ),
        (
            "InstanceField",
            "value",
            &["focused", "dragging", "disabled"][..],
        ),
        (
            "ObjectField",
            "value",
            &["focused", "dragging", "disabled"][..],
        ),
        (
            "Group",
            "expanded",
            &["expanded", "focused", "disabled"][..],
        ),
        (
            "Foldout",
            "expanded",
            &["expanded", "focused", "disabled"][..],
        ),
        (
            "ContextActionMenu",
            "options",
            &["focused", "selected", "popup_open"][..],
        ),
        ("ListRow", "text", &["selected", "focused"][..]),
        ("TreeRow", "text", &["expanded", "selected", "focused"][..]),
    ] {
        let descriptor = registry
            .descriptor(component_id)
            .unwrap_or_else(|| panic!("missing component descriptor `{component_id}`"));
        assert_has_prop(descriptor, property);
        for state in expected_states {
            assert_has_state(descriptor, state);
        }
        if let Some(options_prop) = descriptor.prop("options") {
            assert!(
                options_prop.options.iter().any(|option| option.disabled),
                "component `{component_id}` should advertise at least one disabled option"
            );
            if matches!(
                component_id,
                "Dropdown" | "ComboBox" | "EnumField" | "FlagsField" | "SearchSelect"
            ) {
                assert_has_prop(descriptor, "selection_state");
                assert_has_prop(descriptor, "value_text");
            }
        }
    }

    for (component_id, property) in [
        ("IconButton", "text"),
        ("ToggleButton", "text"),
        ("Checkbox", "text"),
        ("Radio", "text"),
        ("Group", "text"),
        ("Foldout", "text"),
        ("PropertyRow", "text"),
        ("PropertyRow", "value"),
        ("InspectorSection", "text"),
        ("ListRow", "text"),
        ("ListRow", "value"),
        ("Separator", "text"),
        ("Spinner", "text"),
        ("SegmentedControl", "value"),
        ("ContextActionMenu", "value"),
    ] {
        let descriptor = registry
            .descriptor(component_id)
            .unwrap_or_else(|| panic!("missing component descriptor `{component_id}`"));
        assert_has_prop(descriptor, property);
    }

    for component_id in [
        "Dropdown",
        "ComboBox",
        "EnumField",
        "FlagsField",
        "SearchSelect",
        "ContextActionMenu",
    ] {
        let descriptor = registry.descriptor(component_id).unwrap();
        assert_has_event(descriptor, UiComponentEventKind::Focus);
        assert_has_event(descriptor, UiComponentEventKind::OpenPopup);
        assert_has_event(descriptor, UiComponentEventKind::ClosePopup);
        assert_has_event(descriptor, UiComponentEventKind::SelectOption);
    }

    let segmented = registry.descriptor("SegmentedControl").unwrap();
    assert_has_event(segmented, UiComponentEventKind::Focus);
    assert_has_event(segmented, UiComponentEventKind::SelectOption);

    for component_id in ["NumberField"] {
        let descriptor = registry.descriptor(component_id).unwrap();
        assert_has_event(descriptor, UiComponentEventKind::Focus);
        assert_has_event(descriptor, UiComponentEventKind::BeginDrag);
        assert_has_event(descriptor, UiComponentEventKind::DragDelta);
        assert_has_event(descriptor, UiComponentEventKind::EndDrag);
        assert_has_event(descriptor, UiComponentEventKind::Commit);
        assert_has_event(descriptor, UiComponentEventKind::ValueChanged);
    }

    for component_id in ["RangeField"] {
        let descriptor = registry.descriptor(component_id).unwrap();
        assert_has_event(descriptor, UiComponentEventKind::Focus);
        assert_has_event(descriptor, UiComponentEventKind::DragDelta);
        assert_has_event(descriptor, UiComponentEventKind::Commit);
        assert_has_event(descriptor, UiComponentEventKind::ValueChanged);
    }

    for component_id in ["Group", "Foldout"] {
        let descriptor = registry.descriptor(component_id).unwrap();
        assert_has_event(descriptor, UiComponentEventKind::ToggleExpanded);
    }

    let input = registry.descriptor("InputField").unwrap();
    assert_has_event(input, UiComponentEventKind::ValueChanged);
    assert_has_event(input, UiComponentEventKind::Commit);
    assert_has_event(input, UiComponentEventKind::Focus);
}

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
        number_state.validation().level,
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
    assert!(!group_state.flags().expanded);

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
    assert_eq!(state.validation().level, UiValidationLevel::Error);
    assert!(state
        .validation()
        .message
        .as_deref()
        .is_some_and(|message| message.contains("disabled option")));
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
    assert_eq!(asset_state.validation().level, UiValidationLevel::Error);

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
    assert_eq!(asset_state.validation().level, UiValidationLevel::Error);
    assert!(asset_state
        .validation()
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
fn component_state_applies_dropdown_multiple_selection_and_special_options() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let dropdown = registry.descriptor("Dropdown").unwrap();
    assert!(
        dropdown
            .prop("options")
            .unwrap()
            .options
            .iter()
            .any(|option| option.special_condition.is_some()),
        "selection controls should expose a special-condition option for mixed inspector states"
    );

    let mut state = UiComponentState::new().with_value("multiple", UiValue::Bool(true));
    state
        .apply_event(
            dropdown,
            UiComponentEvent::SelectOption {
                property: "value".to_string(),
                option_id: "runtime".to_string(),
                selected: true,
            },
        )
        .unwrap();
    state
        .apply_event(
            dropdown,
            UiComponentEvent::SelectOption {
                property: "value".to_string(),
                option_id: "editor".to_string(),
                selected: true,
            },
        )
        .unwrap();
    state
        .apply_event(
            dropdown,
            UiComponentEvent::SelectOption {
                property: "value".to_string(),
                option_id: "runtime".to_string(),
                selected: false,
            },
        )
        .unwrap();

    assert_eq!(
        state.value("value"),
        Some(&UiValue::Array(vec![UiValue::Enum("editor".to_string())]))
    );
}

fn assert_has_state(descriptor: &UiComponentDescriptor, name: &str) {
    assert!(
        descriptor
            .state_schema
            .iter()
            .any(|state| state.name == name),
        "component {} missing state schema entry `{}`",
        descriptor.id,
        name
    );
}

fn assert_has_prop(descriptor: &UiComponentDescriptor, name: &str) {
    assert!(
        descriptor.prop(name).is_some(),
        "component {} missing prop schema entry `{}`",
        descriptor.id,
        name
    );
}

fn assert_has_event(descriptor: &UiComponentDescriptor, event: UiComponentEventKind) {
    assert!(
        descriptor.supports_event(event),
        "component {} missing event support {:?}",
        descriptor.id,
        event
    );
}
