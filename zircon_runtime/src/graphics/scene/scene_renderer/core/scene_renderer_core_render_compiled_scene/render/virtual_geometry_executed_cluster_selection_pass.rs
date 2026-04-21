use std::collections::HashSet;
use std::sync::Arc;

use crate::core::framework::render::RenderVirtualGeometrySelectedCluster;
use crate::graphics::scene::scene_renderer::mesh::MeshDraw;
use crate::graphics::types::VirtualGeometryClusterSelection;
use wgpu::util::DeviceExt;

#[derive(Default)]
pub(super) struct VirtualGeometryExecutedClusterSelectionPassOutput {
    pub(super) selections: Vec<VirtualGeometryClusterSelection>,
    pub(super) selected_cluster_count: u32,
    pub(super) selected_cluster_buffer: Option<Arc<wgpu::Buffer>>,
}

pub(super) fn execute_virtual_geometry_executed_cluster_selection_pass(
    device: &wgpu::Device,
    cluster_selections: Option<&[VirtualGeometryClusterSelection]>,
    indirect_execution_draws: &[&MeshDraw],
) -> VirtualGeometryExecutedClusterSelectionPassOutput {
    let executed_submission_keys = indirect_execution_draws
        .iter()
        .filter_map(|draw| {
            let detail = draw.virtual_geometry_submission_detail?;
            Some((detail.entity, detail.submission_index))
        })
        .collect::<HashSet<_>>();
    let selections = collect_execution_cluster_selections_from_submission_keys(
        cluster_selections,
        &executed_submission_keys,
    );
    let selected_clusters = selections
        .iter()
        .copied()
        .map(VirtualGeometryClusterSelection::to_selected_cluster)
        .collect::<Vec<_>>();
    VirtualGeometryExecutedClusterSelectionPassOutput {
        selections,
        selected_cluster_count: u32::try_from(selected_clusters.len()).unwrap_or(u32::MAX),
        selected_cluster_buffer: create_selected_cluster_buffer(device, &selected_clusters),
    }
}

fn collect_execution_cluster_selections_from_submission_keys(
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

fn create_selected_cluster_buffer(
    device: &wgpu::Device,
    selected_clusters: &[RenderVirtualGeometrySelectedCluster],
) -> Option<Arc<wgpu::Buffer>> {
    if selected_clusters.is_empty() {
        return None;
    }

    let packed_words = selected_clusters
        .iter()
        .flat_map(RenderVirtualGeometrySelectedCluster::packed_words)
        .collect::<Vec<_>>();
    Some(Arc::new(device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("zircon-vg-executed-selected-clusters"),
            contents: bytemuck::cast_slice(&packed_words),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
        },
    )))
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::collect_execution_cluster_selections_from_submission_keys;
    use crate::graphics::types::{
        VirtualGeometryClusterSelection, VirtualGeometryPrepareClusterState,
    };

    #[test]
    fn executed_cluster_selection_pass_filters_deduplicates_and_sorts_cluster_selections() {
        let entity_a = 42_u64;
        let entity_b = 43_u64;
        let mut executed_submission_keys = HashSet::new();
        executed_submission_keys.insert((entity_a, 7));
        executed_submission_keys.insert((entity_b, 3));

        let selections = collect_execution_cluster_selections_from_submission_keys(
            Some(&[
                selection(
                    Some(1),
                    entity_a,
                    7,
                    30,
                    1,
                    300,
                    0,
                    VirtualGeometryPrepareClusterState::Resident,
                ),
                selection(
                    Some(1),
                    entity_a,
                    9,
                    40,
                    2,
                    400,
                    0,
                    VirtualGeometryPrepareClusterState::Resident,
                ),
                selection(
                    None,
                    entity_b,
                    3,
                    50,
                    0,
                    500,
                    0,
                    VirtualGeometryPrepareClusterState::PendingUpload,
                ),
                selection(
                    Some(1),
                    entity_a,
                    7,
                    20,
                    0,
                    200,
                    0,
                    VirtualGeometryPrepareClusterState::Resident,
                ),
                selection(
                    Some(1),
                    entity_a,
                    7,
                    20,
                    0,
                    200,
                    0,
                    VirtualGeometryPrepareClusterState::Resident,
                ),
            ]),
            &executed_submission_keys,
        );

        assert_eq!(
            selections,
            vec![
                selection(
                    Some(1),
                    entity_a,
                    7,
                    20,
                    0,
                    200,
                    0,
                    VirtualGeometryPrepareClusterState::Resident,
                ),
                selection(
                    Some(1),
                    entity_a,
                    7,
                    30,
                    1,
                    300,
                    0,
                    VirtualGeometryPrepareClusterState::Resident,
                ),
                selection(
                    None,
                    entity_b,
                    3,
                    50,
                    0,
                    500,
                    0,
                    VirtualGeometryPrepareClusterState::PendingUpload,
                ),
            ],
            "expected the shared compat executed-cluster seam to drop non-executed submissions, deduplicate repeated clusters, and emit the exact stable ordering that both VisBuffer64 and HardwareRasterization consume"
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
