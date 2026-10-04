use super::super::super::super::super::super::paint_frame::HostRecordedPaintKind;
use super::*;

#[test]
fn recent_project_rows_keep_idle_surfaces_flat_and_reserve_outline_for_warning_state() {
    let row = FrameRect {
        x: 8.0,
        y: 8.0,
        width: 104.0,
        height: 32.0,
    };
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 128.0,
        height: 48.0,
    };

    let mut idle_frame = HostRgbaFrame::recording_only(128, 48);
    draw_recent_project_row_surface(&mut idle_frame, &row, &clip, false);
    let idle_commands = idle_frame.into_recorded_commands();
    assert_eq!(idle_commands.len(), 2);
    assert!(idle_commands
        .iter()
        .all(|command| matches!(&command.kind, HostRecordedPaintKind::Quad { .. })));
    match &idle_commands[1].kind {
        HostRecordedPaintKind::Quad {
            color,
            corner_radius,
        } => {
            assert_eq!(*color, SEPARATOR);
            assert_eq!(*corner_radius, 0.0);
        }
        ref kind => panic!("idle row separator should be a flat quad, got {kind:?}"),
    }
    let metrics = current_host_metrics();
    assert_eq!(idle_commands[1].frame.height, metrics.border_width);

    let mut warning_frame = HostRgbaFrame::recording_only(128, 48);
    draw_recent_project_row_surface(&mut warning_frame, &row, &clip, true);
    let warning_commands = warning_frame.into_recorded_commands();
    assert_eq!(warning_commands.len(), 3);
    match &warning_commands[2].kind {
        HostRecordedPaintKind::Border {
            color,
            width,
            corner_radius,
        } => {
            assert_eq!(*color, WELCOME_WARNING);
            assert_eq!(*width, metrics.border_width);
            assert_eq!(*corner_radius, metrics.radius_control);
        }
        ref kind => panic!("warning row should add one semantic outline, got {kind:?}"),
    }
}

#[test]
fn recent_project_actions_use_shared_control_radius_for_fill_and_border() {
    let mut frame = HostRgbaFrame::recording_only(160, 48);
    let open = FrameRect {
        x: 8.0,
        y: 8.0,
        width: 52.0,
        height: 24.0,
    };
    let recover = FrameRect {
        x: 100.0,
        y: 8.0,
        width: 24.0,
        height: 24.0,
    };
    let remove = FrameRect {
        x: 132.0,
        y: 8.0,
        width: 24.0,
        height: 24.0,
    };
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 160.0,
        height: 48.0,
    };

    let safe = FrameRect {
        x: 68.0,
        y: 8.0,
        width: 24.0,
        height: 24.0,
    };

    draw_recent_project_row_actions(&mut frame, &open, &safe, &recover, &remove, &clip, false);

    let metrics = current_host_metrics();
    let commands = frame.into_recorded_commands();
    let surface_commands = commands
        .iter()
        .filter(|command| {
            matches!(
                &command.kind,
                HostRecordedPaintKind::Quad { .. } | HostRecordedPaintKind::Border { .. }
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(surface_commands.len(), 8);
    for command in surface_commands {
        match &command.kind {
            HostRecordedPaintKind::Quad { corner_radius, .. } => {
                assert_eq!(*corner_radius, metrics.radius_control);
            }
            HostRecordedPaintKind::Border {
                width,
                corner_radius,
                ..
            } => {
                assert_eq!(*width, metrics.border_width);
                assert_eq!(*corner_radius, metrics.radius_control);
            }
            _ => unreachable!("surface filter only retains quads and borders"),
        }
    }
    let text_commands = commands
        .iter()
        .filter(|command| matches!(&command.kind, HostRecordedPaintKind::Text { .. }))
        .collect::<Vec<_>>();
    assert_eq!(text_commands.len(), 4);
    for (command, action) in text_commands.iter().zip([&open, &safe, &recover, &remove]) {
        match &command.kind {
            HostRecordedPaintKind::Text {
                font_size,
                line_height,
                ..
            } => {
                assert_eq!(*font_size, metrics.font_body);
                assert_eq!(*line_height, metrics.line_height(metrics.font_body).round());
            }
            _ => unreachable!("text filter only retains text commands"),
        }
        let label_center_x = command.frame.x + command.frame.width * 0.5;
        let label_center_y = command.frame.y + command.frame.height * 0.5;
        assert!((label_center_x - (action.x + action.width * 0.5)).abs() <= 1.0);
        assert!((label_center_y - (action.y + action.height * 0.5)).abs() <= 1.0);
    }
    assert!(matches!(
        &text_commands[1].kind,
        HostRecordedPaintKind::Text { text, .. } if text == "S"
    ));
    assert!(matches!(
        &text_commands[2].kind,
        HostRecordedPaintKind::Text { text, .. } if text == "R"
    ));
    assert!(matches!(
        &text_commands[3].kind,
        HostRecordedPaintKind::Text { text, .. } if text == "×"
    ));
}
