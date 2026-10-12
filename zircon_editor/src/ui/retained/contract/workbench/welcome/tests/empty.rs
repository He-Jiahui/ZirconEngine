use super::super::super::super::super::paint_frame::HostRecordedPaintKind;
use super::*;

#[test]
fn recent_empty_state_uses_shared_typography_and_centers_content_in_list() {
    let mut frame = HostRgbaFrame::recording_only(320, 180);
    let list = FrameRect {
        x: 12.0,
        y: 8.0,
        width: 296.0,
        height: 164.0,
    };
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 320.0,
        height: 180.0,
    };

    draw_recent_projects_empty_state(&mut frame, &list, &clip);

    let commands = frame.into_recorded_commands();
    assert_eq!(commands.len(), 2);
    let metrics = current_host_metrics();
    for (command, expected_font_size) in
        commands.iter().zip([metrics.font_body, metrics.font_small])
    {
        let command_center_x = command.frame.x + command.frame.width * 0.5;
        assert!((command_center_x - (list.x + list.width * 0.5)).abs() <= 1.0);
        match &command.kind {
            HostRecordedPaintKind::Text { font_size, .. } => {
                assert_eq!(*font_size, expected_font_size);
            }
            kind => panic!("recent empty state should record text only, got {kind:?}"),
        }
    }
    let content_top = commands[0].frame.y;
    let content_bottom = commands[1].frame.y + commands[1].frame.height;
    assert!((content_top - list.y) > metrics.gap_l);
    assert!((list.y + list.height - content_bottom) > metrics.gap_l);
    assert!(((content_top - list.y) - (list.y + list.height - content_bottom)).abs() <= 1.0);
}
