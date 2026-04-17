use crate::types::VirtualGeometryPrepareFrame;
use crate::VisibilityVirtualGeometryCluster;

use super::super::virtual_geometry_runtime_state::VirtualGeometryRuntimeState;
use super::available_slots::available_slots;
use super::evictable_pages::evictable_pages;
use super::pending_page_requests::pending_page_requests;
use super::prepare_visible_clusters::prepare_visible_clusters;
use super::resident_pages::resident_pages;

impl VirtualGeometryRuntimeState {
    pub(crate) fn build_prepare_frame(
        &self,
        visible_clusters: &[VisibilityVirtualGeometryCluster],
    ) -> VirtualGeometryPrepareFrame {
        let prepared_visible_clusters = prepare_visible_clusters(self, visible_clusters);

        VirtualGeometryPrepareFrame {
            visible_entities: prepared_visible_clusters.visible_entities,
            visible_clusters: prepared_visible_clusters.visible_clusters,
            cluster_draw_segments: prepared_visible_clusters.cluster_draw_segments,
            resident_pages: resident_pages(self),
            pending_page_requests: pending_page_requests(self),
            available_slots: available_slots(self),
            evictable_pages: evictable_pages(self),
        }
    }
}
