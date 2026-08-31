use super::*;

#[test]
fn material_editor_foundation_catalog_covers_editor_descriptor_contracts() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let tree_view = registry
        .descriptor("TreeView")
        .expect("TreeView descriptor");
    assert_has_prop(tree_view, "query");
    assert_has_prop(tree_view, "expanded");
    assert_has_event(tree_view, UiComponentEventKind::ToggleExpanded);
    assert_has_event(tree_view, UiComponentEventKind::Commit);

    let property_grid = registry
        .descriptor("PropertyGrid")
        .expect("PropertyGrid descriptor");
    assert_has_prop(property_grid, "selection_summary");

    let inspector_section = registry
        .descriptor("InspectorSection")
        .expect("InspectorSection descriptor");
    assert_has_prop(inspector_section, "text");
    assert_has_prop(inspector_section, "expanded");
    assert_has_event(inspector_section, UiComponentEventKind::ToggleExpanded);

    let drawer = registry.descriptor("Drawer").expect("Drawer descriptor");
    assert_has_prop(drawer, "slot");
    assert_has_prop(drawer, "mode");
    assert_has_prop(drawer, "active_view");
    assert_has_event(drawer, UiComponentEventKind::SelectOption);

    let view = registry.descriptor("View").expect("View descriptor");
    assert_has_prop(view, "view_id");
    assert_has_prop(view, "dirty");
    assert_has_event(view, UiComponentEventKind::Focus);

    button_inputs::assert_descriptors(&registry);
    data_display::assert_descriptors(&registry);
    feedback::assert_descriptors(&registry);
    let text_field = registry
        .descriptor("TextField")
        .expect("TextField descriptor");
    assert_enum_options(text_field, "variant", &["outlined", "filled", "standard"]);
    for prop in [
        "value_text",
        "label",
        "placeholder",
        "helper_text",
        "multiline",
        "select_mode",
        "required",
        "min_length",
        "max_length",
        "validation_timing",
        "validation_message",
        "validation_dirty",
        "validation_touched",
        "composition_clauses",
    ] {
        assert_has_prop(text_field, prop);
    }
    let composition_clauses = text_field
        .prop("composition_clauses")
        .expect("TextField composition clauses state");
    assert_eq!(composition_clauses.value_kind, UiValueKind::Array);
    assert_eq!(
        composition_clauses.default_value,
        Some(UiValue::Array(Vec::new()))
    );
    assert_eq!(
        text_field
            .default_props
            .iter()
            .find(|(name, _)| name == "variant")
            .map(|(_, value)| value),
        Some(&UiValue::Enum("outlined".to_string())),
        "TextField should default to outlined Material field styling"
    );
    for event in [
        UiComponentEventKind::Focus,
        UiComponentEventKind::ValueChanged,
        UiComponentEventKind::Commit,
    ] {
        assert_has_event(text_field, event);
    }
    assert!(
        text_field
            .required_host_capabilities
            .contains(&UiHostCapability::TextInput)
    );

    let textarea = registry
        .descriptor("TextareaAutosize")
        .expect("TextareaAutosize descriptor");
    assert_enum_options(textarea, "variant", &["outlined", "filled", "standard"]);
    for prop in [
        "value_text",
        "placeholder",
        "helper_text",
        "multiline",
        "autosize",
        "min_rows",
        "max_rows",
        "required",
        "min_length",
        "max_length",
        "validation_timing",
        "validation_message",
        "validation_dirty",
        "validation_touched",
        "composition_clauses",
    ] {
        assert_has_prop(textarea, prop);
    }
    for (prop, expected) in [("multiline", true), ("autosize", true)] {
        assert_eq!(
            textarea
                .default_props
                .iter()
                .find(|(name, _)| name == prop)
                .map(|(_, value)| value),
            Some(&UiValue::Bool(expected)),
            "TextareaAutosize should default `{prop}` to `{expected}`"
        );
    }
    for (prop, expected) in [("min_rows", 2), ("max_rows", 8)] {
        assert_eq!(
            textarea
                .default_props
                .iter()
                .find(|(name, _)| name == prop)
                .map(|(_, value)| value),
            Some(&UiValue::Int(expected)),
            "TextareaAutosize should default `{prop}` to `{expected}`"
        );
    }
    for event in [
        UiComponentEventKind::Focus,
        UiComponentEventKind::ValueChanged,
        UiComponentEventKind::Commit,
    ] {
        assert_has_event(textarea, event);
    }
    assert!(
        textarea
            .required_host_capabilities
            .contains(&UiHostCapability::TextInput)
    );

    inputs::assert_descriptors(&registry);
    selection_inputs::assert_descriptors(&registry);
    form_controls::assert_descriptors(&registry);
    data_display_subcomponents::assert_descriptors(&registry);
    surface_subcomponents::assert_descriptors(&registry);
    navigation_editor::assert_descriptors(&registry);
    navigation_secondary::assert_descriptors(&registry);
    lab_subcomponents::assert_descriptors(&registry);

    let window = registry.descriptor("Window").expect("Window descriptor");
    assert_has_prop(window, "window_id");
    assert_has_prop(window, "dock_policy");
    assert_has_prop(window, "floating");
    assert_has_event(window, UiComponentEventKind::BeginDrag);

    let workbench_shell = registry
        .descriptor("WorkbenchShell")
        .expect("WorkbenchShell descriptor");
    assert_has_prop(workbench_shell, "skin_id");
    assert_has_prop(workbench_shell, "panel_preset_id");
    assert_has_prop(workbench_shell, "shell_preset_id");
    assert_has_prop(workbench_shell, "window_model_preset_id");

    let dock_host = registry
        .descriptor("DockHost")
        .expect("DockHost descriptor");
    assert_eq!(dock_host.descriptor_kind, UiComponentDescriptorKind::Layout);
    assert_eq!(dock_host.layout_role, UiComponentLayoutRole::EditorDock);
    assert!(
        dock_host
            .required_host_capabilities
            .contains(&UiHostCapability::Editor)
    );

    let virtual_list = registry
        .descriptor("VirtualList")
        .expect("VirtualList descriptor");
    assert_eq!(virtual_list.layout_role, UiComponentLayoutRole::VirtualList);
    assert_has_prop(virtual_list, "item_count");
    assert_has_prop(virtual_list, "item_extent");
    assert_has_prop(virtual_list, "overscan");
    assert_has_event(virtual_list, UiComponentEventKind::SetVisibleRange);
    assert!(
        virtual_list
            .required_host_capabilities
            .contains(&UiHostCapability::VirtualizedLayout)
    );
    assert!(
        virtual_list
            .required_render_capabilities
            .contains(&UiRenderCapability::VirtualizedLayout)
    );

    let tree_view = registry
        .descriptor("TreeView")
        .expect("TreeView descriptor");
    assert_has_prop(tree_view, "query");
    assert_has_event(tree_view, UiComponentEventKind::ToggleExpanded);
    assert_has_event(tree_view, UiComponentEventKind::OpenPopupAt);

    let property_grid = registry
        .descriptor("PropertyGrid")
        .expect("PropertyGrid descriptor");
    assert_has_event(property_grid, UiComponentEventKind::ValueChanged);

    let search_field = registry
        .descriptor("SearchField")
        .expect("SearchField descriptor");
    assert_has_prop(search_field, "query");
    for prop in [
        "required",
        "min_length",
        "max_length",
        "validation_timing",
        "validation_message",
        "validation_dirty",
        "validation_touched",
    ] {
        assert_has_prop(search_field, prop);
    }
    assert_has_event(search_field, UiComponentEventKind::Focus);
    assert_has_event(search_field, UiComponentEventKind::ValueChanged);
    assert_has_event(search_field, UiComponentEventKind::Commit);
    assert!(
        search_field
            .required_host_capabilities
            .contains(&UiHostCapability::TextInput)
    );

    let field_editor = registry
        .descriptor("FieldEditor")
        .expect("FieldEditor descriptor");
    assert_has_prop(field_editor, "text");
    assert_has_prop(field_editor, "value_text");
    assert!(field_editor.slot_schema("field").is_some());
    assert_has_event(field_editor, UiComponentEventKind::Focus);
    assert_has_event(field_editor, UiComponentEventKind::ValueChanged);
    assert_has_event(field_editor, UiComponentEventKind::Commit);
    assert!(
        field_editor
            .required_host_capabilities
            .contains(&UiHostCapability::TextInput)
    );

    let asset_grid = registry
        .descriptor("AssetGrid")
        .expect("AssetGrid descriptor");
    assert_has_prop(asset_grid, "item_count");
    assert_has_event(asset_grid, UiComponentEventKind::OpenReference);
    assert_has_event(asset_grid, UiComponentEventKind::LocateReference);

    let viewport_host = registry
        .descriptor("ViewportHost")
        .expect("ViewportHost descriptor");
    assert_eq!(
        viewport_host.descriptor_kind,
        UiComponentDescriptorKind::Layout
    );
    assert_eq!(viewport_host.layout_role, UiComponentLayoutRole::Canvas);
    assert!(
        viewport_host
            .required_host_capabilities
            .contains(&UiHostCapability::CanvasRender)
    );
    assert!(
        viewport_host
            .required_render_capabilities
            .contains(&UiRenderCapability::Canvas)
    );
    assert_has_event(viewport_host, UiComponentEventKind::SetWorldSurface);

    let graph_canvas = registry
        .descriptor("GraphCanvas")
        .expect("GraphCanvas descriptor");
    assert_eq!(graph_canvas.layout_role, UiComponentLayoutRole::Canvas);
    assert!(graph_canvas.slot_schema("nodes").is_some());
    assert!(graph_canvas.slot_schema("edges").is_some());
    assert_has_event(graph_canvas, UiComponentEventKind::DropHover);

    let source_editor = registry
        .descriptor("SourceEditor")
        .expect("SourceEditor descriptor");
    assert_has_prop(source_editor, "text");
    assert_has_event(source_editor, UiComponentEventKind::Focus);
    assert_has_event(source_editor, UiComponentEventKind::ValueChanged);
    assert_has_event(source_editor, UiComponentEventKind::Commit);
    assert!(
        source_editor
            .required_host_capabilities
            .contains(&UiHostCapability::TextInput)
    );

    let timeline = registry
        .descriptor("Timeline")
        .expect("Timeline descriptor");
    assert_has_prop(timeline, "time");
    assert_has_prop(timeline, "duration");
    assert_has_prop(timeline, "position");
    assert_has_event(timeline, UiComponentEventKind::DragDelta);

    let drawer = registry.descriptor("Drawer").expect("Drawer descriptor");
    assert_has_prop(drawer, "slot");
    assert_has_prop(drawer, "mode");
    assert_has_prop(drawer, "active_view");
    assert_has_event(drawer, UiComponentEventKind::SelectOption);

    let view = registry.descriptor("View").expect("View descriptor");
    assert_has_prop(view, "view_id");
    assert_has_prop(view, "dirty");
    assert_has_event(view, UiComponentEventKind::Focus);

    let window = registry.descriptor("Window").expect("Window descriptor");
    assert_has_prop(window, "window_id");
    assert_has_prop(window, "dock_policy");
    assert_has_prop(window, "floating");
    assert_has_event(window, UiComponentEventKind::BeginDrag);

    let document_node = registry
        .descriptor("DocumentNode")
        .expect("DocumentNode descriptor");
    assert_eq!(
        document_node.descriptor_kind,
        UiComponentDescriptorKind::Layout
    );
    assert_eq!(document_node.layout_role, UiComponentLayoutRole::EditorDock);
    assert_has_prop(document_node, "node_kind");

    let tab_stack = registry
        .descriptor("TabStack")
        .expect("TabStack descriptor");
    assert_has_prop(tab_stack, "active_tab");
    assert!(tab_stack.slot_schema("tabs").is_some());
    assert!(tab_stack.slot_schema("content").is_some());
    assert_has_event(tab_stack, UiComponentEventKind::SelectOption);

    let floating_window = registry
        .descriptor("FloatingWindow")
        .expect("FloatingWindow descriptor");
    assert_has_prop(floating_window, "window_id");
    assert_has_prop(floating_window, "focused_view");
    assert_has_event(floating_window, UiComponentEventKind::Focus);
    assert_has_event(floating_window, UiComponentEventKind::BeginDrag);

    let workbench_shell = registry
        .descriptor("WorkbenchShell")
        .expect("WorkbenchShell descriptor");
    assert_has_prop(workbench_shell, "skin_id");
    assert_has_prop(workbench_shell, "panel_preset_id");
    assert_has_prop(workbench_shell, "shell_preset_id");
    assert_has_prop(workbench_shell, "window_model_preset_id");
}
