use super::super::data::FrameRect;
use super::super::paint_frame::{HostRecordedPaintKind, HostRgbaFrame};
use super::{
    draw_border_clipped, draw_rect_clipped, draw_rounded_border_clipped, draw_rounded_box_clipped,
    draw_rounded_rect_clipped,
};

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

    assert_eq!(&frame.as_bytes()[0..4], &[80, 88, 97, 255]);
    assert_eq!(&frame.as_bytes()[4..8], &[80, 88, 97, 255]);
}

#[test]
fn fractional_square_fill_resolves_partial_edge_coverage() {
    let mut frame = HostRgbaFrame::filled(3, 3, [0, 0, 0, 255]);
    draw_rect_clipped(
        &mut frame,
        FrameRect {
            x: 0.5,
            y: 0.5,
            width: 1.0,
            height: 1.0,
        },
        None,
        [255, 255, 255, 255],
    );

    for (x, y) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
        let offset = ((y * 3 + x) * 4) as usize;
        assert!((136..=138).contains(&frame.as_bytes()[offset]));
        assert_eq!(frame.as_bytes()[offset], frame.as_bytes()[offset + 1]);
        assert_eq!(frame.as_bytes()[offset + 1], frame.as_bytes()[offset + 2]);
        assert_eq!(frame.as_bytes()[offset + 3], 255);
    }
    assert_eq!(&frame.as_bytes()[32..36], &[0, 0, 0, 255]);
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
fn recording_only_square_fills_preserve_fractional_physical_frame() {
    let mut frame = HostRgbaFrame::recording_only(64, 64);
    let rect = FrameRect {
        x: 4.25,
        y: 6.5,
        width: 24.5,
        height: 18.75,
    };

    draw_rect_clipped(&mut frame, rect.clone(), None, [10, 20, 30, 255]);

    let commands = frame.into_recorded_commands();
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].frame, rect);
    assert!(matches!(
        commands[0].kind,
        HostRecordedPaintKind::Quad {
            corner_radius: 0.0,
            ..
        }
    ));
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

#[test]
fn recording_only_rounded_borders_preserve_fractional_physical_width() {
    let mut frame = HostRgbaFrame::recording_only(64, 64);
    let rect = FrameRect {
        x: 4.25,
        y: 6.5,
        width: 24.5,
        height: 18.75,
    };

    draw_rounded_border_clipped(&mut frame, rect, None, [10, 20, 30, 255], 1.25, 6.0);

    let commands = frame.into_recorded_commands();
    assert_eq!(commands.len(), 1);
    assert!(matches!(
        commands[0].kind,
        HostRecordedPaintKind::Border {
            width: 1.25,
            corner_radius: 6.0,
            ..
        }
    ));
}

#[test]
fn recording_only_rounded_borders_preserve_the_existing_width_limit() {
    let mut frame = HostRgbaFrame::recording_only(64, 64);
    let rect = FrameRect {
        x: 4.0,
        y: 6.0,
        width: 40.0,
        height: 32.0,
    };

    draw_rounded_border_clipped(&mut frame, rect, None, [10, 20, 30, 255], 12.5, 8.0);

    let commands = frame.into_recorded_commands();
    assert_eq!(commands.len(), 1);
    assert!(matches!(
        commands[0].kind,
        HostRecordedPaintKind::Border {
            width: 8.0,
            corner_radius: 8.0,
            ..
        }
    ));
}

#[test]
fn rounded_box_outer_alpha_matches_a_single_rounded_fill() {
    let rect = FrameRect {
        x: 1.0,
        y: 1.0,
        width: 8.0,
        height: 8.0,
    };
    let mut fill = HostRgbaFrame::filled(10, 10, [0, 0, 0, 0]);
    draw_rounded_rect_clipped(&mut fill, rect.clone(), None, [255; 4], 4.0);
    let mut combined = HostRgbaFrame::filled(10, 10, [0, 0, 0, 0]);
    draw_rounded_box_clipped(&mut combined, rect, None, [255; 4], [255; 4], 1.5, 4.0);

    let fill_alpha = fill
        .as_bytes()
        .chunks_exact(4)
        .map(|pixel| pixel[3])
        .collect::<Vec<_>>();
    let combined_alpha = combined
        .as_bytes()
        .chunks_exact(4)
        .map(|pixel| pixel[3])
        .collect::<Vec<_>>();
    assert_eq!(combined_alpha, fill_alpha);
}

#[test]
fn rounded_box_resolves_fill_and_border_without_an_alpha_gap() {
    let mut frame = HostRgbaFrame::filled(12, 12, [0, 0, 0, 0]);
    draw_rounded_box_clipped(
        &mut frame,
        FrameRect {
            x: 1.0,
            y: 1.0,
            width: 10.0,
            height: 10.0,
        },
        None,
        [24, 32, 44, 255],
        [96, 174, 255, 255],
        2.0,
        4.0,
    );
    let pixel = |x: usize, y: usize| {
        let offset = (y * 12 + x) * 4;
        &frame.as_bytes()[offset..offset + 4]
    };

    assert_eq!(pixel(6, 6), [24, 32, 44, 255]);
    assert_eq!(pixel(6, 1), [96, 174, 255, 255]);
    assert_eq!(pixel(6, 3)[3], 255);
}

#[test]
fn recording_only_rounded_box_preserves_the_two_source_commands() {
    let mut frame = HostRgbaFrame::recording_only(64, 64);
    draw_rounded_box_clipped(
        &mut frame,
        FrameRect {
            x: 4.0,
            y: 6.0,
            width: 40.0,
            height: 32.0,
        },
        None,
        [24, 32, 44, 255],
        [96, 174, 255, 255],
        1.25,
        8.0,
    );

    let commands = frame.into_recorded_commands();
    assert!(matches!(
        commands[0].kind,
        HostRecordedPaintKind::Quad { .. }
    ));
    assert!(matches!(
        commands[1].kind,
        HostRecordedPaintKind::Border { .. }
    ));
}
