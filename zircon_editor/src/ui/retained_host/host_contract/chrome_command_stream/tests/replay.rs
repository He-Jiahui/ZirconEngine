use super::super::{
    build_chrome_command_stream, paint_chrome_command_stream_to_frame,
    repaint_chrome_command_stream_region, ChromeCommandKind,
};
use super::support::{
    first_pixel_difference, pixel, presentation_with_root_overlay_image,
    presentation_with_viewport_image, root_overlay_image_command, solid_rgba, ROOT_OVERLAY_COLOR,
    ROOT_OVERLAY_FRAME_SIZE, ROOT_OVERLAY_IMAGE_HEIGHT, ROOT_OVERLAY_IMAGE_WIDTH,
    ROOT_OVERLAY_UPLOAD_BYTES,
};
use crate::ui::retained_host::host_contract::data::{FrameRect, HostClosePromptData};
use crate::ui::retained_host::host_contract::paint_workbench::{
    paint_host_frame, repaint_host_frame_region,
};

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
fn full_command_stream_replays_root_overlay_image_pixels() {
    let presentation = presentation_with_root_overlay_image();
    let legacy = paint_host_frame(
        ROOT_OVERLAY_FRAME_SIZE.0,
        ROOT_OVERLAY_FRAME_SIZE.1,
        &presentation,
    );
    let stream = build_chrome_command_stream(&presentation, ROOT_OVERLAY_FRAME_SIZE, None, true);
    let replayed = paint_chrome_command_stream_to_frame(
        ROOT_OVERLAY_FRAME_SIZE.0,
        ROOT_OVERLAY_FRAME_SIZE.1,
        &stream,
    );
    let overlay_rgba = solid_rgba(ROOT_OVERLAY_COLOR);

    let image = root_overlay_image_command(&stream, &overlay_rgba)
        .expect("root overlay should be recorded as an image command");
    assert!(!image.resource_key.is_empty());
    assert_eq!(image.width, ROOT_OVERLAY_IMAGE_WIDTH);
    assert_eq!(image.height, ROOT_OVERLAY_IMAGE_HEIGHT);
    assert_eq!(image.upload_bytes, ROOT_OVERLAY_UPLOAD_BYTES);
    assert_eq!(
        pixel(legacy.as_bytes(), ROOT_OVERLAY_FRAME_SIZE.0, 24, 16),
        ROOT_OVERLAY_COLOR
    );
    assert_eq!(
        first_pixel_difference(
            replayed.as_bytes(),
            legacy.as_bytes(),
            ROOT_OVERLAY_FRAME_SIZE.0
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
fn patch_command_stream_repaints_root_overlay_image_damage_pixels() {
    let presentation = presentation_with_root_overlay_image();
    let damage = FrameRect {
        x: 8.0,
        y: 8.0,
        width: 32.0,
        height: 24.0,
    };
    let mut legacy = paint_host_frame(
        ROOT_OVERLAY_FRAME_SIZE.0,
        ROOT_OVERLAY_FRAME_SIZE.1,
        &presentation,
    );
    let mut replayed = paint_host_frame(
        ROOT_OVERLAY_FRAME_SIZE.0,
        ROOT_OVERLAY_FRAME_SIZE.1,
        &presentation,
    );
    let stream =
        build_chrome_command_stream(&presentation, ROOT_OVERLAY_FRAME_SIZE, Some(&damage), true);
    let overlay_rgba = solid_rgba(ROOT_OVERLAY_COLOR);

    let legacy_damage = repaint_host_frame_region(&mut legacy, &presentation, &damage)
        .expect("legacy painter should repaint visible root overlay damage");
    let replayed_damage = repaint_chrome_command_stream_region(&mut replayed, &stream)
        .expect("command stream should repaint visible root overlay damage");

    assert_eq!(replayed_damage, legacy_damage);
    assert!(root_overlay_image_command(&stream, &overlay_rgba).is_some());
    assert_eq!(
        first_pixel_difference(
            replayed.as_bytes(),
            legacy.as_bytes(),
            ROOT_OVERLAY_FRAME_SIZE.0
        ),
        None
    );
}
