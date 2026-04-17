use std::collections::BTreeSet;

use crate::types::{
    VirtualGeometryPrepareCluster, VirtualGeometryPrepareClusterState,
    VirtualGeometryPrepareDrawSegment,
};
use crate::VisibilityVirtualGeometryCluster;

use super::super::virtual_geometry_runtime_state::VirtualGeometryRuntimeState;
use super::prepared_visible_clusters::PreparedVisibleClusters;

pub(super) fn prepare_visible_clusters(
    state: &VirtualGeometryRuntimeState,
    visible_clusters: &[VisibilityVirtualGeometryCluster],
) -> PreparedVisibleClusters {
    let mut visible_entities = BTreeSet::new();
    let mut cluster_draw_segments: Vec<VirtualGeometryPrepareDrawSegment> = Vec::new();
    let visible_clusters = visible_clusters
        .iter()
        .map(|cluster| {
            let resident_slot = state.resident_slots.get(&cluster.page_id).copied();
            let cluster_state = cluster_state(state, cluster.page_id, resident_slot);

            if !matches!(cluster_state, VirtualGeometryPrepareClusterState::Missing) {
                visible_entities.insert(cluster.entity);
                if let Some(previous) = cluster_draw_segments.last_mut() {
                    let previous_end = previous
                        .cluster_ordinal
                        .saturating_add(previous.cluster_span_count);
                    let same_submission_segment = previous.entity == cluster.entity
                        && previous.page_id == cluster.page_id
                        && previous.resident_slot == resident_slot
                        && previous.cluster_count == cluster.cluster_count
                        && previous.lod_level == cluster.lod_level
                        && previous.state == cluster_state
                        && previous_end == cluster.cluster_ordinal;
                    if same_submission_segment {
                        previous.cluster_span_count = previous.cluster_span_count.saturating_add(1);
                    } else {
                        cluster_draw_segments.push(VirtualGeometryPrepareDrawSegment {
                            entity: cluster.entity,
                            cluster_id: cluster.cluster_id,
                            page_id: cluster.page_id,
                            resident_slot,
                            cluster_ordinal: cluster.cluster_ordinal,
                            cluster_span_count: 1,
                            cluster_count: cluster.cluster_count,
                            lod_level: cluster.lod_level,
                            state: cluster_state,
                        });
                    }
                } else {
                    cluster_draw_segments.push(VirtualGeometryPrepareDrawSegment {
                        entity: cluster.entity,
                        cluster_id: cluster.cluster_id,
                        page_id: cluster.page_id,
                        resident_slot,
                        cluster_ordinal: cluster.cluster_ordinal,
                        cluster_span_count: 1,
                        cluster_count: cluster.cluster_count,
                        lod_level: cluster.lod_level,
                        state: cluster_state,
                    });
                }
            }

            VirtualGeometryPrepareCluster {
                entity: cluster.entity,
                cluster_id: cluster.cluster_id,
                page_id: cluster.page_id,
                lod_level: cluster.lod_level,
                resident_slot,
                state: cluster_state,
            }
        })
        .collect::<Vec<_>>();

    PreparedVisibleClusters {
        visible_entities: visible_entities.into_iter().collect(),
        visible_clusters,
        cluster_draw_segments,
    }
}

fn cluster_state(
    state: &VirtualGeometryRuntimeState,
    page_id: u32,
    resident_slot: Option<u32>,
) -> VirtualGeometryPrepareClusterState {
    if resident_slot.is_some() {
        VirtualGeometryPrepareClusterState::Resident
    } else if state.pending_pages.contains(&page_id) {
        VirtualGeometryPrepareClusterState::PendingUpload
    } else {
        VirtualGeometryPrepareClusterState::Missing
    }
}
