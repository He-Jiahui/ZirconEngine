use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::{UiFrame, UiPoint},
    surface::{
        UiDamageDebugReport, UiDebugOverlayPrimitive, UiDebugOverlayPrimitiveKind,
        UiHitGridDebugStats, UiHitTestDebugDump, UiHitTestQuery, UiHitTestReject,
        UiHitTestRejectReason, UiMaterialBatchDebugStat, UiOverdrawDebugStats,
        UiRenderDebugSnapshot, UiRenderDebugStats, UiRenderDebugStatsV2, UiRenderVisualizerOverlay,
        UiRenderVisualizerOverlayKind, UiRenderVisualizerSnapshot, UiRenderVisualizerStats,
        UiRenderVisualizerTextStats, UiRendererParitySnapshot, UiRendererParityStats,
        UiSurfaceDebugCaptureContext, UiSurfaceDebugSnapshot, UiSurfaceRebuildDebugStats,
        UiWidgetReflectorNode,
    },
    tree::{UiInputPolicy, UiVisibility},
};

use super::export::{load_snapshot_json, snapshot_to_json};
use super::model::EditorUiDebugReflectorModel;
use super::overlay::EditorUiDebugReflectorOverlayState;
use super::selection::EditorUiDebugReflectorSelection;

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
