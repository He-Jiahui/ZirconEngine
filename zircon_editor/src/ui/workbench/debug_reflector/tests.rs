use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::{
        UiFrame, UiLayoutEngineCapability, UiLayoutEngineFamily, UiLayoutEngineRequest,
        UiLayoutEngineSelection, UiLayoutEngineSelectionReport, UiPoint,
    },
    surface::{
        UiDamageDebugReport, UiDebugOverlayPrimitive, UiDebugOverlayPrimitiveKind,
        UiDebugTimelineFrameHandle, UiDebugTimelineFrameSummary, UiDebugTimelineRetention,
        UiDebugTimelineSnapshot, UiHitGridDebugStats, UiHitTestDebugDump, UiHitTestQuery,
        UiHitTestReject, UiHitTestRejectReason, UiMaterialBatchDebugStat, UiOverdrawDebugStats,
        UiRenderDebugSnapshot, UiRenderDebugStats, UiRenderDebugStatsV2, UiRenderVisualizerOverlay,
        UiRenderVisualizerOverlayKind, UiRenderVisualizerSnapshot, UiRenderVisualizerStats,
        UiRenderVisualizerTextStats, UiRendererParitySnapshot, UiRendererParityStats,
        UiSurfaceDebugCaptureContext, UiSurfaceDebugOptions, UiSurfaceDebugSnapshot,
        UiSurfaceRebuildDebugStats, UiWidgetReflectorNode,
    },
    tree::{UiInputPolicy, UiVisibility},
};

use super::export::{load_snapshot_json, snapshot_to_json};
use super::model::{EditorUiDebugReflectorModel, EditorUiDebugReflectorSection};
use super::overlay::EditorUiDebugReflectorOverlayState;
use super::selection::EditorUiDebugReflectorSelection;
use super::EditorUiDebugTimelineModel;

#[test]
fn ui_debug_reflector_model_reports_no_active_surface_state() {
    let model = EditorUiDebugReflectorModel::no_active_surface();

    assert_eq!(
        model.summary.title,
        "UI Debug Reflector: no active surface snapshot"
    );
    assert!(model.summary.export_status.contains("Export unavailable"));
    assert!(model.nodes.is_empty());
    assert!(model.details.contains(
        &"Waiting for Runtime Diagnostics to receive a UiSurfaceDebugSnapshot".to_string()
    ));
}

#[test]
fn ui_debug_reflector_model_projects_snapshot_rows_and_sections() {
    let snapshot = snapshot_fixture(Some(UiNodeId::new(2)));
    let model = EditorUiDebugReflectorModel::from_snapshot(&snapshot);

    assert_eq!(
        model.summary.title,
        "UI Debug Reflector: 2 nodes, 3 commands, schema v1"
    );
    assert_eq!(model.nodes.len(), 2);
    assert!(model.nodes[0].label.contains("root"));
    assert!(model.nodes[1].selected);
    assert!(model.nodes[1].label.contains("node=2"));
    assert!(model.nodes[1].label.contains("visibility=Visible"));
    assert!(model.nodes[1].label.contains("input=Receive"));
    assert!(model
        .details
        .iter()
        .any(|detail| detail.contains("Selected: root/button")));
    assert!(model
        .details
        .iter()
        .any(|detail| detail.contains("Focus: focused=Some(2), captured=Some(2)")));
    assert!(model
        .details
        .iter()
        .any(|detail| detail.contains("hit_path=[1, 2]") && detail.contains("bubble=[2, 1]")));
    assert!(model
        .details
        .iter()
        .any(|detail| detail.contains("Reject: node=3")
            && detail.contains("reason=Disabled")
            && detail.contains("node is disabled")));
    assert!(model
        .sections
        .iter()
        .any(|section| section.title == "Render" && section.lines[0] == "commands: 3"));
    assert!(model.sections.iter().any(|section| {
        section.title == "Layout Engine"
            && section.lines.iter().any(|line| line == "requests: 2")
            && section
                .lines
                .iter()
                .any(|line| line == "selected: taffy=1 zircon=1")
            && section
                .lines
                .iter()
                .any(|line| line == "fallbacks: 1 unsupported: 0")
            && section.lines.iter().any(|line| {
                line.contains("node=1")
                    && line.contains("family=Flex")
                    && line.contains("selected=Taffy")
                    && line.contains("support=Native")
            })
            && section.lines.iter().any(|line| {
                line.contains("node=2")
                    && line.contains("family=Overlay")
                    && line.contains("selected=LegacyZircon")
                    && line.contains("reason=ZirconOwnedSemantics")
            })
    }));
    assert!(model.sections.iter().any(|section| {
        section.title == "Render"
            && section.lines.iter().any(|line| {
                line.contains("batch breaks:") && line.contains("kind=Quad;unclipped;opaque;text")
            })
    }));
    assert!(model.sections.iter().any(|section| {
        section.title == "Invalidation"
            && section
                .lines
                .iter()
                .any(|line| line.contains("dirty flags:"))
    }));
    assert!(model.warnings.is_empty());
}

#[test]
fn ui_debug_reflector_model_displays_unsupported_layout_routes() {
    let mut snapshot = snapshot_fixture(None);
    snapshot.layout_engine_report = unsupported_layout_engine_report_fixture();
    let model = EditorUiDebugReflectorModel::from_snapshot(&snapshot);

    assert!(model.sections.iter().any(|section| {
        section.title == "Layout Engine"
            && section.lines.iter().any(|line| line == "requests: 1")
            && section
                .lines
                .iter()
                .any(|line| line == "fallbacks: 0 unsupported: 1")
            && section.lines.iter().any(|line| {
                line.contains("node=42")
                    && line.contains("family=Block")
                    && line.contains("support=Unsupported")
                    && line.contains("reason=UnsupportedFamily")
            })
    }));
}

#[test]
fn ui_debug_reflector_model_flattens_sections_for_runtime_diagnostics_display() {
    let model = EditorUiDebugReflectorModel {
        sections: vec![
            EditorUiDebugReflectorSection {
                title: "Layout Engine".to_string(),
                lines: vec![
                    "requests: 2".to_string(),
                    "selected: taffy=1 zircon=1".to_string(),
                ],
            },
            EditorUiDebugReflectorSection {
                title: "  ".to_string(),
                lines: vec!["ignored".to_string()],
            },
        ],
        ..EditorUiDebugReflectorModel::default()
    };

    assert_eq!(
        model.section_display_lines(),
        vec![
            "Layout Engine:".to_string(),
            "  requests: 2".to_string(),
            "  selected: taffy=1 zircon=1".to_string(),
        ]
    );
}

#[test]
fn ui_debug_reflector_model_projects_render_visualizer_and_parity_sections() {
    let snapshot = snapshot_fixture(Some(UiNodeId::new(2)));
    let model = EditorUiDebugReflectorModel::from_snapshot(&snapshot);

    assert!(model
        .summary
        .export_status
        .contains("3 render visualizer overlays"));
    assert!(model.sections.iter().any(|section| {
        section.title == "Render Visualizer"
            && section.lines.iter().any(|line| line == "paint elements: 5")
            && section.lines.iter().any(|line| line == "batch groups: 2")
            && section.lines.iter().any(|line| line == "draw calls: 2")
            && section.lines.iter().any(|line| {
                line.contains("overlays: 3")
                    && line.contains("overdraw_regions=1")
                    && line.contains("resources=1")
            })
            && section.lines.iter().any(|line| {
                line.contains("text: elements=2")
                    && line.contains("native=1")
                    && line.contains("sdf=1")
                    && line.contains("glyphs=42")
            })
            && section
                .lines
                .iter()
                .any(|line| line == "cache: reused=2 rebuilt=3")
    }));
    assert!(model.sections.iter().any(|section| {
        section.title == "Renderer Parity"
            && section
                .lines
                .iter()
                .any(|line| line == "paint=5 batches=2 clipped=1 resources=1 text=2")
    }));
}

#[test]
fn ui_debug_reflector_model_warns_when_selected_node_is_stale() {
    let snapshot = snapshot_fixture(Some(UiNodeId::new(99)));
    let model = EditorUiDebugReflectorModel::from_snapshot(&snapshot);

    assert_eq!(
        model.warnings,
        vec!["Selected node 99 is not present in snapshot tree".to_string()]
    );
}

#[test]
fn ui_debug_reflector_selection_prefers_pick_top_hit() {
    let snapshot = snapshot_fixture(Some(UiNodeId::new(1)));
    let selection = EditorUiDebugReflectorSelection::from_snapshot_top_hit(&snapshot);

    assert_eq!(selection.selected_node, Some(UiNodeId::new(2)));
    assert_eq!(selection.pick_point, Some(UiPoint::new(24.0, 28.0)));
}

#[test]
fn ui_debug_reflector_export_roundtrips_json_and_reports_parse_error() {
    let snapshot = snapshot_fixture(Some(UiNodeId::new(2)));
    let json = snapshot_to_json(&snapshot).expect("snapshot serializes");
    let loaded = load_snapshot_json(&json).expect("snapshot parses");

    assert_eq!(loaded.capture.selected_node, Some(UiNodeId::new(2)));
    assert!(json.contains("overlay_primitives"));
    assert!(load_snapshot_json("not json").is_err());
}

#[test]
fn ui_debug_reflector_overlay_toggles_filter_shared_primitives() {
    let primitive = UiDebugOverlayPrimitive {
        kind: UiDebugOverlayPrimitiveKind::HitCell,
        node_id: Some(UiNodeId::new(2)),
        frame: UiFrame::new(0.0, 0.0, 10.0, 10.0),
        label: Some("hit:1".to_string()),
        severity: None,
    };
    let mut state = EditorUiDebugReflectorOverlayState::default();

    assert!(state.allows(&primitive));
    state.hit_grid = false;
    assert!(!state.allows(&primitive));
}

#[test]
fn ui_debug_reflector_overlay_derives_damage_region_from_snapshot_report() {
    let mut snapshot = snapshot_fixture(Some(UiNodeId::new(2)));
    snapshot.overlay_primitives.clear();
    snapshot.render_batches.visualizer.overlays.clear();
    snapshot.damage = UiDamageDebugReport {
        damage_region: Some(UiFrame::new(4.0, 5.0, 30.0, 20.0)),
        ..UiDamageDebugReport::default()
    };

    let primitives = EditorUiDebugReflectorOverlayState {
        selected_frame: false,
        clip_frame: false,
        wireframe: false,
        hit_grid: false,
        hit_path: false,
        rejected_bounds: false,
        overdraw: false,
        material_batches: false,
        text_debug: false,
        resource_atlas: false,
        damage: true,
    }
    .primitives_from_snapshot(&snapshot);

    assert_eq!(primitives.len(), 1);
    assert_eq!(
        primitives[0].kind,
        UiDebugOverlayPrimitiveKind::DamageRegion
    );
    assert_eq!(primitives[0].frame, UiFrame::new(4.0, 5.0, 30.0, 20.0));

    let primitives = EditorUiDebugReflectorOverlayState {
        selected_frame: false,
        clip_frame: false,
        wireframe: false,
        hit_grid: false,
        hit_path: false,
        rejected_bounds: false,
        overdraw: false,
        material_batches: false,
        text_debug: false,
        resource_atlas: false,
        damage: false,
    }
    .primitives_from_snapshot(&snapshot);

    assert!(primitives.is_empty());
}

#[test]
fn ui_debug_reflector_overlay_derives_render_visualizer_primitives() {
    let mut snapshot = snapshot_fixture(Some(UiNodeId::new(2)));
    snapshot.overlay_primitives.clear();
    snapshot.damage.damage_region = None;

    let primitives =
        EditorUiDebugReflectorOverlayState::default().primitives_from_snapshot(&snapshot);

    assert!(primitives.iter().any(|primitive| primitive.kind
        == UiDebugOverlayPrimitiveKind::MaterialBatchBounds
        && primitive.label.as_deref() == Some("batch:0")));
    assert!(primitives
        .iter()
        .any(|primitive| primitive.kind == UiDebugOverlayPrimitiveKind::OverdrawCell));
    assert!(primitives.iter().any(|primitive| primitive.kind
        == UiDebugOverlayPrimitiveKind::TextGlyphBounds
        && primitive.node_id == Some(UiNodeId::new(2))));

    let primitives = EditorUiDebugReflectorOverlayState {
        material_batches: false,
        overdraw: false,
        text_debug: false,
        ..EditorUiDebugReflectorOverlayState::default()
    }
    .primitives_from_snapshot(&snapshot);

    assert!(!primitives
        .iter()
        .any(|primitive| primitive.kind == UiDebugOverlayPrimitiveKind::MaterialBatchBounds));
    assert!(!primitives
        .iter()
        .any(|primitive| primitive.kind == UiDebugOverlayPrimitiveKind::OverdrawCell));
    assert!(!primitives
        .iter()
        .any(|primitive| primitive.kind == UiDebugOverlayPrimitiveKind::TextGlyphBounds));
}

#[test]
fn ui_debug_timeline_model_projects_selected_historical_frame() {
    let timeline = timeline_fixture(Some(UiDebugTimelineFrameHandle(2)));

    let model = EditorUiDebugTimelineModel::from_timeline(&timeline);

    assert_eq!(
        model.retention,
        "Timeline: 3/4 frames retained, dropped 5, first=1, latest=3, selected=2"
    );
    assert_eq!(
        model.selected,
        "Selected frame: handle=2 frame=102 source=Frame 102 nodes=2 commands=5"
    );
    assert_eq!(
        model.latest,
        "Latest frame: handle=3 frame=103 source=Frame 103"
    );
    assert_eq!(model.previous_frame, Some(UiDebugTimelineFrameHandle(1)));
    assert_eq!(model.next_frame, Some(UiDebugTimelineFrameHandle(3)));
    assert_eq!(model.frame_rows.len(), 3);
    assert!(model.frame_rows[1].contains("handle=2"));
    assert!(model.frame_rows[1].contains("selected_node=Some(2)"));
    assert_eq!(
        model.selected_reflector.summary.title,
        "UI Debug Reflector: 2 nodes, 5 commands, schema v1"
    );
    assert!(model
        .selected_reflector
        .details
        .iter()
        .any(|detail| detail.contains("Selected: root/button")));
}

#[test]
fn ui_debug_timeline_model_falls_back_to_latest_when_selected_handle_is_missing() {
    let timeline = timeline_fixture(Some(UiDebugTimelineFrameHandle(99)));

    let model = EditorUiDebugTimelineModel::from_timeline(&timeline);

    assert_eq!(
        model.selected,
        "Selected frame: handle=3 frame=103 source=Frame 103 nodes=2 commands=7"
    );
    assert_eq!(model.previous_frame, Some(UiDebugTimelineFrameHandle(2)));
    assert_eq!(model.next_frame, None);
    assert_eq!(
        model.selected_reflector.summary.title,
        "UI Debug Reflector: 2 nodes, 7 commands, schema v1"
    );
}

#[test]
fn ui_debug_timeline_model_reports_empty_timeline_as_no_active_surface() {
    let model = EditorUiDebugTimelineModel::from_timeline(&UiDebugTimelineSnapshot::default());

    assert_eq!(
        model.retention,
        "Timeline: 0/0 frames retained, dropped 0, first=none, latest=none, selected=none"
    );
    assert_eq!(model.selected, "Selected frame: none");
    assert_eq!(model.latest, "Latest frame: none");
    assert!(model.frame_rows.is_empty());
    assert_eq!(
        model.selected_reflector.summary.title,
        "UI Debug Reflector: no active surface snapshot"
    );
}

fn snapshot_fixture(selected_node: Option<UiNodeId>) -> UiSurfaceDebugSnapshot {
    UiSurfaceDebugSnapshot {
        capture: UiSurfaceDebugCaptureContext {
            selected_node,
            pick_query: Some(UiHitTestQuery::new(UiPoint::new(24.0, 28.0))),
            ..UiSurfaceDebugCaptureContext::default()
        },
        tree_id: UiTreeId::new("editor.reflector.test"),
        roots: vec![UiNodeId::new(1)],
        nodes: vec![
            reflector_node(UiNodeId::new(1), "root", None, 0),
            reflector_node(UiNodeId::new(2), "root/button", Some(UiNodeId::new(1)), 1),
        ],
        rebuild: UiSurfaceRebuildDebugStats::default(),
        render: UiRenderDebugStats {
            command_count: 3,
            material_batch_count: 2,
            estimated_draw_calls: 3,
            material_batches: vec![UiMaterialBatchDebugStat {
                key: "material:button".to_string(),
                break_reason: "kind=Quad;unclipped;opaque;text".to_string(),
                command_kind: zircon_runtime_interface::ui::surface::UiRenderCommandKind::Quad,
                command_count: 2,
                clipped_command_count: 0,
                node_ids: vec![UiNodeId::new(1), UiNodeId::new(2)],
            }],
            ..UiRenderDebugStats::default()
        },
        hit_test: UiHitGridDebugStats {
            entry_count: 2,
            cell_count: 4,
            occupied_cell_count: 1,
            ..UiHitGridDebugStats::default()
        },
        pick_hit_test: Some(UiHitTestDebugDump {
            point: UiPoint::new(24.0, 28.0),
            hit_stack: vec![UiNodeId::new(2), UiNodeId::new(1)],
            hit_path: zircon_runtime_interface::ui::surface::UiHitPath {
                target: Some(UiNodeId::new(2)),
                root_to_leaf: vec![UiNodeId::new(1), UiNodeId::new(2)],
                bubble_route: vec![UiNodeId::new(2), UiNodeId::new(1)],
                virtual_pointer: None,
            },
            inspected: 2,
            rejected: vec![UiHitTestReject {
                node_id: UiNodeId::new(3),
                control_id: Some("disabled.button".to_string()),
                reason: UiHitTestRejectReason::Disabled,
                message: "node is disabled".to_string(),
            }],
            tree_id: UiTreeId::new("editor.reflector.test"),
        }),
        overdraw: UiOverdrawDebugStats {
            covered_cells: 2,
            overdrawn_cells: 1,
            max_layers: 2,
            ..UiOverdrawDebugStats::default()
        },
        layout_engine_report: layout_engine_report_fixture(),
        overlay_primitives: vec![UiDebugOverlayPrimitive {
            kind: UiDebugOverlayPrimitiveKind::SelectedFrame,
            node_id: Some(UiNodeId::new(2)),
            frame: UiFrame::new(12.0, 16.0, 64.0, 24.0),
            label: Some("button".to_string()),
            severity: None,
        }],
        focus_state: zircon_runtime_interface::ui::surface::UiFocusState {
            focused: Some(UiNodeId::new(2)),
            captured: Some(UiNodeId::new(2)),
            hovered: vec![UiNodeId::new(2), UiNodeId::new(1)],
            ..Default::default()
        },
        render_batches: render_debug_snapshot_fixture(),
        ..UiSurfaceDebugSnapshot::default()
    }
}

fn layout_engine_report_fixture() -> UiLayoutEngineSelectionReport {
    let taffy = UiLayoutEngineCapability::taffy_flex_grid_block();
    let zircon = UiLayoutEngineCapability::legacy_zircon();
    UiLayoutEngineSelectionReport::from_selections(vec![
        UiLayoutEngineSelection::select(
            &UiLayoutEngineRequest::new(UiLayoutEngineFamily::Flex),
            &taffy,
            &zircon,
        )
        .with_node_id(UiNodeId::new(1)),
        UiLayoutEngineSelection::select(
            &UiLayoutEngineRequest::new(UiLayoutEngineFamily::Overlay),
            &taffy,
            &zircon,
        )
        .with_node_id(UiNodeId::new(2)),
    ])
}

fn unsupported_layout_engine_report_fixture() -> UiLayoutEngineSelectionReport {
    let mut taffy = UiLayoutEngineCapability::taffy_flex_grid_block();
    taffy.supported_families.clear();
    let mut zircon = UiLayoutEngineCapability::legacy_zircon();
    zircon.supported_families.clear();
    UiLayoutEngineSelectionReport::from_selections(vec![UiLayoutEngineSelection::select(
        &UiLayoutEngineRequest::new(UiLayoutEngineFamily::Block),
        &taffy,
        &zircon,
    )
    .with_node_id(UiNodeId::new(42))])
}

fn timeline_fixture(selected_frame: Option<UiDebugTimelineFrameHandle>) -> UiDebugTimelineSnapshot {
    let frames = vec![
        timeline_snapshot(1, 101, "Frame 101", 3),
        timeline_snapshot(2, 102, "Frame 102", 5),
        timeline_snapshot(3, 103, "Frame 103", 7),
    ];
    let summaries = frames
        .iter()
        .enumerate()
        .map(|(index, snapshot)| timeline_summary(index as u64 + 1, snapshot))
        .collect::<Vec<_>>();

    UiDebugTimelineSnapshot {
        selected_frame,
        summaries,
        frames,
        retention: UiDebugTimelineRetention {
            capacity: 4,
            len: 3,
            first_frame: Some(UiDebugTimelineFrameHandle(1)),
            latest_frame: Some(UiDebugTimelineFrameHandle(3)),
            selected_frame,
            dropped_frame_count: 5,
        },
    }
}

fn timeline_snapshot(
    handle: u64,
    frame_index: u64,
    label: &str,
    command_count: usize,
) -> UiSurfaceDebugSnapshot {
    let mut snapshot = snapshot_fixture(Some(UiNodeId::new(2)));
    snapshot.capture.frame_index = Some(frame_index);
    snapshot.capture.captured_at_millis = Some(frame_index * 10);
    snapshot.capture.surface_name = Some(label.to_string());
    snapshot.render.command_count = command_count;
    snapshot
        .render
        .command_records
        .truncate(command_count.min(snapshot.render.command_records.len()));
    snapshot.events = Vec::new();
    assert_eq!(handle, frame_index - 100);
    snapshot
}

fn timeline_summary(handle: u64, snapshot: &UiSurfaceDebugSnapshot) -> UiDebugTimelineFrameSummary {
    UiDebugTimelineFrameSummary {
        handle: UiDebugTimelineFrameHandle(handle),
        frame_index: snapshot.capture.frame_index.expect("frame index"),
        captured_at_millis: snapshot.capture.captured_at_millis,
        source_target_id: snapshot.tree_id.0.clone(),
        source_label: snapshot
            .capture
            .surface_name
            .clone()
            .expect("surface label"),
        schema_version: snapshot.capture.schema_version,
        node_count: snapshot.nodes.len(),
        render_command_count: snapshot.render.command_count,
        hit_grid_cell_count: snapshot.hit_test.cell_count,
        invalidation_dirty_count: snapshot.invalidation.dirty_node_count,
        has_damage_region: snapshot.damage.damage_region.is_some(),
        warning_count: snapshot.invalidation.warnings.len() + snapshot.damage.warnings.len(),
        selected_node: snapshot.capture.selected_node,
        capture_options: UiSurfaceDebugOptions::default(),
    }
}

fn render_debug_snapshot_fixture() -> UiRenderDebugSnapshot {
    UiRenderDebugSnapshot {
        tree_id: UiTreeId::new("editor.reflector.test"),
        stats: UiRenderDebugStatsV2 {
            element_count: 5,
            batch_count: 2,
            draw_call_count: 2,
        },
        parity: UiRendererParitySnapshot {
            tree_id: UiTreeId::new("editor.reflector.test"),
            stats: UiRendererParityStats {
                paint_element_count: 5,
                batch_count: 2,
                clipped_paint_count: 1,
                resource_bound_paint_count: 1,
                text_paint_count: 2,
            },
            ..UiRendererParitySnapshot::default()
        },
        visualizer: UiRenderVisualizerSnapshot {
            overlays: vec![
                UiRenderVisualizerOverlay {
                    kind: UiRenderVisualizerOverlayKind::BatchBounds,
                    frame: UiFrame::new(12.0, 16.0, 64.0, 24.0),
                    node_id: None,
                    paint_index: None,
                    batch_index: Some(0),
                    label: None,
                    color: None,
                    intensity: 1.0,
                },
                UiRenderVisualizerOverlay {
                    kind: UiRenderVisualizerOverlayKind::OverdrawHeat,
                    frame: UiFrame::new(18.0, 20.0, 32.0, 18.0),
                    node_id: Some(UiNodeId::new(1)),
                    paint_index: Some(1),
                    batch_index: Some(0),
                    label: Some("heat:2".to_string()),
                    color: None,
                    intensity: 0.75,
                },
                UiRenderVisualizerOverlay {
                    kind: UiRenderVisualizerOverlayKind::TextGlyphBounds,
                    frame: UiFrame::new(24.0, 24.0, 8.0, 12.0),
                    node_id: Some(UiNodeId::new(2)),
                    paint_index: Some(2),
                    batch_index: Some(1),
                    label: Some("glyph:42".to_string()),
                    color: None,
                    intensity: 1.0,
                },
            ],
            text: UiRenderVisualizerTextStats {
                text_element_count: 2,
                native_text_count: 1,
                sdf_text_count: 1,
                shaped_line_count: 2,
                glyph_count: 42,
                decoration_count: 3,
                selection_count: 1,
                caret_count: 1,
                composition_count: 1,
                ..UiRenderVisualizerTextStats::default()
            },
            stats: UiRenderVisualizerStats {
                paint_element_count: 5,
                batch_group_count: 2,
                draw_call_count: 2,
                overlay_count: 3,
                overdraw_region_count: 1,
                resource_binding_count: 1,
                text_element_count: 2,
                sdf_text_element_count: 1,
                cached_paint_count: 2,
                rebuilt_paint_count: 3,
                ..UiRenderVisualizerStats::default()
            },
            ..UiRenderVisualizerSnapshot::default()
        },
        ..UiRenderDebugSnapshot::default()
    }
}

fn reflector_node(
    node_id: UiNodeId,
    path: &str,
    parent: Option<UiNodeId>,
    paint_order: u64,
) -> UiWidgetReflectorNode {
    UiWidgetReflectorNode {
        node_id,
        node_path: UiNodePath::new(path),
        parent,
        children: Vec::new(),
        frame: UiFrame::new(12.0, 16.0, 64.0, 24.0),
        clip_frame: UiFrame::new(10.0, 14.0, 68.0, 28.0),
        z_index: 1,
        paint_order,
        visibility: UiVisibility::Visible,
        input_policy: UiInputPolicy::Receive,
        enabled: true,
        clickable: true,
        hoverable: true,
        focusable: false,
        control_id: Some(path.to_string()),
        render_command_count: 1,
        hit_entry_count: 1,
        hit_cell_count: 1,
    }
}
