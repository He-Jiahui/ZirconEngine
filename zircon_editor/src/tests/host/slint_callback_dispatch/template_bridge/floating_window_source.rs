use super::super::support::*;

#[test]
fn builtin_floating_window_source_template_bridge_recomputes_surface_backed_frames_with_shell_size()
{
    let _guard = env_lock().lock().unwrap();

    let mut bridge =
        BuiltinFloatingWindowSourceTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    assert_eq!(
        bridge.source_frames().center_band_frame,
        Some(UiFrame::new(0.0, 40.0, 1280.0, 656.0))
    );
    assert_eq!(
        bridge.source_frames().document_frame,
        Some(UiFrame::new(56.0, 40.0, 1224.0, 656.0))
    );

    bridge.recompute_layout(UiSize::new(960.0, 540.0)).unwrap();

    assert_eq!(
        bridge.source_frames().center_band_frame,
        Some(UiFrame::new(0.0, 40.0, 960.0, 476.0))
    );
    assert_eq!(
        bridge.source_frames().document_frame,
        Some(UiFrame::new(56.0, 40.0, 904.0, 476.0))
    );
}
