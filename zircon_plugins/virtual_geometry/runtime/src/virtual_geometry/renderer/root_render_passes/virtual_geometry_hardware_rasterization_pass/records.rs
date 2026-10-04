use crate::virtual_geometry::types::VirtualGeometryClusterSelection;
use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryHardwareRasterizationRecord, RenderVirtualGeometrySelectedCluster,
    RenderVirtualGeometrySelectedClusterSource,
};

use super::super::packed_words::collect_fixed_packed_words;
use super::super::virtual_geometry_executed_cluster_selection_pass::VirtualGeometryExecutedClusterSelectionPassOutput;

pub(super) fn collect_execution_hardware_rasterization_records(
    executed_cluster_selections: &[VirtualGeometryClusterSelection],
) -> Vec<RenderVirtualGeometryHardwareRasterizationRecord> {
    executed_cluster_selections
        .iter()
        .map(|selection| {
            build_hardware_rasterization_record(selection, &selection.to_selected_cluster())
        })
        .collect()
}

// 仅在 pass 提供一一对应的已选簇时复用该投影；缺失或长度不符时从 selection 重建记录。
pub(super) fn collect_execution_hardware_rasterization_records_from_pass(
    executed_cluster_selection_pass: &VirtualGeometryExecutedClusterSelectionPassOutput,
) -> Vec<RenderVirtualGeometryHardwareRasterizationRecord> {
    if executed_cluster_selection_pass.source()
        == RenderVirtualGeometrySelectedClusterSource::Unavailable
        || executed_cluster_selection_pass.selected_clusters().len()
            != executed_cluster_selection_pass.selections().len()
    {
        return collect_execution_hardware_rasterization_records(
            executed_cluster_selection_pass.selections(),
        );
    }

    executed_cluster_selection_pass
        .selections()
        .iter()
        .zip(executed_cluster_selection_pass.selected_clusters().iter())
        .map(|(selection, selected_cluster)| {
            build_hardware_rasterization_record(selection, selected_cluster)
        })
        .collect()
}

fn build_hardware_rasterization_record(
    selection: &VirtualGeometryClusterSelection,
    selected_cluster: &RenderVirtualGeometrySelectedCluster,
) -> RenderVirtualGeometryHardwareRasterizationRecord {
    RenderVirtualGeometryHardwareRasterizationRecord {
        instance_index: selected_cluster.instance_index,
        entity: selected_cluster.entity,
        cluster_id: selected_cluster.cluster_id,
        cluster_ordinal: selected_cluster.cluster_ordinal,
        page_id: selected_cluster.page_id,
        lod_level: selected_cluster.lod_level,
        submission_index: selection.submission_index,
        submission_page_id: selection.submission_page_id,
        submission_lod_level: selection.submission_lod_level,
        entity_cluster_start_ordinal: u32::try_from(selection.entity_cluster_start_ordinal)
            .unwrap_or(u32::MAX),
        entity_cluster_span_count: u32::try_from(selection.entity_cluster_span_count)
            .unwrap_or(u32::MAX),
        entity_cluster_total_count: u32::try_from(selection.entity_cluster_total_count)
            .unwrap_or(u32::MAX),
        lineage_depth: selection.lineage_depth,
        frontier_rank: selection.frontier_rank,
        resident_slot: selection.resident_slot,
        submission_slot: selection.submission_slot,
        state: selected_cluster.state,
    }
}

pub(super) fn pack_hardware_rasterization_records(
    records: &[RenderVirtualGeometryHardwareRasterizationRecord],
) -> Vec<u32> {
    collect_fixed_packed_words(
        records,
        RenderVirtualGeometryHardwareRasterizationRecord::packed_words,
    )
}

#[cfg(test)]
#[path = "tests/records.rs"]
mod tests;
