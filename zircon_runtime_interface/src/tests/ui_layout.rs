use crate::ui::{
    event_ui::{UiNodeId, UiTreeId},
    layout::{
        Anchor, Pivot, Position, UiAlignment, UiAlignment2D, UiCanvasSlotPlacement,
        UiFlowDirection, UiFrame, UiGeometry, UiLayoutMetrics, UiLayoutTransform,
        UiLinearSlotSizeRule, UiLinearSlotSizing, UiMargin, UiPixelSnapping, UiPoint,
        UiRenderTransform, UiSize, UiSlot, UiSlotKind,
    },
    tree::UiTree,
};

#[test]
fn ui_layout_geometry_slot_and_metrics_contracts_construct() {
    let parent_id = UiNodeId::new(100);
    let child_id = UiNodeId::new(101);
    let layout_frame = UiFrame::new(10.0, 20.0, 120.0, 48.0);
    let paint_frame = UiFrame::new(10.0, 20.0, 121.0, 49.0);
    let metrics = UiLayoutMetrics {
        logical_size: UiSize::new(800.0, 600.0),
        physical_size: UiSize::new(1600.0, 1200.0),
        dpi_scale: 2.0,
        font_scale: 1.25,
        layout_scale: 1.5,
        flow_direction: UiFlowDirection::RightToLeft,
        pixel_snapping: UiPixelSnapping::Enabled,
    };
    let geometry = UiGeometry {
        local_size: UiSize::new(120.0, 48.0),
        local_offset: UiPoint::new(10.0, 20.0),
        layout_transform: UiLayoutTransform::new(UiPoint::new(10.0, 20.0), UiPoint::new(1.5, 1.5)),
        render_transform: UiRenderTransform::new(UiPoint::new(0.5, 0.25), UiPoint::new(1.0, 1.0)),
        absolute_frame: layout_frame,
        render_bounds: paint_frame,
        clip_frame: Some(UiFrame::new(0.0, 0.0, 128.0, 64.0)),
        pixel_snapping: UiPixelSnapping::Disabled,
    };
    let slot = UiSlot::new(parent_id, child_id, UiSlotKind::Linear)
        .with_padding(UiMargin::new(4.0, 6.0, 8.0, 10.0))
        .with_alignment(UiAlignment2D::new(UiAlignment::Fill, UiAlignment::Start))
        .with_linear_sizing(
            UiLinearSlotSizing::new(UiLinearSlotSizeRule::StretchContent)
                .with_value(2.0)
                .with_shrink_value(0.5)
                .with_min(24.0)
                .with_max(160.0),
        )
        .with_order(7)
        .with_z_order(11)
        .with_dirty_revision(42);

    let serialized = serde_json::to_string(&(metrics, geometry, slot.clone())).unwrap();
    let (round_trip_metrics, round_trip_geometry, round_trip_slot): (
        UiLayoutMetrics,
        UiGeometry,
        UiSlot,
    ) = serde_json::from_str(&serialized).unwrap();

    assert_eq!(round_trip_metrics, metrics);
    assert_eq!(round_trip_geometry, geometry);
    assert_eq!(round_trip_geometry.absolute_frame, layout_frame);
    assert_eq!(round_trip_geometry.render_bounds, paint_frame);
    assert_eq!(
        round_trip_geometry.pixel_snapping,
        UiPixelSnapping::Disabled
    );
    assert_eq!(round_trip_slot.parent_id, parent_id);
    assert_eq!(round_trip_slot.child_id, child_id);
    assert_eq!(round_trip_slot.alignment, slot.alignment);
    assert_eq!(round_trip_slot.kind, UiSlotKind::Linear);
    assert_eq!(round_trip_slot.padding.horizontal(), 12.0);
    assert_eq!(round_trip_slot.padding.vertical(), 16.0);
    assert_eq!(round_trip_slot.linear_sizing, slot.linear_sizing);
    assert_eq!(round_trip_slot.order, 7);
    assert_eq!(round_trip_slot.z_order, 11);
    assert_eq!(round_trip_slot.dirty_revision, 42);
    assert_eq!(UiLayoutMetrics::default().dpi_scale, 1.0);
    assert_eq!(
        UiGeometry::from_frame(layout_frame).render_bounds,
        layout_frame
    );

    let sparse_metrics: UiLayoutMetrics =
        serde_json::from_str(r#"{"logical_size":{"width":320.0,"height":180.0}}"#).unwrap();
    let sparse_geometry: UiGeometry = serde_json::from_str(
        r#"{"absolute_frame":{"x":1.0,"y":2.0,"width":3.0,"height":4.0},"layout_transform":{"translation":{"x":2.0,"y":3.0}}}"#,
    )
    .unwrap();
    let sparse_slot: UiSlot = serde_json::from_str(
        r#"{"parent_id":100,"child_id":101,"padding":{"left":4.0},"alignment":{"horizontal":"fill"}}"#,
    )
    .unwrap();

    assert_eq!(sparse_metrics.dpi_scale, 1.0);
    assert_eq!(sparse_metrics.font_scale, 1.0);
    assert_eq!(sparse_metrics.layout_scale, 1.0);
    assert_eq!(sparse_geometry.pixel_snapping, UiPixelSnapping::Enabled);
    assert_eq!(
        sparse_geometry.layout_transform.translation,
        UiPoint::new(2.0, 3.0)
    );
    assert_eq!(
        sparse_geometry.layout_transform.scale,
        UiPoint::new(1.0, 1.0)
    );
    assert_eq!(
        sparse_geometry.render_transform,
        UiRenderTransform::identity()
    );
    assert_eq!(sparse_slot.kind, UiSlotKind::Free);
    assert_eq!(sparse_slot.linear_sizing, None);
    assert_eq!(sparse_slot.z_order, 0);
    assert_eq!(sparse_slot.padding, UiMargin::new(4.0, 0.0, 0.0, 0.0));
    assert_eq!(
        sparse_slot.alignment,
        UiAlignment2D::new(UiAlignment::Fill, UiAlignment::Start)
    );

    let sparse_linear_sizing: UiLinearSlotSizing =
        serde_json::from_str(r#"{"rule":"stretch_content"}"#).unwrap();
    assert_eq!(
        sparse_linear_sizing.rule,
        UiLinearSlotSizeRule::StretchContent
    );
    assert_eq!(sparse_linear_sizing.value, 1.0);
    assert_eq!(sparse_linear_sizing.shrink_value, 1.0);
    assert_eq!(sparse_linear_sizing.min, 0.0);
    assert_eq!(sparse_linear_sizing.max, -1.0);
}

#[test]
fn ui_canvas_slot_placement_contract_round_trips_and_defaults() {
    let parent_id = UiNodeId::new(120);
    let child_id = UiNodeId::new(121);
    let placement = UiCanvasSlotPlacement::new(
        Anchor::new(0.75, 0.25),
        Pivot::new(1.0, 0.5),
        Position::new(-12.0, 18.0),
    )
    .with_offset(UiMargin::new(2.0, 4.0, 120.0, 40.0))
    .with_auto_size(true);
    let slot = UiSlot::new(parent_id, child_id, UiSlotKind::Free).with_canvas_placement(placement);

    let round_trip: UiSlot = serde_json::from_str(&serde_json::to_string(&slot).unwrap()).unwrap();

    assert_eq!(round_trip.kind, UiSlotKind::Free);
    assert_eq!(round_trip.canvas_placement, Some(placement));

    let sparse_slot: UiSlot = serde_json::from_str(
        r#"{"parent_id":120,"child_id":121,"canvas_placement":{"anchor":{"x":0.5,"y":1.0},"position":{"x":16.0,"y":-8.0}}}"#,
    )
    .unwrap();
    let sparse_placement = sparse_slot.canvas_placement.expect("sparse placement");
    assert_eq!(sparse_slot.kind, UiSlotKind::Free);
    assert_eq!(sparse_placement.anchor, Anchor::new(0.5, 1.0));
    assert_eq!(sparse_placement.pivot, Pivot::default());
    assert_eq!(sparse_placement.position, Position::new(16.0, -8.0));
    assert_eq!(sparse_placement.offset, UiMargin::default());
    assert!(!sparse_placement.auto_size);

    let legacy_slot: UiSlot = serde_json::from_str(r#"{"parent_id":120,"child_id":121}"#).unwrap();
    assert_eq!(legacy_slot.canvas_placement, None);
}

#[test]
fn ui_tree_slot_contract_defaults_missing_slots_and_round_trips_slots() {
    let legacy_tree: UiTree =
        serde_json::from_str(r#"{"tree_id":"legacy.template.tree","roots":[],"nodes":{}}"#)
            .unwrap();
    assert!(legacy_tree.slots.is_empty());

    let parent_id = UiNodeId::new(1);
    let child_id = UiNodeId::new(2);
    let slot = UiSlot::new(parent_id, child_id, UiSlotKind::Linear)
        .with_padding(UiMargin::new(2.0, 4.0, 6.0, 8.0))
        .with_alignment(UiAlignment2D::new(UiAlignment::Center, UiAlignment::Fill))
        .with_order(5)
        .with_z_order(9);
    let tree = UiTree {
        tree_id: UiTreeId::new("slot.template.tree"),
        roots: vec![parent_id],
        nodes: Default::default(),
        slots: vec![slot.clone()],
    };

    let round_trip: UiTree = serde_json::from_str(&serde_json::to_string(&tree).unwrap()).unwrap();

    assert_eq!(round_trip.slots, vec![slot]);
}
