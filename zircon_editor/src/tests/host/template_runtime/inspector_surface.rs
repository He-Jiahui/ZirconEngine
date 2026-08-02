use super::support::*;

#[test]
fn editor_ui_host_runtime_projects_builtin_inspector_surface_template_into_retained_projection() {
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
        .project_document("res://ui/editor/host/inspector_surface_controls.zui")
        .unwrap();

    assert_eq!(
        projection.document_id,
        "res://ui/editor/host/inspector_surface_controls.zui"
    );
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
            "InspectorActionsRow",
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
fn inspector_actions_stay_in_one_dense_responsive_row() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets/ui/editor/host/inspector_surface_controls.zui"),
    )
    .expect("inspector controls template should be readable");

    assert!(
        source.contains(
            "component = \"HorizontalGroup\"\ncontrol_id = \"InspectorActionsRow\"\nlayout = { container = { kind = \"HorizontalBox\", gap = \"$editor.density.gap.small\" }, width = { stretch = \"Stretch\" }, height = { min = \"$editor.control.height.dense\", preferred = \"$editor.control.height.dense\", max = \"$editor.control.height.dense\", stretch = \"Fixed\" } }"
        ),
        "inspector commands must share one dense action row rather than stretch independently in the vertical form"
    );
    for control_id in ["ApplyBatchButton", "DeleteSelected"] {
        let control_start = source
            .find(&format!("control_id = \"{control_id}\""))
            .expect("inspector action must remain in the template");
        let control = &source[control_start..];
        let layout_end = control
            .find("\nevents =")
            .expect("inspector action must declare an event after its layout");
        assert!(
            control[..layout_end].contains("width = { stretch = \"Stretch\" }"),
            "{control_id} must share the available action-row width"
        );
    }
    assert!(
        !source.contains("preferred = 84.0") && !source.contains("preferred = 88.0"),
        "inspector actions must not keep fixed desktop-only widths"
    );
}

#[test]
fn editor_ui_host_runtime_projects_inspector_controls_through_material_roots() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();

    let projection = runtime
        .project_document("res://ui/editor/host/inspector_surface_controls.zui")
        .unwrap();
    let mut surface = runtime
        .build_shared_surface("res://ui/editor/host/inspector_surface_controls.zui")
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
        Some(28.0)
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
