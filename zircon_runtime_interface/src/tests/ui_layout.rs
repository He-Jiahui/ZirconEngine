use crate::ui::{
    event_ui::UiNodeId,
    layout::{
        UiAlignment, UiAlignment2D, UiFlowDirection, UiFrame, UiGeometry, UiLayoutMetrics,
        UiLayoutTransform, UiMargin, UiPixelSnapping, UiPoint, UiRenderTransform, UiSize, UiSlot,
        UiSlotKind,
    },
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
    let slot = UiSlot::new(parent_id, child_id, UiSlotKind::Overlay)
        .with_padding(UiMargin::new(4.0, 6.0, 8.0, 10.0))
        .with_alignment(UiAlignment2D::new(UiAlignment::Fill, UiAlignment::Start))
        .with_order(7)
        .with_dirty_revision(42);

    let serialized = serde_json::to_string(&(metrics, geometry, slot)).unwrap();
    let (round_trip_metrics, round_trip_geometry, round_trip_slot): (
        UiLayoutMetrics,
        UiGeometry,
        UiSlot,
    ) = serde_json::from_str(&serialized).unwrap();

    assert_eq!(round_trip_metrics, metrics);
    assert_eq!(round_trip_geometry.absolute_frame, layout_frame);
    assert_eq!(round_trip_geometry.render_bounds, paint_frame);
    assert_eq!(
        round_trip_geometry.pixel_snapping,
        UiPixelSnapping::Disabled
    );
    assert_eq!(round_trip_slot.parent_id, parent_id);
    assert_eq!(round_trip_slot.child_id, child_id);
    assert_eq!(round_trip_slot.kind, UiSlotKind::Overlay);
    assert_eq!(round_trip_slot.padding.horizontal(), 12.0);
    assert_eq!(round_trip_slot.padding.vertical(), 16.0);
    assert_eq!(round_trip_slot.order, 7);
    assert_eq!(round_trip_slot.dirty_revision, 42);
    assert_eq!(UiLayoutMetrics::default().dpi_scale, 1.0);
    assert_eq!(
        UiGeometry::from_frame(layout_frame).render_bounds,
        layout_frame
    );
}
