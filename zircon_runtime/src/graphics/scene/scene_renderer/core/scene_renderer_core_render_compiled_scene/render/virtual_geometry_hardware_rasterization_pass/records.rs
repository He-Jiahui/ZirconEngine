use crate::core::framework::render::{
    RenderVirtualGeometryHardwareRasterizationRecord, RenderVirtualGeometrySelectedCluster,
    RenderVirtualGeometrySelectedClusterSource,
};
use crate::graphics::types::VirtualGeometryClusterSelection;

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
    records
        .iter()
        .flat_map(|record| record.packed_words())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        collect_execution_hardware_rasterization_records,
        collect_execution_hardware_rasterization_records_from_pass,
    };
    use crate::core::framework::render::{
        RenderVirtualGeometryExecutionState, RenderVirtualGeometryHardwareRasterizationRecord,
        RenderVirtualGeometrySelectedCluster, RenderVirtualGeometrySelectedClusterSource,
    };
    use crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene::render::virtual_geometry_executed_cluster_selection_pass::VirtualGeometryExecutedClusterSelectionPassOutput;
    use crate::graphics::types::{
        VirtualGeometryClusterSelection, VirtualGeometryPrepareClusterState,
    };

    #[test]
    fn hardware_rasterization_pass_records_follow_shared_executed_cluster_selection_order_and_preserve_startup_parameters(
    ) {
        let records = collect_execution_hardware_rasterization_records(&[
            selection(
                Some(0),
                42,
                9,
                40,
                2,
                400,
                1,
                401,
                2,
                2,
                1,
                3,
                9,
                Some(8),
                Some(9),
                VirtualGeometryPrepareClusterState::PendingUpload,
            ),
            selection(
                Some(0),
                42,
                7,
                30,
                1,
                300,
                0,
                300,
                0,
                1,
                1,
                2,
                4,
                Some(3),
                Some(4),
                VirtualGeometryPrepareClusterState::Resident,
            ),
            selection(
                None,
                43,
                3,
                50,
                0,
                500,
                0,
                500,
                0,
                0,
                1,
                1,
                2,
                Some(7),
                Some(8),
                VirtualGeometryPrepareClusterState::PendingUpload,
            ),
        ]);

        assert_eq!(
            records,
            vec![
                RenderVirtualGeometryHardwareRasterizationRecord {
                    instance_index: Some(0),
                    entity: 42,
                    cluster_id: 40,
                    cluster_ordinal: 2,
                    page_id: 400,
                    lod_level: 1,
                    submission_index: 9,
                    submission_page_id: 401,
                    submission_lod_level: 2,
                    entity_cluster_start_ordinal: 2,
                    entity_cluster_span_count: 1,
                    entity_cluster_total_count: 3,
                    lineage_depth: 1,
                    frontier_rank: 9,
                    resident_slot: Some(8),
                    submission_slot: Some(9),
                    state: RenderVirtualGeometryExecutionState::PendingUpload,
                },
                RenderVirtualGeometryHardwareRasterizationRecord {
                    instance_index: Some(0),
                    entity: 42,
                    cluster_id: 30,
                    cluster_ordinal: 1,
                    page_id: 300,
                    lod_level: 0,
                    submission_index: 7,
                    submission_page_id: 300,
                    submission_lod_level: 0,
                    entity_cluster_start_ordinal: 1,
                    entity_cluster_span_count: 1,
                    entity_cluster_total_count: 2,
                    lineage_depth: 1,
                    frontier_rank: 4,
                    resident_slot: Some(3),
                    submission_slot: Some(4),
                    state: RenderVirtualGeometryExecutionState::Resident,
                },
                RenderVirtualGeometryHardwareRasterizationRecord {
                    instance_index: None,
                    entity: 43,
                    cluster_id: 50,
                    cluster_ordinal: 0,
                    page_id: 500,
                    lod_level: 0,
                    submission_index: 3,
                    submission_page_id: 500,
                    submission_lod_level: 0,
                    entity_cluster_start_ordinal: 0,
                    entity_cluster_span_count: 1,
                    entity_cluster_total_count: 1,
                    lineage_depth: 1,
                    frontier_rank: 2,
                    resident_slot: Some(7),
                    submission_slot: Some(8),
                    state: RenderVirtualGeometryExecutionState::PendingUpload,
                },
            ],
            "expected the hardware-rasterization baseline pass to preserve the ordering and startup fields of the shared executed-cluster seam instead of rebuilding its own execution filtering layer"
        );
    }

    #[test]
    fn hardware_rasterization_pass_prefers_pass_owned_selected_clusters() {
        let records = collect_execution_hardware_rasterization_records_from_pass(
            &VirtualGeometryExecutedClusterSelectionPassOutput::from_test_parts(
                vec![selection(
                    Some(0),
                    42,
                    7,
                    30,
                    1,
                    300,
                    0,
                    301,
                    2,
                    1,
                    1,
                    3,
                    4,
                    Some(3),
                    Some(4),
                    VirtualGeometryPrepareClusterState::PendingUpload,
                )],
                vec![selected_cluster(
                    Some(5),
                    900,
                    20,
                    0,
                    200,
                    1,
                    RenderVirtualGeometryExecutionState::Resident,
                )],
                RenderVirtualGeometrySelectedClusterSource::RenderPathExecutionSelections,
                1,
                None,
            ),
        );

        assert_eq!(
            records,
            vec![RenderVirtualGeometryHardwareRasterizationRecord {
                instance_index: Some(5),
                entity: 900,
                cluster_id: 20,
                cluster_ordinal: 0,
                page_id: 200,
                lod_level: 1,
                submission_index: 7,
                submission_page_id: 301,
                submission_lod_level: 2,
                entity_cluster_start_ordinal: 1,
                entity_cluster_span_count: 1,
                entity_cluster_total_count: 3,
                lineage_depth: 1,
                frontier_rank: 4,
                resident_slot: Some(3),
                submission_slot: Some(4),
                state: RenderVirtualGeometryExecutionState::Resident,
            }],
            "expected the hardware-rasterization baseline pass to trust the pass-owned selected-cluster seam for cluster identity and execution state while still preserving startup-only submission metadata from the internal selection DTO"
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn selection(
        instance_index: Option<u32>,
        entity: u64,
        submission_index: u32,
        cluster_id: u32,
        cluster_ordinal: u32,
        page_id: u32,
        lod_level: u8,
        submission_page_id: u32,
        submission_lod_level: u8,
        entity_cluster_start_ordinal: usize,
        entity_cluster_span_count: usize,
        entity_cluster_total_count: usize,
        frontier_rank: u32,
        resident_slot: Option<u32>,
        submission_slot: Option<u32>,
        state: VirtualGeometryPrepareClusterState,
    ) -> VirtualGeometryClusterSelection {
        VirtualGeometryClusterSelection {
            submission_index,
            instance_index,
            entity,
            cluster_id,
            cluster_ordinal,
            page_id,
            lod_level,
            submission_page_id,
            submission_lod_level,
            entity_cluster_start_ordinal,
            entity_cluster_span_count,
            entity_cluster_total_count,
            lineage_depth: 1,
            frontier_rank,
            resident_slot,
            submission_slot,
            state,
        }
    }

    fn selected_cluster(
        instance_index: Option<u32>,
        entity: u64,
        cluster_id: u32,
        cluster_ordinal: u32,
        page_id: u32,
        lod_level: u8,
        state: RenderVirtualGeometryExecutionState,
    ) -> RenderVirtualGeometrySelectedCluster {
        RenderVirtualGeometrySelectedCluster {
            instance_index,
            entity,
            cluster_id,
            cluster_ordinal,
            page_id,
            lod_level,
            state,
        }
    }
}
