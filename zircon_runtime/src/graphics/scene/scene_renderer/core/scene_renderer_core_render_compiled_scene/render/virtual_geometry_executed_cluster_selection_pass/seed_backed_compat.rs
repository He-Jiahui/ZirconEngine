use std::collections::{HashMap, HashSet};

use crate::core::framework::render::{
    RenderVirtualGeometryCluster, RenderVirtualGeometryExtract,
    RenderVirtualGeometryNodeAndClusterCullInstanceSeed,
    RenderVirtualGeometryNodeAndClusterCullSource, RenderVirtualGeometrySelectedCluster,
};
use crate::graphics::types::{VirtualGeometryClusterSelection, VirtualGeometryPrepareClusterState};

use super::{ExecutedClusterSelectionCollection, VirtualGeometryNodeAndClusterCullPassOutput};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SeedBackedExecutionSelectionRecord {
    pub(super) selection: VirtualGeometryClusterSelection,
    pub(super) selected_cluster: RenderVirtualGeometrySelectedCluster,
}

#[derive(Clone, Copy)]
pub(super) struct SeedBackedClusterOrdering {
    cluster_ordinal: u32,
    entity_cluster_total_count: usize,
}

#[derive(Default)]
struct SeedBackedFrontierRanking {
    unresolved_page_rank_by_page: HashMap<u32, u32>,
    next_unresolved_page_rank: u32,
}

pub(super) fn collect_execution_cluster_selection_collection_from_root_seeds(
    extract: Option<&RenderVirtualGeometryExtract>,
    node_and_cluster_cull_pass: &VirtualGeometryNodeAndClusterCullPassOutput,
) -> ExecutedClusterSelectionCollection {
    let Some(extract) = extract else {
        return ExecutedClusterSelectionCollection::default();
    };
    if node_and_cluster_cull_pass.source
        != RenderVirtualGeometryNodeAndClusterCullSource::RenderPathCullInput
    {
        return ExecutedClusterSelectionCollection::default();
    }

    let cluster_budget = node_and_cluster_cull_pass
        .global_state
        .as_ref()
        .map(|global_state| global_state.cull_input.cluster_budget as usize)
        .unwrap_or_default();
    let forced_mip = node_and_cluster_cull_pass
        .global_state
        .as_ref()
        .and_then(|global_state| global_state.cull_input.debug.forced_mip);
    if cluster_budget == 0 {
        return ExecutedClusterSelectionCollection::default();
    }

    let page_residency = extract
        .pages
        .iter()
        .map(|page| (page.page_id, page.resident))
        .collect::<HashMap<_, _>>();
    let clusters_by_id = extract
        .clusters
        .iter()
        .copied()
        .map(|cluster| (cluster.cluster_id, cluster))
        .collect::<HashMap<_, _>>();
    let cluster_ordering = seed_backed_cluster_ordering_from_instance_seeds(
        extract,
        &node_and_cluster_cull_pass.instance_seeds,
    );
    let mut frontier_ranking = SeedBackedFrontierRanking::default();
    let mut compat_records = Vec::new();
    let mut selected_cluster_record_index = HashMap::<(u64, u32), usize>::new();
    for seed in &node_and_cluster_cull_pass.instance_seeds {
        extend_seed_backed_execution_selection_records_with_frontier_ranking(
            extract,
            &clusters_by_id,
            &cluster_ordering,
            &page_residency,
            &mut frontier_ranking,
            &mut compat_records,
            &mut selected_cluster_record_index,
            seed.instance_index,
            seed.entity,
            seed.cluster_offset,
            seed.cluster_count,
            forced_mip,
        );
    }
    compat_records.sort_by_key(seed_backed_record_sort_key);
    if compat_records.len() > cluster_budget {
        compat_records.truncate(cluster_budget);
    }
    refresh_seed_backed_frontier_ranks(&mut compat_records);
    ExecutedClusterSelectionCollection::from_seed_backed_records(compat_records)
}

#[cfg(test)]
pub(super) fn build_seed_backed_execution_selections(
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
) -> Vec<VirtualGeometryClusterSelection> {
    let mut frontier_ranking = SeedBackedFrontierRanking::default();
    build_seed_backed_execution_selections_with_frontier_ranking(
        extract,
        clusters_by_id,
        cluster_ordering,
        page_residency,
        &mut frontier_ranking,
        emitted_clusters,
        instance_index,
        entity,
        cluster_offset,
        cluster_count,
        forced_mip,
    )
}

#[cfg(test)]
pub(super) fn build_seed_backed_execution_selection_records(
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
    records.retain(|record| {
        emitted_clusters.insert((
            record.selected_cluster.entity,
            record.selected_cluster.cluster_id,
        ))
    });
    records.sort_by_key(seed_backed_record_sort_key);
    refresh_seed_backed_frontier_ranks(&mut records);
    records
}

#[cfg(test)]
fn build_seed_backed_execution_selections_with_frontier_ranking(
    extract: &RenderVirtualGeometryExtract,
    clusters_by_id: &HashMap<u32, RenderVirtualGeometryCluster>,
    cluster_ordering: &HashMap<(u64, u32), SeedBackedClusterOrdering>,
    page_residency: &HashMap<u32, bool>,
    frontier_ranking: &mut SeedBackedFrontierRanking,
    emitted_clusters: &mut HashSet<(u64, u32)>,
    instance_index: u32,
    entity: u64,
    cluster_offset: u32,
    cluster_count: u32,
    forced_mip: Option<u8>,
) -> Vec<VirtualGeometryClusterSelection> {
    if cluster_count == 0 {
        return Vec::new();
    }

    let Some(instance) = extract.instances.get(instance_index as usize) else {
        return Vec::new();
    };
    if instance.entity != entity {
        return Vec::new();
    }

    let Some(start) = usize::try_from(cluster_offset).ok() else {
        return Vec::new();
    };
    let Some(available_count) = usize::try_from(cluster_count).ok() else {
        return Vec::new();
    };
    let end = start
        .saturating_add(available_count)
        .min(extract.clusters.len());
    if start >= end {
        return Vec::new();
    }

    extract.clusters[start..end]
        .iter()
        .enumerate()
        .filter(|(_, cluster)| cluster.entity == entity)
        .filter(|(_, cluster)| forced_mip.is_none_or(|forced_mip| cluster.lod_level == forced_mip))
        .filter_map(|(cluster_index, cluster)| {
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
            if !emitted_clusters.insert((
                record.selected_cluster.entity,
                record.selected_cluster.cluster_id,
            )) {
                return None;
            }

            Some(record.selection)
        })
        .collect()
}

fn extend_seed_backed_execution_selection_records_with_frontier_ranking(
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
        let selected_cluster_key = (
            record.selected_cluster.entity,
            record.selected_cluster.cluster_id,
        );
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

fn build_seed_backed_execution_selection_record(
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
        .unwrap_or(SeedBackedClusterOrdering {
            cluster_ordinal: u32::try_from(local_cluster_ordinal).unwrap_or(u32::MAX),
            entity_cluster_total_count: entity_cluster_total_count.max(1),
        });
    let selected_cluster_ordinal = cluster_ordering
        .get(&(resolved_cluster.entity, resolved_cluster.cluster_id))
        .map(|ordering| ordering.cluster_ordinal)
        .unwrap_or(submission_ordering.cluster_ordinal);
    let submission_state = seed_backed_cluster_state(cluster.page_id, page_residency);
    let selected_state = seed_backed_cluster_state(resolved_cluster.page_id, page_residency);
    let frontier_rank =
        seed_backed_frontier_rank_for_cluster(cluster.page_id, submission_state, frontier_ranking);

    SeedBackedExecutionSelectionRecord {
        selection: VirtualGeometryClusterSelection {
            submission_index: instance_index,
            instance_index: Some(instance_index),
            entity: cluster.entity,
            cluster_id: cluster.cluster_id,
            cluster_ordinal: submission_ordering.cluster_ordinal,
            page_id: cluster.page_id,
            lod_level: cluster.lod_level,
            submission_page_id: cluster.page_id,
            submission_lod_level: cluster.lod_level,
            entity_cluster_start_ordinal: submission_ordering.cluster_ordinal as usize,
            entity_cluster_span_count: 1,
            entity_cluster_total_count: submission_ordering.entity_cluster_total_count,
            lineage_depth: cluster_lineage_depth(cluster, clusters_by_id),
            frontier_rank,
            resident_slot: None,
            submission_slot: None,
            state: submission_state,
        },
        selected_cluster: RenderVirtualGeometrySelectedCluster {
            instance_index: Some(instance_index),
            entity: resolved_cluster.entity,
            cluster_id: resolved_cluster.cluster_id,
            cluster_ordinal: selected_cluster_ordinal,
            page_id: resolved_cluster.page_id,
            lod_level: resolved_cluster.lod_level,
            state: seed_backed_execution_state(selected_state),
        },
    }
}

fn seed_backed_record_sort_key(
    record: &SeedBackedExecutionSelectionRecord,
) -> (u32, u64, u32, u32, u32, u8, u32) {
    let selected_cluster = &record.selected_cluster;
    (
        selected_cluster.instance_index.unwrap_or(u32::MAX),
        selected_cluster.entity,
        selected_cluster.cluster_ordinal,
        selected_cluster.cluster_id,
        selected_cluster.page_id,
        selected_cluster.lod_level,
        record.selection.submission_index,
    )
}

fn refresh_seed_backed_frontier_ranks(records: &mut [SeedBackedExecutionSelectionRecord]) {
    let mut frontier_ranking = SeedBackedFrontierRanking::default();
    for record in records {
        record.selection.frontier_rank = seed_backed_frontier_rank_for_cluster(
            record.selection.page_id,
            record.selection.state,
            &mut frontier_ranking,
        );
    }
}

fn resolve_seed_backed_execution_cluster(
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

fn seed_backed_cluster_state(
    page_id: u32,
    page_residency: &HashMap<u32, bool>,
) -> VirtualGeometryPrepareClusterState {
    match page_residency.get(&page_id).copied() {
        Some(true) => VirtualGeometryPrepareClusterState::Resident,
        Some(false) => VirtualGeometryPrepareClusterState::PendingUpload,
        None => VirtualGeometryPrepareClusterState::Missing,
    }
}

fn seed_backed_execution_state(
    state: VirtualGeometryPrepareClusterState,
) -> crate::core::framework::render::RenderVirtualGeometryExecutionState {
    match state {
        VirtualGeometryPrepareClusterState::Resident => {
            crate::core::framework::render::RenderVirtualGeometryExecutionState::Resident
        }
        VirtualGeometryPrepareClusterState::PendingUpload => {
            crate::core::framework::render::RenderVirtualGeometryExecutionState::PendingUpload
        }
        VirtualGeometryPrepareClusterState::Missing => {
            crate::core::framework::render::RenderVirtualGeometryExecutionState::Missing
        }
    }
}

fn seed_backed_frontier_rank_for_cluster(
    page_id: u32,
    state: VirtualGeometryPrepareClusterState,
    frontier_ranking: &mut SeedBackedFrontierRanking,
) -> u32 {
    if matches!(state, VirtualGeometryPrepareClusterState::Resident) {
        return 0;
    }

    *frontier_ranking
        .unresolved_page_rank_by_page
        .entry(page_id)
        .or_insert_with(|| {
            let rank = frontier_ranking.next_unresolved_page_rank;
            frontier_ranking.next_unresolved_page_rank =
                frontier_ranking.next_unresolved_page_rank.saturating_add(1);
            rank
        })
}

pub(super) fn seed_backed_cluster_ordering(
    extract: &RenderVirtualGeometryExtract,
) -> HashMap<(u64, u32), SeedBackedClusterOrdering> {
    let mut clusters_by_entity = HashMap::<u64, Vec<_>>::new();

    if extract.instances.is_empty() {
        for cluster in extract.clusters.iter().copied() {
            clusters_by_entity
                .entry(cluster.entity)
                .or_default()
                .push(cluster);
        }
    } else {
        for instance in &extract.instances {
            let start = instance.cluster_offset as usize;
            let end = start.saturating_add(instance.cluster_count as usize);
            for cluster in extract
                .clusters
                .get(start..end)
                .into_iter()
                .flatten()
                .copied()
            {
                clusters_by_entity
                    .entry(cluster.entity)
                    .or_default()
                    .push(cluster);
            }
        }
    }

    finalize_seed_backed_cluster_ordering(clusters_by_entity)
}

fn seed_backed_cluster_ordering_from_instance_seeds(
    extract: &RenderVirtualGeometryExtract,
    instance_seeds: &[RenderVirtualGeometryNodeAndClusterCullInstanceSeed],
) -> HashMap<(u64, u32), SeedBackedClusterOrdering> {
    if instance_seeds.is_empty() {
        return seed_backed_cluster_ordering(extract);
    }

    let mut clusters_by_entity = HashMap::<u64, Vec<_>>::new();
    for seed in instance_seeds {
        let Some(instance) = extract.instances.get(seed.instance_index as usize) else {
            continue;
        };
        if instance.entity != seed.entity {
            continue;
        }

        let start = seed.cluster_offset as usize;
        let end = start.saturating_add(seed.cluster_count as usize);
        for cluster in extract
            .clusters
            .get(start..end)
            .into_iter()
            .flatten()
            .copied()
            .filter(|cluster| cluster.entity == seed.entity)
        {
            clusters_by_entity
                .entry(cluster.entity)
                .or_default()
                .push(cluster);
        }
    }

    finalize_seed_backed_cluster_ordering(clusters_by_entity)
}

fn finalize_seed_backed_cluster_ordering(
    clusters_by_entity: HashMap<u64, Vec<RenderVirtualGeometryCluster>>,
) -> HashMap<(u64, u32), SeedBackedClusterOrdering> {
    let mut ordering = HashMap::new();
    for (entity, mut clusters) in clusters_by_entity {
        clusters.sort_by_key(|cluster| cluster.cluster_id);
        clusters.dedup_by_key(|cluster| cluster.cluster_id);
        let entity_cluster_total_count = clusters.len().max(1);
        for (cluster_ordinal, cluster) in clusters.into_iter().enumerate() {
            ordering.insert(
                (entity, cluster.cluster_id),
                SeedBackedClusterOrdering {
                    cluster_ordinal: u32::try_from(cluster_ordinal).unwrap_or(u32::MAX),
                    entity_cluster_total_count,
                },
            );
        }
    }

    ordering
}

fn cluster_lineage_depth(
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
