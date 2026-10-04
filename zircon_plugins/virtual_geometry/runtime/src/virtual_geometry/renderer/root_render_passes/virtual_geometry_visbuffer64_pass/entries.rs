use zircon_runtime::core::framework::render::{
    RenderVirtualGeometrySelectedCluster, RenderVirtualGeometrySelectedClusterSource,
    RenderVirtualGeometryVisBuffer64Entry,
};

use super::super::virtual_geometry_executed_cluster_selection_pass::VirtualGeometryExecutedClusterSelectionPassOutput;

#[cfg(test)]
#[path = "entries/tests/performance_tests.rs"]
mod performance_tests;

pub(super) fn collect_and_pack_execution_visbuffer64_entries(
    executed_selected_clusters: &[RenderVirtualGeometrySelectedCluster],
) -> (Vec<RenderVirtualGeometryVisBuffer64Entry>, Vec<u64>) {
    let mut entries = Vec::with_capacity(executed_selected_clusters.len());
    let mut packed_words = Vec::with_capacity(executed_selected_clusters.len());
    for (entry_index, selected_cluster) in executed_selected_clusters.iter().enumerate() {
        let entry = RenderVirtualGeometryVisBuffer64Entry::from_selected_cluster(
            u32::try_from(entry_index).unwrap_or(u32::MAX),
            selected_cluster,
        );
        packed_words.push(entry.packed_value);
        entries.push(entry);
    }
    (entries, packed_words)
}

// 选择来源不可用时不伪造 VisBuffer 条目；其余情况保持已选簇顺序与 packed 值一致。
pub(super) fn collect_and_pack_execution_visbuffer64_entries_from_pass(
    executed_cluster_selection_pass: &VirtualGeometryExecutedClusterSelectionPassOutput,
) -> (Vec<RenderVirtualGeometryVisBuffer64Entry>, Vec<u64>) {
    if executed_cluster_selection_pass.source()
        == RenderVirtualGeometrySelectedClusterSource::Unavailable
    {
        return (Vec::new(), Vec::new());
    }

    collect_and_pack_execution_visbuffer64_entries(
        executed_cluster_selection_pass.selected_clusters(),
    )
}

#[cfg(test)]
pub(super) fn collect_execution_visbuffer64_entries(
    executed_selected_clusters: &[RenderVirtualGeometrySelectedCluster],
) -> Vec<RenderVirtualGeometryVisBuffer64Entry> {
    executed_selected_clusters
        .iter()
        .enumerate()
        .map(|(entry_index, selected_cluster)| {
            RenderVirtualGeometryVisBuffer64Entry::from_selected_cluster(
                u32::try_from(entry_index).unwrap_or(u32::MAX),
                selected_cluster,
            )
        })
        .collect()
}

#[cfg(test)]
pub(super) fn collect_execution_visbuffer64_entries_from_pass(
    executed_cluster_selection_pass: &VirtualGeometryExecutedClusterSelectionPassOutput,
) -> Vec<RenderVirtualGeometryVisBuffer64Entry> {
    if executed_cluster_selection_pass.source()
        == RenderVirtualGeometrySelectedClusterSource::Unavailable
    {
        return Vec::new();
    }

    collect_execution_visbuffer64_entries(executed_cluster_selection_pass.selected_clusters())
}

#[cfg(test)]
pub(super) fn pack_execution_visbuffer64_entries(
    entries: &[RenderVirtualGeometryVisBuffer64Entry],
) -> Vec<u64> {
    entries.iter().map(|entry| entry.packed_value).collect()
}

#[cfg(test)]
#[path = "tests/entries.rs"]
mod tests;
