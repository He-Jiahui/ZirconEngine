use crate::ui::{
    event_ui::{UiNodeId, UiTreeId},
    layout::UiFrame,
    surface::{
        UiBatchPlan, UiBatchPrimitive, UiBatchShader, UiBatchSplitReason, UiBrushPayload,
        UiClipMode, UiPaintPayload, UiRenderCommand, UiRenderCommandKind, UiRenderDebugSnapshot,
        UiRenderExtract, UiRenderList, UiRenderResourceKey, UiRenderResourceKind, UiResolvedStyle,
        UiResolvedTextLayout, UiResolvedTextLine, UiResolvedTextRun, UiShapedText, UiTextDirection,
        UiTextOverflow, UiTextPaint, UiTextRange, UiTextRenderMode, UiTextRunKind,
        UiVisualAssetRef,
    },
};

fn solid_command(node_id: u64, x: f32, clip_x: f32) -> UiRenderCommand {
    UiRenderCommand {
        node_id: UiNodeId::new(node_id),
        kind: UiRenderCommandKind::Quad,
        frame: UiFrame::new(x, 8.0, 48.0, 20.0),
        clip_frame: Some(UiFrame::new(clip_x, 0.0, 128.0, 64.0)),
        z_index: 2,
        style: UiResolvedStyle {
            background_color: Some("#223344".to_string()),
            border_color: Some("#778899".to_string()),
            border_width: 2.0,
            corner_radius: 6.0,
            ..UiResolvedStyle::default()
        },
        text_layout: None,
        text: None,
        image: None,
        opacity: 0.75,
    }
}

#[test]
fn ui_paint_element_derives_brush_payload_from_legacy_render_command() {
    let element = solid_command(7, 4.0, 0.0).to_paint_element(3);

    assert_eq!(element.node_id, UiNodeId::new(7));
    assert_eq!(element.geometry.absolute_frame, UiFrame::new(4.0, 8.0, 48.0, 20.0));
    assert_eq!(element.geometry.render_bounds, UiFrame::new(4.0, 8.0, 48.0, 20.0));
    assert_eq!(element.clip.as_ref().unwrap().mode, UiClipMode::Scissor);
    assert_eq!(element.z_index, 2);
    assert_eq!(element.paint_order, 3);
    assert_eq!(element.effects.opacity, 0.75);

    let UiPaintPayload::Brush { brushes } = &element.payload else {
        panic!("expected brush payload");
    };
    assert!(matches!(brushes.fill, Some(UiBrushPayload::Rounded(_))));
    assert!(matches!(brushes.border, Some(UiBrushPayload::Border(_))));

    let serialized = serde_json::to_string(&element).unwrap();
    assert!(serialized.contains("rounded"));
    assert!(serialized.contains("#223344"));
}

#[test]
fn ui_paint_element_derives_text_shape_payload_from_text_layout() {
    let layout = UiResolvedTextLayout {
        direction: UiTextDirection::LeftToRight,
        overflow: UiTextOverflow::Ellipsis,
        font_size: 18.0,
        line_height: 22.0,
        measured_width: 36.0,
        measured_height: 22.0,
        source_range: UiTextRange { start: 0, end: 4 },
        lines: vec![UiResolvedTextLine {
            text: "Zirc".to_string(),
            frame: UiFrame::new(4.0, 8.0, 36.0, 22.0),
            source_range: UiTextRange { start: 0, end: 4 },
            visual_range: UiTextRange { start: 0, end: 4 },
            measured_width: 36.0,
            baseline: 16.0,
            direction: UiTextDirection::LeftToRight,
            runs: vec![UiResolvedTextRun {
                kind: UiTextRunKind::Strong,
                text: "Zirc".to_string(),
                source_range: UiTextRange { start: 0, end: 4 },
                visual_range: UiTextRange { start: 0, end: 4 },
                direction: UiTextDirection::LeftToRight,
            }],
            ellipsized: true,
        }],
        ..UiResolvedTextLayout::default()
    };
    let command = UiRenderCommand {
        node_id: UiNodeId::new(9),
        kind: UiRenderCommandKind::Text,
        frame: UiFrame::new(4.0, 8.0, 40.0, 24.0),
        clip_frame: None,
        z_index: 5,
        style: UiResolvedStyle {
            foreground_color: Some("#ffffff".to_string()),
            font: Some("fonts/Inter.ttf".to_string()),
            font_family: Some("Inter".to_string()),
            font_size: 18.0,
            line_height: 22.0,
            text_render_mode: UiTextRenderMode::Sdf,
            ..UiResolvedStyle::default()
        },
        text_layout: Some(layout),
        text: Some("Zirc".to_string()),
        image: None,
        opacity: 1.0,
    };

    let element = command.to_paint_element(1);

    let UiPaintPayload::Text { text } = &element.payload else {
        panic!("expected text payload");
    };
    assert_eq!(text.render_mode, UiTextRenderMode::Sdf);
    assert_eq!(text.shaped.as_ref().unwrap().lines[0].clusters[0].kind, UiTextRunKind::Strong);
    assert_eq!(text.shaped.as_ref().unwrap().lines[0].clusters[0].source_range.end, 4);
}

#[test]
fn ui_brush_material_and_resource_payloads_are_serializable() {
    let image_resource = UiRenderResourceKey::new(UiRenderResourceKind::Image, "textures/ui/panel.png")
        .with_revision(4)
        .with_atlas_page(2);
    let image_brush = UiBrushPayload::image(image_resource.clone()).with_tint("#aabbcc");
    let material_brush = UiBrushPayload::material("material/ui/frosted")
        .with_material_variant("hdr")
        .with_material_revision(8)
        .with_fallback_color("#102030");

    let image_json = serde_json::to_string(&image_brush).unwrap();
    let material_json = serde_json::to_string(&material_brush).unwrap();

    assert!(image_json.contains("atlas_page"));
    assert!(image_json.contains("textures/ui/panel.png"));
    assert!(material_json.contains("material"));
    assert!(material_json.contains("frosted"));
    assert_eq!(image_resource.atlas_page, Some(2));
}

#[test]
fn ui_batch_plan_merges_same_key_and_explains_splits() {
    let commands = vec![
        solid_command(1, 0.0, 0.0),
        solid_command(2, 48.0, 0.0),
        solid_command(3, 96.0, 8.0),
    ];
    let elements = commands
        .iter()
        .enumerate()
        .map(|(index, command)| command.to_paint_element(index as u64))
        .collect::<Vec<_>>();

    let plan = UiBatchPlan::from_paint_elements(&elements);

    assert_eq!(plan.batches.len(), 2);
    assert_eq!(plan.batches[0].range.element_count, 2);
    assert_eq!(plan.batches[0].split_reason, UiBatchSplitReason::FirstBatch);
    assert_eq!(plan.batches[1].split_reason, UiBatchSplitReason::ClipChanged);
    assert_eq!(plan.batches[0].key.primitive, UiBatchPrimitive::RoundedRect);
    assert_eq!(plan.batches[0].key.shader, UiBatchShader::Color);
    assert_eq!(plan.stats.draw_call_count, 2);
}

#[test]
fn ui_render_debug_snapshot_exports_batch_replay_data() {
    let extract = UiRenderExtract {
        tree_id: UiTreeId::new("ui.render.debug"),
        list: UiRenderList {
            commands: vec![solid_command(1, 0.0, 0.0), solid_command(2, 48.0, 0.0)],
        },
    };

    let snapshot = UiRenderDebugSnapshot::from_render_extract(&extract);

    assert_eq!(snapshot.tree_id, UiTreeId::new("ui.render.debug"));
    assert_eq!(snapshot.stats.element_count, 2);
    assert_eq!(snapshot.stats.batch_count, 1);
    assert_eq!(snapshot.batches[0].node_ids, vec![UiNodeId::new(1), UiNodeId::new(2)]);
    assert!(serde_json::to_string(&snapshot).unwrap().contains("draw_call_count"));
}

#[test]
fn ui_shaped_text_contract_preserves_runs_and_ranges() {
    let layout = UiResolvedTextLayout {
        direction: UiTextDirection::Mixed,
        font_size: 14.0,
        line_height: 18.0,
        source_range: UiTextRange { start: 0, end: 5 },
        lines: vec![UiResolvedTextLine {
            text: "Hello".to_string(),
            frame: UiFrame::new(0.0, 0.0, 42.0, 18.0),
            source_range: UiTextRange { start: 0, end: 5 },
            visual_range: UiTextRange { start: 0, end: 5 },
            measured_width: 42.0,
            baseline: 13.0,
            direction: UiTextDirection::LeftToRight,
            runs: vec![UiResolvedTextRun {
                kind: UiTextRunKind::Plain,
                text: "Hello".to_string(),
                source_range: UiTextRange { start: 0, end: 5 },
                visual_range: UiTextRange { start: 0, end: 5 },
                direction: UiTextDirection::LeftToRight,
            }],
            ellipsized: false,
        }],
        ..UiResolvedTextLayout::default()
    };

    let shaped = UiShapedText::from_resolved_layout("Hello", &layout, UiTextRenderMode::Native);
    let paint = UiTextPaint::from_shaped_text(shaped.clone(), Some("#ffffff".to_string()));

    assert_eq!(shaped.source_text, "Hello");
    assert_eq!(shaped.lines[0].clusters[0].visual_range.end, 5);
    assert_eq!(paint.color.as_deref(), Some("#ffffff"));
    assert_eq!(paint.render_mode, UiTextRenderMode::Native);
}

#[test]
fn ui_image_command_derives_resource_batch_key() {
    let command = UiRenderCommand {
        node_id: UiNodeId::new(12),
        kind: UiRenderCommandKind::Image,
        frame: UiFrame::new(0.0, 0.0, 32.0, 32.0),
        clip_frame: None,
        z_index: 1,
        style: UiResolvedStyle::default(),
        text_layout: None,
        text: None,
        image: Some(UiVisualAssetRef::Icon("toolbar.save".to_string())),
        opacity: 1.0,
    };

    let element = command.to_paint_element(0);
    let plan = UiBatchPlan::from_paint_elements(&[element]);

    assert_eq!(plan.batches[0].key.primitive, UiBatchPrimitive::Image);
    assert_eq!(plan.batches[0].key.shader, UiBatchShader::Image);
    assert_eq!(
        plan.batches[0].key.resource.as_ref().unwrap().kind,
        UiRenderResourceKind::Icon
    );
}
