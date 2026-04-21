use std::sync::Arc;

use crate::core::framework::render::{
    RenderVirtualGeometryVisBuffer64Entry, RenderVirtualGeometryVisBuffer64Source,
};
use wgpu::util::DeviceExt;

use super::virtual_geometry_executed_cluster_selection_pass::VirtualGeometryExecutedClusterSelectionPassOutput;

pub(super) struct VirtualGeometryVisBuffer64PassOutput {
    pub(super) clear_value: u64,
    pub(super) source: RenderVirtualGeometryVisBuffer64Source,
    pub(super) entry_count: u32,
    pub(super) buffer: Option<Arc<wgpu::Buffer>>,
}

pub(super) fn execute_virtual_geometry_visbuffer64_pass(
    device: &wgpu::Device,
    pass_enabled: bool,
    executed_cluster_selection_pass: &VirtualGeometryExecutedClusterSelectionPassOutput,
) -> VirtualGeometryVisBuffer64PassOutput {
    let packed_words =
        collect_execution_visbuffer64_words(&executed_cluster_selection_pass.selections);
    VirtualGeometryVisBuffer64PassOutput {
        clear_value: RenderVirtualGeometryVisBuffer64Entry::CLEAR_VALUE,
        source: if !pass_enabled {
            RenderVirtualGeometryVisBuffer64Source::Unavailable
        } else if packed_words.is_empty() {
            RenderVirtualGeometryVisBuffer64Source::RenderPathClearOnly
        } else {
            RenderVirtualGeometryVisBuffer64Source::RenderPathExecutionSelections
        },
        entry_count: u32::try_from(packed_words.len()).unwrap_or(u32::MAX),
        buffer: create_visbuffer64_buffer(device, &packed_words),
    }
}

fn collect_execution_visbuffer64_words(
    executed_cluster_selections: &[crate::graphics::types::VirtualGeometryClusterSelection],
) -> Vec<u64> {
    executed_cluster_selections
        .into_iter()
        .map(|selection| {
            let selected_cluster = (*selection).to_selected_cluster();
            RenderVirtualGeometryVisBuffer64Entry::packed_value_for(
                selected_cluster.instance_index,
                selected_cluster.cluster_id,
                selected_cluster.page_id,
                selected_cluster.lod_level,
                selected_cluster.state,
            )
        })
        .collect()
}

fn create_visbuffer64_buffer(
    device: &wgpu::Device,
    packed_words: &[u64],
) -> Option<Arc<wgpu::Buffer>> {
    if packed_words.is_empty() {
        return None;
    }

    Some(Arc::new(device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("zircon-vg-execution-visbuffer64"),
            contents: bytemuck::cast_slice(packed_words),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
        },
    )))
}

#[cfg(test)]
mod tests {
    use super::collect_execution_visbuffer64_words;
    use crate::core::framework::render::{
        RenderVirtualGeometryExecutionState, RenderVirtualGeometryVisBuffer64Entry,
    };
    use crate::graphics::types::{
        VirtualGeometryClusterSelection, VirtualGeometryPrepareClusterState,
    };

    #[test]
    fn visbuffer64_pass_words_follow_shared_executed_cluster_selection_order() {
        let packed_words = collect_execution_visbuffer64_words(&[
            selection(
                Some(0),
                42,
                7,
                20,
                0,
                400,
                0,
                VirtualGeometryPrepareClusterState::Resident,
            ),
            selection(
                Some(0),
                42,
                7,
                30,
                1,
                300,
                0,
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
                VirtualGeometryPrepareClusterState::PendingUpload,
            ),
        ]);

        assert_eq!(
            packed_words,
            vec![
                RenderVirtualGeometryVisBuffer64Entry::packed_value_for(
                    Some(0),
                    20,
                    400,
                    0,
                    RenderVirtualGeometryExecutionState::Resident,
                ),
                RenderVirtualGeometryVisBuffer64Entry::packed_value_for(
                    Some(0),
                    30,
                    300,
                    0,
                    RenderVirtualGeometryExecutionState::Resident,
                ),
                RenderVirtualGeometryVisBuffer64Entry::packed_value_for(
                    None,
                    50,
                    500,
                    0,
                    RenderVirtualGeometryExecutionState::PendingUpload,
                ),
            ],
            "expected the VisBuffer64 compat pass to preserve the ordering of the shared executed-cluster seam instead of rebuilding its own submission-key filter, dedupe, or sort layer"
        );
    }

    fn selection(
        instance_index: Option<u32>,
        entity: u64,
        submission_index: u32,
        cluster_id: u32,
        cluster_ordinal: u32,
        page_id: u32,
        lod_level: u8,
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
            submission_page_id: page_id,
            submission_lod_level: lod_level,
            entity_cluster_start_ordinal: cluster_ordinal as usize,
            entity_cluster_span_count: 1,
            entity_cluster_total_count: 3,
            lineage_depth: 0,
            frontier_rank: 0,
            resident_slot: Some(0),
            submission_slot: Some(0),
            state,
        }
    }
}
