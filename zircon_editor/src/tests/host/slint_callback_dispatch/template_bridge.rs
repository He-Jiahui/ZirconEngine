use super::support::*;

#[test]
fn builtin_workbench_template_bridge_recomputes_surface_backed_frames_with_shell_size() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge = BuiltinWorkbenchTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    let initial = bridge
        .host_projection()
        .node_by_control_id("DocumentHostRoot")
        .expect("document host control should exist")
        .frame;
    assert_eq!(initial, UiFrame::new(56.0, 40.0, 1224.0, 656.0));

    bridge.recompute_layout(UiSize::new(960.0, 540.0)).unwrap();

    let recomputed = bridge
        .host_projection()
        .node_by_control_id("DocumentHostRoot")
        .expect("document host control should exist after recompute")
        .frame;
    assert_eq!(recomputed, UiFrame::new(56.0, 40.0, 904.0, 476.0));
}
