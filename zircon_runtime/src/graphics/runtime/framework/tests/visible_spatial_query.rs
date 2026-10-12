use crate::core::framework::render::{
    RenderSpatialBounds, RenderViewportDescriptor, RenderViewportHandle, RenderWorldSnapshotHandle,
};
use crate::core::math::{UVec2, Vec3};
use crate::graphics::VisibilityContext;

use super::super::viewport_record::ViewportRecord;

#[test]
fn viewport_record_replaces_visible_spatial_snapshot_by_rendered_generation() {
    let viewport = RenderViewportHandle::new(5);
    let mut record = ViewportRecord::new(RenderViewportDescriptor::new(UVec2::new(64, 64)));
    let context = VisibilityContext::default();

    record.store_visible_spatial_query(viewport, RenderWorldSnapshotHandle::new(3), 7, &context);
    record.store_visible_spatial_query(viewport, RenderWorldSnapshotHandle::new(3), 8, &context);

    let snapshot = record
        .visible_spatial_query()
        .expect("successful render publishes a visible spatial query");
    assert_eq!(snapshot.identity().world, RenderWorldSnapshotHandle::new(3));
    assert_eq!(snapshot.identity().viewport, viewport);
    assert_eq!(snapshot.identity().frame_generation, 8);
    assert!(snapshot
        .query_bounds(RenderSpatialBounds::new(Vec3::ZERO, 1.0))
        .entities
        .is_empty());
}

#[test]
fn viewport_record_returns_owned_visible_snapshot_without_consuming_storage() {
    let viewport = RenderViewportHandle::new(5);
    let world = RenderWorldSnapshotHandle::new(3);
    let mut record = ViewportRecord::new(RenderViewportDescriptor::new(UVec2::new(64, 64)));
    let context = VisibilityContext::default();

    record.store_visible_spatial_query(viewport, world, 7, &context);
    let first = record
        .visible_spatial_query()
        .expect("first query clones the stored snapshot");
    let second = record
        .visible_spatial_query()
        .expect("stored snapshot remains available");
    record.store_visible_spatial_query(viewport, world, 8, &context);

    assert_eq!(first.identity().frame_generation, 7);
    assert_eq!(second.identity().frame_generation, 7);
    assert_eq!(
        record
            .visible_spatial_query()
            .expect("replacement snapshot remains available")
            .identity()
            .frame_generation,
        8
    );
}
