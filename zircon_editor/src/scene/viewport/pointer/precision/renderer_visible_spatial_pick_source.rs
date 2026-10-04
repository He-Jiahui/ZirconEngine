//! 渲染空间命中映射回编辑器呈现 owner，索引在采用产品时建立；事件复用投影，只处理空间查询返回的实体。

use std::{collections::HashMap, mem, sync::Arc};

use zircon_runtime::core::framework::render::RenderVisibleSpatialQuerySnapshot;
use zircon_runtime_interface::{
    math::{UVec2, Vec2},
    ui::layout::UiPoint,
};

use crate::scene::viewport::pointer::{
    candidates::renderable_candidate,
    viewport_renderable_pick_candidate::ViewportRenderablePickCandidate,
};
use crate::scene::viewport::{projection::ViewportProjectionContext, ViewportCameraSnapshot};

use super::PrecisionCandidate;

/// Maps renderer-owned spatial hits back to editor-owned pick presentation data.
///
/// The owner table is materialized only when a rendered generation is adopted. Pointer events
/// perform one renderer spatial query and project only its returned owners.
#[derive(Clone)]
pub(in crate::scene::viewport::pointer) struct RendererVisibleSpatialPickSource {
    snapshot: RenderVisibleSpatialQuerySnapshot,
    // The renderer returns owners in deterministic query order; this is only an O(1) lookup index.
    renderables_by_owner: Arc<HashMap<u64, ViewportRenderablePickCandidate>>,
    projection: ViewportProjectionContext,
}

impl RendererVisibleSpatialPickSource {
    pub(in crate::scene::viewport::pointer) fn new(
        snapshot: RenderVisibleSpatialQuerySnapshot,
        renderables: &[ViewportRenderablePickCandidate],
        camera: ViewportCameraSnapshot,
        viewport: UVec2,
    ) -> Self {
        let mut renderables_by_owner = HashMap::with_capacity(renderables.len());
        for renderable in renderables {
            renderables_by_owner
                .entry(renderable.owner)
                .or_insert_with(|| renderable.clone());
        }
        let projection = ViewportProjectionContext::new(&camera, viewport);
        let source = Self {
            snapshot,
            renderables_by_owner: Arc::new(renderables_by_owner),
            projection,
        };
        source.record_owner_map_metrics(
            source
                .renderables_by_owner
                .len()
                .saturating_mul(mem::size_of::<ViewportRenderablePickCandidate>()),
        );
        source
    }

    /// 复用 owner 索引只适用于同一份交互提取；来源所有者变更后必须重新构造索引。
    pub(in crate::scene::viewport::pointer) fn with_snapshot(
        &self,
        snapshot: RenderVisibleSpatialQuerySnapshot,
        camera: ViewportCameraSnapshot,
        viewport: UVec2,
    ) -> Self {
        let projection = ViewportProjectionContext::new(&camera, viewport);
        let source = Self {
            snapshot,
            renderables_by_owner: Arc::clone(&self.renderables_by_owner),
            projection,
        };
        source.record_owner_map_metrics(0);
        source
    }

    pub(in crate::scene::viewport::pointer) fn is_current_for(
        &self,
        snapshot: &RenderVisibleSpatialQuerySnapshot,
        camera: &ViewportCameraSnapshot,
        viewport: UVec2,
    ) -> bool {
        self.snapshot.identity() == snapshot.identity()
            && self
                .projection
                .matches_camera_and_viewport(camera, viewport)
    }

    /// 事件路径只查询采用时固定的空间快照与投影，避免重新扫描全部世界网格或借用实时场景。
    pub(in crate::scene::viewport::pointer) fn candidates_at(
        &self,
        point: UiPoint,
    ) -> Vec<PrecisionCandidate> {
        zircon_runtime::profile_scope!("editor", "viewport.pointer", "visible_spatial_query");
        let query = self
            .snapshot
            .query_ray(self.projection.spatial_ray_at(Vec2::new(point.x, point.y)));
        zircon_runtime::profile_counter!(
            "editor",
            "viewport.pointer.visible_spatial_query_visited_node_count",
            query.stats.visited_node_count,
        );
        zircon_runtime::profile_counter!(
            "editor",
            "viewport.pointer.visible_spatial_query_candidate_count",
            query.stats.candidate_count,
        );
        zircon_runtime::profile_counter!(
            "editor",
            "viewport.pointer.visible_spatial_query_hit_count",
            query.stats.hit_count,
        );
        let candidates = query
            .entities
            .into_iter()
            .filter_map(|owner| self.renderables_by_owner.get(&owner))
            .filter_map(|renderable| renderable_candidate(renderable, &self.projection))
            .collect::<Vec<_>>();
        zircon_runtime::profile_counter!(
            "editor",
            "viewport.pointer.visible_spatial_query_projected_candidate_count",
            candidates.len(),
        );
        candidates
    }

    // Counts copied candidate values only. Hash-table buckets and allocator overhead belong to
    // the native allocation profile instead of this deterministic payload measure.
    fn record_owner_map_metrics(&self, candidate_copy_payload_bytes: usize) {
        zircon_runtime::profile_counter!(
            "editor",
            "viewport.pointer.visible_spatial_owner_map_entry_count",
            self.renderables_by_owner.len(),
        );
        zircon_runtime::profile_counter!(
            "editor",
            "viewport.pointer.visible_spatial_owner_map_candidate_copy_payload_bytes",
            candidate_copy_payload_bytes,
        );
    }
}

#[cfg(test)]
#[path = "tests/renderer_visible_spatial_pick_source.rs"]
mod tests;
