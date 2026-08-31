use std::sync::Arc;

use crate::core::framework::render::{
    RenderViewportHandle, RenderVisibleSpatialQuerySnapshot, RenderVisibleSpatialQuerySnapshotId,
    RenderVisibleSpatialQueryView, RenderWorldSnapshotHandle,
};
use crate::graphics::visibility::{VisibilityContext, VisibleSpatialQuery};

use super::viewport_record::ViewportRecord;

impl ViewportRecord {
    pub(in crate::graphics::runtime::render_framework) fn store_visible_spatial_query(
        &mut self,
        viewport: RenderViewportHandle,
        world: RenderWorldSnapshotHandle,
        frame_generation: u64,
        visibility_context: &VisibilityContext,
    ) {
        let identity = RenderVisibleSpatialQuerySnapshotId::new(
            world,
            viewport,
            frame_generation,
            RenderVisibleSpatialQueryView::MainCamera,
        );
        let query = Arc::new(VisibleSpatialQuery::from_context(visibility_context));
        self.last_visible_spatial_query =
            Some(RenderVisibleSpatialQuerySnapshot::new(identity, query));
    }

    pub(in crate::graphics::runtime::render_framework) fn visible_spatial_query(
        &self,
    ) -> Option<RenderVisibleSpatialQuerySnapshot> {
        self.last_visible_spatial_query.as_ref().cloned()
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        RenderSpatialBounds, RenderViewportDescriptor, RenderViewportHandle,
        RenderWorldSnapshotHandle,
    };
    use crate::core::math::{UVec2, Vec3};
    use crate::graphics::VisibilityContext;

    use super::super::viewport_record::ViewportRecord;

    #[test]
    fn viewport_record_replaces_visible_spatial_snapshot_by_rendered_generation() {
        let viewport = RenderViewportHandle::new(5);
        let mut record = ViewportRecord::new(RenderViewportDescriptor::new(UVec2::new(64, 64)));
        let context = VisibilityContext::default();

        record.store_visible_spatial_query(
            viewport,
            RenderWorldSnapshotHandle::new(3),
            7,
            &context,
        );
        record.store_visible_spatial_query(
            viewport,
            RenderWorldSnapshotHandle::new(3),
            8,
            &context,
        );

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
}
