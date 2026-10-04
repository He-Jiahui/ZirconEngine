use super::*;

#[test]
fn componentized_workbench_mount_starts_after_host_chrome_and_keeps_window_bottom() {
    let frames = BuiltinHostRootShellFrames {
        menu_bar_frame: Some(UiFrame::new(0.0, 0.0, 1280.0, 24.0)),
        host_page_strip_frame: Some(UiFrame::new(0.0, 24.0, 1280.0, 32.0)),
        host_body_frame: Some(UiFrame::new(0.0, 57.0, 1280.0, 639.0)),
        ..Default::default()
    };

    assert_eq!(
        frames.componentized_workbench_mount_frame(UiSize::new(1280.0, 720.0)),
        UiFrame::new(0.0, 57.0, 1280.0, 663.0)
    );
}
