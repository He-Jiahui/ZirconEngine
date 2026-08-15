use super::*;
use crate::ui::retained_host::host_contract::chrome_command_stream::{
    ChromeCommand, ChromeCommandKind, ChromeCommandLayer, ChromeImagePayload, ChromeImageUvRect,
};
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_text::{
    font_request_for_face, runtime_font_family_for_face, take_runtime_text_face_capture_count,
    HostTextFontFace,
};
use zircon_runtime::rhi::{
    UiSurfaceCommandKind, UiSurfaceImageUvRect, UiSurfaceResolvedCommandKind, UiSurfaceTextStyle,
};
use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextRunPaintStyle};

#[test]
fn runtime_draw_list_preserves_chrome_corner_radius() {
    let mut stream = ChromeCommandStream::full_rebuild((64, 64));
    stream.push_quad(
        ChromeCommandLayer::Static,
        1,
        FrameRect {
            x: 4.0,
            y: 6.0,
            width: 20.0,
            height: 12.0,
        },
        None,
        [255, 0, 0, 255],
        9.0,
    );
    stream.push_border(
        ChromeCommandLayer::Static,
        2,
        FrameRect {
            x: 4.0,
            y: 24.0,
            width: 20.0,
            height: 12.0,
        },
        None,
        [0, 255, 0, 255],
        2.0,
        8.0,
    );

    let draw_list = ui_surface_draw_list_from_stream(&stream);

    assert!(matches!(
        draw_list.commands[0].kind,
        UiSurfaceCommandKind::Quad {
            color: [255, 0, 0, 255],
            corner_radius: 9.0,
        }
    ));
    assert!(matches!(
        draw_list.commands[1].kind,
        UiSurfaceCommandKind::Border {
            color: [0, 255, 0, 255],
            width: 2.0,
            corner_radius: 8.0,
        }
    ));
}

#[test]
fn runtime_draw_list_forwards_atlas_uv_to_runtime_surface_payload() {
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

    let draw_list = ui_surface_draw_list_from_stream(&stream);

    let UiSurfaceCommandKind::Image { payload } = &draw_list.commands[0].kind else {
        panic!("expected runtime image command");
    };
    assert_eq!(payload.resource_key, "atlas://editor/icons");
    assert_eq!(
        payload.atlas_uv,
        Some(UiSurfaceImageUvRect {
            min: [0.5, 0.25],
            max: [0.75, 0.5],
        })
    );
}

#[test]
fn borrowed_and_owned_runtime_draw_lists_project_resolved_editor_text_style() {
    let _ = take_runtime_text_face_capture_count();
    let mut stream = ChromeCommandStream::full_rebuild((128, 64));
    stream.push_command_for_test(ChromeCommand {
        layer: ChromeCommandLayer::Text,
        z_index: 1,
        frame: FrameRect {
            x: 4.0,
            y: 6.0,
            width: 80.0,
            height: 16.0,
        },
        clip: None,
        kind: ChromeCommandKind::Text {
            text: "Code".to_string(),
            color: [220, 220, 220, 255],
            size: 12.0,
            line_height: 14.0,
            style: UiTextRunPaintStyle {
                code: true,
                ..UiTextRunPaintStyle::default()
            },
        },
    });
    stream.push_command_for_test(ChromeCommand {
        layer: ChromeCommandLayer::Text,
        z_index: 2,
        frame: FrameRect {
            x: 4.0,
            y: 24.0,
            width: 80.0,
            height: 16.0,
        },
        clip: None,
        kind: ChromeCommandKind::Text {
            text: "Strong".to_string(),
            color: [240, 240, 240, 255],
            size: 12.0,
            line_height: 14.0,
            style: UiTextRunPaintStyle {
                strong: true,
                ..UiTextRunPaintStyle::default()
            },
        },
    });

    let borrowed_draw_list = ui_surface_draw_list_from_stream(&stream);
    let owned_draw_list = ui_surface_draw_list_from_owned_stream(stream);
    let mono_family = runtime_font_family_for_face(HostTextFontFace::Mono);
    let strong_family = runtime_font_family_for_face(HostTextFontFace::UiStrong);
    assert_eq!(
        take_runtime_text_face_capture_count(),
        2,
        "borrowed and owned draw-list conversions should each capture one coherent face set"
    );

    for draw_list in [&borrowed_draw_list, &owned_draw_list] {
        let UiSurfaceCommandKind::Text {
            font_family,
            font_weight,
            style,
            ..
        } = &draw_list.commands[0].kind
        else {
            panic!("expected code text command");
        };
        assert_eq!(font_family.as_deref(), Some(mono_family.as_ref()));
        assert_eq!(
            *font_weight,
            UiResolvedStyle::normalized_font_weight(
                font_request_for_face(HostTextFontFace::Mono).weight
            )
        );
        assert_eq!(*style, UiSurfaceTextStyle::Regular);

        let UiSurfaceCommandKind::Text {
            font_family,
            font_weight,
            style,
            ..
        } = &draw_list.commands[1].kind
        else {
            panic!("expected strong text command");
        };
        assert_eq!(font_family.as_deref(), Some(strong_family.as_ref()));
        assert_eq!(
            *font_weight,
            UiResolvedStyle::normalized_font_weight(
                font_request_for_face(HostTextFontFace::UiStrong).weight
            )
        );
        assert_eq!(*style, UiSurfaceTextStyle::Strong);
    }
}

#[test]
fn runtime_draw_list_font_projection_work_is_bounded_per_stream() {
    let _ = take_runtime_text_face_capture_count();
    let mut text_stream = ChromeCommandStream::full_rebuild((128, 64));
    for index in 0..1_000 {
        text_stream.push_command_for_test(ChromeCommand {
            layer: ChromeCommandLayer::Text,
            z_index: index,
            frame: FrameRect::default(),
            clip: None,
            kind: ChromeCommandKind::Text {
                text: format!("label-{index}"),
                color: [255; 4],
                size: 12.0,
                line_height: 14.0,
                style: UiTextRunPaintStyle::default(),
            },
        });
    }

    let draw_list = ui_surface_draw_list_from_stream(&text_stream);
    let owned_draw_list = ui_surface_draw_list_from_owned_stream(text_stream.clone());

    assert_eq!(draw_list.commands.len(), 1_000);
    assert_eq!(owned_draw_list.commands.len(), 1_000);
    assert_eq!(
        take_runtime_text_face_capture_count(),
        2,
        "borrowed and owned font face capture work must not scale with text command count"
    );

    let mut quad_stream = ChromeCommandStream::full_rebuild((16, 16));
    quad_stream.push_quad(
        ChromeCommandLayer::Static,
        0,
        FrameRect::default(),
        None,
        [0; 4],
        0.0,
    );
    let _ = ui_surface_draw_list_from_owned_stream(quad_stream);
    assert_eq!(
        take_runtime_text_face_capture_count(),
        0,
        "a non-text stream must not resolve any font face"
    );
}

#[test]
fn owned_runtime_draw_list_moves_image_pixels_into_the_draw_list_resource_table() {
    let mut stream = ChromeCommandStream::full_rebuild((64, 64));
    stream.push_command_for_test(ChromeCommand {
        layer: ChromeCommandLayer::Text,
        z_index: 1,
        frame: FrameRect::default(),
        clip: None,
        kind: ChromeCommandKind::Text {
            text: "move-me".to_string(),
            color: [255; 4],
            size: 12.0,
            line_height: 14.0,
            style: UiTextRunPaintStyle::default(),
        },
    });
    stream.push_image(
        2,
        FrameRect::default(),
        None,
        ChromeImagePayload {
            resource_key: "image://move-me".to_string(),
            resource_generation: 0,
            width: 2,
            height: 2,
            upload_bytes: 16,
            rgba: Some(vec![7; 16].into()),
            atlas_uv: None,
        },
    );

    let ChromeCommandKind::Text { text, .. } = &stream.commands()[0].kind else {
        panic!("expected text command");
    };
    let text_ptr = text.as_ptr();
    let ChromeCommandKind::Image { payload } = &stream.commands()[1].kind else {
        panic!("expected image command");
    };
    let resource_key_ptr = payload.resource_key.as_ptr();
    let rgba_ptr = payload.rgba.as_ref().expect("image bytes").as_ptr();

    let draw_list = ui_surface_draw_list_from_owned_stream(stream);

    let Some(UiSurfaceResolvedCommandKind::Text { text, .. }) =
        draw_list.resolved_kind(&draw_list.commands[0])
    else {
        panic!("expected runtime text command");
    };
    assert_eq!(text.as_ptr(), text_ptr);
    let UiSurfaceCommandKind::Image { payload } = &draw_list.commands[1].kind else {
        panic!("expected runtime image command");
    };
    assert_eq!(payload.resource_key.as_ptr(), resource_key_ptr);
    assert!(payload.rgba.is_none());
    assert_eq!(
        draw_list
            .image_resource("image://move-me", 0)
            .expect("draw list owns the image pixels")
            .rgba
            .as_ptr(),
        rgba_ptr
    );
}

#[test]
fn borrowed_runtime_draw_list_skips_resident_image_resource_pixels() {
    let mut stream = ChromeCommandStream::full_rebuild((64, 64));
    stream.push_image(
        1,
        FrameRect::default(),
        None,
        ChromeImagePayload {
            resource_key: "image://resident".to_string(),
            resource_generation: 7,
            width: 2,
            height: 2,
            upload_bytes: 16,
            rgba: Some(vec![9; 16].into()),
            atlas_uv: None,
        },
    );
    stream.compact_image_resources();

    let draw_list = ui_surface_draw_list_from_stream_with_residency(&stream, |key, generation| {
        key == "image://resident" && generation == 7
    });

    assert!(draw_list.image_resource("image://resident", 7).is_none());
    assert!(matches!(
        &draw_list.commands[0].kind,
        UiSurfaceCommandKind::Image { payload } if payload.rgba.is_none()
    ));
}

#[test]
fn borrowed_runtime_draw_list_keeps_shared_image_pixels_out_of_commands() {
    let mut stream = ChromeCommandStream::full_rebuild((64, 64));
    for z_index in [1, 2] {
        stream.push_image(
            z_index,
            FrameRect::default(),
            None,
            ChromeImagePayload {
                resource_key: "image://shared".to_string(),
                resource_generation: 3,
                width: 2,
                height: 2,
                upload_bytes: 16,
                rgba: Some(vec![3; 16].into()),
                atlas_uv: None,
            },
        );
    }
    stream.compact_image_resources();
    let source_rgba_ptr = stream
        .image_resource("image://shared", 3)
        .expect("stream owns the shared source")
        .rgba
        .as_ptr();

    let draw_list = ui_surface_draw_list_from_stream(&stream);

    assert!(draw_list.commands.iter().all(|command| matches!(
        &command.kind,
        UiSurfaceCommandKind::Image { payload } if payload.rgba.is_none()
    )));
    assert_eq!(
        draw_list
            .image_resource("image://shared", 3)
            .expect("shared image source must be copied once into the resource table")
            .rgba
            .as_ref(),
        &[3; 16]
    );
    assert_eq!(
        draw_list
            .image_resource("image://shared", 3)
            .expect("draw list shares the image source")
            .rgba
            .as_ptr(),
        source_rgba_ptr
    );
}

#[test]
fn owned_runtime_draw_list_preserves_distinct_generations_of_one_resource_key() {
    let mut stream = ChromeCommandStream::full_rebuild((64, 64));
    for (z_index, generation) in [(1, 4), (2, 5)] {
        stream.push_image(
            z_index,
            FrameRect::default(),
            None,
            ChromeImagePayload {
                resource_key: "atlas://editor/icons".to_string(),
                resource_generation: generation,
                width: 2,
                height: 2,
                upload_bytes: 16,
                rgba: Some(vec![generation as u8; 16].into()),
                atlas_uv: None,
            },
        );
    }

    let draw_list = ui_surface_draw_list_from_owned_stream(stream);

    assert!(draw_list.commands.iter().all(|command| matches!(
        &command.kind,
        UiSurfaceCommandKind::Image { payload } if payload.rgba.is_none()
    )));
    assert_eq!(
        draw_list
            .image_resource("atlas://editor/icons", 4)
            .expect("older image generation")
            .rgba
            .as_ref(),
        &[4; 16]
    );
    assert_eq!(
        draw_list
            .image_resource("atlas://editor/icons", 5)
            .expect("newer image generation")
            .rgba
            .as_ref(),
        &[5; 16]
    );
}

#[test]
fn borrowed_runtime_draw_list_falls_back_to_inline_pixels_without_resource_entry() {
    let mut stream = ChromeCommandStream::full_rebuild((64, 64));
    stream.push_image(
        1,
        FrameRect::default(),
        None,
        ChromeImagePayload {
            resource_key: "image://inline-fallback".to_string(),
            resource_generation: 4,
            width: 2,
            height: 2,
            upload_bytes: 16,
            rgba: Some(vec![4; 16].into()),
            atlas_uv: None,
        },
    );

    let draw_list = ui_surface_draw_list_from_stream(&stream);

    assert!(matches!(
        &draw_list.commands[0].kind,
        UiSurfaceCommandKind::Image { payload } if payload.rgba.is_none()
    ));
    assert_eq!(
        draw_list
            .image_resource("image://inline-fallback", 4)
            .expect("uncompacted stream must retain its inline source once")
            .rgba
            .as_ref(),
        &[4; 16]
    );
}

#[test]
fn resident_atlas_stays_out_of_owned_draw_list_resources() {
    let mut stream = ChromeCommandStream::full_rebuild((64, 64));
    stream.push_image(
        1,
        FrameRect::default(),
        None,
        ChromeImagePayload {
            resource_key: "atlas://editor/icons".to_string(),
            resource_generation: 8,
            width: 2,
            height: 2,
            upload_bytes: 16,
            rgba: Some(vec![8; 16].into()),
            atlas_uv: Some(ChromeImageUvRect {
                min: [0.0, 0.0],
                max: [0.5, 0.5],
            }),
        },
    );
    stream.compact_image_resources_with_residency(|resource_key, generation| {
        resource_key == "atlas://editor/icons" && generation == 8
    });

    let draw_list = ui_surface_draw_list_from_owned_stream_with_generation_and_residency(
        stream,
        19,
        |resource_key, generation| resource_key == "atlas://editor/icons" && generation == 8,
    );

    assert!(draw_list
        .image_resource("atlas://editor/icons", 8)
        .is_none());
    assert!(matches!(
        &draw_list.commands[0].kind,
        UiSurfaceCommandKind::Image { payload } if payload.rgba.is_none()
    ));
}

#[test]
fn versioned_owned_runtime_draw_list_preserves_producer_generation() {
    let draw_list = ui_surface_draw_list_from_owned_stream_with_generation(
        ChromeCommandStream::full_rebuild((64, 64)),
        17,
    );

    assert_eq!(draw_list.generation(), Some(17));
}

#[test]
fn versioned_owned_runtime_draw_list_interns_repeated_chrome_styles() {
    let mut stream = ChromeCommandStream::full_rebuild((64, 64));
    for x in [0.0, 12.0, 24.0] {
        stream.push_quad(
            ChromeCommandLayer::Static,
            0,
            FrameRect {
                x,
                y: 0.0,
                width: 8.0,
                height: 8.0,
            },
            None,
            [32, 48, 64, 255],
            0.0,
        );
    }

    let draw_list = ui_surface_draw_list_from_owned_stream_with_generation(stream, 18);

    assert_eq!(draw_list.style_count(), 1);
    assert!(draw_list
        .commands
        .iter()
        .all(|command| matches!(command.kind, UiSurfaceCommandKind::Styled { .. })));
}
