//! 可见空间查询快照跟随成功帧发布，拾取与诊断查询必须使用对应世界及视口代际。
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
#[path = "tests/visible_spatial_query.rs"]
mod tests;
