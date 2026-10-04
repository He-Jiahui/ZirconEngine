use super::super::super::super::super::super::paint_frame::HostRecordedPaintKind;
use super::*;

#[test]
fn recent_project_title_reserves_measured_status_slot_and_uses_shared_text_metrics() {
    let mut frame = HostRgbaFrame::recording_only(320, 64);
    let row = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 300.0,
        height: 54.0,
    };
    let text = FrameRect {
        x: 8.0,
        y: 0.0,
        width: 220.0,
        height: 54.0,
    };
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 320.0,
        height: 64.0,
    };

    draw_recent_project_row_text(
        &mut frame,
        &row,
        &text,
        &clip,
        "A very long project name that must not overlap status",
        "E:/Projects/Zircon",
        "Missing",
        true,
    );

    let commands = frame
        .into_recorded_commands()
        .into_iter()
        .filter(|command| matches!(&command.kind, HostRecordedPaintKind::Text { .. }))
        .collect::<Vec<_>>();
    assert_eq!(commands.len(), 3);
    let metrics = current_host_metrics();
    let expected_line_height = metrics
        .line_height(metrics.font_body)
        .round()
        .max(metrics.font_body.ceil());
    for command in &commands {
        match &command.kind {
            HostRecordedPaintKind::Text {
                font_size,
                line_height,
                ..
            } => {
                assert_eq!(*font_size, metrics.font_body);
                assert_eq!(*line_height, expected_line_height);
            }
            _ => unreachable!("text filter only retains text commands"),
        }
    }
    assert!(commands[0].frame.x + commands[0].frame.width <= commands[2].frame.x);
    assert!(commands[1].frame.width > commands[0].frame.width);
    assert!(matches!(
        &commands[2].kind,
        HostRecordedPaintKind::Text { text, color, .. }
            if text == "Missing" && *color == WELCOME_WARNING
    ));
}
