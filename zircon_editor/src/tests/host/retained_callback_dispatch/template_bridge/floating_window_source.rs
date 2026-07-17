use super::super::support::*;

#[test]
fn builtin_floating_window_source_template_bridge_recomputes_surface_backed_frames_with_shell_size()
{
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinFloatingWindowSourceTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    assert_eq!(
        bridge.source_frames().center_band_frame,
        Some(UiFrame::new(0.0, 30.0, 1280.0, 668.0))
    );
    assert_eq!(
        bridge.source_frames().document_frame,
        Some(UiFrame::new(34.0, 30.0, 1246.0, 668.0))
    );

    assert!(bridge.recompute_layout(UiSize::new(960.0, 540.0)).unwrap());

    assert_eq!(
        bridge.source_frames().center_band_frame,
        Some(UiFrame::new(0.0, 30.0, 960.0, 488.0))
    );
    assert_eq!(
        bridge.source_frames().document_frame,
        Some(UiFrame::new(34.0, 30.0, 926.0, 488.0))
    );
}

#[test]
fn builtin_floating_window_source_template_bridge_reuses_surface_across_layout_recompute() {
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinFloatingWindowSourceTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    let first_node_ids = bridge.debug_surface_node_ids();
    let first_render_commands = bridge.debug_render_command_count();
    let first_rebuild = bridge.debug_last_rebuild_report();

    assert!(!bridge.recompute_layout(UiSize::new(1280.0, 720.0)).unwrap());

    assert_eq!(bridge.debug_surface_node_ids(), first_node_ids);
    assert_eq!(bridge.debug_render_command_count(), first_render_commands);
    assert_eq!(bridge.debug_last_rebuild_report(), first_rebuild);
}
