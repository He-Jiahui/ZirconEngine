use crate::virtual_geometry::VirtualGeometryPrepareFrame;
use zircon_runtime::graphics::{
    VisibilityVirtualGeometryCluster, VisibilityVirtualGeometryDrawSegment,
};

use super::super::VirtualGeometryRuntimeState;
use super::available_slots::available_slots;
use super::evictable_pages::evictable_pages;
use super::pending_page_requests::pending_page_requests;
use super::prepare_visible_clusters::prepare_visible_clusters;
use super::resident_pages::resident_pages;

impl VirtualGeometryRuntimeState {
    pub(crate) fn build_prepare_frame_with_segments(
        &self,
        visible_clusters: &[VisibilityVirtualGeometryCluster],
        visibility_draw_segments: &[VisibilityVirtualGeometryDrawSegment],
    ) -> VirtualGeometryPrepareFrame {
        let prepared_visible_clusters =
            prepare_visible_clusters(self, visible_clusters, visibility_draw_segments);
        let (visible_entities, visible_clusters, cluster_draw_segments) =
            prepared_visible_clusters.into_parts();

        VirtualGeometryPrepareFrame {
            visible_entities,
            visible_clusters,
            cluster_draw_segments,
            resident_pages: resident_pages(self),
            pending_page_requests: pending_page_requests(self),
            available_slots: available_slots(self),
            evictable_pages: evictable_pages(self),
        }
    }
}
