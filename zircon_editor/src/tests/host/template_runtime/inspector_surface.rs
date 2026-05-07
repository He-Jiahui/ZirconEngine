use super::support::*;

#[test]
fn editor_ui_host_runtime_projects_builtin_inspector_surface_template_into_slint_projection() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();

    assert_eq!(
        runtime
            .component_descriptor("InspectorSurfaceControls")
            .unwrap()
            .binding_namespace,
        "InspectorView"
    );

    let projection = runtime
        .project_document("inspector.surface_controls")
        .unwrap();

    assert_eq!(projection.document_id, "inspector.surface_controls");
    assert_eq!(projection.root.component, "InspectorSurfaceControls");
    assert_eq!(
        projection
            .root
            .children
            .iter()
            .map(|node| node.control_id.as_deref().unwrap_or_default())
            .collect::<Vec<_>>(),
        vec![
            "NameField",
            "ParentField",
            "PositionXField",
            "PositionYField",
            "PositionZField",
            "ApplyBatchButton",
            "DeleteSelected",
        ]
    );

    let name = projection
        .bindings
        .iter()
        .find(|binding| binding.binding_id == "InspectorView/NameField")
        .unwrap();
    assert_eq!(name.binding.path().event_kind, UiEventKind::Change);
    assert_eq!(name.binding.path().view_id, "InspectorView");
    assert_eq!(name.binding.path().control_id, "NameField");

    let apply = projection
        .bindings
        .iter()
        .find(|binding| binding.binding_id == "InspectorView/ApplyBatchButton")
        .unwrap();
    assert_eq!(apply.binding.path().event_kind, UiEventKind::Click);
    assert_eq!(apply.binding.path().view_id, "InspectorView");
    assert_eq!(apply.binding.path().control_id, "ApplyBatchButton");
}

#[test]
fn editor_ui_host_runtime_projects_inspector_controls_through_material_roots() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();

    let projection = runtime
        .project_document("inspector.surface_controls")
        .unwrap();
    let mut surface = runtime
        .build_shared_surface("inspector.surface_controls")
        .unwrap();
    surface.compute_layout(UiSize::new(360.0, 240.0)).unwrap();
    let host_model = runtime
        .build_host_model_with_surface(&projection, &surface)
        .unwrap();

    let name = host_model
        .node_by_control_id("NameField")
        .expect("inspector name field should project as a Material line edit root");
    assert_eq!(name.component, "InputField");
    assert_eq!(
        name.attributes.get("placeholder").and_then(Value::as_str),
        Some("Name")
    );
    assert_eq!(
        name.attributes
            .get("input_focusable")
            .and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        name.attributes
            .get("layout_min_height")
            .and_then(Value::as_float),
        Some(32.0)
    );

    let position_x = host_model
        .node_by_control_id("PositionXField")
        .expect("inspector position field should project as a Material spin box root");
    assert_eq!(position_x.component, "NumberField");
    assert_eq!(
        position_x.attributes.get("step").and_then(Value::as_float),
        Some(0.1)
    );
    assert_eq!(
        position_x
            .attributes
            .get("large_step")
            .and_then(Value::as_float),
        Some(1.0)
    );
    assert_eq!(
        position_x
            .attributes
            .get("input_clickable")
            .and_then(Value::as_bool),
        Some(true)
    );

    let apply = host_model
        .node_by_control_id("ApplyBatchButton")
        .expect("inspector apply should project as a Material button root");
    assert_eq!(apply.component, "Button");
    assert_eq!(
        apply.attributes.get("text").and_then(Value::as_str),
        Some("Apply")
    );
    assert_eq!(
        apply
            .attributes
            .get("button_variant")
            .and_then(Value::as_str),
        Some("primary")
    );
    assert!(apply.bindings.iter().any(|binding| {
        binding.binding_id == "InspectorView/ApplyBatchButton"
            && binding.event_kind == UiEventKind::Click
    }));

    let delete = host_model
        .node_by_control_id("DeleteSelected")
        .expect("inspector delete should project as a Material button root");
    assert_eq!(delete.component, "Button");
    assert_eq!(
        delete.attributes.get("text_tone").and_then(Value::as_str),
        Some("danger")
    );
    assert!(delete.bindings.iter().any(|binding| {
        binding.binding_id == "InspectorView/DeleteSelected"
            && binding.event_kind == UiEventKind::Click
    }));
}
