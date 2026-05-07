use super::support::*;

#[test]
fn editor_ui_host_runtime_projects_pane_surface_controls_through_material_button() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();

    assert_eq!(
        runtime
            .component_descriptor("PaneSurfaceControls")
            .unwrap()
            .binding_namespace,
        "PaneSurface"
    );

    let projection = runtime.project_document("pane.surface_controls").unwrap();

    assert_eq!(projection.document_id, "pane.surface_controls");
    assert_eq!(projection.root.component, "PaneSurfaceControls");
    assert_eq!(
        projection
            .root
            .children
            .iter()
            .map(|node| node.control_id.as_deref().unwrap_or_default())
            .collect::<Vec<_>>(),
        vec!["TriggerAction"]
    );

    let trigger = projection
        .bindings
        .iter()
        .find(|binding| binding.binding_id == "PaneSurface/TriggerAction")
        .unwrap();
    assert_eq!(trigger.binding.path().event_kind, UiEventKind::Click);
    assert_eq!(trigger.binding.path().view_id, "PaneSurface");
    assert_eq!(trigger.binding.path().control_id, "TriggerAction");

    let mut surface = runtime
        .build_shared_surface("pane.surface_controls")
        .unwrap();
    surface.compute_layout(UiSize::new(300.0, 32.0)).unwrap();
    let host_model = runtime
        .build_host_model_with_surface(&projection, &surface)
        .unwrap();

    let root = host_model
        .node_by_control_id("PaneSurfaceControlsRoot")
        .expect("pane surface controls root should remain a host container");
    assert_eq!(root.component, "PaneSurfaceControls");
    assert_eq!(root.frame, UiFrame::new(0.0, 0.0, 300.0, 32.0));

    let button = host_model
        .node_by_control_id("TriggerAction")
        .expect("pane trigger action should project as a Material button root");
    assert_eq!(button.component, "Button");
    assert_eq!(button.frame, UiFrame::new(0.0, 0.0, 128.0, 32.0));
    assert_eq!(
        button.attributes.get("text").and_then(Value::as_str),
        Some("Open Project")
    );
    assert_eq!(
        button
            .attributes
            .get("input_interactive")
            .and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        button
            .attributes
            .get("input_clickable")
            .and_then(Value::as_bool),
        Some(true)
    );
}
