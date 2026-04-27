use std::sync::Arc;

use crate::core::framework::render::{
    RenderVirtualGeometrySelectedCluster, RenderVirtualGeometrySelectedClusterSource,
    RenderVirtualGeometryVisBuffer64Entry, RenderVirtualGeometryVisBuffer64Source,
};
use wgpu::util::DeviceExt;

use super::virtual_geometry_executed_cluster_selection_pass::VirtualGeometryExecutedClusterSelectionPassOutput;

pub(in crate::graphics::scene::scene_renderer::core) struct VirtualGeometryVisBuffer64PassOutput {
    pub(in crate::graphics::scene::scene_renderer::core) clear_value: u64,
    pub(in crate::graphics::scene::scene_renderer::core) entries:
        Vec<RenderVirtualGeometryVisBuffer64Entry>,
    pub(in crate::graphics::scene::scene_renderer::core) source:
        RenderVirtualGeometryVisBuffer64Source,
    pub(in crate::graphics::scene::scene_renderer::core) entry_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) buffer: Option<Arc<wgpu::Buffer>>,
}

pub(super) fn execute_virtual_geometry_visbuffer64_pass(
    device: &wgpu::Device,
    pass_enabled: bool,
    executed_cluster_selection_pass: &VirtualGeometryExecutedClusterSelectionPassOutput,
) -> VirtualGeometryVisBuffer64PassOutput {
    let entries = collect_execution_visbuffer64_entries_from_pass(executed_cluster_selection_pass);
    let packed_words = pack_execution_visbuffer64_entries(&entries);
    VirtualGeometryVisBuffer64PassOutput {
        clear_value: RenderVirtualGeometryVisBuffer64Entry::CLEAR_VALUE,
        entries,
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

fn collect_execution_visbuffer64_entries(
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

fn collect_execution_visbuffer64_entries_from_pass(
    executed_cluster_selection_pass: &VirtualGeometryExecutedClusterSelectionPassOutput,
) -> Vec<RenderVirtualGeometryVisBuffer64Entry> {
    if executed_cluster_selection_pass.source
        == RenderVirtualGeometrySelectedClusterSource::Unavailable
    {
        return Vec::new();
    }

    collect_execution_visbuffer64_entries(&executed_cluster_selection_pass.selected_clusters)
}

fn pack_execution_visbuffer64_entries(
    entries: &[RenderVirtualGeometryVisBuffer64Entry],
) -> Vec<u64> {
    entries.iter().map(|entry| entry.packed_value).collect()
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
    use super::{
        collect_execution_visbuffer64_entries, collect_execution_visbuffer64_entries_from_pass,
        pack_execution_visbuffer64_entries, VirtualGeometryExecutedClusterSelectionPassOutput,
    };
    use crate::core::framework::render::{
        RenderVirtualGeometryExecutionState, RenderVirtualGeometrySelectedCluster,
        RenderVirtualGeometrySelectedClusterSource, RenderVirtualGeometryVisBuffer64Entry,
    };
    use crate::graphics::types::{
        VirtualGeometryClusterSelection, VirtualGeometryPrepareClusterState,
    };

    #[test]
    fn visbuffer64_pass_entries_follow_shared_executed_cluster_selection_order() {
        let entries = collect_execution_visbuffer64_entries(&[
            selected_cluster(
                Some(0),
                42,
                20,
                0,
                400,
                0,
                RenderVirtualGeometryExecutionState::Resident,
            ),
            selected_cluster(
                Some(0),
                42,
                30,
                1,
                300,
                0,
                RenderVirtualGeometryExecutionState::Resident,
            ),
            selected_cluster(
                None,
                43,
                50,
                0,
                500,
                0,
                RenderVirtualGeometryExecutionState::PendingUpload,
            ),
        ]);

        assert_eq!(
            entries,
            vec![
                RenderVirtualGeometryVisBuffer64Entry::from_selected_cluster(
                    0,
                    &selected_cluster(
                        Some(0),
                        42,
                        20,
                        0,
                        400,
                        0,
                        RenderVirtualGeometryExecutionState::Resident,
                    ),
                ),
                RenderVirtualGeometryVisBuffer64Entry::from_selected_cluster(
                    1,
                    &selected_cluster(
                        Some(0),
                        42,
                        30,
                        1,
                        300,
                        0,
                        RenderVirtualGeometryExecutionState::Resident,
                    ),
                ),
                RenderVirtualGeometryVisBuffer64Entry::from_selected_cluster(
                    2,
                    &selected_cluster(
                        None,
                        43,
                        50,
                        0,
                        500,
                        0,
                        RenderVirtualGeometryExecutionState::PendingUpload,
                    ),
                ),
            ],
            "expected the VisBuffer64 compat pass to preserve the ordering and typed identity of the shared executed-cluster seam instead of rebuilding its own submission-key filter, dedupe, or sort layer"
        );
    }

    #[test]
    fn visbuffer64_pass_prefers_pass_owned_selected_clusters() {
        let entries = collect_execution_visbuffer64_entries_from_pass(
            &VirtualGeometryExecutedClusterSelectionPassOutput {
                selections: vec![selection(
                    Some(0),
                    42,
                    7,
                    30,
                    1,
                    300,
                    0,
                    VirtualGeometryPrepareClusterState::Resident,
                )],
                selected_clusters: vec![selected_cluster(
                    Some(0),
                    42,
                    20,
                    0,
                    200,
                    0,
                    RenderVirtualGeometryExecutionState::Resident,
                )],
                source: RenderVirtualGeometrySelectedClusterSource::RenderPathExecutionSelections,
                selected_cluster_count: 1,
                selected_cluster_buffer: None,
            },
        );

        assert_eq!(
            entries,
            vec![RenderVirtualGeometryVisBuffer64Entry::from_selected_cluster(
                0,
                &selected_cluster(
                    Some(0),
                    42,
                    20,
                    0,
                    200,
                    0,
                    RenderVirtualGeometryExecutionState::Resident,
                ),
            )],
            "expected the VisBuffer64 compat pass to project typed entries from the executed pass-owned selected-cluster seam directly once that seam exists instead of re-projecting a second cluster identity list from the internal selection DTO"
        );
    }

    #[test]
    fn visbuffer64_pass_packs_words_from_pass_owned_entries() {
        let entries = vec![
            RenderVirtualGeometryVisBuffer64Entry::from_selected_cluster(
                0,
                &selected_cluster(
                    Some(0),
                    42,
                    20,
                    0,
                    200,
                    0,
                    RenderVirtualGeometryExecutionState::Resident,
                ),
            ),
        ];

        assert_eq!(
            pack_execution_visbuffer64_entries(&entries),
            vec![RenderVirtualGeometryVisBuffer64Entry::packed_value_for(
                Some(0),
                20,
                200,
                0,
                RenderVirtualGeometryExecutionState::Resident,
            )],
            "expected the VisBuffer64 compat pass to derive packed buffer words from its pass-owned typed entry list so later store/readback code can share that same seam"
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
