use super::*;

#[test]
fn shared_component_catalog_views_reuse_process_registries() {
    assert!(std::ptr::eq(
        UiComponentDescriptorRegistry::editor_showcase_shared(),
        UiComponentDescriptorRegistry::editor_showcase_shared(),
    ));
    assert!(std::ptr::eq(
        UiComponentDescriptorRegistry::material_editor_foundation_shared(),
        UiComponentDescriptorRegistry::material_editor_foundation_shared(),
    ));
}

#[test]
fn runtime_component_catalog_contains_showcase_v1_controls() {
    let registry = UiComponentDescriptorRegistry::editor_showcase();

    let showcase_v1_components = [
        "Container",
        "Overlay",
        "ListView",
        "FlexBox",
        "HorizontalBox",
        "HorizontalGroup",
        "VerticalBox",
        "VerticalGroup",
        "FlowBox",
        "GridBox",
        "GridGroup",
        "ScrollableBox",
        "CanvasBox",
        "SizeBox",
        "Space",
        "Label",
        "RichLabel",
        "Text",
        "Image",
        "Icon",
        "Canvas",
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
        "RadioField",
        "Toggle",
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
        "Popup",
        "PropertyRow",
        "InspectorSection",
        "WorldSpaceSurface",
        "ArrayField",
        "MapField",
        "ListRow",
        "VirtualList",
        "PagedList",
        "TreeRow",
        "TreeView",
        "EditableTable",
        "Table",
        "MessageBox",
        "ContextActionMenu",
        "SvgIcon",
        "Svg",
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
            "Text",
            "Separator",
            "Canvas",
            "Svg",
            "SvgIcon",
        ],
    );
    assert_category_component_ids(
        &registry,
        UiComponentCategory::Feedback,
        &["Badge", "HelpRow", "MessageBox", "ProgressBar", "Spinner"],
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
            "RadioField",
            "SegmentedControl",
            "TextField",
            "Toggle",
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
        &[
            "Container",
            "CanvasBox",
            "FlexBox",
            "Foldout",
            "FlowBox",
            "GridBox",
            "GridGroup",
            "Group",
            "HorizontalBox",
            "HorizontalGroup",
            "InspectorSection",
            "ListView",
            "Overlay",
            "Popup",
            "PropertyRow",
            "ScrollableBox",
            "SizeBox",
            "Space",
            "VerticalBox",
            "VerticalGroup",
            "WorldSpaceSurface",
        ],
    );
    assert_category_component_ids(
        &registry,
        UiComponentCategory::Collection,
        &[
            "ArrayField",
            "EditableTable",
            "ListRow",
            "MapField",
            "PagedList",
            "Table",
            "TreeRow",
            "TreeView",
            "VirtualList",
        ],
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

    let world_space_surface = registry.descriptor("WorldSpaceSurface").unwrap();
    assert_has_prop(world_space_surface, "world_position");
    assert_has_prop(world_space_surface, "world_rotation");
    assert_has_prop(world_space_surface, "world_scale");
    assert_has_prop(world_space_surface, "world_size");
    assert_has_prop(world_space_surface, "pixels_per_meter");
    assert_has_prop(world_space_surface, "billboard");
    assert_has_prop(world_space_surface, "depth_test");
    assert_has_prop(world_space_surface, "render_order");
    assert_has_prop(world_space_surface, "camera_target");
    assert!(world_space_surface.slot_schema("content").is_some());

    let list_row = registry.descriptor("ListRow").unwrap();
    assert_has_prop(list_row, "value");
    assert_has_prop(list_row, "selected");
    assert_has_prop(list_row, "focused");
    assert_has_prop(list_row, "hovered");

    let virtual_list = registry.descriptor("VirtualList").unwrap();
    assert_has_prop(virtual_list, "items");
    assert_has_prop(virtual_list, "total_count");
    assert_has_prop(virtual_list, "viewport_start");
    assert_has_prop(virtual_list, "viewport_count");
    assert_has_prop(virtual_list, "item_extent");
    assert_has_prop(virtual_list, "overscan");
    assert_has_state(virtual_list, "viewport_start");
    assert_has_state(virtual_list, "viewport_count");
    assert!(virtual_list.slot_schema("row").is_some());

    let paged_list = registry.descriptor("PagedList").unwrap();
    assert_has_prop(paged_list, "items");
    assert_has_prop(paged_list, "total_count");
    assert_has_prop(paged_list, "page_index");
    assert_has_prop(paged_list, "page_size");
    assert_has_prop(paged_list, "page_count");
    assert_has_state(paged_list, "page_index");
    assert_has_state(paged_list, "page_size");
    assert!(paged_list.slot_schema("page").is_some());

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

    let text = registry.descriptor("Text").unwrap();
    assert_has_prop(text, "text");

    let popup = registry.descriptor("Popup").unwrap();
    assert_has_prop(popup, "popup_open");
    assert!(popup.slot_schema("content").is_some());

    let table = registry.descriptor("EditableTable").unwrap();
    assert_has_prop(table, "rows");
    assert_has_prop(table, "columns");
    assert!(table.slot_schema("cell").is_some());

    let message_box = registry.descriptor("MessageBox").unwrap();
    assert_has_prop(message_box, "rich_text");
    assert_has_event(message_box, UiComponentEventKind::OpenPopup);
}
