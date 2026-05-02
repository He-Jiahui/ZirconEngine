use std::collections::HashMap;
#[cfg(test)]
use std::collections::HashSet;

use super::frontier_ranking::{seed_backed_frontier_rank_for_cluster, SeedBackedFrontierRanking};
use super::ordering::SeedBackedClusterOrdering;
use super::record::SeedBackedExecutionSelectionRecord;
use super::state::{
    cluster_lineage_depth, resolve_seed_backed_execution_cluster, seed_backed_cluster_state,
    seed_backed_execution_state,
};
use crate::virtual_geometry::types::{
    VirtualGeometryClusterSelection, VirtualGeometryNodeAndClusterCullClusterWorkItem,
};
use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryCluster, RenderVirtualGeometryExtract,
    RenderVirtualGeometrySelectedCluster,
};

#[cfg(test)]
pub(in crate::virtual_geometry::renderer::root_render_passes::virtual_geometry_executed_cluster_selection_pass) fn build_seed_backed_execution_selection_records(
    extract: &RenderVirtualGeometryExtract,
    clusters_by_id: &HashMap<u32, RenderVirtualGeometryCluster>,
    cluster_ordering: &HashMap<(u64, u32), SeedBackedClusterOrdering>,
    page_residency: &HashMap<u32, bool>,
    emitted_clusters: &mut HashSet<(u64, u32)>,
    instance_index: u32,
    entity: u64,
    cluster_offset: u32,
    cluster_count: u32,
    forced_mip: Option<u8>,
) -> Vec<SeedBackedExecutionSelectionRecord> {
    let mut frontier_ranking = SeedBackedFrontierRanking::default();
    let mut records = Vec::new();
    let mut selected_cluster_record_index = HashMap::<(u64, u32), usize>::new();
    extend_seed_backed_execution_selection_records_with_frontier_ranking(
        extract,
        clusters_by_id,
        cluster_ordering,
        page_residency,
        &mut frontier_ranking,
        &mut records,
        &mut selected_cluster_record_index,
        instance_index,
        entity,
        cluster_offset,
        cluster_count,
        forced_mip,
    );
    records.retain(|record| emitted_clusters.insert(record.selected_cluster_key()));
    records.sort_by_key(seed_backed_record_sort_key);
    refresh_seed_backed_frontier_ranks(&mut records);
    records
}

#[cfg(test)]
pub(super) fn extend_seed_backed_execution_selection_records_with_frontier_ranking(
    extract: &RenderVirtualGeometryExtract,
    clusters_by_id: &HashMap<u32, RenderVirtualGeometryCluster>,
    cluster_ordering: &HashMap<(u64, u32), SeedBackedClusterOrdering>,
    page_residency: &HashMap<u32, bool>,
    frontier_ranking: &mut SeedBackedFrontierRanking,
    records: &mut Vec<SeedBackedExecutionSelectionRecord>,
    selected_cluster_record_index: &mut HashMap<(u64, u32), usize>,
    instance_index: u32,
    entity: u64,
    cluster_offset: u32,
    cluster_count: u32,
    forced_mip: Option<u8>,
) {
    if cluster_count == 0 {
        return;
    }

    let Some(instance) = extract.instances.get(instance_index as usize) else {
        return;
    };
    if instance.entity != entity {
        return;
    }

    let Some(start) = usize::try_from(cluster_offset).ok() else {
        return;
    };
    let Some(available_count) = usize::try_from(cluster_count).ok() else {
        return;
    };
    let end = start
        .saturating_add(available_count)
        .min(extract.clusters.len());
    if start >= end {
        return;
    }

    for (cluster_index, cluster) in extract.clusters[start..end]
        .iter()
        .enumerate()
        .filter(|(_, cluster)| cluster.entity == entity)
        .filter(|(_, cluster)| forced_mip.is_none_or(|forced_mip| cluster.lod_level == forced_mip))
    {
        let record = build_seed_backed_execution_selection_record(
            *cluster,
            start.saturating_add(cluster_index),
            end.saturating_sub(start).max(1),
            instance_index,
            clusters_by_id,
            cluster_ordering,
            page_residency,
            frontier_ranking,
            forced_mip,
        );
        let selected_cluster_key = record.selected_cluster_key();
        if let Some(record_index) = selected_cluster_record_index
            .get(&selected_cluster_key)
            .copied()
        {
            records[record_index] = record;
            continue;
        }

        selected_cluster_record_index.insert(selected_cluster_key, records.len());
        records.push(record);
    }
}

pub(super) fn extend_seed_backed_execution_selection_records_from_cluster_work_item(
    clusters_by_id: &HashMap<u32, RenderVirtualGeometryCluster>,
    cluster_ordering: &HashMap<(u64, u32), SeedBackedClusterOrdering>,
    page_residency: &HashMap<u32, bool>,
    frontier_ranking: &mut SeedBackedFrontierRanking,
    records: &mut Vec<SeedBackedExecutionSelectionRecord>,
    selected_cluster_record_index: &mut HashMap<(u64, u32), usize>,
    extract: &RenderVirtualGeometryExtract,
    work_item: &VirtualGeometryNodeAndClusterCullClusterWorkItem,
) {
    let Some(cluster) = extract
        .clusters
        .get(work_item.cluster_array_index as usize)
        .copied()
    else {
        return;
    };
    if cluster.entity != work_item.entity {
        return;
    }
    if work_item
        .forced_mip
        .is_some_and(|forced_mip| cluster.lod_level != forced_mip)
    {
        return;
    }

    let record = build_seed_backed_execution_selection_record(
        cluster,
        work_item.cluster_array_index as usize,
        1,
        work_item.instance_index,
        clusters_by_id,
        cluster_ordering,
        page_residency,
        frontier_ranking,
        work_item.forced_mip,
    );
    let selected_cluster_key = record.selected_cluster_key();
    if let Some(record_index) = selected_cluster_record_index
        .get(&selected_cluster_key)
        .copied()
    {
        records[record_index] = record;
        return;
    }

    selected_cluster_record_index.insert(selected_cluster_key, records.len());
    records.push(record);
}

pub(super) fn build_seed_backed_execution_selection_record(
    cluster: RenderVirtualGeometryCluster,
    local_cluster_ordinal: usize,
    entity_cluster_total_count: usize,
    instance_index: u32,
    clusters_by_id: &HashMap<u32, RenderVirtualGeometryCluster>,
    cluster_ordering: &HashMap<(u64, u32), SeedBackedClusterOrdering>,
    page_residency: &HashMap<u32, bool>,
    frontier_ranking: &mut SeedBackedFrontierRanking,
    forced_mip: Option<u8>,
) -> SeedBackedExecutionSelectionRecord {
    let resolved_cluster =
        resolve_seed_backed_execution_cluster(cluster, clusters_by_id, page_residency, forced_mip);
    let submission_ordering = cluster_ordering
        .get(&(cluster.entity, cluster.cluster_id))
        .copied()
        .unwrap_or_else(|| {
            SeedBackedClusterOrdering::new(
                u32::try_from(local_cluster_ordinal).unwrap_or(u32::MAX),
                entity_cluster_total_count.max(1),
            )
        });
    let selected_cluster_ordinal = cluster_ordering
        .get(&(resolved_cluster.entity, resolved_cluster.cluster_id))
        .map(|ordering| ordering.cluster_ordinal())
        .unwrap_or_else(|| submission_ordering.cluster_ordinal());
    let submission_state = seed_backed_cluster_state(cluster.page_id, page_residency);
    let selected_state = seed_backed_cluster_state(resolved_cluster.page_id, page_residency);
    let frontier_rank =
        seed_backed_frontier_rank_for_cluster(cluster.page_id, submission_state, frontier_ranking);

    SeedBackedExecutionSelectionRecord::new(
        VirtualGeometryClusterSelection {
            submission_index: instance_index,
            instance_index: Some(instance_index),
            entity: cluster.entity,
            cluster_id: cluster.cluster_id,
            cluster_ordinal: submission_ordering.cluster_ordinal(),
            page_id: cluster.page_id,
            lod_level: cluster.lod_level,
            submission_page_id: cluster.page_id,
            submission_lod_level: cluster.lod_level,
            entity_cluster_start_ordinal: submission_ordering.cluster_ordinal() as usize,
            entity_cluster_span_count: 1,
            entity_cluster_total_count: submission_ordering.entity_cluster_total_count(),
            lineage_depth: cluster_lineage_depth(cluster, clusters_by_id),
            frontier_rank,
            resident_slot: None,
            submission_slot: None,
            state: submission_state,
        },
        RenderVirtualGeometrySelectedCluster {
            instance_index: Some(instance_index),
            entity: resolved_cluster.entity,
            cluster_id: resolved_cluster.cluster_id,
            cluster_ordinal: selected_cluster_ordinal,
            page_id: resolved_cluster.page_id,
            lod_level: resolved_cluster.lod_level,
            state: seed_backed_execution_state(selected_state),
        },
    )
}

pub(super) fn seed_backed_record_sort_key(
    record: &SeedBackedExecutionSelectionRecord,
) -> (u32, u64, u32, u32, u32, u8, u32) {
    let selected_cluster = record.selected_cluster();
    (
        selected_cluster.instance_index.unwrap_or(u32::MAX),
        selected_cluster.entity,
        selected_cluster.cluster_ordinal,
        selected_cluster.cluster_id,
        selected_cluster.page_id,
        selected_cluster.lod_level,
        record.selection().submission_index,
    )
}

pub(super) fn refresh_seed_backed_frontier_ranks(
    records: &mut [SeedBackedExecutionSelectionRecord],
) {
    let mut frontier_ranking = SeedBackedFrontierRanking::default();
    for record in records {
        let frontier_rank = seed_backed_frontier_rank_for_cluster(
            record.selection().page_id,
            record.selection().state,
            &mut frontier_ranking,
        );
        record.assign_frontier_rank(frontier_rank);
    }
}
