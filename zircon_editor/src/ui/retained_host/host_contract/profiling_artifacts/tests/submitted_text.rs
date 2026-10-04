use super::*;
use crate::ui::retained_host::host_contract::{
    chrome_command_stream::{ChromeCommand, ChromeCommandKind, ChromeCommandLayer},
    data::FrameRect,
    paint_frame::{HostRenderCommandSource, HostRenderSourceTable},
};
use std::sync::Arc;
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    surface::{UiRenderFrameCommandRef, UiSurfaceFrame},
};
#[test]
fn submitted_source_identity_uses_actual_source_table_without_guessed_metadata() {
    let frame = Arc::new(UiSurfaceFrame {
        generation: 19,
        ..Default::default()
    });
    let mut table = HostRenderSourceTable::default();
    let key = table.register(&frame).unwrap();
    let stream = ChromeCommandStream::from_extracted_commands(
        (640, 520),
        None,
        vec![ChromeCommand {
            layer: ChromeCommandLayer::Dynamic,
            z_index: 0,
            frame: FrameRect::default(),
            clip: None,
            source: Some(HostRenderCommandSource {
                surface_key: key,
                command_ref: UiRenderFrameCommandRef::new(UiNodeId::new(31), 2),
                fragment_index: 4,
            }),
            kind: ChromeCommandKind::Clip,
        }],
        table,
    );
    let rows = source_rows(&stream, &HostWindowPresentationData::default());
    let row = rows[0].as_ref().unwrap();
    assert_eq!(row["sourceRef"]["generation"], 19);
    assert_eq!(row["sourceRef"]["nodeCommandIndex"], 2);
    assert_eq!(row["sourceRef"]["fragmentIndex"], 4);
    assert_eq!(row["sourceRef"]["authority"], "surface-node-owner");
    assert_eq!(row["sourceRef"]["primitiveIdentity"], false);
    assert!(row["propertyFieldId"].is_null());
    assert!(row["bounds"].is_null());
}
#[test]
fn submitted_binding_retains_disabled_property_identity_and_physical_lines() {
    let snapshot = UiSurfaceTextLayoutSnapshot {
        presented_frame_count: 7,
        projection_size: (960, 780),
        damage: None,
        prepared_this_present: true,
        retained_cache_copy_bytes: 0,
        draw_list_generation: None,
        runs: vec![zircon_runtime::rhi::UiSurfaceTextLayoutRun {
            command_index: 1,
            text: "Readonly".into(),
            clip: zircon_runtime::rhi::UiSurfaceRect::new(12.0, 30.0, 90.0, 30.0),
            lines: vec![zircon_runtime::rhi::UiSurfaceTextLine {
                original_line_text: "Readonly".into(),
                line_index: 0,
                byte_range: Some((0, 8)),
                frame: zircon_runtime::rhi::UiSurfaceRect::new(12.75, 31.5, 60.0, 24.0),
                baseline_y: 49.5,
                font_ids: vec![],
            }],
            faces: vec![],
        }],
    };
    let identity =
        json!({"propertyFieldId":"camera.fov","itemKey":"stable-materializer-key","disabled":true});
    let result = bind_submitted(snapshot, &[None, Some(identity.clone())], false);
    assert_eq!(result["runs"][0]["source"], identity);
    assert_eq!(result["runs"][0]["lines"][0]["frame"]["x"], 12.75);
    assert_eq!(result["paintControls"][0]["disabled"], true);
    assert_eq!(result["presentedFrameCount"], 7);
    // This observation path does not construct or modify any hit grid.
}
#[test]
fn readonly_paint_identity_rejects_foreign_tree_regeneration_and_missing_model() {
    use crate::ui::retained_host::host_contract::data::{
        TemplateNodeFrameData, TemplatePaneNodeData,
    };
    use zircon_runtime_interface::ui::{
        layout::UiFrame,
        surface::{UiArrangedNode, UiArrangedTree},
    };
    let id = UiNodeId::new(42);
    let arranged = UiArrangedNode {
        node_id: id,
        node_path: Default::default(),
        parent: None,
        children: vec![],
        frame: UiFrame::new(12.75, 30.0, 90.0, 30.0),
        clip_frame: UiFrame::new(0.0, 0.0, 960.0, 780.0),
        z_index: 0,
        paint_order: 0,
        visibility: Default::default(),
        input_policy: Default::default(),
        pointer_events: Default::default(),
        enabled: false,
        clickable: false,
        hoverable: false,
        focusable: false,
        clip_to_bounds: false,
        control_id: Some("PhysicalPoolRow7".into()),
        slot: None,
    };
    assert!(!arranged.supports_pointer());
    let frame = Arc::new(UiSurfaceFrame {
        generation: 20,
        arranged_tree: Arc::new(UiArrangedTree {
            nodes: std::iter::once(arranged).collect(),
            ..Default::default()
        }),
        ..Default::default()
    });
    let before = serde_json::to_value(frame.as_ref()).unwrap();
    let mut table = HostRenderSourceTable::default();
    let key = table.register(&frame).unwrap();
    let stream = ChromeCommandStream::from_extracted_commands(
        (960, 780),
        None,
        vec![ChromeCommand {
            layer: ChromeCommandLayer::Dynamic,
            z_index: 0,
            frame: FrameRect {
                x: 12.75,
                y: 30.0,
                width: 90.0,
                height: 30.0,
            },
            clip: None,
            source: Some(HostRenderCommandSource {
                surface_key: key,
                command_ref: UiRenderFrameCommandRef::new(id, 0),
                fragment_index: 0,
            }),
            kind: ChromeCommandKind::Clip,
        }],
        table,
    );
    let node = TemplatePaneNodeData {
        source_surface_frame: Some(Arc::clone(&frame)),
        surface_node_id: Some(id),
        control_id: "PhysicalPoolRow7".into(),
        source_path: "inspector_panel.zui".into(),
        source_node_id: "WorkbenchComponentPropertySlot04Row".into(),
        inspector_property_field_id: "camera.fov".into(),
        inspector_property_item_key: "real-key-from-materializer".into(),
        disabled: true,
        frame: TemplateNodeFrameData {
            x: 12.75,
            y: 30.0,
            width: 90.0,
            height: 30.0,
        },
        ..Default::default()
    };
    let mut presentation = HostWindowPresentationData {
        workbench_window_nodes: crate::ui::layouts::common::model_rc(vec![node.clone()]),
        ..Default::default()
    };
    let rows = source_rows(&stream, &presentation);
    assert_eq!(rows[0].as_ref().unwrap()["propertyFieldId"], "camera.fov");
    assert_eq!(
        rows[0].as_ref().unwrap()["itemKey"],
        "real-key-from-materializer"
    );
    assert_eq!(rows[0].as_ref().unwrap()["bounds"]["x"], 12.75);
    assert_eq!(rows[0].as_ref().unwrap()["disabled"], true);
    assert_eq!(before, serde_json::to_value(frame.as_ref()).unwrap());
    assert!(!frame.arranged_tree.get(id).unwrap().supports_pointer());
    let mut rebound = node.clone();
    let fresh = Arc::new(UiSurfaceFrame {
        generation: 21,
        ..(*frame).clone()
    });
    rebound.source_surface_frame = Some(Arc::clone(&fresh));
    rebound.inspector_property_field_id = "light.intensity".into();
    rebound.inspector_property_item_key = "new-authoritative-key".into();
    presentation.workbench_window_nodes = crate::ui::layouts::common::model_rc(vec![rebound]);
    assert!(source_rows(&stream, &presentation)[0].as_ref().unwrap()["itemKey"].is_null());
    let mut fresh_table = HostRenderSourceTable::default();
    let fresh_key = fresh_table.register(&fresh).unwrap();
    let fresh_stream = ChromeCommandStream::from_extracted_commands(
        (960, 780),
        None,
        vec![ChromeCommand {
            layer: ChromeCommandLayer::Dynamic,
            z_index: 0,
            frame: FrameRect::default(),
            clip: None,
            source: Some(HostRenderCommandSource {
                surface_key: fresh_key,
                command_ref: UiRenderFrameCommandRef::new(id, 0),
                fragment_index: 0,
            }),
            kind: ChromeCommandKind::Clip,
        }],
        fresh_table,
    );
    assert_eq!(
        source_rows(&fresh_stream, &presentation)[0]
            .as_ref()
            .unwrap()["propertyFieldId"],
        "light.intensity"
    );
    assert_eq!(
        source_rows(&fresh_stream, &presentation)[0]
            .as_ref()
            .unwrap()["itemKey"],
        "new-authoritative-key"
    );
    // A unique matching pool slot from another publication is not ownership proof.
    let foreign = Arc::new(UiSurfaceFrame {
        tree_id: zircon_runtime_interface::ui::event_ui::UiTreeId::new("other-tree"),
        generation: 20,
        ..(*frame).clone()
    });
    let mut wrong = node.clone();
    wrong.source_surface_frame = Some(foreign);
    presentation.workbench_window_nodes = crate::ui::layouts::common::model_rc(vec![wrong]);
    assert!(source_rows(&stream, &presentation)[0].as_ref().unwrap()["propertyFieldId"].is_null());
    let regenerated = Arc::new(UiSurfaceFrame {
        generation: 21,
        ..(*frame).clone()
    });
    let mut stale = node.clone();
    stale.source_surface_frame = Some(regenerated);
    presentation.workbench_window_nodes = crate::ui::layouts::common::model_rc(vec![stale]);
    assert!(source_rows(&stream, &presentation)[0].as_ref().unwrap()["itemKey"].is_null());
    presentation.workbench_window_nodes = Default::default();
    assert!(source_rows(&stream, &presentation)[0].as_ref().unwrap()["bounds"].is_null());
    presentation.workbench_window_nodes = crate::ui::layouts::common::model_rc(vec![node.clone()]);
    presentation.root_template_nodes = crate::ui::layouts::common::model_rc(vec![node]);
    let ambiguous = source_rows(&stream, &presentation);
    assert!(ambiguous[0].as_ref().unwrap()["propertyFieldId"].is_null());
    assert_eq!(
        ambiguous[0].as_ref().unwrap()["sourceRef"]["generation"],
        20
    );
}
#[test]
fn actual_workbench_publication_reaches_shared_painter_source_table() {
    use crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowTemplateSurfaceBridge;
    use crate::ui::retained_host::host_contract::{
        chrome_command_stream::build_chrome_command_stream, data::HostWindowPresentationData,
        profiling_artifacts::source_rows,
    };
    use crate::ui::retained_host::ui::to_host_contract_workbench_window_nodes;
    use std::sync::Arc;
    use zircon_runtime_interface::ui::layout::UiSize;
    let bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
    let projection = bridge.host_projection();
    let source = projection
        .source_surface_frame
        .as_ref()
        .expect("real bridge publication");
    let nodes = to_host_contract_workbench_window_nodes(Some(projection));
    let mapped = nodes
        .iter()
        .filter(|node| node.source_surface_frame.is_some())
        .count();
    assert!(
        mapped > 16,
        "actual authored Workbench labels and controls have real node command ownership"
    );
    for node in nodes
        .iter()
        .filter(|node| node.source_surface_frame.is_some())
    {
        assert!(Arc::ptr_eq(
            node.source_surface_frame.as_ref().unwrap(),
            source
        ));
        let owner_ref = node.surface_render_command_ref.unwrap();
        let range = source
            .render_extract
            .command_range(owner_ref.node_id)
            .unwrap();
        assert!(range.contains(&(range.start + owner_ref.node_command_index as usize)));
        assert!(source.render_extract.command_by_ref(owner_ref).is_some());
    }
    assert!(
        nodes.iter().any(|node| node
            .surface_render_command_ref
            .is_some_and(|reference| source
                .render_extract
                .command_range(reference.node_id)
                .is_some_and(|range| range.start > 0))),
        "real publication has nonzero global command range"
    );
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: nodes,
        ..Default::default()
    };
    let stream = build_chrome_command_stream(&presentation, (1280, 800), None, true);
    let rows = source_rows(&stream, &presentation);
    assert!(
        rows.iter().flatten().any(|row| row["sourcePath"]
            .as_str()
            .is_some_and(|value| !value.is_empty())),
        "production shared painter exports qualified source metadata"
    );
}
