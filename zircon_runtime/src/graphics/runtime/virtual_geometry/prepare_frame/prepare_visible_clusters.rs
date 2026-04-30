use std::collections::{BTreeMap, BTreeSet};

use crate::graphics::types::{
    VirtualGeometryPrepareCluster, VirtualGeometryPrepareClusterState,
    VirtualGeometryPrepareDrawSegment,
};
use crate::{VisibilityVirtualGeometryCluster, VisibilityVirtualGeometryDrawSegment};

use super::super::VirtualGeometryRuntimeState;
use super::prepared_visible_clusters::PreparedVisibleClusters;

pub(super) fn prepare_visible_clusters(
    state: &VirtualGeometryRuntimeState,
    visible_clusters: &[VisibilityVirtualGeometryCluster],
    visibility_draw_segments: &[VisibilityVirtualGeometryDrawSegment],
) -> PreparedVisibleClusters {
    let mut visible_entities = BTreeSet::new();
    let prepared_visible_clusters = visible_clusters
        .iter()
        .map(|cluster| {
            let resident_slot = state.resident_slot(cluster.page_id);
            let cluster_state = cluster_state(state, cluster.page_id, resident_slot);

            if !matches!(cluster_state, VirtualGeometryPrepareClusterState::Missing) {
                visible_entities.insert(cluster.entity);
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
    let prepared_clusters_by_id = prepared_visible_clusters
        .iter()
        .map(|cluster| ((cluster.entity, cluster.cluster_id), cluster))
        .collect::<BTreeMap<_, _>>();
    let cluster_draw_segments = if visibility_draw_segments.is_empty() {
        compact_cluster_draw_segments(visible_clusters, &prepared_clusters_by_id)
    } else {
        visibility_draw_segments
            .iter()
            .filter_map(|draw_segment| {
                let prepared_cluster =
                    prepared_clusters_by_id.get(&(draw_segment.entity, draw_segment.cluster_id))?;
                if matches!(
                    prepared_cluster.state,
                    VirtualGeometryPrepareClusterState::Missing
                ) {
                    return None;
                }

                Some(VirtualGeometryPrepareDrawSegment {
                    entity: draw_segment.entity,
                    cluster_id: draw_segment.cluster_id,
                    page_id: prepared_cluster.page_id,
                    resident_slot: prepared_cluster.resident_slot,
                    cluster_ordinal: draw_segment.cluster_ordinal,
                    cluster_span_count: draw_segment.cluster_span_count.max(1),
                    cluster_count: draw_segment.cluster_count.max(1),
                    lineage_depth: draw_segment.lineage_depth,
                    lod_level: draw_segment.lod_level,
                    state: prepared_cluster.state,
                })
            })
            .collect()
    };

    PreparedVisibleClusters::new(
        visible_entities.into_iter().collect(),
        prepared_visible_clusters,
        cluster_draw_segments,
    )
}

fn cluster_state(
    state: &VirtualGeometryRuntimeState,
    page_id: u32,
    resident_slot: Option<u32>,
) -> VirtualGeometryPrepareClusterState {
    if resident_slot.is_some() {
        VirtualGeometryPrepareClusterState::Resident
    } else if state.has_pending_page(page_id) {
        VirtualGeometryPrepareClusterState::PendingUpload
    } else {
        VirtualGeometryPrepareClusterState::Missing
    }
}

fn compact_cluster_draw_segments(
    visible_clusters: &[VisibilityVirtualGeometryCluster],
    prepared_clusters_by_id: &BTreeMap<(u64, u32), &VirtualGeometryPrepareCluster>,
) -> Vec<VirtualGeometryPrepareDrawSegment> {
    let mut cluster_draw_segments: Vec<VirtualGeometryPrepareDrawSegment> = Vec::new();

    for cluster in visible_clusters {
        let Some(prepared_cluster) =
            prepared_clusters_by_id.get(&(cluster.entity, cluster.cluster_id))
        else {
            continue;
        };
        if matches!(
            prepared_cluster.state,
            VirtualGeometryPrepareClusterState::Missing
        ) {
            continue;
        }

        if let Some(previous) = cluster_draw_segments.last_mut() {
            let previous_end = previous
                .cluster_ordinal
                .saturating_add(previous.cluster_span_count);
            let same_submission_segment = previous.entity == cluster.entity
                && previous.page_id == cluster.page_id
                && previous.resident_slot == prepared_cluster.resident_slot
                && previous.cluster_count == cluster.cluster_count
                && previous.lod_level == cluster.lod_level
                && previous.state == prepared_cluster.state
                && previous_end == cluster.cluster_ordinal;
            if same_submission_segment {
                previous.cluster_span_count = previous.cluster_span_count.saturating_add(1);
                continue;
            }
        }

        cluster_draw_segments.push(VirtualGeometryPrepareDrawSegment {
            entity: cluster.entity,
            cluster_id: cluster.cluster_id,
            page_id: cluster.page_id,
            resident_slot: prepared_cluster.resident_slot,
            cluster_ordinal: cluster.cluster_ordinal,
            cluster_span_count: 1,
            cluster_count: cluster.cluster_count,
            lineage_depth: u32::from(cluster.lod_level),
            lod_level: cluster.lod_level,
            state: prepared_cluster.state,
        });
    }

    cluster_draw_segments
}
