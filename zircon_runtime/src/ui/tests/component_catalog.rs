use std::collections::BTreeSet;

use crate::ui::component::{
    UiComponentCategory, UiComponentDescriptor, UiComponentDescriptorRegistry, UiComponentEvent,
    UiComponentEventKind, UiComponentState, UiDragPayload, UiDragPayloadKind, UiDragSourceMetadata,
    UiPropSchema, UiValidationLevel, UiValue, UiValueKind,
};

#[test]
fn runtime_component_catalog_contains_showcase_v1_controls() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();

    let showcase_v1_components = [
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
    ];

    assert!(!registry.is_empty());
    assert_eq!(
        registry.len(),
        showcase_v1_components.len(),
        "editor showcase registry should expose exactly the V1 component catalog"
    );
    assert_eq!(
        registry.component_ids().collect::<BTreeSet<_>>(),
        showcase_v1_components
            .iter()
            .copied()
            .collect::<BTreeSet<_>>(),
        "editor showcase registry component id set should match the authored V1 catalog"
    );
    assert_eq!(
        registry.categories().collect::<BTreeSet<_>>(),
        [
            UiComponentCategory::Visual,
            UiComponentCategory::Input,
            UiComponentCategory::Numeric,
            UiComponentCategory::Selection,
            UiComponentCategory::Reference,
            UiComponentCategory::Collection,
            UiComponentCategory::Container,
            UiComponentCategory::Feedback,
        ]
        .iter()
        .copied()
        .collect::<BTreeSet<_>>(),
        "editor showcase registry should expose the full V1 category set"
    );
    assert_category_component_ids(
        &registry,
        UiComponentCategory::Visual,
        &[
            "Icon",
            "Image",
            "Label",
            "RichLabel",
            "Separator",
            "SvgIcon",
        ],
    );
    assert_category_component_ids(
        &registry,
        UiComponentCategory::Feedback,
        &["Badge", "HelpRow", "ProgressBar", "Spinner"],
    );
    assert_category_component_ids(
        &registry,
        UiComponentCategory::Input,
        &[
            "Button",
            "Checkbox",
            "ContextActionMenu",
            "IconButton",
            "InputField",
            "Radio",
            "SegmentedControl",
            "TextField",
            "ToggleButton",
        ],
    );
    assert_category_component_ids(
        &registry,
        UiComponentCategory::Numeric,
        &[
            "ColorField",
            "NumberField",
            "RangeField",
            "Vector2Field",
            "Vector3Field",
            "Vector4Field",
        ],
    );
    assert_category_component_ids(
        &registry,
        UiComponentCategory::Selection,
        &[
            "ComboBox",
            "Dropdown",
            "EnumField",
            "FlagsField",
            "SearchSelect",
        ],
    );
    assert_category_component_ids(
        &registry,
        UiComponentCategory::Reference,
        &["AssetField", "InstanceField", "ObjectField"],
    );
    assert_category_component_ids(
        &registry,
        UiComponentCategory::Container,
        &["Foldout", "Group", "InspectorSection", "PropertyRow"],
    );
    assert_category_component_ids(
        &registry,
        UiComponentCategory::Collection,
        &["ArrayField", "ListRow", "MapField", "TreeRow"],
    );

    for component_id in showcase_v1_components {
        assert!(
            registry.contains(component_id),
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
    assert_has_prop(dropdown, "disabled_options");
    assert_has_prop(dropdown, "special_options");
    assert_has_prop(dropdown, "focused_options");
    assert_has_prop(dropdown, "hovered_options");
    assert_has_prop(dropdown, "pressed_options");
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
    assert_has_state(asset, "drop_hovered");
    assert_has_state(asset, "active_drag_target");
    assert_has_prop(asset, "validation_level");
    assert_has_prop(asset, "drop_hovered");
    assert_has_prop(asset, "active_drag_target");

    let number = registry.descriptor("NumberField").unwrap();
    assert_has_state(number, "focused");
    assert_has_state(number, "dragging");

    let image = registry.descriptor("Image").unwrap();
    assert_has_prop(image, "value");

    let icon = registry.descriptor("Icon").unwrap();
    assert_has_prop(icon, "value");

    let svg_icon = registry.descriptor("SvgIcon").unwrap();
    assert_has_state(svg_icon, "source");

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
    assert_has_prop(group, "validation_level");
    assert!(group.slot_schema("content").is_some());

    let menu = registry.descriptor("ContextActionMenu").unwrap();
    assert_has_prop(menu, "value");
    assert_has_prop(menu, "popup_open");
    assert_has_prop(menu, "popup_anchor_x");
    assert_has_prop(menu, "popup_anchor_y");
    assert_has_prop(menu, "menu_items");
    assert_has_state(menu, "popup_open");
    assert_has_state(menu, "popup_anchor_x");
    assert_has_state(menu, "popup_anchor_y");

    let prop_row = registry.descriptor("PropertyRow").unwrap();
    assert_has_prop(prop_row, "text");
    assert_has_prop(prop_row, "value");
    assert!(prop_row.slot_schema("label").is_some());
    assert!(prop_row.slot_schema("field").is_some());

    let inspector_section = registry.descriptor("InspectorSection").unwrap();
    assert_has_prop(inspector_section, "text");
    assert_has_prop(inspector_section, "expanded");

    let list_row = registry.descriptor("ListRow").unwrap();
    assert_has_prop(list_row, "value");
    assert_has_prop(list_row, "selected");
    assert_has_prop(list_row, "focused");
    assert_has_prop(list_row, "hovered");

    let tree_row = registry.descriptor("TreeRow").unwrap();
    assert_has_prop(tree_row, "tree_depth");
    assert_has_prop(tree_row, "tree_indent_px");

    let icon_button = registry.descriptor("IconButton").unwrap();
    assert_has_prop(icon_button, "text");

    let separator = registry.descriptor("Separator").unwrap();
    assert_has_prop(separator, "text");

    let spinner = registry.descriptor("Spinner").unwrap();
    assert_has_prop(spinner, "text");

    let segmented = registry.descriptor("SegmentedControl").unwrap();
    assert_has_prop(segmented, "value");
    assert_has_prop(segmented, "selection_state");

    let checkbox = registry.descriptor("Checkbox").unwrap();
    assert_has_prop(checkbox, "checked");

    let toggle_button = registry.descriptor("ToggleButton").unwrap();
    assert_has_prop(toggle_button, "checked");

    let radio = registry.descriptor("Radio").unwrap();
    assert_has_prop(radio, "checked");

    let array_field = registry.descriptor("ArrayField").unwrap();
    assert_has_prop(array_field, "validation_level");

    let map_field = registry.descriptor("MapField").unwrap();
    assert_has_prop(map_field, "validation_level");
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
            &[
                "focused",
                "dragging",
                "drop_hovered",
                "active_drag_target",
                "disabled",
            ][..],
        ),
        (
            "InstanceField",
            "value",
            &[
                "focused",
                "dragging",
                "drop_hovered",
                "active_drag_target",
                "disabled",
            ][..],
        ),
        (
            "ObjectField",
            "value",
            &[
                "focused",
                "dragging",
                "drop_hovered",
                "active_drag_target",
                "disabled",
            ][..],
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
        (
            "ListRow",
            "text",
            &["selected", "focused", "hovered", "pressed"][..],
        ),
        (
            "TreeRow",
            "text",
            &["expanded", "selected", "focused", "hovered", "pressed"][..],
        ),
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
                assert_has_prop(descriptor, "popup_open");
                assert_has_prop(descriptor, "disabled_options");
                assert_has_prop(descriptor, "special_options");
                assert_has_prop(descriptor, "focused_options");
                assert_has_prop(descriptor, "hovered_options");
                assert_has_prop(descriptor, "pressed_options");
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

    let context_menu = registry.descriptor("ContextActionMenu").unwrap();
    assert_has_event(context_menu, UiComponentEventKind::OpenPopupAt);

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

    for component_id in ["ListRow", "TreeRow"] {
        let descriptor = registry.descriptor(component_id).unwrap();
        assert_has_event(descriptor, UiComponentEventKind::Hover);
        assert_has_event(descriptor, UiComponentEventKind::Press);
    }

    for component_id in ["AssetField", "InstanceField", "ObjectField"] {
        let descriptor = registry.descriptor(component_id).unwrap();
        assert_has_event(descriptor, UiComponentEventKind::DropHover);
        assert_has_event(descriptor, UiComponentEventKind::ActiveDragTarget);
    }
}

#[test]
fn runtime_component_catalog_schemas_are_normalized_and_type_consistent() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();

    for descriptor in registry.descriptors() {
        assert_unique_schema_names(descriptor, "prop", &descriptor.prop_schema);
        assert_unique_schema_names(descriptor, "state", &descriptor.state_schema);
        assert_unique_slot_names(descriptor);
        assert_unique_events(descriptor);

        for schema in &descriptor.prop_schema {
            assert!(
                descriptor.prop(&schema.name).is_some(),
                "component {} prop lookup should find schema `{}`",
                descriptor.id,
                schema.name
            );
        }

        for schema in &descriptor.state_schema {
            assert!(
                descriptor.state_prop(&schema.name).is_some(),
                "component {} state lookup should find schema `{}`",
                descriptor.id,
                schema.name
            );
        }

        for slot in &descriptor.slot_schema {
            assert!(
                descriptor.slot_schema(&slot.name).is_some(),
                "component {} slot lookup should find schema `{}`",
                descriptor.id,
                slot.name
            );
        }

        for (name, value) in &descriptor.default_props {
            let schema = descriptor.prop(name).unwrap_or_else(|| {
                panic!(
                    "component {} default prop `{}` must have a matching prop schema",
                    descriptor.id, name
                )
            });
            assert_value_matches_schema_kind(descriptor, name, schema.value_kind, value);
        }

        for schema in descriptor
            .prop_schema
            .iter()
            .chain(descriptor.state_schema.iter())
        {
            if let Some(default_value) = &schema.default_value {
                assert_value_matches_schema_kind(
                    descriptor,
                    &schema.name,
                    schema.value_kind,
                    default_value,
                );
            }

            if let (Some(min), Some(max)) = (schema.min, schema.max) {
                assert!(
                    min <= max,
                    "component {} schema `{}` has inverted range {min}..{max}",
                    descriptor.id,
                    schema.name
                );
            }

            if let Some(step) = schema.step {
                assert!(
                    step > 0.0,
                    "component {} schema `{}` must use a positive step, got {step}",
                    descriptor.id,
                    schema.name
                );
            }
        }
    }
}

mod component_state;
fn test_asset_source() -> UiDragSourceMetadata {
    UiDragSourceMetadata::asset(
        "browser",
        "AssetBrowserContentPanel",
        "asset-uuid-1",
        "res://textures/grid.albedo.png",
        "Grid Albedo",
        "Texture",
        "png",
    )
}

fn assert_has_state(descriptor: &UiComponentDescriptor, name: &str) {
    assert!(
        descriptor.state_prop(name).is_some(),
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

fn assert_category_component_ids(
    registry: &UiComponentDescriptorRegistry,
    category: UiComponentCategory,
    expected_ids: &[&str],
) {
    assert_eq!(
        registry
            .descriptors_in_category(category)
            .map(|descriptor| descriptor.id.as_str())
            .collect::<BTreeSet<_>>(),
        expected_ids.iter().copied().collect::<BTreeSet<_>>(),
        "component category {category:?} should expose the expected V1 component ids"
    );
}

fn assert_unique_schema_names(
    descriptor: &UiComponentDescriptor,
    schema_label: &str,
    schemas: &[UiPropSchema],
) {
    let mut names = BTreeSet::new();
    for schema in schemas {
        assert!(
            names.insert(schema.name.as_str()),
            "component {} has duplicate {} schema `{}`",
            descriptor.id,
            schema_label,
            schema.name
        );
    }
}

fn assert_unique_slot_names(descriptor: &UiComponentDescriptor) {
    let mut names = BTreeSet::new();
    for slot in &descriptor.slot_schema {
        assert!(
            names.insert(slot.name.as_str()),
            "component {} has duplicate slot schema `{}`",
            descriptor.id,
            slot.name
        );
    }
}

fn assert_unique_events(descriptor: &UiComponentDescriptor) {
    let mut events = BTreeSet::new();
    for event in &descriptor.events {
        assert!(
            events.insert(format!("{event:?}")),
            "component {} has duplicate event {:?}",
            descriptor.id,
            event
        );
    }
}

fn assert_value_matches_schema_kind(
    descriptor: &UiComponentDescriptor,
    name: &str,
    expected_kind: UiValueKind,
    value: &UiValue,
) {
    if expected_kind == UiValueKind::Any {
        return;
    }

    assert_eq!(
        value.kind(),
        expected_kind,
        "component {} schema `{}` default value kind mismatch",
        descriptor.id,
        name
    );
}
