use super::support::*;

#[test]
fn editor_ui_host_runtime_projects_an_empty_fallback_pane_shell() {
    let template = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets/ui/editor/host/pane_surface_controls.zui"),
    )
    .expect("pane surface controls template should be readable");
    assert!(
        template.contains("gap = \"$editor.density.gap.small\""),
        "pane surface controls must consume the shared dense spacing token"
    );
    assert!(
        !template.contains("gap = 6.0"),
        "pane surface controls must not keep a local spacing literal"
    );

    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();

    assert_eq!(
        runtime
            .component_descriptor("PaneSurfaceControls")
            .unwrap()
            .binding_namespace,
        "PaneSurface"
    );

    let projection = runtime
        .project_document("res://ui/editor/host/pane_surface_controls.zui")
        .unwrap();

    assert_eq!(
        projection.document_id,
        "res://ui/editor/host/pane_surface_controls.zui"
    );
    assert_eq!(projection.root.component, "HorizontalGroup");
    assert!(projection.root.children.is_empty());
    assert!(projection.bindings.is_empty());

    let mut surface = runtime
        .build_shared_surface("res://ui/editor/host/pane_surface_controls.zui")
        .unwrap();
    surface.compute_layout(UiSize::new(300.0, 32.0)).unwrap();
    let host_model = runtime
        .build_host_model_with_surface(&projection, &surface)
        .unwrap();

    let root = host_model
        .node_by_control_id("PaneSurfaceControlsRoot")
        .expect("pane surface controls root should remain a host container");
    assert_eq!(root.component, "HorizontalGroup");
    assert_eq!(root.frame, UiFrame::new(0.0, 0.0, 300.0, 32.0));
    assert!(host_model.node_by_control_id("TriggerAction").is_none());
}
