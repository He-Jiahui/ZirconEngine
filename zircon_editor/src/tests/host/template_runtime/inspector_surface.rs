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
