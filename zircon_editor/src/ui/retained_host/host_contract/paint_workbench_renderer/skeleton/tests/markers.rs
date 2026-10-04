use super::*;
use crate::ui::retained_host::host_contract::paint_frame::HostRecordedPaintKind;
use crate::ui::retained_host::host_contract::paint_theme::METRICS;

#[test]
fn project_marker_uses_a_finite_runtime_text_slot_with_ellipsis() {
    let top_bar = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 116.0,
        height: 24.0,
    };
    let mut frame = HostRgbaFrame::recording_only(128, 32);
    let palette = RootSkeletonPalette {
        top_bar: [0, 0, 0, 0],
        center_band: [0, 0, 0, 0],
        dock: [0, 0, 0, 0],
        document: [0, 0, 0, 0],
        viewport: [0, 0, 0, 0],
        status: [0, 0, 0, 0],
        separator: [0, 0, 0, 0],
        accent: [60, 199, 214, 255],
        text_muted: [164, 174, 180, 255],
        marker_surface: [0, 0, 0, 0],
    };

    draw_project_marker(
        &mut frame,
        "res://projects/a-very-long-workbench-project-name",
        &top_bar,
        palette,
        METRICS,
    );

    let command = frame
        .into_recorded_commands()
        .into_iter()
        .find(|command| matches!(&command.kind, HostRecordedPaintKind::Text { .. }))
        .expect("project marker should use Runtime Text");
    let HostRecordedPaintKind::Text { text, .. } = &command.kind else {
        unreachable!("filtered command should be text");
    };

    assert!(text.ends_with('\u{2026}'));
    assert!(command.frame.x >= top_bar.x);
    assert!(command.frame.x + command.frame.width <= top_bar.x + top_bar.width);
}
