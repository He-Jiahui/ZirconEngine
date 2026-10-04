use std::sync::Arc;

use super::*;
use crate::ui::retained_host::host_contract::chrome_command_stream::{
    ChromeCommandLayer, ChromeImagePayload,
};
use crate::ui::retained_host::host_contract::data::FrameRect;

#[test]
fn distinct_icons_share_one_stable_atlas_resource_and_keep_distinct_uvs() {
    let mut atlas = EditorIconAtlas::default();
    let mut first_frame = vec![icon("icon:save", 11), icon("template-icon:close", 22)];

    atlas.pack(&mut first_frame);

    let first = payload(&first_frame[0]);
    let second = payload(&first_frame[1]);
    assert_eq!(first.resource_key, second.resource_key);
    assert_eq!(first.resource_generation, second.resource_generation);
    assert_ne!(first.atlas_uv, second.atlas_uv);
    assert!(Arc::ptr_eq(
        first.rgba.as_ref().expect("atlas pixels"),
        second.rgba.as_ref().expect("shared atlas pixels")
    ));

    let generation = first.resource_generation;
    let mut second_frame = vec![icon("icon:save", 11), icon("template-icon:close", 22)];
    atlas.pack(&mut second_frame);
    assert_eq!(payload(&second_frame[0]).resource_generation, generation);
    assert_eq!(payload(&second_frame[1]).resource_generation, generation);
}

#[test]
fn adding_an_icon_keeps_published_pages_immutable() {
    let mut atlas = EditorIconAtlas::default();
    let mut first_frame = vec![icon("icon:save", 11)];
    atlas.pack(&mut first_frame);
    let first_generation = payload(&first_frame[0]).resource_generation;

    let mut second_frame = vec![icon("icon:save", 11), icon("icon:close", 22)];
    atlas.pack(&mut second_frame);

    let existing = payload(&second_frame[0]);
    let added = payload(&second_frame[1]);
    assert_eq!(existing.resource_generation, first_generation);
    assert_ne!(added.resource_generation, first_generation);
    assert_ne!(added.resource_key, existing.resource_key);
}

#[test]
fn changed_icon_content_does_not_advance_an_unrelated_page_generation() {
    let mut atlas = EditorIconAtlas::default();
    let mut first_frame = vec![
        icon("icon-raster:retained-image:save-v1", 11),
        icon("icon-raster:retained-image:close-v1", 22),
    ];
    atlas.pack(&mut first_frame);
    let shared_generation = payload(&first_frame[0]).resource_generation;

    let mut changed_frame = vec![
        icon("icon-raster:retained-image:save-v2", 33),
        icon("icon-raster:retained-image:close-v1", 22),
    ];
    atlas.pack(&mut changed_frame);

    assert_ne!(
        payload(&changed_frame[0]).resource_generation,
        shared_generation
    );
    assert_eq!(
        payload(&changed_frame[1]).resource_generation,
        shared_generation
    );
}

#[test]
fn atlas_pages_are_lru_bounded_without_rekeying_surviving_pages() {
    let mut atlas = EditorIconAtlas::default();
    let mut first_generation = 0;
    for index in 0..=MAX_ICON_ATLAS_PAGES {
        let mut commands = vec![icon(&format!("icon:bounded-{index}"), index as u8)];
        atlas.pack(&mut commands);
        if index == 0 {
            first_generation = payload(&commands[0]).resource_generation;
        }
    }

    assert_eq!(atlas.pages.len(), MAX_ICON_ATLAS_PAGES);
    assert!(atlas.resident_bytes <= MAX_ICON_ATLAS_BYTES);
    assert!(!atlas.slots.contains_key("icon:bounded-0"));

    let mut latest = vec![icon(
        &format!("icon:bounded-{MAX_ICON_ATLAS_PAGES}"),
        MAX_ICON_ATLAS_PAGES as u8,
    )];
    atlas.pack(&mut latest);
    assert_ne!(payload(&latest[0]).resource_generation, first_generation);
}

#[test]
fn ordinary_images_remain_standalone_resources() {
    let mut atlas = EditorIconAtlas::default();
    let mut commands = vec![icon("image:preview", 11)];

    atlas.pack(&mut commands);

    let payload = payload(&commands[0]);
    assert_eq!(payload.resource_key, "image:preview");
    assert!(payload.atlas_uv.is_none());
}

fn icon(resource_key: &str, color: u8) -> ChromeCommand {
    ChromeCommand {
        layer: ChromeCommandLayer::Static,
        z_index: 0,
        frame: FrameRect {
            x: 0.0,
            y: 0.0,
            width: 2.0,
            height: 2.0,
        },
        clip: None,
        source: None,
        kind: ChromeCommandKind::Image {
            payload: ChromeImagePayload {
                resource_key: resource_key.to_string(),
                resource_generation: 0,
                width: 2,
                height: 2,
                upload_bytes: 16,
                rgba: Some(vec![color; 16].into()),
                atlas_uv: None,
            },
        },
    }
}

fn payload(command: &ChromeCommand) -> &ChromeImagePayload {
    let ChromeCommandKind::Image { payload } = &command.kind else {
        panic!("expected image command");
    };
    payload
}
