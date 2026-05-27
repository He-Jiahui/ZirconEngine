use super::*;
use crate::ui::retained_host::host_contract::painter::{HostPaintAtlasImage, HostPaintImageUvRect};

#[test]
fn recorded_atlas_images_keep_shared_resource_key_and_distinct_uvs() {
    let mut stream = ChromeCommandStream::full_rebuild((64, 64));

    stream.push_recorded_command(
        recorded_atlas_image(
            0,
            0.0,
            atlas_image("lib://editor-sprite-atlases/a.png", [0.0, 0.0], [0.25, 0.5]),
        ),
        true,
    );
    stream.push_recorded_command(
        recorded_atlas_image(
            1,
            16.0,
            atlas_image("lib://editor-sprite-atlases/a.png", [0.25, 0.0], [0.5, 0.5]),
        ),
        true,
    );
    stream.push_recorded_command(
        recorded_atlas_image(
            2,
            32.0,
            atlas_image("lib://editor-sprite-atlases/b.png", [0.0, 0.5], [0.25, 1.0]),
        ),
        true,
    );

    let images = stream
        .commands()
        .iter()
        .filter_map(|command| match &command.kind {
            ChromeCommandKind::Image { payload } => Some(payload),
            _ => None,
        })
        .collect::<Vec<_>>();
    let resource_keys = images
        .iter()
        .map(|payload| payload.resource_key.as_str())
        .collect::<Vec<_>>();
    let atlas_uvs = images
        .iter()
        .filter_map(|payload| payload.atlas_uv)
        .collect::<Vec<_>>();

    assert_eq!(resource_keys.len(), 3);
    assert_eq!(
        resource_keys,
        vec![
            "lib://editor-sprite-atlases/a.png",
            "lib://editor-sprite-atlases/a.png",
            "lib://editor-sprite-atlases/b.png",
        ]
    );
    assert_eq!(
        resource_keys
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        2
    );
    assert_eq!(
        atlas_uvs,
        vec![
            ChromeImageUvRect {
                min: [0.0, 0.0],
                max: [0.25, 0.5],
            },
            ChromeImageUvRect {
                min: [0.25, 0.0],
                max: [0.5, 0.5],
            },
            ChromeImageUvRect {
                min: [0.0, 0.5],
                max: [0.25, 1.0],
            },
        ]
    );
    assert!(images.iter().all(|payload| payload.width == 4));
    assert!(images.iter().all(|payload| payload.height == 4));
    assert!(images
        .iter()
        .all(|payload| payload.rgba.as_ref().is_some_and(|rgba| rgba.len() == 64)));
}

#[test]
fn recorded_atlas_image_uses_atlas_texture_payload_not_source_payload() {
    let mut stream = ChromeCommandStream::full_rebuild((64, 64));
    stream.push_recorded_command(
        recorded_atlas_image(
            0,
            0.0,
            atlas_image("lib://editor-sprite-atlases/a.png", [0.25, 0.0], [0.5, 0.5]),
        ),
        true,
    );

    let payload = stream
        .commands()
        .iter()
        .find_map(|command| match &command.kind {
            ChromeCommandKind::Image { payload } => Some(payload),
            _ => None,
        })
        .expect("atlas image should record an image payload");

    assert_eq!(payload.resource_key, "lib://editor-sprite-atlases/a.png");
    assert_eq!(payload.width, 4);
    assert_eq!(payload.height, 4);
    assert_eq!(payload.upload_bytes, 64);
    assert_eq!(payload.rgba.as_ref().map(Vec::len), Some(64));
    assert_eq!(
        payload.atlas_uv,
        Some(ChromeImageUvRect {
            min: [0.25, 0.0],
            max: [0.5, 0.5],
        })
    );
}

#[test]
fn command_stream_replay_samples_atlas_uv_from_embedded_atlas_bytes() {
    let mut stream = ChromeCommandStream::full_rebuild((1, 1));
    stream.push_image(
        0,
        FrameRect {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        },
        None,
        ChromeImagePayload {
            resource_key: "lib://editor-sprite-atlases/a.png".to_string(),
            width: 2,
            height: 1,
            upload_bytes: 8,
            rgba: Some(vec![255, 0, 0, 255, 0, 0, 255, 255]),
            atlas_uv: Some(ChromeImageUvRect {
                min: [0.5, 0.0],
                max: [1.0, 1.0],
            }),
        },
    );

    let frame = paint_chrome_command_stream_to_frame(1, 1, &stream);

    assert_eq!(frame.as_bytes(), &[0, 0, 255, 255]);
}

fn recorded_atlas_image(
    z_index: i32,
    x: f32,
    atlas: HostPaintAtlasImage,
) -> HostRecordedPaintCommand {
    HostRecordedPaintCommand {
        frame: FrameRect {
            x,
            y: 0.0,
            width: 8.0,
            height: 8.0,
        },
        clip_frame: None,
        z_index,
        kind: HostRecordedPaintKind::Image {
            resource_key: format!("source:{z_index}"),
            width: 1,
            height: 1,
            rgba: Some(vec![z_index as u8, 0, 0, 255]),
            atlas: Some(atlas),
        },
    }
}

fn atlas_image(resource_key: &str, min: [f32; 2], max: [f32; 2]) -> HostPaintAtlasImage {
    HostPaintAtlasImage {
        resource_key: resource_key.to_string(),
        width: 4,
        height: 4,
        rgba: Some(vec![255; 64]),
        uv: HostPaintImageUvRect { min, max },
    }
}
