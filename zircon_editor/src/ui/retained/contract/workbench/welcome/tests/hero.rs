use super::super::super::super::super::paint_frame::HostRecordedPaintKind;
use super::*;

fn test_clip() -> FrameRect {
    FrameRect {
        x: 0.0,
        y: 0.0,
        width: 360.0,
        height: 160.0,
    }
}

#[test]
fn welcome_hero_uses_runtime_text_metrics_and_measured_accent_width() {
    let mut frame = HostRgbaFrame::recording_only(360, 160);
    let mut pane = PaneData::default();
    pane.welcome.title = "Create a project".into();
    pane.welcome.subtitle = "Start from a renderable template".into();
    let hero = FrameRect {
        x: 16.0,
        y: 12.0,
        width: 328.0,
        height: 84.0,
    };

    draw_welcome_hero(&mut frame, &pane, &hero, &test_clip());

    let commands = frame.into_recorded_commands();
    assert_eq!(commands.len(), 4);
    let metrics = current_host_metrics();
    assert!(matches!(
        &commands[0].kind,
        HostRecordedPaintKind::Text { text, font_size, .. }
            if text == "Create a project" && *font_size == metrics.font_large
    ));
    assert!(matches!(
        &commands[1].kind,
        HostRecordedPaintKind::Text { text, font_size, .. }
            if text == "Start from a renderable template" && *font_size == metrics.font_body
    ));
    assert_eq!(commands[0].frame.x, hero.x);
    assert_eq!(commands[0].frame.width, hero.width);
    assert_eq!(commands[3].frame.height, metrics.selection_indicator_width);
    assert!(commands[3].frame.width < hero.width);
    assert!(commands[3].frame.width > metrics.gap_l);
}

#[test]
fn welcome_status_paints_project_admission_failure_in_visible_status_region() {
    let reason = "project admission requires the BuildSet App authenticated during startup";
    let mut pane = PaneData::default();
    pane.welcome.status_message = reason.into();
    let mut frame = HostRgbaFrame::recording_only(680, 120);
    let status = FrameRect {
        x: 16.0,
        y: 8.0,
        width: 648.0,
        height: 64.0,
    };
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 680.0,
        height: 120.0,
    };
    draw_welcome_status(&mut frame, &pane, &status, &clip);
    let commands = frame.into_recorded_commands();
    assert!(commands.iter().any(|command| matches!(
        &command.kind,
        HostRecordedPaintKind::Text { text, .. } if text.contains("project admission")
    )));
}

#[test]
fn welcome_status_uses_shared_radius_border_and_top_aligned_wrapped_runtime_text() {
    let mut frame = HostRgbaFrame::recording_only(360, 64);
    let status = FrameRect {
        x: 16.0,
        y: 12.0,
        width: 328.0,
        height: 30.0,
    };

    draw_welcome_status(&mut frame, &PaneData::default(), &status, &test_clip());

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
    assert!(matches!(
        &commands[3].kind,
        HostRecordedPaintKind::Text { text, font_size, .. }
            if text == "Ready" && *font_size == metrics.font_body
    ));
    assert_eq!(commands[3].frame.y, status.y);
}
