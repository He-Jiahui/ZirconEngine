use super::*;
use crate::ui::retained_host::host_contract::paint_template_nodes::visual_assets::{
    ICON_TINT, ICON_TINT_ACTIVE,
};

fn icon_node() -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        icon_name: "zircon_editor_shell/toolbar/compile.svg".into(),
        ..TemplatePaneNodeData::default()
    }
}

#[test]
fn focused_icon_image_does_not_use_active_tint() {
    let node = TemplatePaneNodeData {
        focused: true,
        ..icon_node()
    };

    assert_eq!(template_node_image_tint(&node), Some(ICON_TINT));
}

#[test]
fn checked_icon_image_uses_active_tint_without_focus() {
    let node = TemplatePaneNodeData {
        checked: true,
        ..icon_node()
    };

    assert_eq!(template_node_image_tint(&node), Some(ICON_TINT_ACTIVE));
}

#[test]
fn popup_open_icon_image_uses_active_tint_without_focus() {
    let node = TemplatePaneNodeData {
        popup_open: true,
        ..icon_node()
    };

    assert_eq!(template_node_image_tint(&node), Some(ICON_TINT_ACTIVE));
}

#[test]
fn unknown_image_aspect_uses_the_visible_container_for_materialization() {
    let node = TemplatePaneNodeData {
        role: "Image".into(),
        media_source: "ui/editor/showcase_checker.svg".into(),
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 120.0,
        height: 80.0,
    };

    let materialization_rect = image_materialization_rect(&node, &rect, 0, 0);

    assert_eq!(materialization_rect.x, rect.x);
    assert_eq!(materialization_rect.y, rect.y);
    assert_eq!(materialization_rect.width, rect.width);
    assert_eq!(materialization_rect.height, rect.height);
}

#[test]
fn vector_raster_bucket_does_not_change_the_command_frame() {
    let node = TemplatePaneNodeData {
        role: "SvgIcon".into(),
        icon_name: "folder-open-outline".into(),
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 54.0,
        height: 54.0,
    };
    let expected_frame = image_materialization_rect(&node, &rect, 0, 0);
    let mut commands = Vec::new();

    push_template_image_command(&mut commands, &node, &rect, &rect, 7, 1.0);

    assert_eq!(commands.len(), 1);
    let command = &commands[0];
    assert_eq!(command.frame.x, expected_frame.x);
    assert_eq!(command.frame.y, expected_frame.y);
    assert_eq!(command.frame.width, expected_frame.width);
    assert_eq!(command.frame.height, expected_frame.height);
    let pixels = command
        .image_pixels
        .as_ref()
        .expect("SVG icon command should carry cached raster pixels");
    assert_eq!((pixels.width, pixels.height), (48, 48));
    assert_ne!(pixels.width as f32, command.frame.width);
}

#[test]
fn opacity_variants_keep_distinct_resources_through_compaction_and_replay() {
    use crate::ui::retained_host::host_contract::chrome_command_stream::{
        chrome_command_from_recorded_for_test, paint_chrome_command_stream_to_frame,
        ChromeCommandKind, ChromeCommandStream,
    };
    use crate::ui::retained_host::host_contract::paint_frame::{
        HostRecordedPaintKind, HostRenderSourceTable, HostRgbaFrame,
    };
    use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::draw_host_paint_commands;
    use std::sync::Arc;

    let source: Arc<[u8]> = Arc::from([255_u8, 255, 255, 255]);
    let commands = [1.0, 0.5, 0.25, 0.5]
        .into_iter()
        .enumerate()
        .map(|(index, opacity)| {
            HostPaintCommand::image_pixels(
                FrameRect {
                    x: index as f32,
                    y: 0.0,
                    width: 1.0,
                    height: 1.0,
                },
                None,
                index as i32,
                "shared-opacity-source".into(),
                1,
                1,
                Arc::clone(&source),
                None,
                opacity,
            )
        })
        .collect::<Vec<_>>();
    let mut frame = HostRgbaFrame::recording_only(4, 1);
    assert!(draw_host_paint_commands(&mut frame, &commands));
    let recorded = frame.into_recorded_commands();
    assert_eq!(recorded.len(), 4);
    let HostRecordedPaintKind::Image {
        resource_key,
        rgba: Some(rgba),
        ..
    } = &recorded[0].kind
    else {
        panic!("opaque image must retain its shared pixels");
    };
    assert_eq!(resource_key, "shared-opacity-source");
    assert!(Arc::ptr_eq(rgba, &source));
    let extracted = recorded
        .into_iter()
        .map(|command| chrome_command_from_recorded_for_test(command, true, true).unwrap())
        .collect();
    let mut stream = ChromeCommandStream::from_extracted_commands(
        (4, 1),
        None,
        extracted,
        HostRenderSourceTable::default(),
    );
    stream.compact_image_resources();
    assert_eq!(stream.image_resources().len(), 3);
    let mut keys = Vec::new();
    for (command, alpha) in stream.commands().iter().zip([255_u8, 128, 64, 128]) {
        let ChromeCommandKind::Image { payload } = &command.kind else {
            panic!("image command must survive extraction");
        };
        assert!(
            payload.rgba.is_none(),
            "replay must resolve the compacted resource table"
        );
        let resource = stream
            .image_resource(&payload.resource_key, payload.resource_generation)
            .unwrap();
        assert_eq!(resource.rgba.as_ref(), &[255, 255, 255, alpha]);
        keys.push(payload.resource_key.clone());
    }
    assert_ne!(keys[0], keys[1]);
    assert_ne!(keys[0], keys[2]);
    assert_ne!(keys[1], keys[2]);
    assert_eq!(keys[1], keys[3]);
    assert!(Arc::ptr_eq(
        &stream.image_resource(&keys[0], 0).unwrap().rgba,
        &source,
    ));
    let replayed = paint_chrome_command_stream_to_frame(4, 1, &stream);
    // White over opaque black uses the existing linear-light sRGB renderer.
    assert_eq!(
        replayed.as_bytes(),
        &[255, 255, 255, 255, 188, 188, 188, 255, 137, 137, 137, 255, 188, 188, 188, 255,]
    );
}

#[test]
fn opaque_images_preserve_the_atlas_fast_path() {
    use crate::ui::retained_host::host_contract::paint_frame::{
        HostPaintAtlasImage, HostPaintImageUvRect, HostRecordedPaintKind, HostRgbaFrame,
    };
    use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::draw_host_paint_commands;
    use std::sync::Arc;

    let atlas = HostPaintAtlasImage {
        resource_key: "unchanged-opacity-atlas".into(),
        resource_generation: 7,
        width: 1,
        height: 1,
        rgba: Some(vec![255, 255, 255, 255]),
        uv: HostPaintImageUvRect {
            min: [0.0, 0.0],
            max: [1.0, 1.0],
        },
    };
    let command = HostPaintCommand::image_pixels(
        FrameRect {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        },
        None,
        0,
        "original-opacity-source".into(),
        1,
        1,
        Arc::<[u8]>::from([255_u8, 255, 255, 255]),
        Some(atlas.clone()),
        1.0,
    );
    let mut frame = HostRgbaFrame::recording_only(1, 1);
    assert!(draw_host_paint_commands(&mut frame, &[command]));
    let recorded = frame.into_recorded_commands();
    assert_eq!(recorded.len(), 1);
    let HostRecordedPaintKind::Image {
        resource_key,
        rgba,
        atlas: actual,
        ..
    } = &recorded[0].kind
    else {
        panic!("opaque atlas image must remain an image");
    };
    assert_eq!(resource_key, &atlas.resource_key);
    assert!(
        rgba.is_none(),
        "opaque atlas must not materialize derived RGBA"
    );
    assert_eq!(actual.as_ref(), Some(&atlas));
}
