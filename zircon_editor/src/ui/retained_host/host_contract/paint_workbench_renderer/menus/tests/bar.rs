use super::*;
use crate::ui::retained_host::host_contract::paint_frame::HostRecordedPaintKind;
use crate::ui::retained_host::host_contract::paint_theme::METRICS;

#[test]
fn menu_bar_label_frame_is_finite_and_centered() {
    let menu_frame = FrameRect {
        x: 24.0,
        y: 8.0,
        width: 96.0,
        height: 28.0,
    };

    let label = menu_bar_label_frame(&menu_frame, 6.0, 16.0);

    assert_eq!(label.x, 30.0);
    assert_eq!(label.y, 14.0);
    assert_eq!(label.width, 84.0);
    assert_eq!(label.height, 16.0);
}

#[test]
fn menu_bar_control_uses_quiet_resting_material_and_surface_open_feedback() {
    let mut palette = crate::ui::retained_host::host_contract::paint_theme::PALETTE;
    palette.surface_hover = [32, 48, 60, 255];
    palette.text = [230, 232, 235, 255];
    palette.text_muted = [140, 148, 155, 255];

    let resting = menu_bar_control_visual(false, palette);
    let open = menu_bar_control_visual(true, palette);

    assert_eq!(resting.background, None);
    assert_eq!(resting.text_color, palette.text_muted);
    assert_eq!(open.background, Some(palette.surface_hover));
    assert_eq!(open.text_color, palette.text);
}

#[test]
fn menu_bar_label_uses_a_finite_runtime_text_slot_with_ellipsis() {
    let menu_frame = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 72.0,
        height: 28.0,
    };
    let mut frame = HostRgbaFrame::recording_only(80, 32);

    draw_menu_bar_label(
        &mut frame,
        "A long workbench menu label that must ellipsize",
        &menu_frame,
        Some(&menu_frame),
        [230, 230, 230, 255],
        METRICS,
    );

    let command = frame
        .into_recorded_commands()
        .into_iter()
        .find(|command| matches!(&command.kind, HostRecordedPaintKind::Text { .. }))
        .expect("menu bar label should use Runtime Text");
    let HostRecordedPaintKind::Text { text, .. } = &command.kind else {
        unreachable!("filtered command should be text");
    };

    assert!(text.ends_with('\u{2026}'));
    assert!(command.frame.x >= menu_frame.x);
    assert!(command.frame.x + command.frame.width <= menu_frame.x + menu_frame.width);
}
