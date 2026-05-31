use std::path::PathBuf;
use std::rc::Rc;

use super::*;
use crate::ui::layouts::views::load_preview_image;
use crate::ui::retained_host::host_contract::data::{
    HostClosePromptData, HostDocumentDockSurfaceData, HostWindowLayoutData, PaneData,
    TemplateNodeFrameData, TemplatePaneNodeData,
};
use crate::ui::retained_host::host_contract::painter::{
    paint_host_frame, repaint_host_frame_region,
};
use crate::ui::retained_host::primitives::{Image, ModelRc, VecModel};

const WORKBENCH_REFERENCE_IMAGE_CONTROL_ID: &str = "WorkbenchShellReferenceImage";
const WORKBENCH_REFERENCE_IMAGE_PATH: &str = "ui/editor/reference/workbench.png";
const WORKBENCH_REFERENCE_SOURCE_PATH: &str = "docs/ui-and-layout/workbench.png";
const WORKBENCH_REFERENCE_WIDTH: u32 = 1672;
const WORKBENCH_REFERENCE_HEIGHT: u32 = 941;
const WORKBENCH_REFERENCE_UPLOAD_BYTES: u64 =
    (WORKBENCH_REFERENCE_WIDTH as u64) * (WORKBENCH_REFERENCE_HEIGHT as u64) * 4;

fn presentation_with_viewport_image() -> HostWindowPresentationData {
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_layout = test_layout();
    presentation.host_scene_data.layout = test_layout();
    presentation.host_scene_data.menu_chrome.template_nodes =
        model_rc(vec![template_node("ProjectAction", "Button", "Create")]);
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        region_frame: FrameRect {
            x: 24.0,
            y: 50.0,
            width: 150.0,
            height: 120.0,
        },
        header_frame: FrameRect {
            x: 0.0,
            y: 0.0,
            width: 150.0,
            height: 30.0,
        },
        content_frame: FrameRect {
            x: 0.0,
            y: 32.0,
            width: 150.0,
            height: 86.0,
        },
        pane: PaneData {
            kind: "Scene".into(),
            title: "Scene".into(),
            show_toolbar: false,
            ..PaneData::default()
        },
        ..HostDocumentDockSurfaceData::default()
    };
    presentation.host_shell.project_path = "res://project".into();
    presentation.host_shell.status_secondary = "Ready".into();
    presentation.viewport_image = Some(super::super::super::data::HostViewportImageData {
        resource_key: "viewport:test-initial".into(),
        width: 2,
        height: 2,
        rgba: vec![255; 16],
    });
    presentation
}

fn presentation_with_workbench_reference_overlay() -> HostWindowPresentationData {
    let mut presentation = HostWindowPresentationData::default();
    presentation.root_template_nodes = model_rc(vec![workbench_reference_image_node()]);
    presentation
}

fn test_layout() -> HostWindowLayoutData {
    HostWindowLayoutData {
        center_band_frame: FrameRect {
            x: 0.0,
            y: 36.0,
            width: 200.0,
            height: 144.0,
        },
        viewport_content_frame: FrameRect {
            x: 40.0,
            y: 92.0,
            width: 80.0,
            height: 60.0,
        },
        status_bar_frame: FrameRect {
            x: 0.0,
            y: 180.0,
            width: 200.0,
            height: 20.0,
        },
        document_region_frame: FrameRect {
            x: 24.0,
            y: 50.0,
            width: 150.0,
            height: 120.0,
        },
        ..HostWindowLayoutData::default()
    }
}

fn workbench_reference_image_node() -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: "workbench_reference_image".into(),
        control_id: WORKBENCH_REFERENCE_IMAGE_CONTROL_ID.into(),
        role: "Image".into(),
        component_role: "image".into(),
        media_source: WORKBENCH_REFERENCE_IMAGE_PATH.into(),
        has_preview_image: true,
        preview_image: load_preview_image(WORKBENCH_REFERENCE_IMAGE_PATH, ""),
        frame: TemplateNodeFrameData {
            x: 0.0,
            y: 0.0,
            width: WORKBENCH_REFERENCE_WIDTH as f32,
            height: WORKBENCH_REFERENCE_HEIGHT as f32,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn template_node(control_id: &str, role: &str, text: &str) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: role.into(),
        text: text.into(),
        surface_variant: "panel".into(),
        border_width: 1.0,
        frame: TemplateNodeFrameData {
            x: 12.0,
            y: 12.0,
            width: 72.0,
            height: 24.0,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn reference_png_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_editor manifest should live under the repository root")
        .join(WORKBENCH_REFERENCE_SOURCE_PATH)
}

fn model_rc<T: Clone + 'static>(values: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(values)))
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
        .any(|command| matches!(command.kind, ChromeCommandKind::Image { .. })));
    assert!(stream.commands().iter().any(|command| {
        matches!(
            &command.kind,
            ChromeCommandKind::Text { text, .. } if text == "Create"
        )
    }));
}

#[test]
fn recorded_commands_preserve_corner_radius_in_chrome_stream() {
    let mut stream = ChromeCommandStream::full_rebuild((64, 64));
    stream.push_recorded_command(
        HostRecordedPaintCommand {
            frame: FrameRect {
                x: 4.0,
                y: 4.0,
                width: 24.0,
                height: 16.0,
            },
            clip_frame: None,
            z_index: 3,
            kind: HostRecordedPaintKind::Quad {
                color: [10, 20, 30, 255],
                corner_radius: 8.0,
            },
        },
        false,
    );
    stream.push_recorded_command(
        HostRecordedPaintCommand {
            frame: FrameRect {
                x: 4.0,
                y: 24.0,
                width: 24.0,
                height: 16.0,
            },
            clip_frame: None,
            z_index: 4,
            kind: HostRecordedPaintKind::Border {
                color: [40, 50, 60, 255],
                width: 2.0,
                corner_radius: 7.0,
            },
        },
        false,
    );

    assert!(matches!(
        stream.commands()[0].kind,
        ChromeCommandKind::Quad {
            color: [10, 20, 30, 255],
            corner_radius: 8.0,
        }
    ));
    assert!(matches!(
        stream.commands()[1].kind,
        ChromeCommandKind::Border {
            color: [40, 50, 60, 255],
            width: 2.0,
            corner_radius: 7.0,
        }
    ));
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
    assert_eq!(image.rgba.as_deref(), Some(&[255; 16][..]));
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

#[test]
fn command_stream_executor_repaints_close_prompt_without_legacy_painter() {
    let mut presentation = presentation_with_viewport_image();
    presentation.close_prompt = HostClosePromptData {
        visible: true,
        title: "Unsaved".into(),
        message: "Save changes?".into(),
        details: "Project".into(),
        can_save: true,
        overlay_frame: FrameRect {
            x: 10.0,
            y: 20.0,
            width: 160.0,
            height: 120.0,
        },
        dialog_frame: FrameRect {
            x: 30.0,
            y: 34.0,
            width: 120.0,
            height: 90.0,
        },
        save_button_frame: FrameRect {
            x: 42.0,
            y: 92.0,
            width: 32.0,
            height: 18.0,
        },
        discard_button_frame: FrameRect {
            x: 78.0,
            y: 92.0,
            width: 32.0,
            height: 18.0,
        },
        cancel_button_frame: FrameRect {
            x: 114.0,
            y: 92.0,
            width: 32.0,
            height: 18.0,
        },
        ..HostClosePromptData::default()
    };

    let stream = build_chrome_command_stream(&presentation, (200, 200), None, true);
    let frame = paint_chrome_command_stream_to_frame(200, 200, &stream);

    assert_ne!(pixel(frame.as_bytes(), 200, 12, 22), [0, 0, 0, 255]);
    assert!(stream.commands().iter().any(|command| {
        matches!(
            &command.kind,
            ChromeCommandKind::Text { text, .. } if text == "Unsaved"
        )
    }));
}

#[test]
fn full_command_stream_matches_legacy_painter_pixels() {
    let presentation = presentation_with_viewport_image();
    let legacy = paint_host_frame(200, 200, &presentation);
    let stream = build_chrome_command_stream(&presentation, (200, 200), None, true);
    let replayed = paint_chrome_command_stream_to_frame(200, 200, &stream);

    assert_eq!(
        first_pixel_difference(replayed.as_bytes(), legacy.as_bytes(), 200),
        None
    );
}

#[test]
fn full_command_stream_replays_workbench_reference_overlay_pixels() {
    let presentation = presentation_with_workbench_reference_overlay();
    let legacy = paint_host_frame(
        WORKBENCH_REFERENCE_WIDTH,
        WORKBENCH_REFERENCE_HEIGHT,
        &presentation,
    );
    let stream = build_chrome_command_stream(
        &presentation,
        (WORKBENCH_REFERENCE_WIDTH, WORKBENCH_REFERENCE_HEIGHT),
        None,
        true,
    );
    let replayed = paint_chrome_command_stream_to_frame(
        WORKBENCH_REFERENCE_WIDTH,
        WORKBENCH_REFERENCE_HEIGHT,
        &stream,
    );
    let reference = Image::load_from_path(&reference_png_path())
        .expect("docs workbench reference image should load")
        .to_rgba8()
        .expect("docs workbench reference image should convert to RGBA");

    let image = workbench_reference_image_command(&stream, reference.as_bytes())
        .expect("workbench reference overlay should be recorded as an image command");
    assert!(!image.resource_key.is_empty());
    assert_eq!(image.width, WORKBENCH_REFERENCE_WIDTH);
    assert_eq!(image.height, WORKBENCH_REFERENCE_HEIGHT);
    assert_eq!(image.upload_bytes, WORKBENCH_REFERENCE_UPLOAD_BYTES);
    assert_eq!(
        first_pixel_difference(
            legacy.as_bytes(),
            reference.as_bytes(),
            WORKBENCH_REFERENCE_WIDTH
        ),
        None
    );
    assert_eq!(
        first_pixel_difference(
            replayed.as_bytes(),
            reference.as_bytes(),
            WORKBENCH_REFERENCE_WIDTH
        ),
        None
    );
}

#[test]
fn patch_command_stream_matches_legacy_region_repaint_pixels() {
    let mut presentation = presentation_with_viewport_image();
    let damage = FrameRect {
        x: 40.0,
        y: 92.0,
        width: 80.0,
        height: 60.0,
    };
    let mut legacy = paint_host_frame(200, 200, &presentation);
    let mut replayed = paint_host_frame(200, 200, &presentation);

    presentation.viewport_image = Some(super::super::super::data::HostViewportImageData {
        resource_key: "viewport:test-patch".into(),
        width: 2,
        height: 2,
        rgba: vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255,
        ],
    });
    let stream = build_chrome_command_stream(&presentation, (200, 200), Some(&damage), true);

    let legacy_damage = repaint_host_frame_region(&mut legacy, &presentation, &damage)
        .expect("legacy painter should repaint visible viewport damage");
    let replayed_damage = repaint_chrome_command_stream_region(&mut replayed, &stream)
        .expect("command stream should repaint visible viewport damage");

    assert_eq!(replayed_damage, legacy_damage);
    assert_eq!(
        first_pixel_difference(replayed.as_bytes(), legacy.as_bytes(), 200),
        None
    );
}

#[test]
fn patch_command_stream_repaints_workbench_reference_overlay_damage_pixels() {
    let presentation = presentation_with_workbench_reference_overlay();
    let damage = FrameRect {
        x: 512.0,
        y: 300.0,
        width: 320.0,
        height: 220.0,
    };
    let mut legacy = paint_host_frame(
        WORKBENCH_REFERENCE_WIDTH,
        WORKBENCH_REFERENCE_HEIGHT,
        &presentation,
    );
    let mut replayed = paint_host_frame(
        WORKBENCH_REFERENCE_WIDTH,
        WORKBENCH_REFERENCE_HEIGHT,
        &presentation,
    );
    let stream = build_chrome_command_stream(
        &presentation,
        (WORKBENCH_REFERENCE_WIDTH, WORKBENCH_REFERENCE_HEIGHT),
        Some(&damage),
        true,
    );
    let reference = Image::load_from_path(&reference_png_path())
        .expect("docs workbench reference image should load")
        .to_rgba8()
        .expect("docs workbench reference image should convert to RGBA");

    let legacy_damage = repaint_host_frame_region(&mut legacy, &presentation, &damage)
        .expect("legacy painter should repaint visible workbench reference damage");
    let replayed_damage = repaint_chrome_command_stream_region(&mut replayed, &stream)
        .expect("command stream should repaint visible workbench reference damage");

    assert_eq!(replayed_damage, legacy_damage);
    assert!(workbench_reference_image_command(&stream, reference.as_bytes()).is_some());
    assert_eq!(
        first_pixel_difference(
            replayed.as_bytes(),
            legacy.as_bytes(),
            WORKBENCH_REFERENCE_WIDTH
        ),
        None
    );
    assert_eq!(
        first_pixel_difference(
            replayed.as_bytes(),
            reference.as_bytes(),
            WORKBENCH_REFERENCE_WIDTH
        ),
        None
    );
}

fn workbench_reference_image_command<'a>(
    stream: &'a ChromeCommandStream,
    reference_rgba: &[u8],
) -> Option<&'a ChromeImagePayload> {
    stream
        .commands()
        .iter()
        .find_map(|command| match &command.kind {
            ChromeCommandKind::Image { payload }
                if payload.width == WORKBENCH_REFERENCE_WIDTH
                    && payload.height == WORKBENCH_REFERENCE_HEIGHT
                    && payload.upload_bytes == WORKBENCH_REFERENCE_UPLOAD_BYTES
                    && payload.rgba.as_deref() == Some(reference_rgba) =>
            {
                Some(payload)
            }
            _ => None,
        })
}

fn first_pixel_difference(
    left: &[u8],
    right: &[u8],
    width: u32,
) -> Option<(u32, u32, [u8; 4], [u8; 4])> {
    left.chunks_exact(4)
        .zip(right.chunks_exact(4))
        .enumerate()
        .find_map(|(index, (left, right))| {
            (left != right).then(|| {
                let x = index as u32 % width;
                let y = index as u32 / width;
                (
                    x,
                    y,
                    [left[0], left[1], left[2], left[3]],
                    [right[0], right[1], right[2], right[3]],
                )
            })
        })
}

fn pixel(bytes: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let offset = ((y as usize * width as usize) + x as usize) * 4;
    [
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]
}
