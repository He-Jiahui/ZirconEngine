use std::collections::{HashMap, HashSet};

use crate::core::framework::render::{
    RenderVirtualGeometryCluster, RenderVirtualGeometryExecutionState,
};
use crate::graphics::types::VirtualGeometryPrepareClusterState;

pub(super) fn resolve_seed_backed_execution_cluster(
    cluster: RenderVirtualGeometryCluster,
    clusters_by_id: &HashMap<u32, RenderVirtualGeometryCluster>,
    page_residency: &HashMap<u32, bool>,
    forced_mip: Option<u8>,
) -> RenderVirtualGeometryCluster {
    if forced_mip.is_some()
        || seed_backed_cluster_state(cluster.page_id, page_residency)
            == VirtualGeometryPrepareClusterState::Resident
    {
        return cluster;
    }

    let mut current_parent_cluster_id = cluster.parent_cluster_id;
    let mut visited_cluster_ids = HashSet::from([cluster.cluster_id]);
    while let Some(parent_cluster_id) = current_parent_cluster_id {
        if !visited_cluster_ids.insert(parent_cluster_id) {
            break;
        }

        let Some(parent_cluster) = clusters_by_id.get(&parent_cluster_id).copied() else {
            break;
        };
        if parent_cluster.entity != cluster.entity {
            break;
        }
        if seed_backed_cluster_state(parent_cluster.page_id, page_residency)
            == VirtualGeometryPrepareClusterState::Resident
        {
            return parent_cluster;
        }
        current_parent_cluster_id = parent_cluster.parent_cluster_id;
    }

    cluster
}

pub(super) fn seed_backed_cluster_state(
    page_id: u32,
    page_residency: &HashMap<u32, bool>,
) -> VirtualGeometryPrepareClusterState {
    match page_residency.get(&page_id).copied() {
        Some(true) => VirtualGeometryPrepareClusterState::Resident,
        Some(false) => VirtualGeometryPrepareClusterState::PendingUpload,
        None => VirtualGeometryPrepareClusterState::Missing,
    }
}

pub(super) fn seed_backed_execution_state(
    state: VirtualGeometryPrepareClusterState,
) -> RenderVirtualGeometryExecutionState {
    match state {
        VirtualGeometryPrepareClusterState::Resident => {
            RenderVirtualGeometryExecutionState::Resident
        }
        VirtualGeometryPrepareClusterState::PendingUpload => {
            RenderVirtualGeometryExecutionState::PendingUpload
        }
        VirtualGeometryPrepareClusterState::Missing => RenderVirtualGeometryExecutionState::Missing,
    }
}

pub(super) fn cluster_lineage_depth(
    cluster: RenderVirtualGeometryCluster,
    clusters_by_id: &HashMap<u32, RenderVirtualGeometryCluster>,
) -> u32 {
    let mut depth = 0_u32;
    let mut current_parent_cluster_id = cluster.parent_cluster_id;
    let mut visited_cluster_ids = HashSet::new();

    while let Some(parent_cluster_id) = current_parent_cluster_id {
        if !visited_cluster_ids.insert(parent_cluster_id) {
            break;
        }
        depth = depth.saturating_add(1);
        current_parent_cluster_id = clusters_by_id
            .get(&parent_cluster_id)
            .and_then(|parent| parent.parent_cluster_id);
    }

    depth
}
