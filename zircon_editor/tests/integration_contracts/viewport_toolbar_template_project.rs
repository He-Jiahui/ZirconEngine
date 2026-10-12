use zircon_editor::ui::control::EditorUiControlService;
use zircon_editor::ui::template_runtime::EditorUiHostRuntime;
use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};

#[test]
fn viewport_toolbar_template_projects_surface_backed_group_frames() {
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
        .expect("scene mode activation group should exist");
    assert_eq!(
        activate_scene_mode.frame,
        UiFrame::new(0.0, 0.0, 28.0, 28.0)
    );

    let set_transform_space = host_model
        .node_by_control_id("SetTransformSpace")
        .expect("transform space group should exist");
    assert_eq!(
        set_transform_space.frame,
        UiFrame::new(
            activate_scene_mode.frame.x + activate_scene_mode.frame.width + 4.0,
            0.0,
            28.0,
            28.0
        )
    );

    let set_pivot_mode = host_model
        .node_by_control_id("SetPivotMode")
        .expect("pivot mode control should exist");
    assert_eq!(
        set_pivot_mode.frame,
        UiFrame::new(
            set_transform_space.frame.x + set_transform_space.frame.width + 4.0,
            0.0,
            28.0,
            28.0
        )
    );

    let set_projection_mode = host_model
        .node_by_control_id("SetProjectionMode")
        .expect("projection mode group should exist");

    let align_view = host_model
        .node_by_control_id("AlignView")
        .expect("align view group should exist");
    assert_eq!(set_projection_mode.frame.height, 28.0);
    assert_eq!(align_view.frame.height, 28.0);
    assert_eq!(
        align_view.frame.x,
        set_projection_mode.frame.x + set_projection_mode.frame.width + 4.0
    );
    assert_eq!(
        align_view.frame.x + align_view.frame.width,
        root.frame.width
    );

    let frame_selection = host_model
        .node_by_control_id("FrameSelection")
        .expect("frame selection control should exist");
    assert_eq!(frame_selection.frame.width, 28.0);
    assert_eq!(frame_selection.frame.height, 28.0);
    assert!(frame_selection.frame.x > set_transform_space.frame.x);
}
