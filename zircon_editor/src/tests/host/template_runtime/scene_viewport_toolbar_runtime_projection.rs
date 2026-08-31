use super::support::*;

#[test]
fn editor_ui_host_runtime_projects_builtin_viewport_toolbar_template_into_retained_projection() {
    let template = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets/ui/editor/host/scene_viewport_toolbar.zui"),
    )
    .expect("viewport toolbar template should be readable");
    assert_eq!(
        template.matches("icon_placement = \"icon_only\"").count(),
        16,
        "every 28px viewport toolbar action must retain only its semantic label"
    );

    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();

    assert_eq!(
        runtime
            .component_descriptor("SceneViewportToolbar")
            .unwrap()
            .binding_namespace,
        "ViewportToolbar"
    );

    let projection = runtime
        .project_document("res://ui/editor/host/scene_viewport_toolbar.zui")
        .unwrap();

    assert_eq!(
        projection.document_id,
        "res://ui/editor/host/scene_viewport_toolbar.zui"
    );
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
            "SceneViewportToolbarFill",
            "SceneViewportToolbarRightGroup",
        ]
    );

    let activate_scene_mode = projection
        .bindings
        .iter()
        .find(|binding| binding.binding_id == "ViewportToolbar/ActivateSceneMode")
        .unwrap();
    assert_eq!(
        activate_scene_mode.binding.path().event_kind,
        UiEventKind::Change
    );
    assert_eq!(
        activate_scene_mode.binding.path().view_id,
        "ViewportToolbar"
    );
    assert_eq!(
        activate_scene_mode.binding.path().control_id,
        "ActivateSceneMode"
    );

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
    let mut projection = runtime
        .project_document("res://ui/editor/host/scene_viewport_toolbar.zui")
        .unwrap();
    let mut service = EditorUiControlService::default();
    runtime
        .register_projection_routes(&mut service, &mut projection)
        .unwrap();
    let mut surface = runtime
        .build_shared_surface("res://ui/editor/host/scene_viewport_toolbar.zui")
        .unwrap();
    surface.compute_layout(UiSize::new(1280.0, 28.0)).unwrap();

    let host_model = runtime
        .build_host_model_with_surface(&projection, &surface)
        .unwrap();

    let root = host_model
        .node_by_control_id("SceneViewportToolbarRoot")
        .expect("viewport toolbar root should exist");
    assert_eq!(root.frame, UiFrame::new(0.0, 0.0, 1280.0, 28.0));

    let activate_scene_mode = host_model
        .node_by_control_id("ActivateSceneMode")
        .expect("scene mode group should exist");
    assert_eq!(
        activate_scene_mode.frame,
        UiFrame::new(0.0, 0.0, 28.0, 28.0)
    );
    assert_eq!(activate_scene_mode.component, "IconButton");
    assert_eq!(
        activate_scene_mode
            .attributes
            .get("label")
            .and_then(Value::as_str),
        Some("Tool")
    );

    let set_transform_space = host_model
        .node_by_control_id("SetTransformSpace")
        .expect("transform space group should exist");
    assert_eq!(set_transform_space.component, "IconButton");
    assert_eq!(
        set_transform_space.frame,
        UiFrame::new(32.0, 0.0, 28.0, 28.0)
    );

    let set_pivot_mode = host_model
        .node_by_control_id("SetPivotMode")
        .expect("pivot mode control should exist");
    assert_eq!(set_pivot_mode.component, "IconButton");
    assert_eq!(set_pivot_mode.frame, UiFrame::new(64.0, 0.0, 28.0, 28.0));

    let set_projection_mode = host_model
        .node_by_control_id("SetProjectionMode")
        .expect("projection mode group should exist");
    assert_eq!(
        set_projection_mode.frame,
        UiFrame::new(1184.0, 0.0, 46.0, 28.0)
    );

    let align_view = host_model
        .node_by_control_id("AlignView")
        .expect("align view group should exist");
    assert_eq!(align_view.frame, UiFrame::new(1234.0, 0.0, 46.0, 28.0));

    let frame_selection = host_model
        .node_by_control_id("FrameSelection")
        .expect("frame selection control should exist");
    assert_eq!(frame_selection.component, "IconButton");
    assert_eq!(frame_selection.frame, UiFrame::new(352.0, 0.0, 28.0, 28.0));
}

#[test]
fn editor_ui_host_runtime_keeps_play_and_view_controls_non_overlapping_at_640px() {
    let mut runtime = EditorUiHostRuntime::default();
    let loaded = runtime.load_builtin_host_templates();
    assert!(loaded.is_ok());
    if loaded.is_err() {
        return;
    }
    let projection = runtime.project_document("res://ui/editor/host/scene_viewport_toolbar.zui");
    assert!(projection.is_ok());
    let Ok(projection) = projection else {
        return;
    };
    let surface = runtime.build_shared_surface("res://ui/editor/host/scene_viewport_toolbar.zui");
    assert!(surface.is_ok());
    let Ok(mut surface) = surface else {
        return;
    };
    let computed = surface.compute_layout(UiSize::new(640.0, 28.0));
    assert!(computed.is_ok());
    if computed.is_err() {
        return;
    }
    let host_model = runtime.build_host_model_with_surface(&projection, &surface);
    assert!(host_model.is_ok());
    let Ok(host_model) = host_model else {
        return;
    };

    assert!(host_model.node_by_control_id("EnterPlayMode").is_some());
    assert!(host_model.node_by_control_id("ExitPlayMode").is_some());
    assert!(host_model.node_by_control_id("SetProjectionMode").is_some());
    assert!(host_model.node_by_control_id("AlignView").is_some());
    let Some(play) = host_model.node_by_control_id("EnterPlayMode") else {
        return;
    };
    let Some(stop) = host_model.node_by_control_id("ExitPlayMode") else {
        return;
    };
    let Some(projection_mode) = host_model.node_by_control_id("SetProjectionMode") else {
        return;
    };
    let Some(align_view) = host_model.node_by_control_id("AlignView") else {
        return;
    };

    assert_eq!(play.frame, UiFrame::new(384.0, 0.0, 28.0, 28.0));
    assert_eq!(stop.frame, UiFrame::new(416.0, 0.0, 28.0, 28.0));
    assert!(stop.frame.x + stop.frame.width <= projection_mode.frame.x);
    assert!(projection_mode.frame.x + projection_mode.frame.width <= align_view.frame.x);
    assert!(align_view.frame.x + align_view.frame.width <= 640.0);
}
