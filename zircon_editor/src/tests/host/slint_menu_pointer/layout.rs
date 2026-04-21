use super::support::*;

#[test]
fn shared_menu_pointer_layout_prefers_shared_root_menu_bar_projection_over_stale_legacy_frames() {
    let harness = EventRuntimeHarness::new("zircon_slint_menu_pointer_root_projection");
    let chrome = harness.runtime.chrome_snapshot();
    let layout = build_workbench_menu_pointer_layout(
        &chrome,
        UiSize::new(1280.0, 720.0),
        &["alpha-00".to_string(), "alpha-01".to_string()],
        Some("compact"),
        Some(&BuiltinWorkbenchRootShellFrames {
            shell_frame: Some(UiFrame::new(32.0, 18.0, 1440.0, 900.0)),
            menu_bar_frame: Some(UiFrame::new(32.0, 18.0, 1440.0, 40.0)),
            ..Default::default()
        }),
    );

    assert_eq!(layout.shell_frame, UiFrame::new(32.0, 18.0, 1440.0, 900.0));
    assert_eq!(
        layout.button_frames,
        [
            UiFrame::new(40.0, 19.0, 40.0, 22.0),
            UiFrame::new(82.0, 19.0, 42.0, 22.0),
            UiFrame::new(126.0, 19.0, 74.0, 22.0),
            UiFrame::new(202.0, 19.0, 42.0, 22.0),
            UiFrame::new(246.0, 19.0, 56.0, 22.0),
            UiFrame::new(304.0, 19.0, 40.0, 22.0),
        ],
        "shared root menu bar projection should own top-level menu button frames"
    );
    assert_eq!(layout.active_preset_name, "compact");
    assert_eq!(layout.resolved_preset_name, "compact");
}

#[test]
fn shared_menu_pointer_layout_derives_button_frames_from_shared_shell_when_menu_bar_frame_is_missing(
) {
    let harness = EventRuntimeHarness::new("zircon_slint_menu_pointer_shell_projection");
    let chrome = harness.runtime.chrome_snapshot();
    let layout = build_workbench_menu_pointer_layout(
        &chrome,
        UiSize::new(1280.0, 720.0),
        &["alpha-00".to_string(), "alpha-01".to_string()],
        Some("compact"),
        Some(&BuiltinWorkbenchRootShellFrames {
            shell_frame: Some(UiFrame::new(32.0, 18.0, 1440.0, 900.0)),
            menu_bar_frame: None,
            ..Default::default()
        }),
    );

    assert_eq!(
        layout.button_frames,
        [
            UiFrame::new(40.0, 19.0, 40.0, 22.0),
            UiFrame::new(82.0, 19.0, 42.0, 22.0),
            UiFrame::new(126.0, 19.0, 74.0, 22.0),
            UiFrame::new(202.0, 19.0, 42.0, 22.0),
            UiFrame::new(246.0, 19.0, 56.0, 22.0),
            UiFrame::new(304.0, 19.0, 40.0, 22.0),
        ],
        "shared shell projection should still own top-level menu button frames when menu bar frame is temporarily unavailable"
    );
}
