use std::collections::{HashMap, HashSet};

use crate::virtual_geometry::types::VirtualGeometryPrepareClusterState;
use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryCluster, RenderVirtualGeometryExecutionState,
};

pub(super) fn resolve_seed_backed_execution_cluster_state_and_lineage(
    cluster: RenderVirtualGeometryCluster,
    clusters_by_id: &HashMap<u32, RenderVirtualGeometryCluster>,
    page_residency: &HashMap<u32, bool>,
    forced_mip: Option<u8>,
) -> (
    RenderVirtualGeometryCluster,
    u32,
    VirtualGeometryPrepareClusterState,
    VirtualGeometryPrepareClusterState,
) {
    let submission_state = seed_backed_cluster_state(cluster.page_id, page_residency);
    let mut resolved_cluster = cluster;
    let mut selected_state = submission_state;
    let mut resolution_search_active =
        forced_mip.is_none() && submission_state != VirtualGeometryPrepareClusterState::Resident;
    let mut lineage_depth = 0_u32;
    let mut current_parent_cluster_id = cluster.parent_cluster_id;
    let mut visited_cluster_ids = HashSet::new();
    while let Some(parent_cluster_id) = current_parent_cluster_id {
        if !visited_cluster_ids.insert(parent_cluster_id) {
            break;
        }
        lineage_depth = lineage_depth.saturating_add(1);

        let Some(parent_cluster) = clusters_by_id.get(&parent_cluster_id).copied() else {
            break;
        };
        if resolution_search_active {
            if parent_cluster_id == cluster.cluster_id || parent_cluster.entity != cluster.entity {
                resolution_search_active = false;
            } else if seed_backed_cluster_state(parent_cluster.page_id, page_residency)
                == VirtualGeometryPrepareClusterState::Resident
            {
                resolved_cluster = parent_cluster;
                selected_state = VirtualGeometryPrepareClusterState::Resident;
                resolution_search_active = false;
            }
        }
        current_parent_cluster_id = parent_cluster.parent_cluster_id;
    }

    (
        resolved_cluster,
        lineage_depth,
        submission_state,
        selected_state,
    )
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

#[cfg(test)]
mod performance_tests;
