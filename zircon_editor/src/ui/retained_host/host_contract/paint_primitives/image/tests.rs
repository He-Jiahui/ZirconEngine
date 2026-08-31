use super::super::super::paint_frame::{
    HostPaintAtlasImage, HostPaintImageUvRect, HostRecordedPaintKind,
};
use super::*;
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_frame::HostRgbaFrame;
use std::sync::Arc;

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
    assert_eq!(&frame.as_bytes()[0..4], &[80, 88, 97, 255]);
    assert_eq!(&frame.as_bytes()[4..8], &[80, 88, 97, 255]);
}

#[test]
fn scaled_rgba_image_uses_bilinear_center_sampling() {
    let mut frame = HostRgbaFrame::filled(3, 3, [0, 0, 0, 255]);
    let rgba = vec![
        255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
    ];

    assert!(draw_rgba_image_clipped(
        &mut frame,
        FrameRect {
            x: 0.0,
            y: 0.0,
            width: 3.0,
            height: 3.0,
        },
        None,
        2,
        2,
        &rgba,
    ));

    assert_eq!(rgba_pixel(&frame, 1, 1), [188, 188, 188, 255]);
}

#[test]
fn bilinear_sampling_does_not_bleed_rgb_from_transparent_texels() {
    let mut frame = HostRgbaFrame::filled(3, 1, [0, 0, 0, 255]);
    let rgba = vec![255, 0, 0, 0, 0, 0, 255, 255];

    assert!(draw_rgba_image_clipped(
        &mut frame,
        FrameRect {
            x: 0.0,
            y: 0.0,
            width: 3.0,
            height: 1.0,
        },
        None,
        2,
        1,
        &rgba,
    ));

    assert_eq!(rgba_pixel(&frame, 1, 0), [0, 0, 188, 255]);
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
fn shared_rgba_recording_reuses_the_cached_pixel_allocation() {
    let mut frame = HostRgbaFrame::recording_only(1, 1);
    let rgba: Arc<[u8]> = vec![255, 0, 0, 255].into();

    assert!(draw_shared_rgba_image_clipped_with_resource_key(
        &mut frame,
        FrameRect {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        },
        None,
        "icon:shared-test",
        1,
        1,
        &rgba,
    ));

    let command = frame.into_recorded_commands().pop().expect("image command");
    let HostRecordedPaintKind::Image {
        rgba: Some(recorded),
        ..
    } = command.kind
    else {
        panic!("expected recorded image pixels");
    };
    assert!(Arc::ptr_eq(&rgba, &recorded));
}

#[test]
fn draw_gpu_image_records_the_external_resource_without_cpu_pixels() {
    let mut frame = HostRgbaFrame::recording_only(2, 1);

    assert!(draw_gpu_image_clipped_with_resource_key(
        &mut frame,
        FrameRect {
            x: 0.0,
            y: 0.0,
            width: 2.0,
            height: 1.0,
        },
        None,
        "viewport:7:13",
        640,
        360,
    ));

    let command = frame.into_recorded_commands().pop().expect("image command");
    let HostRecordedPaintKind::Image {
        resource_key,
        width,
        height,
        rgba,
        atlas,
    } = command.kind
    else {
        panic!("expected recorded image");
    };
    assert_eq!(resource_key, "viewport:7:13");
    assert_eq!((width, height), (640, 360));
    assert!(rgba.is_none());
    assert!(atlas.is_none());
}

#[test]
fn atlas_recording_keeps_one_copy_of_atlas_pixels() {
    let mut frame = HostRgbaFrame::recording_only(1, 1);
    let source_rgba = vec![255, 0, 0, 255];
    let atlas = HostPaintAtlasImage {
        resource_key: "atlas://editor/icons".to_string(),
        resource_generation: 0,
        width: 2,
        height: 2,
        rgba: Some(vec![7; 16].into()),
        uv: HostPaintImageUvRect {
            min: [0.0, 0.0],
            max: [0.5, 0.5],
        },
    };

    assert!(draw_rgba_image_clipped_with_atlas(
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
        &source_rgba,
        &atlas,
    ));

    let command = frame.into_recorded_commands().pop().expect("image command");
    let HostRecordedPaintKind::Image { rgba, atlas, .. } = command.kind else {
        panic!("expected recorded image");
    };
    assert!(rgba.is_none(), "atlas bytes must not be duplicated");
    assert_eq!(atlas.and_then(|atlas| atlas.rgba), Some(vec![7; 16]));
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

fn rgba_pixel(frame: &HostRgbaFrame, x: u32, y: u32) -> [u8; 4] {
    let offset = ((y as usize * frame.width() as usize) + x as usize) * 4;
    frame.as_bytes()[offset..offset + 4]
        .try_into()
        .expect("RGBA pixel")
}
