use zircon_framework::scene::EntityId;

use crate::types::{VirtualGeometryPrepareCluster, VirtualGeometryPrepareDrawSegment};

pub(super) struct PreparedVisibleClusters {
    pub(super) visible_entities: Vec<EntityId>,
    pub(super) visible_clusters: Vec<VirtualGeometryPrepareCluster>,
    pub(super) cluster_draw_segments: Vec<VirtualGeometryPrepareDrawSegment>,
}
