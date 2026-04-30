use super::support::*;

#[test]
fn editor_ui_compatibility_harness_captures_projection_shape_for_parity_checks() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();
    let projection = runtime
        .project_document(UI_HOST_WINDOW_DOCUMENT_ID)
        .unwrap();

    let snapshot = EditorUiCompatibilityHarness::capture_projection_snapshot(&projection);

    assert_eq!(
        snapshot.components,
        vec![
            "UiHostWindow",
            "VerticalBox",
            "UiHostToolbar",
            "IconButton",
            "IconButton",
            "IconButton",
            "Container",
            "Container",
            "HorizontalBox",
            "ActivityRail",
            "IconButton",
            "IconButton",
            "IconButton",
            "DocumentHost",
            "DocumentTabs",
            "PaneSurface",
            "StatusBar",
            "Label",
            "Overlay",
            "Container",
            "Container",
            "Overlay",
            "Container",
            "Container",
            "Overlay",
            "Container",
            "Container",
        ]
    );
    assert!(snapshot
        .control_ids
        .contains(&"WorkbenchMenuBarRoot".to_string()));
    assert!(snapshot
        .binding_ids
        .contains(&"WorkbenchMenuBar/ResetLayout".to_string()));
}

#[test]
fn editor_ui_host_runtime_builds_host_node_model_with_routes_and_attributes() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();
    let mut projection = runtime
        .project_document(UI_HOST_WINDOW_DOCUMENT_ID)
        .unwrap();
    let mut service = EditorUiControlService::default();
    runtime
        .register_projection_routes(&mut service, &mut projection)
        .unwrap();

    let host_model = runtime.build_host_model(&projection).unwrap();

    assert_eq!(host_model.document_id, UI_HOST_WINDOW_DOCUMENT_ID);
    assert_eq!(
        host_model
            .nodes
            .iter()
            .map(|node| node.component.as_str())
            .collect::<Vec<_>>(),
        vec![
            "UiHostWindow",
            "VerticalBox",
            "UiHostToolbar",
            "IconButton",
            "IconButton",
            "IconButton",
            "Container",
            "Container",
            "HorizontalBox",
            "ActivityRail",
            "IconButton",
            "IconButton",
            "IconButton",
            "DocumentHost",
            "DocumentTabs",
            "PaneSurface",
            "StatusBar",
            "Label",
            "Overlay",
            "Container",
            "Container",
            "Overlay",
            "Container",
            "Container",
            "Overlay",
            "Container",
            "Container",
        ]
    );

    let open_project = host_model
        .nodes
        .iter()
        .find(|node| node.control_id.as_deref() == Some("OpenProject"))
        .unwrap();
    assert_eq!(open_project.node_id, "root.0.0.0");
    assert_eq!(open_project.parent_id.as_deref(), Some("root.0.0"));
    assert_eq!(
        open_project.attributes.get("icon"),
        Some(&Value::String("folder-open-outline".to_string()))
    );
    assert_eq!(
        open_project.attributes.get("label"),
        Some(&Value::String("Open".to_string()))
    );
    let open_project_binding = open_project
        .bindings
        .iter()
        .find(|binding| binding.binding_id == "WorkbenchMenuBar/OpenProject")
        .unwrap();
    assert_eq!(open_project_binding.event_kind, UiEventKind::Click);
    let route_id = open_project_binding.route_id.expect("registered route");
    assert_eq!(
        service.route_binding(route_id).unwrap(),
        projection
            .bindings
            .iter()
            .find(|binding| binding.binding_id == "WorkbenchMenuBar/OpenProject")
            .unwrap()
            .binding
            .as_ui_binding()
    );

    let status_text = host_model
        .nodes
        .iter()
        .find(|node| node.control_id.as_deref() == Some("StatusText"))
        .unwrap();
    assert_eq!(status_text.node_id, "root.0.4.0");
    assert_eq!(
        status_text.attributes.get("text"),
        Some(&Value::String("Ready".to_string()))
    );
}

#[test]
fn editor_ui_compatibility_harness_captures_host_model_routes_and_attributes() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();
    let mut projection = runtime
        .project_document(UI_HOST_WINDOW_DOCUMENT_ID)
        .unwrap();
    let mut service = EditorUiControlService::default();
    runtime
        .register_projection_routes(&mut service, &mut projection)
        .unwrap();
    let host_model = runtime.build_host_model(&projection).unwrap();

    let snapshot = EditorUiCompatibilityHarness::capture_host_model_snapshot(&host_model);

    assert!(snapshot
        .host_nodes
        .contains(&"root.0.0.0|IconButton|OpenProject".to_string()));
    assert!(snapshot
        .host_nodes
        .contains(&"root.0.4.0|Label|StatusText".to_string()));
    assert!(snapshot
        .route_bindings
        .iter()
        .any(|entry: &String| entry.starts_with("WorkbenchMenuBar/OpenProject@")));
    assert!(snapshot
        .attribute_entries
        .contains(&"root.0.0.0.icon=folder-open-outline".to_string()));
    assert!(snapshot
        .attribute_entries
        .contains(&"root.0.4.0.text=Ready".to_string()));
}

#[test]
fn slint_ui_host_adapter_builds_generic_projection_from_host_model() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();
    let mut projection = runtime
        .project_document(UI_HOST_WINDOW_DOCUMENT_ID)
        .unwrap();
    let mut service = EditorUiControlService::default();
    runtime
        .register_projection_routes(&mut service, &mut projection)
        .unwrap();
    let host_model = runtime.build_host_model(&projection).unwrap();

    let slint_projection = SlintUiHostAdapter::build_projection(&host_model);

    assert_eq!(slint_projection.document_id, UI_HOST_WINDOW_DOCUMENT_ID);
    assert_eq!(
        slint_projection
            .nodes
            .iter()
            .map(|node| node.kind)
            .collect::<Vec<_>>(),
        vec![
            SlintUiHostComponentKind::Root,
            SlintUiHostComponentKind::VerticalBox,
            SlintUiHostComponentKind::Toolbar,
            SlintUiHostComponentKind::IconButton,
            SlintUiHostComponentKind::IconButton,
            SlintUiHostComponentKind::IconButton,
            SlintUiHostComponentKind::Unknown,
            SlintUiHostComponentKind::Unknown,
            SlintUiHostComponentKind::HorizontalBox,
            SlintUiHostComponentKind::ActivityRail,
            SlintUiHostComponentKind::IconButton,
            SlintUiHostComponentKind::IconButton,
            SlintUiHostComponentKind::IconButton,
            SlintUiHostComponentKind::DocumentHost,
            SlintUiHostComponentKind::TabStrip,
            SlintUiHostComponentKind::PaneSurface,
            SlintUiHostComponentKind::StatusBar,
            SlintUiHostComponentKind::Label,
            SlintUiHostComponentKind::Unknown,
            SlintUiHostComponentKind::Unknown,
            SlintUiHostComponentKind::Unknown,
            SlintUiHostComponentKind::Unknown,
            SlintUiHostComponentKind::Unknown,
            SlintUiHostComponentKind::Unknown,
            SlintUiHostComponentKind::Unknown,
            SlintUiHostComponentKind::Unknown,
            SlintUiHostComponentKind::Unknown,
        ]
    );

    let open_project = slint_projection.node_by_control_id("OpenProject").unwrap();
    assert_eq!(open_project.node_id, "root.0.0.0");
    assert_eq!(open_project.parent_id.as_deref(), Some("root.0.0"));
    assert_eq!(open_project.kind, SlintUiHostComponentKind::IconButton);
    assert_eq!(open_project.text.as_deref(), Some("Open"));
    assert_eq!(open_project.icon.as_deref(), Some("folder-open-outline"));
    assert_eq!(
        open_project.properties.get("label").cloned().unwrap(),
        SlintUiHostValue::String("Open".to_string())
    );
    let click_route = open_project
        .routes
        .iter()
        .find(|route| route.binding_id == "WorkbenchMenuBar/OpenProject")
        .unwrap()
        .route_id
        .expect("click route");
    assert_eq!(
        service.route_binding(click_route).unwrap(),
        projection
            .bindings
            .iter()
            .find(|binding| binding.binding_id == "WorkbenchMenuBar/OpenProject")
            .unwrap()
            .binding
            .as_ui_binding()
    );

    let status_text = slint_projection.node_by_control_id("StatusText").unwrap();
    assert_eq!(status_text.kind, SlintUiHostComponentKind::Label);
    assert_eq!(status_text.text.as_deref(), Some("Ready"));
    assert_eq!(status_text.icon, None);
}

#[test]
fn editor_ui_host_runtime_builds_slint_host_projection_and_snapshot() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();
    let mut projection = runtime
        .project_document(UI_HOST_WINDOW_DOCUMENT_ID)
        .unwrap();
    let mut service = EditorUiControlService::default();
    runtime
        .register_projection_routes(&mut service, &mut projection)
        .unwrap();

    let slint_projection = runtime.build_slint_host_projection(&projection).unwrap();
    let snapshot =
        EditorUiCompatibilityHarness::capture_slint_host_projection_snapshot(&slint_projection);

    assert_eq!(slint_projection.nodes.len(), 27);
    assert!(snapshot
        .slint_nodes
        .contains(&"root.0.0.0|IconButton|OpenProject".to_string()));
    assert!(snapshot
        .slint_nodes
        .contains(&"root.0.4.0|Label|StatusText".to_string()));
    assert!(snapshot
        .text_entries
        .contains(&"root.0.0.0=Open".to_string()));
    assert!(snapshot
        .icon_entries
        .contains(&"root.0.0.0=folder-open-outline".to_string()));
    assert!(snapshot
        .route_bindings
        .iter()
        .any(|entry: &String| entry.starts_with("WorkbenchMenuBar/OpenProject@")));
}
