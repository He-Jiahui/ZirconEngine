use super::*;
use crate::ui::retained_host::host_contract::chrome_command_stream::{
    ChromeCommandLayer, ChromeImagePayload, ChromeImageUvRect,
};
use crate::ui::retained_host::host_contract::data::FrameRect;
use zircon_runtime::rhi::{UiSurfaceCommandKind, UiSurfaceImageUvRect};

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
