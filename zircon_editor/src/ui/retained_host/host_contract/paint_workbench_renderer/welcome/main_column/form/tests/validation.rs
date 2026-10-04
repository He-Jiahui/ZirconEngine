use super::super::super::super::super::super::paint_frame::HostRecordedPaintKind;
use super::*;

#[test]
fn welcome_validation_uses_semantic_marker_and_centered_runtime_text_for_each_state() {
    let validation = FrameRect {
        x: 16.0,
        y: 8.0,
        width: 328.0,
        height: 32.0,
    };
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 360.0,
        height: 48.0,
    };
    let metrics = current_host_metrics();

    for (can_create, expected_message, expected_color) in [
        (false, "Enter a project name and location", WELCOME_WARNING),
        (true, "Project settings are valid", WELCOME_SUCCESS),
    ] {
        let mut frame = HostRgbaFrame::recording_only(360, 48);
        let mut pane = PaneData::default();
        pane.welcome.form.can_create = can_create;

        draw_welcome_validation(&mut frame, &pane, &validation, &clip);

        let commands = frame.into_recorded_commands();
        assert_eq!(commands.len(), 2);
        assert!(matches!(
            &commands[0].kind,
            HostRecordedPaintKind::Quad { color, corner_radius }
                if *color == expected_color && *corner_radius == metrics.gap_m * 0.5
        ));
        assert!(matches!(
            &commands[1].kind,
            HostRecordedPaintKind::Text { text, color, font_size, .. }
                if text == expected_message
                    && *color == expected_color
                    && *font_size == metrics.font_body
        ));
        let text_center_y = commands[1].frame.y + commands[1].frame.height * 0.5;
        assert!((text_center_y - (validation.y + validation.height * 0.5)).abs() <= 1.0);
    }
}
