use super::super::data::FrameRect;
use super::{HostRecordedPaintKind, HostRgbaFrame};

#[test]
fn fill_rect_replaces_contiguous_row_span() {
    let mut frame = HostRgbaFrame::filled(4, 2, [0, 0, 0, 255]);
    frame.fill_rect(
        &FrameRect {
            x: 1.0,
            y: 0.0,
            width: 2.0,
            height: 1.0,
        },
        [10, 20, 30, 255],
    );

    assert_eq!(&frame.as_bytes()[0..4], &[0, 0, 0, 255]);
    assert_eq!(&frame.as_bytes()[4..8], &[10, 20, 30, 255]);
    assert_eq!(&frame.as_bytes()[8..12], &[10, 20, 30, 255]);
    assert_eq!(&frame.as_bytes()[12..16], &[0, 0, 0, 255]);
}

#[test]
fn recording_only_collects_quad_without_allocating_pixels() {
    let mut frame = HostRgbaFrame::recording_only(16, 12);

    frame.fill_rect(
        &FrameRect {
            x: 1.0,
            y: 2.0,
            width: 3.0,
            height: 4.0,
        },
        [10, 20, 30, 255],
    );

    let commands = frame.into_recorded_commands();
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].z_index, 0);
    assert!(matches!(
        commands[0].kind,
        HostRecordedPaintKind::Quad {
            color: [10, 20, 30, 255],
            corner_radius: 0.0,
        }
    ));
}

#[test]
fn fill_rect_respects_active_paint_clip() {
    let mut frame = HostRgbaFrame::filled(4, 2, [0, 0, 0, 255]);
    frame.replace_paint_clip(Some(FrameRect {
        x: 1.0,
        y: 0.0,
        width: 2.0,
        height: 1.0,
    }));

    frame.fill_rect(
        &FrameRect {
            x: 0.0,
            y: 0.0,
            width: 4.0,
            height: 2.0,
        },
        [10, 20, 30, 255],
    );

    assert_eq!(&frame.as_bytes()[0..4], &[0, 0, 0, 255]);
    assert_eq!(&frame.as_bytes()[4..8], &[10, 20, 30, 255]);
    assert_eq!(&frame.as_bytes()[8..12], &[10, 20, 30, 255]);
    assert_eq!(&frame.as_bytes()[12..16], &[0, 0, 0, 255]);
    assert_eq!(&frame.as_bytes()[16..20], &[0, 0, 0, 255]);
}
