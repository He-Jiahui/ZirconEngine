use std::collections::HashSet;

use crate::virtual_geometry::types::VirtualGeometryClusterSelection;

pub(super) fn collect_execution_cluster_selections_from_submission_keys(
    cluster_selections: Option<&[VirtualGeometryClusterSelection]>,
    executed_submission_keys: &HashSet<(u64, u32)>,
) -> Vec<VirtualGeometryClusterSelection> {
    let Some(cluster_selections) = cluster_selections else {
        return Vec::new();
    };
    if executed_submission_keys.is_empty() {
        return Vec::new();
    }

    let initial_capacity = cluster_selections.len().min(executed_submission_keys.len());
    let mut emitted_clusters = HashSet::<(u64, u32)>::with_capacity(initial_capacity);
    let mut executed_selections = Vec::with_capacity(initial_capacity);
    for selection in cluster_selections.iter().copied() {
        if executed_submission_keys.contains(&(selection.entity, selection.submission_index))
            && emitted_clusters.insert((selection.entity, selection.cluster_id))
        {
            executed_selections.push(selection);
        }
    }
    executed_selections.sort_unstable_by_key(execution_selection_sort_key);
    executed_selections
}

fn execution_selection_sort_key(
    selection: &VirtualGeometryClusterSelection,
) -> (u32, u64, u32, u32, u32, u8, u32) {
    (
        selection.instance_index.unwrap_or(u32::MAX),
        selection.entity,
        selection.cluster_ordinal,
        selection.cluster_id,
        selection.page_id,
        selection.lod_level,
        selection.submission_index,
    )
}

#[cfg(test)]
mod allocation_tests;
