use super::super::super::super::super::super::paint_frame::HostRecordedPaintKind;
use super::*;

#[test]
fn welcome_path_preview_uses_shared_surface_and_runtime_text_inset() {
    let mut frame = HostRgbaFrame::recording_only(360, 96);
    let mut pane = PaneData::default();
    pane.welcome.form.project_path_preview = "E:/Projects/ZirconProject".into();
    let preview = FrameRect {
        x: 16.0,
        y: 12.0,
        width: 328.0,
        height: 72.0,
    };
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 360.0,
        height: 96.0,
    };

    draw_welcome_preview(&mut frame, &pane, &preview, &clip);

    let commands = frame.into_recorded_commands();
    assert_eq!(commands.len(), 4);
    let metrics = current_host_metrics();
    assert!(matches!(
        &commands[0].kind,
        HostRecordedPaintKind::Quad { corner_radius, .. }
            if *corner_radius == metrics.radius_control
    ));
    assert!(matches!(
        &commands[1].kind,
        HostRecordedPaintKind::Border { width, corner_radius, .. }
            if *width == metrics.border_width && *corner_radius == metrics.radius_control
    ));
    for (command, expected_font_size) in commands[2..]
        .iter()
        .zip([metrics.font_small, metrics.font_body])
    {
        assert_eq!(command.frame.x, preview.x + metrics.gap_l);
        assert_eq!(command.frame.width, preview.width - metrics.gap_l * 2.0);
        assert!(matches!(
            &command.kind,
            HostRecordedPaintKind::Text { font_size, .. }
                if *font_size == expected_font_size
        ));
    }
    assert!(matches!(
        &commands[3].kind,
        HostRecordedPaintKind::Text { text, color, .. }
            if text == "E:/Projects/ZirconProject" && *color == WELCOME_TEXT
    ));
}
