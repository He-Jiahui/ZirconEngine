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

    let mut emitted_clusters = HashSet::<(u64, u32)>::new();
    let mut executed_selections = cluster_selections
        .iter()
        .copied()
        .filter(|selection| {
            executed_submission_keys.contains(&(selection.entity, selection.submission_index))
        })
        .filter(|selection| emitted_clusters.insert((selection.entity, selection.cluster_id)))
        .collect::<Vec<_>>();
    executed_selections.sort_by_key(|selection| {
        (
            selection.instance_index.unwrap_or(u32::MAX),
            selection.entity,
            selection.cluster_ordinal,
            selection.cluster_id,
            selection.page_id,
            selection.lod_level,
            selection.submission_index,
        )
    });
    executed_selections
}
