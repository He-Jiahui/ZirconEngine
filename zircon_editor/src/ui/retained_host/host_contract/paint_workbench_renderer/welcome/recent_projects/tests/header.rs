use super::super::super::super::super::paint_frame::HostRecordedPaintKind;
use super::*;

#[test]
fn recent_header_uses_shared_typography_and_centers_single_line_content() {
    let mut frame = HostRgbaFrame::recording_only(320, 64);
    let header = FrameRect {
        x: 8.0,
        y: 4.0,
        width: 304.0,
        height: RECENT_HEADER_HEIGHT,
    };
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 320.0,
        height: 64.0,
    };

    draw_recent_projects_header(&mut frame, &header, &clip);

    let commands = frame.into_recorded_commands();
    assert_eq!(commands.len(), 1);
    let metrics = current_host_metrics();
    for command in &commands {
        assert_eq!(command.frame.x, header.x + metrics.gap_l);
        assert_eq!(command.frame.width, header.width - metrics.gap_l * 2.0);
        match &command.kind {
            HostRecordedPaintKind::Text { font_size, .. } => {
                assert_eq!(*font_size, metrics.font_body);
            }
            kind => panic!("recent header should record text only, got {kind:?}"),
        }
    }
    assert!(commands[0].frame.y >= header.y);
    assert!(commands[0].frame.y + commands[0].frame.height <= header.y + header.height);
}
