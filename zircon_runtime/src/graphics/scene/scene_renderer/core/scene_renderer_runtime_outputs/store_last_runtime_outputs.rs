use crate::core::framework::render::{
    RenderVirtualGeometryClusterSelectionInputSource, RenderVirtualGeometryCullInputSnapshot,
    RenderVirtualGeometryDebugSnapshot, RenderVirtualGeometryExtract,
    RenderVirtualGeometryHardwareRasterizationSource,
    RenderVirtualGeometryNodeAndClusterCullSource, RenderVirtualGeometrySelectedClusterSource,
    RenderVirtualGeometryVisBuffer64Entry, RenderVirtualGeometryVisBuffer64Source,
};
use crate::graphics::types::GraphicsError;

use super::super::scene_renderer::{
    SceneRenderer, VirtualGeometryCullOutputUpdate, VirtualGeometryIndirectOutputUpdate,
    VirtualGeometryLastOutputUpdate, VirtualGeometryRenderPathOutputUpdate,
};
use super::super::scene_renderer_core_render_compiled_scene::SceneRendererCompiledSceneOutputs;

#[allow(clippy::too_many_arguments)]
pub(in crate::graphics::scene::scene_renderer::core) fn store_last_runtime_outputs(
    renderer: &mut SceneRenderer,
    runtime_outputs: SceneRendererCompiledSceneOutputs,
    virtual_geometry_debug_snapshot: Option<RenderVirtualGeometryDebugSnapshot>,
    virtual_geometry_cull_input: Option<RenderVirtualGeometryCullInputSnapshot>,
    cluster_selection_input_source: RenderVirtualGeometryClusterSelectionInputSource,
    _virtual_geometry_extract: Option<&RenderVirtualGeometryExtract>,
) -> Result<(), GraphicsError> {
    runtime_outputs.into_parts().collect_into_outputs(
        &renderer.backend.device,
        &mut renderer.advanced_plugin_outputs,
    )?;

    let mut virtual_geometry_debug_snapshot = virtual_geometry_debug_snapshot;
    if let Some(snapshot) = virtual_geometry_debug_snapshot.as_mut() {
        snapshot.cluster_selection_input_source = cluster_selection_input_source;
        snapshot.cull_input.cluster_selection_input_source = cluster_selection_input_source;
    }
    let resolved_cull_input = virtual_geometry_debug_snapshot
        .as_ref()
        .map(|snapshot| snapshot.cull_input)
        .or_else(|| {
            virtual_geometry_cull_input.map(|mut cull_input| {
                cull_input.cluster_selection_input_source = cluster_selection_input_source;
                cull_input
            })
        });
    let snapshot_ref = virtual_geometry_debug_snapshot.as_ref();

    renderer
        .advanced_plugin_outputs
        .store_virtual_geometry_last_outputs(VirtualGeometryLastOutputUpdate {
            node_and_cluster_cull: VirtualGeometryCullOutputUpdate {
                cluster_selection_input_source,
                cull_input_buffer: None,
                node_and_cluster_cull_source: snapshot_ref
                    .map(|snapshot| snapshot.node_and_cluster_cull_source)
                    .unwrap_or(RenderVirtualGeometryNodeAndClusterCullSource::Unavailable),
                node_and_cluster_cull_record_count: snapshot_ref
                    .map(|snapshot| snapshot.node_and_cluster_cull_record_count)
                    .unwrap_or(0),
                node_and_cluster_cull_global_state: snapshot_ref
                    .and_then(|snapshot| snapshot.node_and_cluster_cull_global_state.clone()),
                node_and_cluster_cull_dispatch_group_count: [0, 0, 0],
                node_and_cluster_cull_buffer: None,
                node_and_cluster_cull_dispatch_setup_buffer: None,
                node_and_cluster_cull_launch_worklist_buffer: None,
                node_and_cluster_cull_instance_seed_count: snapshot_ref
                    .map(|snapshot| {
                        saturated_u32_len(snapshot.node_and_cluster_cull_instance_seeds.len())
                    })
                    .unwrap_or(0),
                node_and_cluster_cull_instance_seed_buffer: None,
                node_and_cluster_cull_instance_work_item_count: snapshot_ref
                    .map(|snapshot| {
                        saturated_u32_len(snapshot.node_and_cluster_cull_instance_work_items.len())
                    })
                    .unwrap_or(0),
                node_and_cluster_cull_instance_work_item_buffer: None,
                node_and_cluster_cull_cluster_work_item_count: snapshot_ref
                    .map(|snapshot| {
                        saturated_u32_len(snapshot.node_and_cluster_cull_cluster_work_items.len())
                    })
                    .unwrap_or(0),
                node_and_cluster_cull_cluster_work_item_buffer: None,
                node_and_cluster_cull_hierarchy_child_id_count: snapshot_ref
                    .map(|snapshot| {
                        saturated_u32_len(snapshot.node_and_cluster_cull_hierarchy_child_ids.len())
                    })
                    .unwrap_or(0),
                node_and_cluster_cull_hierarchy_child_id_buffer: None,
                node_and_cluster_cull_child_work_item_count: snapshot_ref
                    .map(|snapshot| {
                        saturated_u32_len(snapshot.node_and_cluster_cull_child_work_items.len())
                    })
                    .unwrap_or(0),
                node_and_cluster_cull_child_work_item_buffer: None,
                node_and_cluster_cull_traversal_record_count: snapshot_ref
                    .map(|snapshot| {
                        saturated_u32_len(snapshot.node_and_cluster_cull_traversal_records.len())
                    })
                    .unwrap_or(0),
                node_and_cluster_cull_traversal_record_buffer: None,
                node_and_cluster_cull_page_request_count: snapshot_ref
                    .map(|snapshot| {
                        saturated_u32_len(snapshot.node_and_cluster_cull_page_request_ids.len())
                    })
                    .unwrap_or(0),
                node_and_cluster_cull_page_request_ids: snapshot_ref
                    .map(|snapshot| snapshot.node_and_cluster_cull_page_request_ids.clone())
                    .unwrap_or_default(),
                node_and_cluster_cull_page_request_buffer: None,
            },
            render_path: VirtualGeometryRenderPathOutputUpdate {
                selected_cluster_count: snapshot_ref
                    .map(|snapshot| saturated_u32_len(snapshot.selected_clusters.len()))
                    .unwrap_or(0),
                selected_cluster_source: snapshot_ref
                    .map(|snapshot| snapshot.selected_clusters_source)
                    .unwrap_or(RenderVirtualGeometrySelectedClusterSource::Unavailable),
                selected_cluster_buffer: None,
                visbuffer64_clear_value: snapshot_ref
                    .map(|snapshot| snapshot.visbuffer64_clear_value)
                    .unwrap_or(RenderVirtualGeometryVisBuffer64Entry::CLEAR_VALUE),
                visbuffer64_source: snapshot_ref
                    .map(|snapshot| snapshot.visbuffer64_source)
                    .unwrap_or(RenderVirtualGeometryVisBuffer64Source::Unavailable),
                visbuffer64_entry_count: snapshot_ref
                    .map(|snapshot| saturated_u32_len(snapshot.visbuffer64_entries.len()))
                    .unwrap_or(0),
                visbuffer64_buffer: None,
                hardware_rasterization_source: snapshot_ref
                    .map(|snapshot| snapshot.hardware_rasterization_source)
                    .unwrap_or(RenderVirtualGeometryHardwareRasterizationSource::Unavailable),
                hardware_rasterization_record_count: snapshot_ref
                    .map(|snapshot| {
                        saturated_u32_len(snapshot.hardware_rasterization_records.len())
                    })
                    .unwrap_or(0),
                hardware_rasterization_buffer: None,
                debug_snapshot: snapshot_ref.cloned(),
            },
            indirect: VirtualGeometryIndirectOutputUpdate {
                indirect_draw_count: 0,
                indirect_buffer_count: 0,
                indirect_segment_count: 0,
                execution_segment_count: snapshot_ref
                    .map(|snapshot| snapshot.execution_segment_count)
                    .unwrap_or(0),
                execution_page_count: snapshot_ref
                    .map(|snapshot| snapshot.execution_page_count)
                    .unwrap_or(0),
                execution_resident_segment_count: snapshot_ref
                    .map(|snapshot| snapshot.execution_resident_segment_count)
                    .unwrap_or(0),
                execution_pending_segment_count: snapshot_ref
                    .map(|snapshot| snapshot.execution_pending_segment_count)
                    .unwrap_or(0),
                execution_missing_segment_count: snapshot_ref
                    .map(|snapshot| snapshot.execution_missing_segment_count)
                    .unwrap_or(0),
                execution_repeated_draw_count: snapshot_ref
                    .map(|snapshot| snapshot.execution_repeated_draw_count)
                    .unwrap_or(0),
                execution_indirect_offsets: snapshot_ref
                    .map(|snapshot| snapshot.execution_indirect_offsets.clone())
                    .unwrap_or_default(),
                mesh_draw_submission_order: snapshot_ref
                    .map(|snapshot| {
                        snapshot
                            .submission_order
                            .iter()
                            .map(|entry| (entry.instance_index, entry.entity, entry.page_id))
                            .collect()
                    })
                    .unwrap_or_default(),
                mesh_draw_submission_records: snapshot_ref
                    .map(|snapshot| {
                        snapshot
                            .submission_records
                            .iter()
                            .map(|record| {
                                (
                                    record.entity,
                                    record.page_id,
                                    record.draw_ref_index.unwrap_or(record.submission_index),
                                    record.original_index as usize,
                                )
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
                mesh_draw_submission_token_records: snapshot_ref
                    .map(|snapshot| {
                        snapshot
                            .submission_records
                            .iter()
                            .map(|record| {
                                (
                                    record.entity,
                                    record.page_id,
                                    record.submission_index,
                                    record.draw_ref_rank,
                                    record.original_index as usize,
                                )
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
                indirect_args_buffer: None,
                indirect_args_count: 0,
                indirect_submission_buffer: None,
                indirect_authority_buffer: None,
                indirect_draw_refs_buffer: None,
                indirect_segments_buffer: None,
                indirect_execution_submission_buffer: None,
                indirect_execution_args_buffer: None,
                indirect_execution_authority_buffer: None,
            },
        });

    if let Some(cull_input) = resolved_cull_input {
        let _ = cull_input;
    }

    Ok(())
}

fn saturated_u32_len(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
}
