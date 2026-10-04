use std::sync::Arc;

use super::super::super::super::data::{HostViewportImageData, HostViewportOverlayImageData};
use super::super::super::super::paint_frame::HostRecordedPaintKind;
use super::*;

#[test]
fn simulate_gizmo_overlay_records_after_the_base_viewport_image() {
    let mut images = HostViewportImageSet::default();
    images.replace_scene(HostViewportImageData {
        resource_key: "play:test-frame".to_string(),
        resource_generation: 0,
        width: 10,
        height: 10,
        rgba: Some(vec![0; 400].into()),
        play_frame_identity: None,
        overlay: Some(Arc::new(HostViewportOverlayImageData {
            resource_key: "play:test-gizmo".to_string(),
            x: 3,
            y: 4,
            width: 2,
            height: 2,
            rgba: vec![255; 16].into(),
        })),
    });
    let pane = PaneData {
        kind: "Scene".into(),
        ..PaneData::default()
    };
    let body = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 100.0,
    };
    let mut frame = HostRgbaFrame::recording_only(200, 200);

    assert!(draw_viewport_image(
        &mut frame, &pane, &body, &body, &images, None,
    ));
    let commands = frame.into_recorded_commands();
    assert_eq!(commands.len(), 2);
    let resource_keys = commands
        .iter()
        .map(|command| match &command.kind {
            HostRecordedPaintKind::Image { resource_key, .. } => resource_key.as_str(),
            _ => panic!("viewport composition must record image commands"),
        })
        .collect::<Vec<_>>();
    assert_eq!(resource_keys, ["play:test-frame", "play:test-gizmo"]);
    assert_eq!(commands[1].frame.x, 40.0);
    assert_eq!(commands[1].frame.y, 60.0);
    assert_eq!(commands[1].frame.width, 20.0);
    assert_eq!(commands[1].frame.height, 20.0);
}

#[test]
fn valid_viewport_content_remains_present_outside_damage() {
    let mut images = HostViewportImageSet::default();
    images.replace_scene(HostViewportImageData {
        resource_key: "scene:test-frame".to_string(),
        resource_generation: 0,
        width: 10,
        height: 10,
        rgba: Some(vec![0; 400].into()),
        play_frame_identity: None,
        overlay: None,
    });
    let pane = PaneData {
        kind: "Scene".into(),
        ..PaneData::default()
    };
    let body = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 100.0,
    };
    let mut frame = HostRgbaFrame::recording_only(300, 300);
    frame.replace_paint_clip(Some(FrameRect {
        x: 200.0,
        y: 200.0,
        width: 20.0,
        height: 20.0,
    }));

    assert!(draw_viewport_image(
        &mut frame, &pane, &body, &body, &images, None,
    ));
    assert!(frame.into_recorded_commands().is_empty());
}

#[test]
fn keyed_leaf_viewport_paint_consumes_only_matching_scene_product() {
    let mut images = HostViewportImageSet::default();
    images.replace_scene_for_surface(
        "document:left",
        HostViewportImageData {
            resource_key: "scene:left".to_string(),
            resource_generation: 0,
            width: 1,
            height: 1,
            rgba: Some(vec![255, 0, 0, 255].into()),
            play_frame_identity: None,
            overlay: None,
        },
    );
    images.replace_scene_for_surface(
        "document:right",
        HostViewportImageData {
            resource_key: "scene:right".to_string(),
            resource_generation: 0,
            width: 1,
            height: 1,
            rgba: Some(vec![0, 0, 255, 255].into()),
            play_frame_identity: None,
            overlay: None,
        },
    );
    let pane = PaneData {
        kind: "Scene".into(),
        ..PaneData::default()
    };
    let body = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 10.0,
        height: 10.0,
    };
    let mut frame = HostRgbaFrame::recording_only(10, 10);
    assert!(draw_viewport_image(
        &mut frame,
        &pane,
        &body,
        &body,
        &images,
        Some("document:right"),
    ));
    let commands = frame.into_recorded_commands();
    assert_eq!(commands.len(), 1);
    assert!(matches!(
        &commands[0].kind,
        HostRecordedPaintKind::Image { resource_key, .. } if resource_key == "scene:right"
    ));
}
