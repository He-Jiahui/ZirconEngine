use crate::core::framework::scene::EntityId;

use crate::graphics::types::{VirtualGeometryPrepareCluster, VirtualGeometryPrepareDrawSegment};

pub(super) struct PreparedVisibleClusters {
    visible_entities: Vec<EntityId>,
    visible_clusters: Vec<VirtualGeometryPrepareCluster>,
    cluster_draw_segments: Vec<VirtualGeometryPrepareDrawSegment>,
}

impl PreparedVisibleClusters {
    pub(super) fn new(
        visible_entities: Vec<EntityId>,
        visible_clusters: Vec<VirtualGeometryPrepareCluster>,
        cluster_draw_segments: Vec<VirtualGeometryPrepareDrawSegment>,
    ) -> Self {
        Self {
            visible_entities,
            visible_clusters,
            cluster_draw_segments,
        }
    }

    pub(super) fn into_parts(
        self,
    ) -> (
        Vec<EntityId>,
        Vec<VirtualGeometryPrepareCluster>,
        Vec<VirtualGeometryPrepareDrawSegment>,
    ) {
        (
            self.visible_entities,
            self.visible_clusters,
            self.cluster_draw_segments,
        )
    }
}
