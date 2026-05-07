use super::support::*;

#[test]
fn editor_ui_host_runtime_projects_builtin_viewport_toolbar_template_into_slint_projection() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();

    assert_eq!(
        runtime
            .component_descriptor("SceneViewportToolbar")
            .unwrap()
            .binding_namespace,
        "ViewportToolbar"
    );

    let projection = runtime.project_document("scene.viewport_toolbar").unwrap();

    assert_eq!(projection.document_id, "scene.viewport_toolbar");
    assert_eq!(projection.root.component, "SceneViewportToolbar");
    assert_eq!(
        projection
            .root
            .children
            .iter()
            .map(|node| node.control_id.as_deref().unwrap_or_default())
            .collect::<Vec<_>>(),
        vec![
            "SceneViewportToolbarLeftGroup",
            "SceneViewportToolbarRightGroup",
        ]
    );

    let set_tool = projection
        .bindings
        .iter()
        .find(|binding| binding.binding_id == "ViewportToolbar/SetTool")
        .unwrap();
    assert_eq!(set_tool.binding.path().event_kind, UiEventKind::Change);
    assert_eq!(set_tool.binding.path().view_id, "ViewportToolbar");
    assert_eq!(set_tool.binding.path().control_id, "SetTool");

    let frame_selection = projection
        .bindings
        .iter()
        .find(|binding| binding.binding_id == "ViewportToolbar/FrameSelection")
        .unwrap();
    assert_eq!(
        frame_selection.binding.path().event_kind,
        UiEventKind::Click
    );
    assert_eq!(frame_selection.binding.path().view_id, "ViewportToolbar");
    assert_eq!(frame_selection.binding.path().control_id, "FrameSelection");
}

#[test]
fn editor_ui_host_runtime_builds_surface_backed_viewport_toolbar_group_frames() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();
    let mut projection = runtime.project_document("scene.viewport_toolbar").unwrap();
    let mut service = EditorUiControlService::default();
    runtime
        .register_projection_routes(&mut service, &mut projection)
        .unwrap();
    let mut surface = runtime
        .build_shared_surface("scene.viewport_toolbar")
        .unwrap();
    surface.compute_layout(UiSize::new(1280.0, 28.0)).unwrap();

    let host_model = runtime
        .build_host_model_with_surface(&projection, &surface)
        .unwrap();

    let root = host_model
        .node_by_control_id("SceneViewportToolbarRoot")
        .expect("viewport toolbar root should exist");
    assert_eq!(root.frame, UiFrame::new(0.0, 0.0, 1280.0, 28.0));

    let set_tool = host_model
        .node_by_control_id("SetTool")
        .expect("set tool group should exist");
    assert_eq!(set_tool.frame, UiFrame::new(8.0, 4.0, 172.0, 20.0));
    assert_eq!(set_tool.component, "Button");
    assert_eq!(
        set_tool.attributes.get("text").and_then(Value::as_str),
        Some("Tool")
    );
    assert_eq!(
        set_tool
            .attributes
            .get("input_interactive")
            .and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        set_tool
            .attributes
            .get("input_clickable")
            .and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        set_tool
            .attributes
            .get("input_hoverable")
            .and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        set_tool
            .attributes
            .get("input_focusable")
            .and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        set_tool
            .attributes
            .get("layout_min_height")
            .and_then(Value::as_float),
        Some(20.0)
    );

    let set_transform_space = host_model
        .node_by_control_id("SetTransformSpace")
        .expect("transform space group should exist");
    assert_eq!(set_transform_space.component, "Button");
    assert_eq!(
        set_transform_space.frame,
        UiFrame::new(189.0, 4.0, 86.0, 20.0)
    );

    let set_projection_mode = host_model
        .node_by_control_id("SetProjectionMode")
        .expect("projection mode group should exist");
    assert_eq!(
        set_projection_mode.frame,
        UiFrame::new(958.0, 4.0, 92.0, 20.0)
    );

    let align_view = host_model
        .node_by_control_id("AlignView")
        .expect("align view group should exist");
    assert_eq!(align_view.frame, UiFrame::new(1054.0, 4.0, 200.0, 20.0));

    let frame_selection = host_model
        .node_by_control_id("FrameSelection")
        .expect("frame selection control should exist");
    assert_eq!(frame_selection.component, "Button");
    assert_eq!(frame_selection.frame, UiFrame::new(649.0, 4.0, 20.0, 20.0));
}
