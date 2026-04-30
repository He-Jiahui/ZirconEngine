use std::collections::{HashMap, HashSet};

use super::build_records::build_seed_backed_execution_selection_record;
use super::frontier_ranking::SeedBackedFrontierRanking;
use super::ordering::SeedBackedClusterOrdering;
use crate::core::framework::render::{RenderVirtualGeometryCluster, RenderVirtualGeometryExtract};
use crate::graphics::types::VirtualGeometryClusterSelection;

pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene::render::virtual_geometry_executed_cluster_selection_pass) fn build_seed_backed_execution_selections(
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
            if !emitted_clusters.insert(record.selected_cluster_key()) {
                return None;
            }

            Some(record.into_selection())
        })
        .collect()
}
