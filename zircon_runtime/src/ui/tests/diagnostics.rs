use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{
        Anchor, AxisConstraint, BoxConstraints, Pivot, Position, StretchMode,
        UiCanvasSlotPlacement, UiContainerKind, UiFrame, UiLayoutEngineBackend,
        UiLayoutEngineFallbackReason, UiLayoutEngineFamily, UiLayoutEngineSupport,
        UiLinearBoxConfig, UiMargin, UiPoint, UiSize, UiSizeBoxConfig, UiSlot, UiSlotKind,
    },
    surface::{
        UiCanvasLayerGroup, UiDebugOverlayPrimitiveKind, UiHitTestRejectReason,
        UiRenderCommandKind, UiSurfaceDebugOptions, UiSurfaceDebugSnapshot,
        UI_SURFACE_DEBUG_SCHEMA_VERSION,
    },
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn surface_debug_snapshot_reports_reflector_render_batch_and_hit_grid_stats() {
    let surface = diagnostic_surface();
    let snapshot = surface.debug_snapshot_with_options(&UiSurfaceDebugOptions {
        overdraw_sample_cell_size: 40.0,
        ..UiSurfaceDebugOptions::default()
    });

    assert_eq!(snapshot.tree_id, UiTreeId::new("runtime.ui.diagnostics"));
    assert_eq!(snapshot.roots, vec![UiNodeId::new(1)]);
    assert_eq!(snapshot.nodes.len(), 3);

    let front = snapshot
        .nodes
        .iter()
        .find(|node| node.node_id == UiNodeId::new(3))
        .expect("front node should be reflected");
    assert_eq!(front.control_id.as_deref(), Some("front.button"));
    assert_eq!(front.render_command_count, 1);
    assert_eq!(front.hit_entry_count, 1);
    assert!(front.hit_cell_count > 0);

    assert_eq!(snapshot.render.command_count, 3);
    assert_eq!(snapshot.render.group_count, 1);
    assert_eq!(snapshot.render.quad_count, 2);
    assert_eq!(snapshot.render.material_batch_count, 2);
    assert!(snapshot.render.estimated_draw_calls >= 2);
    assert!(snapshot
        .render
        .material_batches
        .iter()
        .any(|batch| batch.command_kind == UiRenderCommandKind::Quad
            && batch.break_reason == "kind=Quad;unclipped;opaque;text"
            && batch.command_count == 2
            && batch.node_ids.contains(&UiNodeId::new(2))
            && batch.node_ids.contains(&UiNodeId::new(3))));

    assert_eq!(snapshot.hit_test.entry_count, 2);
    assert!(snapshot.hit_test.occupied_cell_count > 0);
    assert!(snapshot.hit_test.max_entries_per_cell >= 2);

    assert!(snapshot.overdraw.covered_cells > 0);
    assert!(snapshot.overdraw.overdrawn_cells > 0);
    assert!(snapshot.overdraw.max_layers >= 2);
    assert!(snapshot.rebuild.arranged_rebuilt);
    assert!(snapshot.rebuild.hit_grid_rebuilt);
    assert!(snapshot.rebuild.render_rebuilt);
    assert_eq!(snapshot.rebuild.arranged_node_count, 3);
    assert_eq!(
        snapshot.rebuild.render_command_count,
        snapshot.render.command_count
    );
    assert_eq!(
        snapshot.rebuild.hit_grid_entry_count,
        snapshot.hit_test.entry_count
    );
    assert_eq!(
        snapshot.rebuild.hit_grid_cell_count,
        snapshot.hit_test.cell_count
    );
}

#[test]
fn surface_debug_snapshot_reports_command_records_and_hit_cells() {
    let surface = diagnostic_surface();
    let snapshot = surface.debug_snapshot();

    assert_eq!(
        snapshot.capture.schema_version,
        UI_SURFACE_DEBUG_SCHEMA_VERSION
    );
    assert_eq!(
        snapshot.render.command_records.len(),
        snapshot.render.command_count
    );
    assert!(snapshot
        .render
        .command_records
        .iter()
        .any(|record| record.node_id == UiNodeId::new(3)
            && record.visible_frame == Some(UiFrame::new(40.0, 0.0, 80.0, 40.0))
            && record.material_key == record.batch_key
            && record.estimated_draw_calls > 0));
    assert_eq!(
        snapshot.hit_test.cell_records.len(),
        snapshot.hit_test.occupied_cell_count
    );
    assert!(snapshot
        .hit_test
        .cell_records
        .iter()
        .any(|cell| cell.entry_node_ids.contains(&UiNodeId::new(2))
            && cell.entry_node_ids.contains(&UiNodeId::new(3))));
}

#[test]
fn surface_debug_snapshot_reports_stable_reject_reason_codes() {
    let surface = diagnostic_surface_with_disabled_front();
    let snapshot = surface.debug_snapshot_for_pick(
        zircon_runtime_interface::ui::surface::UiHitTestQuery::new(UiPoint::new(60.0, 20.0)),
        &UiSurfaceDebugOptions::default(),
    );
    let dump = surface.debug_hit_test(UiPoint::new(60.0, 20.0));

    assert_eq!(
        snapshot.capture.pick_query.expect("pick query").hit_point(),
        UiPoint::new(60.0, 20.0)
    );
    assert!(snapshot.pick_hit_test.is_some());
    assert!(dump
        .rejected
        .iter()
        .any(|reject| reject.node_id == UiNodeId::new(3)
            && reject.reason == UiHitTestRejectReason::Disabled
            && reject.message == "node is disabled"));
    assert!(snapshot
        .overlay_primitives
        .iter()
        .any(
            |primitive| primitive.kind == UiDebugOverlayPrimitiveKind::RejectedBounds
                && primitive.node_id == Some(UiNodeId::new(3))
        ));
}

#[test]
fn surface_debug_snapshot_reports_overdraw_cells_and_overlay_primitives() {
    let surface = diagnostic_surface();
    let snapshot =
        surface.debug_snapshot_for_selection(UiNodeId::new(3), &UiSurfaceDebugOptions::default());

    assert_eq!(snapshot.capture.selected_node, Some(UiNodeId::new(3)));
    assert!(snapshot.overdraw.cells.iter().any(|cell| {
        cell.layer_count >= 2
            && cell.node_ids.contains(&UiNodeId::new(2))
            && cell.node_ids.contains(&UiNodeId::new(3))
    }));
    assert!(snapshot
        .overlay_primitives
        .iter()
        .any(
            |primitive| primitive.kind == UiDebugOverlayPrimitiveKind::SelectedFrame
                && primitive.node_id == Some(UiNodeId::new(3))
        ));
    assert!(snapshot
        .overlay_primitives
        .iter()
        .any(|primitive| primitive.kind == UiDebugOverlayPrimitiveKind::OverdrawCell));
    assert!(snapshot
        .overlay_primitives
        .iter()
        .any(|primitive| primitive.kind == UiDebugOverlayPrimitiveKind::MaterialBatchBounds));
}

#[test]
fn surface_debug_snapshot_json_roundtrips_export_payload() {
    let surface = diagnostic_surface();
    let json = surface
        .debug_snapshot_json(&UiSurfaceDebugOptions::default())
        .expect("debug snapshot json");
    let snapshot: UiSurfaceDebugSnapshot = serde_json::from_str(&json).expect("roundtrip snapshot");

    assert_eq!(snapshot.tree_id, UiTreeId::new("runtime.ui.diagnostics"));
    assert_eq!(
        snapshot.capture.schema_version,
        UI_SURFACE_DEBUG_SCHEMA_VERSION
    );
    assert!(!snapshot.render.command_records.is_empty());
    assert!(!snapshot.hit_test.cell_records.is_empty());
    assert!(!snapshot.overdraw.cells.is_empty());
}

#[test]
fn surface_debug_snapshot_json_exports_layout_engine_route_report() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.diagnostics.layout_engine"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(10), UiNodePath::new("root")).with_container(
            UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 0.0 }),
        ),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(10),
            UiTreeNode::new(UiNodeId::new(11), UiNodePath::new("root/child")),
        )
        .unwrap();
    surface.compute_layout(UiSize::new(120.0, 24.0)).unwrap();

    let json = surface
        .debug_snapshot_json(&UiSurfaceDebugOptions::default())
        .expect("debug snapshot json");
    assert!(json.contains("\"layout_engine_report\""));
    assert!(json.contains("\"fallback_reason_counts\": []"));

    let snapshot: UiSurfaceDebugSnapshot = serde_json::from_str(&json).expect("roundtrip snapshot");
    let report = &snapshot.layout_engine_report;
    assert_eq!(report.request_count, 1);
    assert_eq!(report.taffy_selected_count, 1);
    assert_eq!(report.zircon_selected_count, 0);
    assert!(report.fallback_reason_counts.is_empty());

    let root = report
        .selections
        .iter()
        .find(|selection| selection.node_id == Some(UiNodeId::new(10)))
        .expect("root layout route selection");
    assert_eq!(root.selected_backend, UiLayoutEngineBackend::Taffy);
    assert_eq!(root.support, UiLayoutEngineSupport::Native);
}

#[test]
fn surface_debug_snapshot_json_exports_zircon_fallback_route_reason() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.diagnostics.layout_fallback"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(20), UiNodePath::new("root")).with_container(
            UiContainerKind::SizeBox(UiSizeBoxConfig { aspect_ratio: 2.0 }),
        ),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(20),
            UiTreeNode::new(UiNodeId::new(21), UiNodePath::new("root/child")),
        )
        .unwrap();
    surface.compute_layout(UiSize::new(100.0, 100.0)).unwrap();

    let json = surface
        .debug_snapshot_json(&UiSurfaceDebugOptions::default())
        .expect("debug snapshot json");
    assert!(json.contains("\"selected_backend\": \"zircon\""));
    assert!(json.contains("\"fallback_reason\": \"zircon_owned_semantics\""));
    assert!(json.contains("\"fallback_reason_counts\""));
    assert!(json.contains("\"reason\": \"zircon_owned_semantics\""));
    assert!(json.contains("\"count\": 1"));

    let snapshot: UiSurfaceDebugSnapshot = serde_json::from_str(&json).expect("roundtrip snapshot");
    let report = &snapshot.layout_engine_report;
    assert_eq!(report.request_count, 1);
    assert_eq!(report.taffy_selected_count, 0);
    assert_eq!(report.zircon_selected_count, 1);
    assert_eq!(report.fallback_count, 1);
    assert_eq!(report.fallback_reason_counts.len(), 1);
    assert_eq!(
        report.fallback_reason_counts[0].reason,
        Some(UiLayoutEngineFallbackReason::ZirconOwnedSemantics)
    );
    assert_eq!(report.fallback_reason_counts[0].count, 1);

    let root = report
        .selections
        .iter()
        .find(|selection| selection.node_id == Some(UiNodeId::new(20)))
        .expect("root layout route selection");
    assert_eq!(root.request.family, UiLayoutEngineFamily::Container);
    assert_eq!(root.selected_backend, UiLayoutEngineBackend::Zircon);
    assert_eq!(root.support, UiLayoutEngineSupport::Fallback);
    assert_eq!(
        root.fallback_reason,
        Some(UiLayoutEngineFallbackReason::ZirconOwnedSemantics)
    );
}

#[test]
fn surface_debug_snapshot_uses_surface_frame_as_single_spatial_source() {
    let surface = diagnostic_surface();
    let frame = surface.surface_frame();
    let snapshot = crate::ui::surface::debug_surface_frame(&frame);
    let hit = crate::ui::surface::hit_test_surface_frame(&frame, UiPoint::new(60.0, 20.0));

    let front = snapshot
        .nodes
        .iter()
        .find(|node| node.node_id == UiNodeId::new(3))
        .expect("front node should be reflected");

    assert_eq!(hit.top_hit, Some(UiNodeId::new(3)));
    assert_eq!(front.frame, UiFrame::new(40.0, 0.0, 80.0, 40.0));
    assert_eq!(front.clip_frame, UiFrame::new(40.0, 0.0, 80.0, 40.0));
    assert_eq!(front.hit_entry_count, 1);
    assert_eq!(front.render_command_count, 1);
}

#[test]
fn surface_debug_snapshot_reports_parent_owned_canvas_slot_provenance() {
    let surface = diagnostic_canvas_slot_surface();
    let snapshot = surface.debug_snapshot();
    let canvas_child = snapshot
        .nodes
        .iter()
        .find(|node| node.node_id == UiNodeId::new(31))
        .expect("canvas child should be reflected");
    let slot = canvas_child
        .slot
        .as_ref()
        .expect("reflector node should expose the parent-owned Canvas slot");
    let placement = slot
        .canvas_placement
        .expect("Canvas slot placement should be preserved for diagnostics");

    assert_eq!(canvas_child.frame, UiFrame::new(64.0, 13.0, 80.0, 40.0));
    assert_eq!(canvas_child.z_index, 8);
    assert_eq!(slot.parent_id, UiNodeId::new(30));
    assert_eq!(slot.child_id, UiNodeId::new(31));
    assert_eq!(slot.kind, UiSlotKind::Canvas);
    assert_eq!(slot.order, 4);
    assert_eq!(slot.z_order, 6);
    assert_eq!(slot.dirty_revision, 2);
    assert_eq!(placement.anchor, Anchor::new(0.5, 0.25));
    assert_eq!(placement.anchor_max, None);
    assert_eq!(placement.pivot, Pivot::new(0.5, 0.5));
    assert_eq!(placement.position, Position::new(4.0, 6.0));
    assert_eq!(placement.offset, UiMargin::new(10.0, 2.0, 80.0, 40.0));
    assert!(!placement.auto_size);
    assert_eq!(
        snapshot.canvas_layers,
        vec![UiCanvasLayerGroup {
            parent_id: UiNodeId::new(30),
            layer_index: 0,
            z_order: 6,
            child_ids: vec![UiNodeId::new(31)],
        }]
    );
}

fn diagnostic_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.diagnostics"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 80.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            button_node(
                2,
                "root/back",
                "back.button",
                UiFrame::new(0.0, 0.0, 80.0, 40.0),
                0,
            ),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            button_node(
                3,
                "root/front",
                "front.button",
                UiFrame::new(40.0, 0.0, 80.0, 40.0),
                10,
            ),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn diagnostic_canvas_slot_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.diagnostics.canvas_slot"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(30), UiNodePath::new("root"))
            .with_container(UiContainerKind::Canvas)
            .with_input_policy(UiInputPolicy::Ignore),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(30),
            button_node(
                31,
                "root/canvas_child",
                "canvas.button",
                UiFrame::default(),
                2,
            )
            .with_constraints(BoxConstraints {
                width: fixed_constraint(32.0),
                height: fixed_constraint(18.0),
            }),
        )
        .unwrap();
    surface.tree.push_layout_slot(
        UiSlot::new(UiNodeId::new(30), UiNodeId::new(31), UiSlotKind::Canvas)
            .with_canvas_placement(
                UiCanvasSlotPlacement::new(
                    Anchor::new(0.5, 0.25),
                    Pivot::new(0.5, 0.5),
                    Position::new(4.0, 6.0),
                )
                .with_offset(UiMargin::new(10.0, 2.0, 80.0, 40.0)),
            )
            .with_order(4)
            .with_z_order(6)
            .with_dirty_revision(2),
    );

    surface.compute_layout(UiSize::new(200.0, 100.0)).unwrap();
    surface
}

fn diagnostic_surface_with_disabled_front() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.diagnostics.disabled"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 80.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            button_node(
                2,
                "root/back",
                "back.button",
                UiFrame::new(0.0, 0.0, 80.0, 40.0),
                0,
            ),
        )
        .unwrap();
    let mut disabled = pointer_state();
    disabled.enabled = false;
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            button_node(
                3,
                "root/front",
                "front.button",
                UiFrame::new(40.0, 0.0, 80.0, 40.0),
                10,
            )
            .with_state_flags(disabled),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn button_node(
    node_id: u64,
    node_path: &str,
    control_id: &str,
    frame: UiFrame,
    z_index: i32,
) -> UiTreeNode {
    UiTreeNode::new(UiNodeId::new(node_id), UiNodePath::new(node_path))
        .with_frame(frame)
        .with_z_index(z_index)
        .with_input_policy(UiInputPolicy::Receive)
        .with_state_flags(pointer_state())
        .with_template_metadata(UiTemplateNodeMetadata {
            component: "MaterialButton".to_string(),
            control_id: Some(control_id.to_string()),
            attributes: toml::from_str(
                r##"
text = "Run"
opacity = 1.0

[background]
color = "#224466"
"##,
            )
            .unwrap(),
            ..Default::default()
        })
}

fn fixed_constraint(size: f32) -> AxisConstraint {
    AxisConstraint {
        min: size,
        max: size,
        preferred: size,
        priority: 100,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}

fn pointer_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: true,
        hoverable: true,
        focusable: true,
        pressed: false,
        checked: false,
        dirty: false,
    }
}
