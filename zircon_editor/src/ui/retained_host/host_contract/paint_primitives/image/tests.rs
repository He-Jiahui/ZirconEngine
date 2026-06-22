use super::super::super::paint_frame::HostRecordedPaintKind;
use super::*;
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_frame::HostRgbaFrame;

#[test]
fn draw_rgba_image_clipped_copies_opaque_identity_rows_inside_clip() {
    let mut frame = HostRgbaFrame::filled(3, 2, [0, 0, 0, 255]);
    let rgba = vec![
        1, 2, 3, 255, 4, 5, 6, 255, 7, 8, 9, 255, 10, 11, 12, 255, 13, 14, 15, 255, 16, 17, 18, 255,
    ];

    let drew = draw_rgba_image_clipped(
        &mut frame,
        FrameRect {
            x: 0.0,
            y: 0.0,
            width: 3.0,
            height: 2.0,
        },
        Some(&FrameRect {
            x: 1.0,
            y: 0.0,
            width: 2.0,
            height: 2.0,
        }),
        3,
        2,
        &rgba,
    );

    assert!(drew);
    assert_eq!(&frame.as_bytes()[0..4], &[0, 0, 0, 255]);
    assert_eq!(&frame.as_bytes()[4..8], &[4, 5, 6, 255]);
    assert_eq!(&frame.as_bytes()[8..12], &[7, 8, 9, 255]);
    assert_eq!(&frame.as_bytes()[12..16], &[0, 0, 0, 255]);
    assert_eq!(&frame.as_bytes()[16..20], &[13, 14, 15, 255]);
    assert_eq!(&frame.as_bytes()[20..24], &[16, 17, 18, 255]);
}

#[test]
fn draw_rgba_image_clipped_blends_translucent_scaled_pixels() {
    let mut frame = HostRgbaFrame::filled(2, 1, [10, 20, 30, 255]);
    let rgba = vec![110, 120, 130, 128];

    let drew = draw_rgba_image_clipped(
        &mut frame,
        FrameRect {
            x: 0.0,
            y: 0.0,
            width: 2.0,
            height: 1.0,
        },
        None,
        1,
        1,
        &rgba,
    );

    assert!(drew);
    assert_eq!(&frame.as_bytes()[0..4], &[60, 70, 80, 255]);
    assert_eq!(&frame.as_bytes()[4..8], &[60, 70, 80, 255]);
}

#[test]
fn draw_rgba_image_clipped_records_content_scoped_resource_keys() {
    let mut frame = HostRgbaFrame::recording_only(2, 1);
    let red = vec![255, 0, 0, 255];
    let blue = vec![0, 0, 255, 255];

    assert!(draw_rgba_image_clipped(
        &mut frame,
        FrameRect {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        },
        None,
        1,
        1,
        &red,
    ));
    assert!(draw_rgba_image_clipped(
        &mut frame,
        FrameRect {
            x: 1.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        },
        None,
        1,
        1,
        &blue,
    ));

    let resource_keys = frame
        .into_recorded_commands()
        .into_iter()
        .filter_map(|command| match command.kind {
            HostRecordedPaintKind::Image { resource_key, .. } => Some(resource_key),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(resource_keys.len(), 2);
    assert_ne!(resource_keys[0], resource_keys[1]);
    assert!(resource_keys
        .iter()
        .all(|key| key.as_str().starts_with("rgba:1x1:")));
}

#[test]
fn draw_rgba_image_clipped_skips_disjoint_active_and_explicit_clips() {
    let mut frame = HostRgbaFrame::filled(2, 2, [0, 0, 0, 255]);
    frame.replace_paint_clip(Some(FrameRect {
        x: 0.0,
        y: 0.0,
        width: 1.0,
        height: 1.0,
    }));
    let rgba = vec![10, 20, 30, 255];

    let drew = draw_rgba_image_clipped(
        &mut frame,
        FrameRect {
            x: 1.0,
            y: 1.0,
            width: 1.0,
            height: 1.0,
        },
        Some(&FrameRect {
            x: 1.0,
            y: 1.0,
            width: 1.0,
            height: 1.0,
        }),
        1,
        1,
        &rgba,
    );

    assert!(!drew);
    assert!(frame
        .as_bytes()
        .chunks_exact(4)
        .all(|pixel| pixel == [0, 0, 0, 255]));
}
