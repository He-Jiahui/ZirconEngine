use std::{collections::BTreeMap, sync::Arc};

use zircon_runtime::core::framework::render::RenderVisibleSpatialQuerySnapshot;
use zircon_runtime_interface::{
    math::{UVec2, Vec2},
    ui::layout::UiPoint,
};

use crate::scene::viewport::pointer::{
    candidates::renderable_candidate,
    viewport_renderable_pick_candidate::ViewportRenderablePickCandidate,
};
use crate::scene::viewport::{ViewportCameraSnapshot, projection::ViewportProjectionContext};

use super::PrecisionCandidate;

/// Maps renderer-owned spatial hits back to editor-owned pick presentation data.
///
/// The owner table is materialized only when a rendered generation is adopted. Pointer events
/// perform one renderer spatial query and project only its returned owners.
#[derive(Clone)]
pub(in crate::scene::viewport::pointer) struct RendererVisibleSpatialPickSource {
    snapshot: RenderVisibleSpatialQuerySnapshot,
    renderables_by_owner: Arc<BTreeMap<u64, ViewportRenderablePickCandidate>>,
    camera: ViewportCameraSnapshot,
    viewport: UVec2,
}

impl RendererVisibleSpatialPickSource {
    pub(in crate::scene::viewport::pointer) fn new(
        snapshot: RenderVisibleSpatialQuerySnapshot,
        renderables: &[ViewportRenderablePickCandidate],
        camera: ViewportCameraSnapshot,
        viewport: UVec2,
    ) -> Self {
        let mut renderables_by_owner = BTreeMap::new();
        for renderable in renderables {
            renderables_by_owner
                .entry(renderable.owner)
                .or_insert_with(|| renderable.clone());
        }
        Self {
            snapshot,
            renderables_by_owner: Arc::new(renderables_by_owner),
            camera,
            viewport,
        }
    }

    pub(in crate::scene::viewport::pointer) fn with_snapshot(
        &self,
        snapshot: RenderVisibleSpatialQuerySnapshot,
        camera: ViewportCameraSnapshot,
        viewport: UVec2,
    ) -> Self {
        Self {
            snapshot,
            renderables_by_owner: Arc::clone(&self.renderables_by_owner),
            camera,
            viewport,
        }
    }

    pub(in crate::scene::viewport::pointer) fn candidates_at(
        &self,
        point: UiPoint,
    ) -> Vec<PrecisionCandidate> {
        let projection = ViewportProjectionContext::new(&self.camera, self.viewport);
        self.snapshot
            .query_ray(projection.spatial_ray_at(Vec2::new(point.x, point.y)))
            .entities
            .into_iter()
            .filter_map(|owner| self.renderables_by_owner.get(&owner))
            .filter_map(|renderable| renderable_candidate(renderable, &projection))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn renderer_visible_source_queries_only_returned_owners_at_event_time() {
        let source = include_str!("renderer_visible_spatial_pick_source.rs")
            .split_once("#[cfg(test)]")
            .map_or(
                include_str!("renderer_visible_spatial_pick_source.rs"),
                |(production, _)| production,
            );

        assert!(source.contains(".query_ray("));
        assert!(source.contains("renderables_by_owner.get(&owner)"));
        assert!(!source.contains("render_meshes()"));
        assert!(source.contains("Arc::clone(&self.renderables_by_owner)"));
    }
}
