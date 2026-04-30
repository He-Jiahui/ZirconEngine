use std::collections::HashMap;

use super::super::super::virtual_geometry_node_and_cluster_cull_pass::VirtualGeometryNodeAndClusterCullPassOutput;
use super::super::selection_collection::ExecutedClusterSelectionCollection;
use super::build_records::{
    extend_seed_backed_execution_selection_records_from_cluster_work_item,
    refresh_seed_backed_frontier_ranks, seed_backed_record_sort_key,
};
use super::frontier_ranking::SeedBackedFrontierRanking;
use super::ordering::seed_backed_cluster_ordering_from_cluster_work_items;
use crate::core::framework::render::{
    RenderVirtualGeometryExtract, RenderVirtualGeometryNodeAndClusterCullSource,
};

pub(crate) fn collect_execution_cluster_selection_collection_from_root_seeds(
    extract: Option<&RenderVirtualGeometryExtract>,
    node_and_cluster_cull_pass: &VirtualGeometryNodeAndClusterCullPassOutput,
) -> ExecutedClusterSelectionCollection {
    let Some(extract) = extract else {
        return ExecutedClusterSelectionCollection::default();
    };
    if node_and_cluster_cull_pass.source()
        != RenderVirtualGeometryNodeAndClusterCullSource::RenderPathCullInput
    {
        return ExecutedClusterSelectionCollection::default();
    }
    if node_and_cluster_cull_pass.instance_work_items().is_empty() {
        return ExecutedClusterSelectionCollection::default();
    }
    let cluster_work_items = node_and_cluster_cull_pass.cluster_work_items();
    if cluster_work_items.is_empty() {
        return ExecutedClusterSelectionCollection::default();
    }

    let cluster_budget = cluster_work_items[0].cluster_budget as usize;
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
    let cluster_ordering =
        seed_backed_cluster_ordering_from_cluster_work_items(extract, cluster_work_items);
    let mut frontier_ranking = SeedBackedFrontierRanking::default();
    let mut execution_records = Vec::new();
    let mut selected_cluster_record_index = HashMap::<(u64, u32), usize>::new();
    for work_item in cluster_work_items {
        extend_seed_backed_execution_selection_records_from_cluster_work_item(
            &clusters_by_id,
            &cluster_ordering,
            &page_residency,
            &mut frontier_ranking,
            &mut execution_records,
            &mut selected_cluster_record_index,
            extract,
            work_item,
        );
    }
    execution_records.sort_by_key(seed_backed_record_sort_key);
    if execution_records.len() > cluster_budget {
        execution_records.truncate(cluster_budget);
    }
    refresh_seed_backed_frontier_ranks(&mut execution_records);
    ExecutedClusterSelectionCollection::from_seed_backed_records(execution_records)
}
