use super::super::data::FrameRect;
use super::super::paint_frame::{HostRecordedPaintKind, HostRgbaFrame};
use super::{draw_border_clipped, draw_rect_clipped, draw_rounded_border_clipped};

#[test]
fn draw_rect_clipped_fills_only_clipped_span() {
    let mut frame = HostRgbaFrame::filled(4, 3, [0, 0, 0, 255]);
    draw_rect_clipped(
        &mut frame,
        FrameRect {
            x: 0.0,
            y: 0.0,
            width: 4.0,
            height: 3.0,
        },
        Some(&FrameRect {
            x: 1.0,
            y: 1.0,
            width: 2.0,
            height: 1.0,
        }),
        [10, 20, 30, 255],
    );

    assert_eq!(&frame.as_bytes()[0..4], &[0, 0, 0, 255]);
    assert_eq!(&frame.as_bytes()[20..24], &[10, 20, 30, 255]);
    assert_eq!(&frame.as_bytes()[24..28], &[10, 20, 30, 255]);
    assert_eq!(&frame.as_bytes()[28..32], &[0, 0, 0, 255]);
}

#[test]
fn draw_rect_clipped_blends_alpha_over_existing_pixels() {
    let mut frame = HostRgbaFrame::filled(2, 1, [10, 20, 30, 255]);
    draw_rect_clipped(
        &mut frame,
        FrameRect {
            x: 0.0,
            y: 0.0,
            width: 2.0,
            height: 1.0,
        },
        None,
        [110, 120, 130, 128],
    );

    assert_eq!(&frame.as_bytes()[0..4], &[60, 70, 80, 255]);
    assert_eq!(&frame.as_bytes()[4..8], &[60, 70, 80, 255]);
}

#[test]
fn draw_rect_clipped_skips_disjoint_active_and_explicit_clips() {
    let mut frame = HostRgbaFrame::filled(4, 4, [0, 0, 0, 255]);
    frame.replace_paint_clip(Some(FrameRect {
        x: 0.0,
        y: 0.0,
        width: 2.0,
        height: 2.0,
    }));

    draw_rect_clipped(
        &mut frame,
        FrameRect {
            x: 2.0,
            y: 2.0,
            width: 2.0,
            height: 2.0,
        },
        Some(&FrameRect {
            x: 2.0,
            y: 2.0,
            width: 2.0,
            height: 2.0,
        }),
        [10, 20, 30, 255],
    );

    assert!(frame
        .as_bytes()
        .chunks_exact(4)
        .all(|pixel| pixel == [0, 0, 0, 255]));
}

#[test]
fn recording_only_square_borders_emit_one_border_command() {
    let mut frame = HostRgbaFrame::recording_only(64, 64);
    let rect = FrameRect {
        x: 4.0,
        y: 6.0,
        width: 24.0,
        height: 18.0,
    };

    draw_border_clipped(&mut frame, rect, None, [10, 20, 30, 255]);

    let commands = frame.into_recorded_commands();
    assert_eq!(commands.len(), 1);
    assert!(matches!(
        commands[0].kind,
        HostRecordedPaintKind::Border {
            width: 1.0,
            corner_radius: 0.0,
            ..
        }
    ));
}

#[test]
fn recording_only_wide_square_borders_emit_one_border_command() {
    let mut frame = HostRgbaFrame::recording_only(64, 64);
    let rect = FrameRect {
        x: 4.0,
        y: 6.0,
        width: 24.0,
        height: 18.0,
    };

    draw_rounded_border_clipped(&mut frame, rect, None, [10, 20, 30, 255], 3.0, 0.0);

    let commands = frame.into_recorded_commands();
    assert_eq!(commands.len(), 1);
    assert!(matches!(
        commands[0].kind,
        HostRecordedPaintKind::Border {
            width: 3.0,
            corner_radius: 0.0,
            ..
        }
    ));
}
