use super::super::image_resources::ChromeImageResource;
use super::ChromeCommandStream;
use crate::ui::retained_host::host_contract::chrome_command_stream::{
    ChromeCommand, ChromeCommandKind, ChromeCommandLayer, ChromeImagePayload,
};
use crate::ui::retained_host::host_contract::data::FrameRect;

#[test]
fn repeated_compaction_keeps_the_canonical_image_allocation() {
    let mut stream = ChromeCommandStream::full_rebuild((2, 2));
    stream.push_command_for_test(ChromeCommand {
        layer: ChromeCommandLayer::Static,
        z_index: 0,
        frame: FrameRect::default(),
        clip: None,
        source: None,
        kind: ChromeCommandKind::Image {
            payload: ChromeImagePayload {
                resource_key: "image://stable".to_string(),
                resource_generation: 9,
                width: 2,
                height: 2,
                upload_bytes: 16,
                rgba: Some(vec![9; 16].into()),
                atlas_uv: None,
            },
        },
    });

    stream.compact_image_resources();
    let pixels_ptr = stream
        .image_resource("image://stable", 9)
        .expect("first compaction stores the resource")
        .rgba
        .as_ptr();

    stream.compact_image_resources();

    assert_eq!(
        stream
            .image_resource("image://stable", 9)
            .expect("second compaction preserves the resource")
            .rgba
            .as_ptr(),
        pixels_ptr
    );
}

#[test]
fn resident_images_keep_their_commands_but_drop_staged_source_bytes() {
    let mut stream = ChromeCommandStream::full_rebuild((64, 64));
    stream.image_resources.insert(
        "atlas://editor/icons".to_string(),
        ChromeImageResource {
            generation: 7,
            width: 2,
            height: 2,
            upload_bytes: 16,
            rgba: vec![7; 16].into(),
        },
    );
    stream.image_resources.insert(
        "atlas://editor/changed".to_string(),
        ChromeImageResource {
            generation: 8,
            width: 2,
            height: 2,
            upload_bytes: 16,
            rgba: vec![8; 16].into(),
        },
    );

    stream.retain_unresident_image_resources(|resource_key, generation| {
        resource_key == "atlas://editor/icons" && generation == 7
    });

    assert!(stream
        .image_resources()
        .get("atlas://editor/icons", 7)
        .is_none());
    assert_eq!(
        stream
            .image_resources()
            .get("atlas://editor/changed", 8)
            .map(|resource| resource.rgba.as_ref()),
        Some(&[8; 16][..])
    );
}

#[test]
fn command_append_reopens_image_resource_compaction() {
    let image = |generation| ChromeCommand {
        layer: ChromeCommandLayer::Static,
        z_index: generation as i32,
        frame: FrameRect {
            width: 2.0,
            height: 2.0,
            ..FrameRect::default()
        },
        clip: None,
        source: None,
        kind: ChromeCommandKind::Image {
            payload: ChromeImagePayload {
                resource_key: "image://compaction-state".to_string(),
                resource_generation: generation,
                width: 2,
                height: 2,
                upload_bytes: 16,
                rgba: Some(vec![generation as u8; 16].into()),
                atlas_uv: None,
            },
        },
    };
    let mut stream = ChromeCommandStream::full_rebuild((64, 64));
    stream.push_command_for_test(image(1));

    assert!(!stream.image_resources_compacted);
    stream.compact_image_resources();
    assert!(stream.image_resources_compacted);
    assert!(stream
        .image_resource("image://compaction-state", 1)
        .is_some());

    stream.push_command_for_test(image(2));
    assert!(!stream.image_resources_compacted);
    stream.compact_image_resources();
    assert!(stream.image_resources_compacted);
    assert!(stream
        .image_resource("image://compaction-state", 2)
        .is_some());
}
