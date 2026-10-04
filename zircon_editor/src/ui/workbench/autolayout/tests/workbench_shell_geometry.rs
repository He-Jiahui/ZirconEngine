use super::*;

#[test]
fn mounted_frame_reuse_ignores_window_constraints_but_not_visible_geometry() {
    let previous = WorkbenchShellGeometry::default();
    let mut next = previous.clone();
    next.window_min_width = 720.0;
    next.window_min_height = 480.0;

    assert!(previous.shares_mounted_layout_frames_with(&next));

    next.viewport_content_frame = ShellFrame::new(1.0, 0.0, 0.0, 0.0);
    assert!(!previous.shares_mounted_layout_frames_with(&next));
}
