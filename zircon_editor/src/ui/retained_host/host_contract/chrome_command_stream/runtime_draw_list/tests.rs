use super::*;
use crate::ui::retained_host::host_contract::chrome_command_stream::{
    ChromeCommand, ChromeCommandKind, ChromeCommandLayer, ChromeImagePayload, ChromeImageUvRect,
};
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_text::{
    HostTextFontFace, font_request_for_face,
};
use zircon_runtime::rhi::{UiSurfaceCommandKind, UiSurfaceImageUvRect, UiSurfaceTextStyle};
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
fn runtime_draw_list_projects_editor_text_family_and_weight_to_runtime_surface() {
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

    let draw_list = ui_surface_draw_list_from_stream(&stream);

    let UiSurfaceCommandKind::Text {
        font_family,
        font_weight,
        style,
        ..
    } = &draw_list.commands[0].kind
    else {
        panic!("expected code text command");
    };
    assert_eq!(
        font_family.as_deref(),
        Some(
            font_request_for_face(HostTextFontFace::Mono)
                .family
                .as_str()
        )
    );
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
    assert_eq!(
        font_family.as_deref(),
        Some(
            font_request_for_face(HostTextFontFace::UiStrong)
                .family
                .as_str()
        )
    );
    assert_eq!(
        *font_weight,
        UiResolvedStyle::normalized_font_weight(
            font_request_for_face(HostTextFontFace::UiStrong).weight
        )
    );
    assert_eq!(*style, UiSurfaceTextStyle::Strong);
}

#[test]
fn owned_runtime_draw_list_moves_text_and_image_allocations() {
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
            width: 2,
            height: 2,
            upload_bytes: 16,
            rgba: Some(vec![7; 16]),
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

    let UiSurfaceCommandKind::Text { text, .. } = &draw_list.commands[0].kind else {
        panic!("expected runtime text command");
    };
    assert_eq!(text.as_ptr(), text_ptr);
    let UiSurfaceCommandKind::Image { payload } = &draw_list.commands[1].kind else {
        panic!("expected runtime image command");
    };
    assert_eq!(payload.resource_key.as_ptr(), resource_key_ptr);
    assert_eq!(
        payload.rgba.as_ref().expect("image bytes").as_ptr(),
        rgba_ptr
    );
}

#[test]
fn versioned_owned_runtime_draw_list_preserves_producer_generation() {
    let draw_list = ui_surface_draw_list_from_owned_stream_with_generation(
        ChromeCommandStream::full_rebuild((64, 64)),
        17,
    );

    assert_eq!(draw_list.generation(), Some(17));
}
