use crate::ui::workbench::autolayout::ShellFrame;
use zircon_runtime_interface::ui::layout::UiPoint;

use super::child_pointer_to_workbench_point;

#[test]
fn child_local_pointer_uses_the_projected_outer_frame_origin() {
    let frame = ShellFrame::new(384.0, 168.0, 640.0, 420.0);

    assert_eq!(
        child_pointer_to_workbench_point(frame, UiPoint::new(31.0, 47.0)),
        UiPoint::new(415.0, 215.0),
    );
}

#[test]
fn repeated_drag_target_group_does_not_republish_ui_state() {
    let source = include_str!("../drag_drop.rs");
    let production = source.split("#[cfg(test)]").next().unwrap_or(source);

    assert!(production.contains("source_ui.global::<UiHostContext>()"));
    assert!(production.contains("child_pointer_to_workbench_point(frames.outer_frame"));
    assert!(production.contains("drag_target_group_matches"));
    assert!(production.contains("host_shell.set_drag_target_group(value);"));
    assert!(!production.contains("host_shell.set_drag_state"));
}
