use std::sync::Arc;

use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiTreeId},
    layout::UiFrame,
    surface::{
        UiRenderCommand, UiRenderCommandKind, UiRenderExtract, UiRenderFrameCommandRef,
        UiRenderFrameExtract, UiRenderList, UiResolvedStyle, UiSurfaceFrame,
    },
};

use super::super::{
    build_chrome_command_stream, ChromeCommandKind, ChromeCommandLayer, ChromeCommandStream,
    ChromeImagePayload, ChromeImageUvRect,
};
use super::support::{
    presentation_with_componentized_workbench_frame_owner, presentation_with_viewport_image,
    stream_has_quad_color, LEGACY_CENTER_BAND, LEGACY_DOCUMENT_PANEL, LEGACY_VIEWPORT_PANEL,
};
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_frame::{
    HostRenderCommandSource, HostRenderSourceTable,
};

#[test]
fn extracted_command_vector_is_adopted_without_reallocation() {
    let damage = FrameRect {
        x: 2.0,
        y: 4.0,
        width: 8.0,
        height: 16.0,
    };
    let mut commands = Vec::with_capacity(1);
    commands.push(super::super::ChromeCommand {
        layer: ChromeCommandLayer::Dynamic,
        z_index: 0,
        frame: damage.clone(),
        clip: Some(damage.clone()),
        source: None,
        kind: ChromeCommandKind::Clip,
    });
    let commands_ptr = commands.as_ptr();

    let stream = ChromeCommandStream::from_extracted_commands(
        (64, 64),
        Some(damage),
        commands,
        Default::default(),
    );

    assert_eq!(stream.commands().as_ptr(), commands_ptr);
    assert!(matches!(stream.commands()[0].kind, ChromeCommandKind::Clip));
    assert!(!stream.is_full_rebuild());
}

#[test]
fn extracted_command_source_resolves_through_the_stream_surface_table() {
    let source_node_id = UiNodeId::new(31);
    let source_frame = Arc::new(UiSurfaceFrame {
        render_extract: Arc::new(UiRenderFrameExtract::from_extract(&UiRenderExtract {
            tree_id: UiTreeId::new("editor.chrome.source"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: source_node_id,
                    kind: UiRenderCommandKind::Quad,
                    frame: UiFrame::new(0.0, 0.0, 4.0, 4.0),
                    clip_frame: None,
                    z_index: 0,
                    style: UiResolvedStyle::default(),
                    text_layout: None,
                    text: None,
                    image: None,
                    opacity: 1.0,
                }],
            },
            raster_scale: 1.0,
        })),
        ..UiSurfaceFrame::default()
    });
    let mut render_sources = HostRenderSourceTable::default();
    let surface_key = render_sources
        .register(&source_frame)
        .expect("source surface key");
    let command_ref = UiRenderFrameCommandRef::new(source_node_id, 0);
    let stream = ChromeCommandStream::from_extracted_commands(
        (64, 64),
        None,
        vec![super::super::ChromeCommand {
            layer: ChromeCommandLayer::Dynamic,
            z_index: 0,
            frame: FrameRect::default(),
            clip: None,
            source: Some(HostRenderCommandSource {
                surface_key,
                command_ref,
                fragment_index: 4,
            }),
            kind: ChromeCommandKind::Clip,
        }],
        render_sources,
    );

    let (resolved_frame, resolved_command, resolved_fragment) =
        stream.resolve_command_source(0).expect("resolved source");
    assert!(Arc::ptr_eq(resolved_frame, &source_frame));
    assert_eq!(resolved_command, command_ref);
    assert_eq!(resolved_fragment, 4);
    let (resolved_frame, runtime_command, resolved_fragment) = stream
        .resolve_runtime_command_source(0)
        .expect("resolved runtime command source");
    assert!(Arc::ptr_eq(resolved_frame, &source_frame));
    assert_eq!(runtime_command.node_id, source_node_id);
    assert_eq!(resolved_fragment, 4);
}

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
            .map(|resource| resource.rgba.as_ref()),
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
