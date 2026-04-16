use zircon_editor::{EditorUiHostRuntime, UiSize};
use zircon_editor_ui::EditorUiControlService;
use zircon_ui::UiFrame;

#[test]
fn viewport_toolbar_template_projects_surface_backed_group_frames() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_workbench_shell().unwrap();
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

    let set_transform_space = host_model
        .node_by_control_id("SetTransformSpace")
        .expect("transform space group should exist");
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
    assert_eq!(frame_selection.frame, UiFrame::new(649.0, 4.0, 20.0, 20.0));
}
