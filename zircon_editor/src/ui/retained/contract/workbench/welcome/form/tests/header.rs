use super::super::super::super::super::super::paint_frame::HostRecordedPaintKind;
use super::*;

#[test]
fn welcome_form_header_uses_shared_two_line_typography_and_relative_centering() {
    let mut frame = HostRgbaFrame::recording_only(360, 64);
    let header = FrameRect {
        x: 16.0,
        y: 12.0,
        width: 328.0,
        height: 36.0,
    };
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 360.0,
        height: 64.0,
    };

    draw_welcome_new_project_header(&mut frame, &PaneData::default(), &header, &clip);

    let commands = frame.into_recorded_commands();
    assert_eq!(commands.len(), 2);
    let metrics = current_host_metrics();
    for (command, expected_font_size) in
        commands.iter().zip([metrics.font_body, metrics.font_small])
    {
        assert_eq!(command.frame.x, header.x);
        assert_eq!(command.frame.width, header.width);
        assert!(matches!(
            &command.kind,
            HostRecordedPaintKind::Text { font_size, .. }
                if *font_size == expected_font_size
        ));
    }
    assert!(matches!(
        &commands[1].kind,
        HostRecordedPaintKind::Text { text, .. } if text == "Renderable Empty"
    ));
    let top_gap = commands[0].frame.y - header.y;
    let bottom_gap = header.y + header.height - (commands[1].frame.y + commands[1].frame.height);
    assert!((top_gap - bottom_gap).abs() <= 1.0);
}
