use super::super::{
    build_chrome_command_stream, ChromeCommandKind, ChromeCommandLayer, ChromeCommandStream,
    ChromeImagePayload, ChromeImageUvRect,
};
use super::support::{
    presentation_with_componentized_workbench_frame_owner, presentation_with_viewport_image,
    stream_has_quad_color, LEGACY_CENTER_BAND, LEGACY_DOCUMENT_PANEL, LEGACY_VIEWPORT_PANEL,
};
use crate::ui::retained_host::host_contract::data::FrameRect;

#[test]
fn full_command_stream_records_full_ui_draw_list() {
    let stream =
        build_chrome_command_stream(&presentation_with_viewport_image(), (200, 200), None, true);

    let stats = stream.stats();
    assert!(stream.is_full_rebuild());
    assert!(stats.static_command_count > 0);
    assert!(stats.text_command_count > 0);
    assert!(stats.draw_call_count > 0);
    assert_eq!(stats.image_upload_bytes, 16);
    assert!(stream
        .commands()
        .iter()
        .any(|command| matches!(&command.kind, ChromeCommandKind::Image { .. })));
    assert!(stream.commands().iter().any(|command| {
        matches!(
            &command.kind,
            ChromeCommandKind::Text { text, .. } if text == "Create"
        )
    }));
}

#[test]
fn componentized_workbench_command_stream_skips_legacy_root_skeleton_quads() {
    let stream = build_chrome_command_stream(
        &presentation_with_componentized_workbench_frame_owner(),
        (200, 200),
        None,
        true,
    );

    assert!(!stream_has_quad_color(&stream, LEGACY_CENTER_BAND));
    assert!(!stream_has_quad_color(&stream, LEGACY_DOCUMENT_PANEL));
    assert!(!stream_has_quad_color(&stream, LEGACY_VIEWPORT_PANEL));
}

#[test]
fn patch_command_stream_does_not_rebuild_static_layer() {
    let damage = FrameRect {
        x: 42.0,
        y: 94.0,
        width: 10.0,
        height: 8.0,
    };

    let stream = build_chrome_command_stream(
        &presentation_with_viewport_image(),
        (200, 200),
        Some(&damage),
        false,
    );

    let stats = stream.stats();
    assert!(!stream.is_full_rebuild());
    assert_eq!(stats.static_command_count, 0);
    assert!(stats.dynamic_command_count > 0);
    assert!(stream
        .commands()
        .iter()
        .all(|command| { !matches!(command.layer, ChromeCommandLayer::Static) }));
}

#[test]
fn viewport_image_patch_can_carry_upload_bytes_for_gpu() {
    let damage = FrameRect {
        x: 42.0,
        y: 94.0,
        width: 10.0,
        height: 8.0,
    };

    let stream = build_chrome_command_stream(
        &presentation_with_viewport_image(),
        (200, 200),
        Some(&damage),
        true,
    );

    let image = stream
        .commands()
        .iter()
        .find_map(|command| match &command.kind {
            ChromeCommandKind::Image { payload } => Some(payload),
            _ => None,
        })
        .expect("viewport damage should keep the viewport image command");
    assert_eq!(image.resource_key, "viewport:test-initial");
    assert_eq!(image.upload_bytes, 16);
    assert!(image.rgba.is_none());
    assert_eq!(
        stream
            .image_resource("viewport:test-initial", 0)
            .map(|resource| resource.rgba.as_slice()),
        Some(&[255; 16][..])
    );
    assert_eq!(image.atlas_uv, None);
}

#[test]
fn command_stream_preserves_atlas_uv_on_image_payload() {
    let mut stream = ChromeCommandStream::full_rebuild((64, 64));

    stream.push_image(
        1,
        FrameRect {
            x: 4.0,
            y: 6.0,
            width: 20.0,
            height: 12.0,
        },
        None,
        ChromeImagePayload {
            resource_key: "atlas://editor/icons".to_string(),
            resource_generation: 0,
            width: 64,
            height: 64,
            upload_bytes: 0,
            rgba: None,
            atlas_uv: Some(ChromeImageUvRect {
                min: [0.5, 0.25],
                max: [0.75, 0.5],
            }),
        },
    );

    let ChromeCommandKind::Image { payload } = &stream.commands()[0].kind else {
        panic!("expected image command");
    };
    assert_eq!(payload.resource_key, "atlas://editor/icons");
    assert_eq!(
        payload.atlas_uv,
        Some(ChromeImageUvRect {
            min: [0.5, 0.25],
            max: [0.75, 0.5],
        })
    );
}
