use super::super::super::{
    chrome_command_stream::{ChromeCommand, ChromeCommandLayer, ChromeImagePayload},
    data::{FrameRect, HostViewportImageData, TemplateNodeFrameData},
    paint_frame::{HostRenderCommandSource, HostRenderSourceTable},
};
use super::*;
use std::sync::Arc;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiTreeId},
    layout::UiFrame,
    surface::{UiArrangedNode, UiArrangedTree, UiRenderFrameCommandRef, UiSurfaceFrame},
};

fn fixture() -> (
    HostWindowPresentationData,
    ChromeCommandStream,
    Vec<Option<Value>>,
) {
    let id = UiNodeId::new(2);
    let arranged = UiArrangedNode {
        node_id: id,
        node_path: Default::default(),
        parent: None,
        children: vec![],
        frame: UiFrame::new(10.0, 20.0, 600.0, 400.0),
        clip_frame: UiFrame::new(0.0, 0.0, 800.0, 600.0),
        z_index: 0,
        paint_order: 0,
        visibility: Default::default(),
        input_policy: Default::default(),
        pointer_events: Default::default(),
        enabled: true,
        clickable: false,
        hoverable: false,
        focusable: false,
        clip_to_bounds: false,
        control_id: Some("WorkbenchViewportSurface".into()),
        slot: None,
    };
    let frame = Arc::new(UiSurfaceFrame {
        tree_id: UiTreeId::new("actual-scene-surface"),
        generation: 9,
        arranged_tree: Arc::new(UiArrangedTree {
            nodes: vec![arranged].into(),
            ..Default::default()
        }),
        ..Default::default()
    });
    let invocation = r#"[{"sourcePath":"res://ui/editor/unreal/level_editor_window.zui","sourceNodeId":"viewport"}]"#;
    let node = |name: &str, parent: &str, control: &str, authored: &str| TemplatePaneNodeData {
        node_id: name.into(),
        parent_node_id: parent.into(),
        control_id: control.into(),
        source_path: "res://ui/editor/unreal/components/viewport.zui".into(),
        source_node_id: authored.into(),
        instance_path: invocation.into(),
        frame: TemplateNodeFrameData {
            x: 10.0,
            y: 20.0,
            width: 600.0,
            height: 400.0,
        },
        ..Default::default()
    };
    let root = TemplatePaneNodeData {
        node_id: "window".into(),
        control_id: "WorkbenchWindowRoot".into(),
        frame: TemplateNodeFrameData {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
        },
        ..Default::default()
    };
    let content = TemplatePaneNodeData {
        node_id: "content".into(),
        parent_node_id: "window".into(),
        control_id: "UnrealLevelEditorContent".into(),
        ..Default::default()
    };
    let viewport = node(
        "viewport",
        "content",
        "WorkbenchMainBandViewportPanel",
        "root",
    );
    let media = TemplatePaneNodeData {
        source_surface_frame: Some(Arc::clone(&frame)),
        surface_node_id: Some(id),
        surface_render_command_ref: Some(UiRenderFrameCommandRef::new(id, 0)),
        ..node(
            "media",
            "viewport",
            "WorkbenchViewportSurface",
            "scene_media",
        )
    };
    let mut presentation = HostWindowPresentationData {
        workbench_window_nodes: crate::ui::layouts::common::model_rc(vec![
            root, content, viewport, media,
        ]),
        ..Default::default()
    };
    presentation.host_scene_data.document_dock.surface_key = "document:scene:current".into();
    presentation.host_scene_data.document_dock.pane.kind = "Scene".into();
    presentation.host_scene_data.document_dock.region_frame = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 800.0,
        height: 600.0,
    };
    presentation.viewport_images.replace_scene_for_surface(
        "document:scene:current",
        HostViewportImageData {
            resource_key: "viewport:exact:5".into(),
            resource_generation: 5,
            width: 600,
            height: 400,
            ..Default::default()
        },
    );
    let mut table = HostRenderSourceTable::default();
    let key = table.register(&frame).unwrap();
    let stream = ChromeCommandStream::from_extracted_commands(
        (800, 600),
        None,
        vec![ChromeCommand {
            layer: ChromeCommandLayer::Viewport,
            z_index: 0,
            frame: FrameRect {
                x: 10.0,
                y: 20.0,
                width: 600.0,
                height: 400.0,
            },
            clip: Some(FrameRect {
                x: 10.0,
                y: 20.0,
                width: 600.0,
                height: 400.0,
            }),
            source: Some(HostRenderCommandSource {
                surface_key: key,
                command_ref: UiRenderFrameCommandRef::new(id, 0),
                fragment_index: 0,
            }),
            kind: ChromeCommandKind::Image {
                payload: ChromeImagePayload {
                    resource_key: "viewport:exact:5".into(),
                    resource_generation: 5,
                    width: 600,
                    height: 400,
                    upload_bytes: 0,
                    rgba: None,
                    atlas_uv: None,
                },
            },
        }],
        table,
    );
    let sources = super::super::source_rows(&stream, &presentation);
    (presentation, stream, sources)
}

#[test]
fn normal_viewport_media_joins_actual_document_surface_source_and_gpu_command() {
    let (presentation, stream, sources) = fixture();
    let before = serde_json::to_value(
        presentation
            .workbench_window_nodes
            .iter()
            .find(|node| node.node_id.as_str() == "media")
            .unwrap()
            .source_surface_frame
            .as_ref()
            .unwrap()
            .as_ref(),
    )
    .unwrap();
    let selected = scene_image_selection(&presentation, &stream, &sources)
        .unwrap()
        .unwrap();
    assert_eq!(selected.request.resource_key, "viewport:exact:5");
    assert_eq!(selected.request.physical_size, (600, 400));
    assert_eq!(selected.evidence["owner"]["sourceNodeId"], "scene_media");
    assert_eq!(selected.evidence["surfaceKey"], "document:scene:current");
    assert_eq!(selected.evidence["sourceRef"]["generation"], 9);
    assert_eq!(selected.evidence["commandBounds"]["x"], 10.0);
    assert_eq!(
        selected.evidence["owner"]["instancePath"],
        sources[0].as_ref().unwrap()["instancePath"]
    );
    let draw =
        super::super::super::chrome_command_stream::ui_surface_draw_list_from_stream(&stream);
    assert!(selected.matches_draw_list(&draw));
    assert_eq!(
        before,
        serde_json::to_value(
            presentation
                .workbench_window_nodes
                .iter()
                .find(|node| node.node_id.as_str() == "media")
                .unwrap()
                .source_surface_frame
                .as_ref()
                .unwrap()
                .as_ref()
        )
        .unwrap()
    );
}

#[test]
fn normal_viewport_media_rejects_stale_or_partial_source_evidence_and_duplicates() {
    let (presentation, stream, sources) = fixture();
    for key in ["sourcePath", "sourceNodeId", "instancePath"] {
        let mut stale = sources.clone();
        stale[0].as_mut().unwrap()[key] = json!("foreign");
        assert!(
            scene_image_selection(&presentation, &stream, &stale).is_err(),
            "{key}"
        );
    }
    let mut stale = sources.clone();
    stale[0].as_mut().unwrap()["sourceRef"]["generation"] = json!(8);
    assert!(scene_image_selection(&presentation, &stream, &stale).is_err());
    let mut duplicate = stream.clone();
    duplicate.push_command_for_test(stream.commands()[0].clone());
    let repeated = super::super::source_rows(&duplicate, &presentation);
    assert!(scene_image_selection(&presentation, &duplicate, &repeated).is_err());
    let mut nodes = presentation
        .workbench_window_nodes
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    let mut duplicate = nodes
        .iter()
        .find(|node| node.node_id.as_str() == "media")
        .unwrap()
        .clone();
    duplicate.node_id = "another-physical-media".into();
    nodes.push(duplicate);
    let data = HostWindowPresentationData {
        workbench_window_nodes: crate::ui::layouts::common::model_rc(nodes),
        ..presentation
    };
    assert!(scene_image_selection(&data, &stream, &sources).is_err());
}

#[test]
fn normal_viewport_media_does_not_replace_scene_with_overlay_neighbor_or_cpu_pixels() {
    let (mut presentation, mut stream, _) = fixture();
    // Another actual command may use the same resource, but has no matching scene source.
    let mut outside = stream.commands()[0].clone();
    outside.source = None;
    stream.push_command_for_test(outside);
    let mut overlay = stream.commands()[0].clone();
    if let ChromeCommandKind::Image { payload } = &mut overlay.kind {
        payload.resource_key = "viewport:overlay".into();
    }
    stream.push_command_for_test(overlay);
    let sources = super::super::source_rows(&stream, &presentation);
    assert_eq!(
        scene_image_selection(&presentation, &stream, &sources)
            .unwrap()
            .unwrap()
            .request
            .command_index,
        0
    );
    let mut neighbor = presentation.host_scene_data.document_dock.clone();
    neighbor.surface_key = "neighbor".into();
    neighbor.region_frame.x = 900.0;
    presentation.host_scene_data.document_leaves = vec![neighbor];
    assert!(scene_image_selection(&presentation, &stream, &sources)
        .unwrap()
        .is_none());
    presentation.host_scene_data.document_leaves.clear();
    presentation.viewport_images.replace_scene_for_surface(
        "document:scene:current",
        HostViewportImageData {
            resource_key: "cpu-scene".into(),
            width: 600,
            height: 400,
            rgba: Some(Arc::from(vec![255; 600 * 400 * 4])),
            ..Default::default()
        },
    );
    assert!(scene_image_selection(&presentation, &stream, &sources).is_err());
}

#[test]
fn normal_viewport_media_rejects_changed_post_conversion_command() {
    let (presentation, stream, sources) = fixture();
    let selected = scene_image_selection(&presentation, &stream, &sources)
        .unwrap()
        .unwrap();
    let draw =
        super::super::super::chrome_command_stream::ui_surface_draw_list_from_stream(&stream);
    for mutation in 0..7 {
        let mut changed = draw.clone();
        match mutation {
            0 => changed.commands[0].frame.x += 1.0,
            1 => changed.commands[0].clip = None,
            _ => {
                if let zircon_runtime::rhi::UiSurfaceCommandKind::Image { payload } =
                    &mut changed.commands[0].kind
                {
                    match mutation {
                        2 => payload.resource_key = "other".into(),
                        3 => payload.resource_generation += 1,
                        4 => payload.width += 1,
                        5 => payload.rgba = Some(vec![]),
                        _ => {
                            payload.atlas_uv = Some(zircon_runtime::rhi::UiSurfaceImageUvRect {
                                min: [0.0, 0.0],
                                max: [0.5, 1.0],
                            })
                        }
                    }
                }
            }
        }
        assert!(!selected.matches_draw_list(&changed), "mutation {mutation}");
    }
}
