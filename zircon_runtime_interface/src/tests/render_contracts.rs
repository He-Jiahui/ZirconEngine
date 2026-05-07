use crate::ui::{
    event_ui::{UiNodeId, UiTreeId},
    layout::{UiFrame, UiLayoutMetrics, UiMargin, UiPixelSnapping},
    surface::{
        UiBatchPlan, UiBatchPrimitive, UiBatchShader, UiBatchSplitReason, UiBrushPayload,
        UiClipMode, UiEditableTextState, UiPaintPayload, UiRenderCacheInvalidationReason,
        UiRenderCachePlan, UiRenderCacheStatus, UiRenderCommand, UiRenderCommandKind,
        UiRenderDebugSnapshot, UiRenderExtract, UiRenderList, UiRenderResourceKey,
        UiRenderResourceKind, UiRenderResourceState, UiRenderVisualizerOverlayKind,
        UiRenderVisualizerSnapshot, UiRendererParityPayloadKind, UiRendererParitySnapshot,
        UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine, UiResolvedTextRun,
        UiResourceUvRect, UiShapedGlyph, UiShapedText, UiTextCaret, UiTextCaretAffinity,
        UiTextComposition, UiTextDirection, UiTextOverflow, UiTextPaint, UiTextPaintDecoration,
        UiTextPaintDecorationKind, UiTextRange, UiTextRenderMode, UiTextRunKind, UiTextSelection,
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

fn parity_extract_fixture(name: &str, commands: Vec<UiRenderCommand>) -> UiRenderExtract {
    UiRenderExtract {
        tree_id: UiTreeId::new(format!("ui.render.parity.{name}")),
        list: UiRenderList { commands },
    }
}

fn image_command(node_id: u64, x: f32, resource_id: &str) -> UiRenderCommand {
    UiRenderCommand {
        node_id: UiNodeId::new(node_id),
        kind: UiRenderCommandKind::Image,
        frame: UiFrame::new(x, 8.0, 32.0, 24.0),
        clip_frame: Some(UiFrame::new(0.0, 0.0, 160.0, 80.0)),
        z_index: 4,
        style: UiResolvedStyle::default(),
        text_layout: None,
        text: None,
        image: Some(UiVisualAssetRef::Icon(resource_id.to_string())),
        opacity: 1.0,
    }
}

fn text_command(node_id: u64, x: f32, mode: UiTextRenderMode) -> UiRenderCommand {
    UiRenderCommand {
        node_id: UiNodeId::new(node_id),
        kind: UiRenderCommandKind::Text,
        frame: UiFrame::new(x, 8.0, 56.0, 24.0),
        clip_frame: None,
        z_index: 5,
        style: UiResolvedStyle {
            foreground_color: Some("#ffffff".to_string()),
            text_render_mode: mode,
            ..UiResolvedStyle::default()
        },
        text_layout: None,
        text: Some("Parity".to_string()),
        image: None,
        opacity: 1.0,
    }
}

#[test]
fn ui_paint_element_derives_brush_payload_from_legacy_render_command() {
    let element = solid_command(7, 4.0, 0.0).to_paint_element(3);

    assert_eq!(element.node_id, UiNodeId::new(7));
    assert_eq!(
        element.geometry.absolute_frame,
        UiFrame::new(4.0, 8.0, 48.0, 20.0)
    );
    assert_eq!(
        element.geometry.render_bounds,
        UiFrame::new(4.0, 8.0, 48.0, 20.0)
    );
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
fn ui_icon_paint_payload_carries_frame_and_dpi_target_size_for_vector_cache() {
    let command = UiRenderCommand {
        node_id: UiNodeId::new(77),
        kind: UiRenderCommandKind::Image,
        frame: UiFrame::new(4.0, 8.0, 32.0, 24.0),
        clip_frame: None,
        z_index: 4,
        style: UiResolvedStyle::default(),
        text_layout: None,
        text: None,
        image: Some(UiVisualAssetRef::Icon(
            "ionicons/options-outline.svg".to_string(),
        )),
        opacity: 1.0,
    };

    let element = command.to_paint_element_with_metrics(
        2,
        UiLayoutMetrics {
            dpi_scale: 1.5,
            pixel_snapping: UiPixelSnapping::Enabled,
            ..UiLayoutMetrics::default()
        },
    );

    let UiPaintPayload::Brush { brushes } = &element.payload else {
        panic!("expected brush payload");
    };
    let Some(UiBrushPayload::Image(payload)) = &brushes.fill else {
        panic!("expected image brush");
    };

    assert_eq!(payload.resource.kind, UiRenderResourceKind::Icon);
    assert_eq!(payload.resource.id, "ionicons/options-outline.svg");
    assert_eq!(payload.resource_state.pixel_size, Some((48.0, 36.0)));
    assert_eq!(
        element.geometry.absolute_frame,
        UiFrame::new(4.0, 8.0, 32.0, 24.0)
    );
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
    assert_eq!(
        text.shaped.as_ref().unwrap().lines[0].clusters[0].kind,
        UiTextRunKind::Strong
    );
    assert_eq!(
        text.shaped.as_ref().unwrap().lines[0].clusters[0]
            .source_range
            .end,
        4
    );
    assert_eq!(
        text.shaped
            .as_ref()
            .unwrap()
            .font_key
            .as_ref()
            .unwrap()
            .kind,
        UiRenderResourceKind::Font
    );
    assert_eq!(
        text.shaped.as_ref().unwrap().font_key.as_ref().unwrap().id,
        "fonts/Inter.ttf"
    );
    assert_eq!(
        text.shaped
            .as_ref()
            .unwrap()
            .atlas_resource
            .as_ref()
            .unwrap()
            .kind,
        UiRenderResourceKind::Texture
    );
    assert_eq!(
        text.shaped.as_ref().unwrap().lines[0].glyphs[0]
            .atlas_resource
            .as_ref()
            .unwrap()
            .kind,
        UiRenderResourceKind::Texture
    );
    assert_eq!(text.runs.len(), 1);
    assert_eq!(text.runs[0].kind, UiTextRunKind::Strong);
    assert_eq!(text.runs[0].text, "Zirc");
    assert_eq!(text.runs[0].frame, UiFrame::new(4.0, 8.0, 36.0, 22.0));
    assert!(text.runs[0].style.strong);
}

#[test]
fn ui_text_paint_carries_editable_caret_selection_and_composition() {
    let editable = UiEditableTextState {
        text: "Hello".to_string(),
        caret: UiTextCaret {
            offset: 4,
            affinity: UiTextCaretAffinity::Downstream,
        },
        selection: Some(UiTextSelection {
            anchor: 1,
            focus: 3,
        }),
        composition: Some(UiTextComposition {
            range: UiTextRange { start: 2, end: 4 },
            text: "ll".to_string(),
            restore_text: None,
        }),
        read_only: false,
    };
    let layout = UiResolvedTextLayout {
        direction: UiTextDirection::LeftToRight,
        overflow: UiTextOverflow::Clip,
        font_size: 10.0,
        line_height: 12.0,
        measured_width: 50.0,
        measured_height: 12.0,
        source_range: UiTextRange { start: 0, end: 5 },
        editable: Some(editable),
        lines: vec![UiResolvedTextLine {
            text: "Hello".to_string(),
            frame: UiFrame::new(10.0, 20.0, 50.0, 12.0),
            source_range: UiTextRange { start: 0, end: 5 },
            visual_range: UiTextRange { start: 0, end: 5 },
            measured_width: 50.0,
            baseline: 8.0,
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
    let command = UiRenderCommand {
        node_id: UiNodeId::new(11),
        kind: UiRenderCommandKind::Text,
        frame: UiFrame::new(10.0, 20.0, 50.0, 12.0),
        clip_frame: None,
        z_index: 5,
        style: UiResolvedStyle {
            font_size: 10.0,
            line_height: 12.0,
            ..UiResolvedStyle::default()
        },
        text_layout: Some(layout),
        text: Some("Hello".to_string()),
        image: None,
        opacity: 1.0,
    };

    let element = command.to_paint_element(1);

    let UiPaintPayload::Text { text } = &element.payload else {
        panic!("expected text payload");
    };
    assert_eq!(text.caret.as_ref().unwrap().offset, 4);
    assert_eq!(
        text.selection.as_ref().unwrap().range(),
        UiTextRange { start: 1, end: 3 }
    );
    assert_eq!(
        text.composition.as_ref().unwrap().range,
        UiTextRange { start: 2, end: 4 }
    );
    assert!(text.decorations.iter().any(|decoration| decoration.kind
        == UiTextPaintDecorationKind::Selection
        && decoration.frame == UiFrame::new(20.0, 20.0, 20.0, 12.0)));
    assert!(text.decorations.iter().any(|decoration| decoration.kind
        == UiTextPaintDecorationKind::CompositionUnderline
        && decoration.frame == UiFrame::new(30.0, 30.0, 20.0, 2.0)));
    assert!(text.decorations.iter().any(|decoration| decoration.kind
        == UiTextPaintDecorationKind::Caret
        && decoration.frame == UiFrame::new(50.0, 20.0, 1.0, 12.0)));
}

#[test]
fn ui_brush_material_and_resource_payloads_are_serializable() {
    let image_resource =
        UiRenderResourceKey::new(UiRenderResourceKind::Image, "textures/ui/panel.png")
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
    assert_eq!(
        plan.batches[1].split_reason,
        UiBatchSplitReason::ClipChanged
    );
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
    assert_eq!(
        snapshot.batches[0].node_ids,
        vec![UiNodeId::new(1), UiNodeId::new(2)]
    );
    assert!(serde_json::to_string(&snapshot)
        .unwrap()
        .contains("draw_call_count"));
}

#[test]
fn ui_renderer_parity_snapshot_exports_canonical_paint_and_batch_rows() {
    let extract = UiRenderExtract {
        tree_id: UiTreeId::new("ui.render.parity"),
        list: UiRenderList {
            commands: vec![
                solid_command(91, 0.0, 0.0),
                UiRenderCommand {
                    node_id: UiNodeId::new(92),
                    kind: UiRenderCommandKind::Image,
                    frame: UiFrame::new(56.0, 8.0, 32.0, 20.0),
                    clip_frame: Some(UiFrame::new(0.0, 0.0, 128.0, 64.0)),
                    z_index: 2,
                    style: UiResolvedStyle::default(),
                    text_layout: None,
                    text: None,
                    image: Some(UiVisualAssetRef::Icon("toolbar.save".to_string())),
                    opacity: 1.0,
                },
                UiRenderCommand {
                    node_id: UiNodeId::new(93),
                    kind: UiRenderCommandKind::Text,
                    frame: UiFrame::new(96.0, 8.0, 48.0, 20.0),
                    clip_frame: None,
                    z_index: 3,
                    style: UiResolvedStyle {
                        foreground_color: Some("#ffffff".to_string()),
                        text_render_mode: UiTextRenderMode::Sdf,
                        ..UiResolvedStyle::default()
                    },
                    text_layout: None,
                    text: Some("Save".to_string()),
                    image: None,
                    opacity: 1.0,
                },
            ],
        },
    };

    let parity = UiRendererParitySnapshot::from_render_extract(&extract);

    assert_eq!(parity.paint_order.len(), 3);
    assert_eq!(parity.batch_order.len(), 3);
    assert_eq!(parity.paint_order[0].node_id, UiNodeId::new(91));
    assert_eq!(
        parity.paint_order[0].payload_kind,
        UiRendererParityPayloadKind::Brush
    );
    assert!(parity.paint_order[0].clip_key.is_some());
    assert_eq!(
        parity.paint_order[1].resource.as_ref().unwrap().kind,
        UiRenderResourceKind::Icon
    );
    assert_eq!(
        parity.paint_order[2].payload_kind,
        UiRendererParityPayloadKind::Text
    );
    assert_eq!(
        parity.paint_order[2].text_render_mode,
        Some(UiTextRenderMode::Sdf)
    );
    assert_eq!(parity.batch_order[1].batch_index, 1);
    assert_eq!(parity.stats.paint_element_count, 3);
    assert_eq!(parity.stats.resource_bound_paint_count, 1);
    assert_eq!(parity.stats.text_paint_count, 1);
    assert_eq!(parity.stats.clipped_paint_count, 2);
    assert!(serde_json::to_string(&parity)
        .unwrap()
        .contains("batch_key"));
}

#[test]
fn ui_renderer_parity_fixture_tiers_compare_semantic_rows_before_pixels() {
    let extract = parity_extract_fixture(
        "tiers",
        vec![
            solid_command(101, 0.0, 0.0),
            image_command(102, 56.0, "toolbar.open"),
            text_command(103, 96.0, UiTextRenderMode::Sdf),
        ],
    );

    let expected = UiRendererParitySnapshot::from_render_extract(&extract);
    let actual = UiRendererParitySnapshot::from_render_extract(&extract);

    assert_eq!(actual, expected);
    assert_eq!(actual.paint_order.len(), 3);
    assert_eq!(actual.batch_order.len(), 3);
    assert_eq!(actual.paint_order[0].node_id, UiNodeId::new(101));
    assert_eq!(
        actual.paint_order[0].payload_kind,
        UiRendererParityPayloadKind::Brush
    );
    assert!(actual.paint_order[0].clip_key.is_some());
    assert_eq!(
        actual.paint_order[1].resource.as_ref().unwrap().kind,
        UiRenderResourceKind::Icon
    );
    assert_eq!(
        actual.paint_order[2].text_render_mode,
        Some(UiTextRenderMode::Sdf)
    );
    assert_eq!(actual.stats.paint_element_count, 3);
    assert_eq!(actual.stats.resource_bound_paint_count, 1);
    assert_eq!(actual.stats.text_paint_count, 1);
    let json = serde_json::to_string(&actual).unwrap();
    assert!(json.contains("paint_order"));
    assert!(json.contains("batch_order"));
}

#[test]
fn ui_renderer_parity_fixture_preserves_material_resource_identity() {
    let fallback = UiRenderResourceKey::new(UiRenderResourceKind::Texture, "ui/fallback-material")
        .with_revision(3);
    let mut element = solid_command(111, 0.0, 0.0).to_paint_element(0);
    element.payload = UiPaintPayload::Brush {
        brushes: crate::ui::surface::UiBrushSet {
            fill: Some(
                UiBrushPayload::material("material/ui/parity-panel")
                    .with_material_variant("hdr")
                    .with_material_revision(17)
                    .with_fallback_resource(fallback.clone())
                    .with_fallback_color("#102030"),
            ),
            border: None,
        },
    };
    let elements = vec![element];
    let plan = UiBatchPlan::from_paint_elements(&elements);
    let snapshot = UiRendererParitySnapshot::from_paint_elements_batches(
        UiTreeId::new("ui.render.parity.material"),
        &elements,
        &plan,
    );
    let resource = snapshot.paint_order[0].resource.as_ref().unwrap();

    assert_eq!(resource.kind, UiRenderResourceKind::Material);
    assert_eq!(resource.id, "material/ui/parity-panel#hdr");
    assert_eq!(resource.revision, Some(17));
    assert_eq!(resource.fallback.as_deref(), Some(&fallback));
    assert_eq!(snapshot.batch_order[0].resource.as_ref(), Some(resource));
}

#[test]
fn ui_render_debug_snapshot_carries_renderer_parity_contract() {
    let extract = UiRenderExtract {
        tree_id: UiTreeId::new("ui.render.debug.parity"),
        list: UiRenderList {
            commands: vec![solid_command(94, 0.0, 0.0)],
        },
    };

    let snapshot = UiRenderDebugSnapshot::from_render_extract(&extract);

    assert_eq!(
        snapshot.parity.paint_order.len(),
        snapshot.stats.element_count
    );
    assert_eq!(
        snapshot.parity.batch_order.len(),
        snapshot.stats.batch_count
    );
    assert_eq!(
        snapshot.parity.paint_order[0].batch_key,
        snapshot.batches[0].key
    );
    let minimal: UiRenderDebugSnapshot = serde_json::from_str(
        r#"{"tree_id":"legacy","stats":{"element_count":0,"batch_count":0,"draw_call_count":0},"batches":[]}"#,
    )
    .unwrap();
    assert!(minimal.parity.paint_order.is_empty());
}

#[test]
fn ui_render_visualizer_snapshot_exports_paint_batch_overlay_and_overdraw_data() {
    let extract = UiRenderExtract {
        tree_id: UiTreeId::new("ui.render.visualizer"),
        list: UiRenderList {
            commands: vec![solid_command(61, 0.0, 0.0), solid_command(62, 24.0, 0.0)],
        },
    };

    let snapshot = UiRenderDebugSnapshot::from_render_extract(&extract);

    assert_eq!(snapshot.visualizer.paint_elements.len(), 2);
    assert_eq!(snapshot.visualizer.batch_groups.len(), 1);
    assert_eq!(snapshot.visualizer.stats.paint_element_count, 2);
    assert_eq!(snapshot.visualizer.stats.overdraw_region_count, 1);
    assert!(snapshot
        .visualizer
        .overlays
        .iter()
        .any(|overlay| overlay.kind == UiRenderVisualizerOverlayKind::Wireframe));
    assert!(snapshot
        .visualizer
        .overlays
        .iter()
        .any(|overlay| overlay.kind == UiRenderVisualizerOverlayKind::ClipScissor));
    assert!(snapshot
        .visualizer
        .overlays
        .iter()
        .any(|overlay| overlay.kind == UiRenderVisualizerOverlayKind::OverdrawHeat));
    assert!(serde_json::to_string(&snapshot.visualizer)
        .unwrap()
        .contains("paint_elements"));
}

#[test]
fn ui_render_visualizer_snapshot_tracks_material_resource_and_sdf_text_stats() {
    let mut material_element = solid_command(71, 0.0, 0.0).to_paint_element(0);
    material_element.payload = UiPaintPayload::Brush {
        brushes: crate::ui::surface::UiBrushSet {
            fill: Some(
                UiBrushPayload::material("material/ui/debug-panel")
                    .with_material_variant("hdr")
                    .with_material_revision(3),
            ),
            border: None,
        },
    };
    let mut shaped = UiShapedText::from_resolved_layout(
        "A",
        &UiResolvedTextLayout {
            direction: UiTextDirection::LeftToRight,
            font_size: 16.0,
            line_height: 20.0,
            source_range: UiTextRange { start: 0, end: 1 },
            lines: vec![UiResolvedTextLine {
                text: "A".to_string(),
                frame: UiFrame::new(4.0, 6.0, 12.0, 20.0),
                source_range: UiTextRange { start: 0, end: 1 },
                visual_range: UiTextRange { start: 0, end: 1 },
                measured_width: 12.0,
                baseline: 15.0,
                direction: UiTextDirection::LeftToRight,
                runs: vec![UiResolvedTextRun {
                    kind: UiTextRunKind::Plain,
                    text: "A".to_string(),
                    source_range: UiTextRange { start: 0, end: 1 },
                    visual_range: UiTextRange { start: 0, end: 1 },
                    direction: UiTextDirection::LeftToRight,
                }],
                ellipsized: false,
            }],
            ..UiResolvedTextLayout::default()
        },
        UiTextRenderMode::Sdf,
    );
    let atlas = UiRenderResourceKey::new(UiRenderResourceKind::Font, "font-atlas/debug")
        .with_revision(5)
        .with_atlas_page(1);
    shaped.font_key = Some(UiRenderResourceKey::new(
        UiRenderResourceKind::Font,
        "res://fonts/debug.font.toml",
    ));
    shaped.atlas_resource = Some(atlas.clone());
    shaped.lines[0].glyphs = vec![UiShapedGlyph::new(
        65,
        UiTextRange { start: 0, end: 1 },
        UiFrame::new(4.0, 6.0, 10.0, 16.0),
        12.0,
    )
    .with_atlas(atlas, UiResourceUvRect::new(0.0, 0.0, 0.125, 0.125))];
    let mut text_element = solid_command(72, 52.0, 0.0).to_paint_element(1);
    text_element.payload = UiPaintPayload::Text {
        text: UiTextPaint::from_shaped_text(shaped, Some("#ffffff".to_string())),
    };

    let elements = vec![material_element, text_element];
    let batch_plan = UiBatchPlan::from_paint_elements(&elements);
    let cache_plan = UiRenderCachePlan::from_paint_elements_and_batches(
        14,
        &elements,
        &batch_plan,
        UiRenderCacheInvalidationReason::ResourceRevisionChanged,
    );
    let visualizer = UiRenderVisualizerSnapshot::from_paint_elements_batches_cache(
        &elements,
        &batch_plan,
        &cache_plan,
    );

    assert_eq!(visualizer.stats.material_batch_count, 1);
    assert_eq!(visualizer.text.sdf_text_count, 1);
    assert_eq!(visualizer.text.glyph_count, 1);
    assert!(visualizer.resource_bindings.len() >= 3);
    assert!(visualizer
        .overlays
        .iter()
        .any(|overlay| overlay.kind == UiRenderVisualizerOverlayKind::TextGlyphBounds));
    assert!(visualizer
        .overlays
        .iter()
        .any(|overlay| overlay.kind == UiRenderVisualizerOverlayKind::ResourceAtlas));
}

#[test]
fn ui_render_visualizer_overdraw_respects_clip_and_counts_stacked_elements() {
    let mut first = solid_command(81, 0.0, 0.0).to_paint_element(0);
    first.clip = None;
    first.geometry.clip_frame = None;
    let mut second = solid_command(82, 8.0, 0.0).to_paint_element(1);
    second.clip = None;
    second.geometry.clip_frame = None;
    let mut third = solid_command(83, 16.0, 0.0).to_paint_element(2);
    third.clip = None;
    third.geometry.clip_frame = None;
    let mut clipped_away = solid_command(84, 0.0, 96.0).to_paint_element(3);
    clipped_away.clip = Some(crate::ui::surface::UiClipState {
        mode: UiClipMode::Scissor,
        frame: UiFrame::new(128.0, 0.0, 32.0, 32.0),
    });
    clipped_away.geometry.clip_frame = Some(UiFrame::new(128.0, 0.0, 32.0, 32.0));

    let elements = vec![first, second, third, clipped_away];
    let batch_plan = UiBatchPlan::from_paint_elements(&elements);
    let cache_plan = UiRenderCachePlan::from_paint_elements_and_batches(
        15,
        &elements,
        &batch_plan,
        UiRenderCacheInvalidationReason::Unchanged,
    );
    let visualizer = UiRenderVisualizerSnapshot::from_paint_elements_batches_cache(
        &elements,
        &batch_plan,
        &cache_plan,
    );

    assert!(visualizer
        .overdraw_regions
        .iter()
        .all(|region| !region.node_ids.contains(&UiNodeId::new(84))));
    assert!(visualizer.overdraw_regions.iter().any(|region| {
        region.paint_count == 3
            && region.node_ids == vec![UiNodeId::new(81), UiNodeId::new(82), UiNodeId::new(83)]
    }));
}

#[test]
fn ui_render_visualizer_snapshot_deserializes_missing_fields_as_defaults() {
    let visualizer: UiRenderVisualizerSnapshot =
        serde_json::from_str(r#"{"paint_elements":[]}"#).unwrap();

    assert!(visualizer.batch_groups.is_empty());
    assert_eq!(visualizer.stats.paint_element_count, 0);
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
fn ui_shaped_text_contract_derives_grapheme_glyph_bounds() {
    let accent = "a\u{0301}";
    let emoji = "\u{1f469}\u{200d}\u{1f4bb}";
    let source = format!("{accent}{emoji}b");
    let layout = UiResolvedTextLayout {
        direction: UiTextDirection::LeftToRight,
        font_size: 16.0,
        line_height: 20.0,
        source_range: UiTextRange {
            start: 0,
            end: source.len(),
        },
        lines: vec![UiResolvedTextLine {
            text: source.clone(),
            frame: UiFrame::new(4.0, 6.0, 90.0, 20.0),
            source_range: UiTextRange {
                start: 0,
                end: source.len(),
            },
            visual_range: UiTextRange {
                start: 0,
                end: source.len(),
            },
            measured_width: 90.0,
            baseline: 14.0,
            direction: UiTextDirection::LeftToRight,
            runs: vec![UiResolvedTextRun {
                kind: UiTextRunKind::Plain,
                text: source.clone(),
                source_range: UiTextRange {
                    start: 0,
                    end: source.len(),
                },
                visual_range: UiTextRange {
                    start: 0,
                    end: source.len(),
                },
                direction: UiTextDirection::LeftToRight,
            }],
            ellipsized: false,
        }],
        ..UiResolvedTextLayout::default()
    };

    let shaped =
        UiShapedText::from_resolved_layout(source.clone(), &layout, UiTextRenderMode::Native);
    let glyphs = &shaped.lines[0].glyphs;

    assert_eq!(glyphs.len(), 3);
    assert_eq!(
        glyphs[0].source_range,
        UiTextRange {
            start: 0,
            end: accent.len()
        }
    );
    assert_eq!(
        glyphs[1].source_range,
        UiTextRange {
            start: accent.len(),
            end: accent.len() + emoji.len()
        }
    );
    assert_eq!(glyphs[1].visual_frame, UiFrame::new(34.0, 6.0, 30.0, 20.0));
    assert_eq!(glyphs[2].advance, 30.0);
    assert!(glyphs.iter().all(|glyph| glyph.glyph_id != 0));
}

#[test]
fn ui_text_decorations_snap_to_grapheme_cluster_edges() {
    let accent = "a\u{0301}";
    let source = format!("{accent}b");
    let layout = UiResolvedTextLayout {
        direction: UiTextDirection::LeftToRight,
        font_size: 10.0,
        line_height: 12.0,
        source_range: UiTextRange {
            start: 0,
            end: source.len(),
        },
        editable: Some(UiEditableTextState {
            text: source.clone(),
            caret: UiTextCaret {
                offset: accent.len(),
                affinity: UiTextCaretAffinity::Downstream,
            },
            selection: Some(UiTextSelection {
                anchor: 1,
                focus: accent.len(),
            }),
            composition: Some(UiTextComposition {
                range: UiTextRange {
                    start: 1,
                    end: accent.len(),
                },
                text: "\u{0301}".to_string(),
                restore_text: None,
            }),
            read_only: false,
        }),
        lines: vec![UiResolvedTextLine {
            text: source.clone(),
            frame: UiFrame::new(0.0, 0.0, 30.0, 12.0),
            source_range: UiTextRange {
                start: 0,
                end: source.len(),
            },
            visual_range: UiTextRange {
                start: 0,
                end: source.len(),
            },
            measured_width: 30.0,
            baseline: 8.0,
            direction: UiTextDirection::LeftToRight,
            runs: vec![UiResolvedTextRun {
                kind: UiTextRunKind::Plain,
                text: source.clone(),
                source_range: UiTextRange {
                    start: 0,
                    end: source.len(),
                },
                visual_range: UiTextRange {
                    start: 0,
                    end: source.len(),
                },
                direction: UiTextDirection::LeftToRight,
            }],
            ellipsized: false,
        }],
        ..UiResolvedTextLayout::default()
    };
    let command = UiRenderCommand {
        node_id: UiNodeId::new(74),
        kind: UiRenderCommandKind::Text,
        frame: UiFrame::new(0.0, 0.0, 30.0, 12.0),
        clip_frame: None,
        z_index: 0,
        style: UiResolvedStyle {
            font_size: 10.0,
            line_height: 12.0,
            ..UiResolvedStyle::default()
        },
        text_layout: Some(layout),
        text: Some(source),
        image: None,
        opacity: 1.0,
    };

    let element = command.to_paint_element(0);
    let UiPaintPayload::Text { text } = element.payload else {
        panic!("expected text payload");
    };

    assert!(text.decorations.iter().any(|decoration| decoration.kind
        == UiTextPaintDecorationKind::Selection
        && decoration.frame == UiFrame::new(0.0, 0.0, 15.0, 12.0)));
    assert!(text.decorations.iter().any(|decoration| decoration.kind
        == UiTextPaintDecorationKind::CompositionUnderline
        && decoration.frame == UiFrame::new(0.0, 10.0, 15.0, 2.0)));
    assert!(text.decorations.iter().any(|decoration| decoration.kind
        == UiTextPaintDecorationKind::Caret
        && decoration.frame == UiFrame::new(15.0, 0.0, 1.0, 12.0)));
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

#[test]
fn ui_resource_state_carries_atlas_uv_revision_and_fallback() {
    let primary = UiRenderResourceKey::new(UiRenderResourceKind::Image, "textures/ui/panel.png")
        .with_revision(4)
        .with_atlas_page(2)
        .with_uv_rect(UiResourceUvRect::new(0.25, 0.0, 0.5, 0.5));
    let fallback = UiRenderResourceKey::new(UiRenderResourceKind::Texture, "ui/fallback-white");
    let brush = UiBrushPayload::image(primary.clone())
        .with_image_size(128.0, 64.0)
        .with_tint("#aabbcc")
        .with_fallback_resource(fallback.clone());

    let UiBrushPayload::Image(image) = brush else {
        panic!("expected image brush");
    };

    assert_eq!(
        image.resource.uv_rect,
        Some(UiResourceUvRect::new(0.25, 0.0, 0.5, 0.5))
    );
    assert_eq!(image.resource_state.revision, Some(4));
    assert_eq!(image.resource_state.atlas_page, Some(2));
    assert_eq!(
        image.resource_state.uv_rect,
        Some(UiResourceUvRect::new(0.25, 0.0, 0.5, 0.5))
    );
    assert_eq!(image.resource_state.pixel_size, Some((128.0, 64.0)));
    assert_eq!(image.resource_state.fallback.as_ref(), Some(&fallback));

    let serialized = serde_json::to_string(&image).unwrap();
    assert!(serialized.contains("fallback"));
    assert!(serialized.contains("uv_rect"));
}

#[test]
fn ui_batch_plan_splits_when_resource_revision_or_uv_changes() {
    let mut first = solid_command(21, 0.0, 0.0).to_paint_element(0);
    let mut second = solid_command(22, 48.0, 0.0).to_paint_element(1);
    let image_a = UiRenderResourceKey::new(UiRenderResourceKind::Image, "textures/ui/panel.png")
        .with_revision(1)
        .with_atlas_page(0)
        .with_uv_rect(UiResourceUvRect::new(0.0, 0.0, 0.5, 0.5));
    let image_b = UiRenderResourceKey::new(UiRenderResourceKind::Image, "textures/ui/panel.png")
        .with_revision(2)
        .with_atlas_page(0)
        .with_uv_rect(UiResourceUvRect::new(0.5, 0.0, 0.5, 0.5));
    first.payload = UiPaintPayload::Brush {
        brushes: crate::ui::surface::UiBrushSet {
            fill: Some(UiBrushPayload::image(image_a.clone())),
            border: None,
        },
    };
    second.payload = UiPaintPayload::Brush {
        brushes: crate::ui::surface::UiBrushSet {
            fill: Some(UiBrushPayload::image(image_b.clone())),
            border: None,
        },
    };

    let plan = UiBatchPlan::from_paint_elements(&[first, second]);

    assert_eq!(plan.batches.len(), 2);
    assert_eq!(plan.batches[0].key.resource.as_ref(), Some(&image_a));
    assert_eq!(plan.batches[1].key.resource.as_ref(), Some(&image_b));
    assert_eq!(
        plan.batches[1].split_reason,
        UiBatchSplitReason::ResourceChanged
    );
}

#[test]
fn ui_material_batch_key_includes_variant_revision_and_fallback_state() {
    let mut element = solid_command(31, 0.0, 0.0).to_paint_element(0);
    let fallback = UiRenderResourceKey::new(UiRenderResourceKind::Texture, "ui/material-fallback")
        .with_revision(9);
    let material = UiBrushPayload::material("material/ui/frosted")
        .with_material_variant("hdr")
        .with_material_revision(8)
        .with_fallback_resource(fallback.clone())
        .with_fallback_color("#102030");
    element.payload = UiPaintPayload::Brush {
        brushes: crate::ui::surface::UiBrushSet {
            fill: Some(material),
            border: None,
        },
    };

    let plan = UiBatchPlan::from_paint_elements(&[element]);
    let resource = plan.batches[0].key.resource.as_ref().unwrap();

    assert_eq!(resource.kind, UiRenderResourceKind::Material);
    assert_eq!(resource.id, "material/ui/frosted#hdr");
    assert_eq!(resource.revision, Some(8));
    assert_eq!(resource.fallback.as_deref(), Some(&fallback));
}

#[test]
fn ui_box_brush_preserves_nine_slice_margin_and_resource_state() {
    let resource = UiRenderResourceKey::new(UiRenderResourceKind::Image, "textures/ui/frame.png")
        .with_revision(5)
        .with_atlas_page(3);
    let brush = UiBrushPayload::box_image(resource.clone(), UiMargin::new(0.2, 0.3, 0.2, 0.3));

    let UiBrushPayload::Box(payload) = brush else {
        panic!("expected box brush");
    };

    assert_eq!(payload.margin, UiMargin::new(0.2, 0.3, 0.2, 0.3));
    assert_eq!(
        payload.resource_state,
        UiRenderResourceState::from_key(&resource)
    );
}

#[test]
fn ui_render_cache_plan_marks_reused_paint_and_batch_entries() {
    let mut elements = vec![
        solid_command(41, 0.0, 0.0).to_paint_element(0),
        solid_command(42, 48.0, 0.0).to_paint_element(1),
    ];
    elements[0].cache_generation = Some(7);
    elements[1].cache_generation = Some(7);
    let batch_plan = UiBatchPlan::from_paint_elements(&elements);

    let cache_plan = UiRenderCachePlan::from_paint_elements_and_batches(
        12,
        &elements,
        &batch_plan,
        UiRenderCacheInvalidationReason::Unchanged,
    );

    assert_eq!(cache_plan.surface_generation, 12);
    assert_eq!(cache_plan.paint_entries.len(), 2);
    assert_eq!(
        cache_plan.paint_entries[0].status,
        UiRenderCacheStatus::Reused
    );
    assert_eq!(cache_plan.batch_entries.len(), 1);
    assert_eq!(cache_plan.stats.reused_paint_count, 2);
    assert_eq!(cache_plan.stats.reused_batch_count, 1);
    assert_eq!(cache_plan.stats.rebuilt_paint_count, 0);
}

#[test]
fn ui_render_cache_plan_marks_resource_revision_as_recache_reason() {
    let mut element = solid_command(51, 0.0, 0.0).to_paint_element(0);
    let image = UiRenderResourceKey::new(UiRenderResourceKind::Image, "textures/ui/panel.png")
        .with_revision(3)
        .with_atlas_page(1);
    element.payload = UiPaintPayload::Brush {
        brushes: crate::ui::surface::UiBrushSet {
            fill: Some(UiBrushPayload::image(image.clone())),
            border: None,
        },
    };
    let batch_plan = UiBatchPlan::from_paint_elements(&[element.clone()]);

    let cache_plan = UiRenderCachePlan::from_paint_elements_and_batches(
        13,
        &[element],
        &batch_plan,
        UiRenderCacheInvalidationReason::ResourceRevisionChanged,
    );

    assert_eq!(
        cache_plan.paint_entries[0].status,
        UiRenderCacheStatus::Rebuilt
    );
    assert_eq!(
        cache_plan.paint_entries[0].reason,
        UiRenderCacheInvalidationReason::ResourceRevisionChanged
    );
    assert_eq!(
        cache_plan.batch_entries[0].reason,
        UiRenderCacheInvalidationReason::ResourceRevisionChanged
    );
    assert_eq!(
        cache_plan.batch_entries[0].batch_key.resource.as_ref(),
        Some(&image)
    );
    assert_eq!(cache_plan.stats.rebuilt_paint_count, 1);
    assert_eq!(cache_plan.stats.rebuilt_batch_count, 1);
}

#[test]
fn ui_shaped_text_contract_preserves_glyph_atlas_and_advance_data() {
    let mut shaped = UiShapedText::from_resolved_layout(
        "AB",
        &UiResolvedTextLayout {
            direction: UiTextDirection::LeftToRight,
            font_size: 16.0,
            line_height: 20.0,
            source_range: UiTextRange { start: 0, end: 2 },
            lines: vec![UiResolvedTextLine {
                text: "AB".to_string(),
                frame: UiFrame::new(0.0, 0.0, 24.0, 20.0),
                source_range: UiTextRange { start: 0, end: 2 },
                visual_range: UiTextRange { start: 0, end: 2 },
                measured_width: 24.0,
                baseline: 15.0,
                direction: UiTextDirection::LeftToRight,
                runs: vec![UiResolvedTextRun {
                    kind: UiTextRunKind::Plain,
                    text: "AB".to_string(),
                    source_range: UiTextRange { start: 0, end: 2 },
                    visual_range: UiTextRange { start: 0, end: 2 },
                    direction: UiTextDirection::LeftToRight,
                }],
                ellipsized: false,
            }],
            ..UiResolvedTextLayout::default()
        },
        UiTextRenderMode::Sdf,
    );
    let atlas = UiRenderResourceKey::new(UiRenderResourceKind::Font, "font-atlas/default")
        .with_revision(3)
        .with_atlas_page(1);
    shaped.font_key = Some(UiRenderResourceKey::new(
        UiRenderResourceKind::Font,
        "res://fonts/default.font.toml",
    ));
    shaped.lines[0].glyphs = vec![
        UiShapedGlyph::new(
            65,
            UiTextRange { start: 0, end: 1 },
            UiFrame::new(0.0, 0.0, 10.0, 16.0),
            12.0,
        )
        .with_atlas(atlas.clone(), UiResourceUvRect::new(0.0, 0.0, 0.125, 0.125)),
        UiShapedGlyph::new(
            66,
            UiTextRange { start: 1, end: 2 },
            UiFrame::new(12.0, 0.0, 10.0, 16.0),
            12.0,
        )
        .with_atlas(
            atlas.clone(),
            UiResourceUvRect::new(0.125, 0.0, 0.125, 0.125),
        ),
    ];

    assert_eq!(shaped.lines[0].glyphs.len(), 2);
    assert_eq!(shaped.lines[0].glyphs[0].glyph_id, 65);
    assert_eq!(shaped.lines[0].glyphs[0].advance, 12.0);
    assert_eq!(
        shaped.lines[0].glyphs[0].atlas_resource.as_ref(),
        Some(&atlas)
    );
    assert!(serde_json::to_string(&shaped).unwrap().contains("glyph_id"));
}

#[test]
fn ui_text_paint_contract_carries_editing_and_overflow_decorations() {
    let mut shaped = UiShapedText::from_resolved_layout(
        "edit",
        &UiResolvedTextLayout {
            direction: UiTextDirection::LeftToRight,
            overflow: UiTextOverflow::Ellipsis,
            font_size: 14.0,
            line_height: 18.0,
            measured_width: 28.0,
            measured_height: 18.0,
            source_range: UiTextRange { start: 0, end: 4 },
            lines: vec![UiResolvedTextLine {
                text: "edit".to_string(),
                frame: UiFrame::new(4.0, 8.0, 28.0, 18.0),
                source_range: UiTextRange { start: 0, end: 4 },
                visual_range: UiTextRange { start: 0, end: 4 },
                measured_width: 28.0,
                baseline: 13.0,
                direction: UiTextDirection::LeftToRight,
                runs: vec![UiResolvedTextRun {
                    kind: UiTextRunKind::Strong,
                    text: "edit".to_string(),
                    source_range: UiTextRange { start: 0, end: 4 },
                    visual_range: UiTextRange { start: 0, end: 4 },
                    direction: UiTextDirection::LeftToRight,
                }],
                ellipsized: true,
            }],
            ..UiResolvedTextLayout::default()
        },
        UiTextRenderMode::Native,
    );
    shaped.ellipsis_range = Some(UiTextRange { start: 3, end: 4 });
    let mut paint = UiTextPaint::from_shaped_text(shaped, Some("#ffffff".to_string()));
    paint.selection = Some(UiTextSelection {
        anchor: 1,
        focus: 3,
    });
    paint.caret = Some(UiTextCaret::default());
    paint.composition = Some(UiTextComposition {
        range: UiTextRange { start: 2, end: 4 },
        text: "it".to_string(),
        restore_text: None,
    });
    paint.decorations.push(UiTextPaintDecoration::selection(
        UiTextRange { start: 1, end: 3 },
        UiFrame::new(10.0, 8.0, 14.0, 18.0),
        "#3355ffaa",
    ));
    paint
        .decorations
        .push(UiTextPaintDecoration::composition_underline(
            UiTextRange { start: 2, end: 4 },
            UiFrame::new(18.0, 25.0, 14.0, 1.0),
            "#88ccff",
        ));

    assert_eq!(
        paint.selection.as_ref().unwrap().range(),
        UiTextRange { start: 1, end: 3 }
    );
    assert_eq!(
        paint.shaped.as_ref().unwrap().ellipsis_range,
        Some(UiTextRange { start: 3, end: 4 })
    );
    assert_eq!(paint.decorations.len(), 2);
    assert!(serde_json::to_string(&paint)
        .unwrap()
        .contains("composition_underline"));
}
