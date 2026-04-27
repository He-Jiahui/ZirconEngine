use crate::core::framework::render::{
    RenderVirtualGeometryClusterSelectionInputSource, RenderVirtualGeometryCullInputSnapshot,
    RenderVirtualGeometryDebugSnapshot, RenderVirtualGeometryExtract,
    RenderVirtualGeometryNodeAndClusterCullChildWorkItem as RenderNodeAndClusterCullChildWorkItem,
    RenderVirtualGeometryNodeAndClusterCullClusterWorkItem as RenderNodeAndClusterCullClusterWorkItem,
    RenderVirtualGeometryNodeAndClusterCullTraversalRecord as RenderNodeAndClusterCullTraversalRecord,
    RenderVirtualGeometrySelectedCluster, RenderVirtualGeometrySelectedClusterSource,
    RenderVirtualGeometrySubmissionEntry, RenderVirtualGeometrySubmissionRecord,
    RenderVirtualGeometryVisBuffer64Entry,
};

use crate::graphics::types::GraphicsError;
use std::collections::HashMap;

use super::super::scene_renderer::{
    SceneRenderer, VirtualGeometryCullOutputUpdate, VirtualGeometryIndirectOutputUpdate,
    VirtualGeometryLastOutputUpdate, VirtualGeometryRenderPathOutputUpdate,
};
use super::super::scene_renderer_core_render_compiled_scene::{
    SceneRendererCompiledSceneOutputs, VirtualGeometryIndirectStats,
};
use super::virtual_geometry_output_buffers::{
    create_cull_input_buffer, create_hardware_rasterization_buffer,
    create_node_and_cluster_cull_child_work_item_buffer,
    create_node_and_cluster_cull_cluster_work_item_buffer,
    create_node_and_cluster_cull_hierarchy_child_id_buffer,
    create_node_and_cluster_cull_instance_work_item_buffer,
    create_node_and_cluster_cull_launch_worklist_buffer,
    create_node_and_cluster_cull_page_request_buffer,
    create_node_and_cluster_cull_traversal_record_buffer, create_selected_cluster_buffer,
    create_visbuffer64_buffer, pack_hardware_rasterization_records,
};
use super::virtual_geometry_snapshot_rebuild::{
    rebuild_selected_clusters_from_execution_segments,
    rebuild_visbuffer64_entries_from_selected_clusters,
    rebuild_visbuffer_debug_marks_from_selected_clusters, resolve_selected_clusters_for_store,
    resolve_visbuffer64_buffer_source, resolve_visbuffer64_entries_for_store,
};

#[allow(clippy::too_many_arguments)]
pub(in crate::graphics::scene::scene_renderer::core) fn store_last_runtime_outputs(
    renderer: &mut SceneRenderer,
    runtime_outputs: SceneRendererCompiledSceneOutputs,
    virtual_geometry_debug_snapshot: Option<RenderVirtualGeometryDebugSnapshot>,
    virtual_geometry_cull_input: Option<RenderVirtualGeometryCullInputSnapshot>,
    cluster_selection_input_source: RenderVirtualGeometryClusterSelectionInputSource,
    virtual_geometry_extract: Option<&RenderVirtualGeometryExtract>,
) -> Result<(), GraphicsError> {
    let (advanced_plugin_readbacks, virtual_geometry_indirect_stats) = runtime_outputs.into_parts();
    advanced_plugin_readbacks.collect_into_outputs(
        &renderer.backend.device,
        &mut renderer.advanced_plugin_outputs,
    )?;
    let VirtualGeometryIndirectStats {
        draw_count: indirect_draw_count,
        buffer_count: indirect_buffer_count,
        segment_count: indirect_segment_count,
        execution_segment_count,
        execution_page_count,
        execution_resident_segment_count,
        execution_pending_segment_count,
        execution_missing_segment_count,
        execution_repeated_draw_count,
        execution_indirect_offsets,
        execution_segments,
        executed_selected_clusters,
        executed_selected_cluster_source: selected_cluster_render_path_source,
        executed_selected_cluster_count,
        executed_selected_cluster_buffer,
        node_and_cluster_cull_pass,
        hardware_rasterization_pass,
        visbuffer64_pass,
        draw_submission_order: indirect_draw_submission_order,
        draw_submission_records: indirect_draw_submission_records,
        draw_submission_token_records: indirect_draw_submission_token_records,
        args_buffer: indirect_args_buffer,
        args_count: indirect_args_count,
        submission_buffer: indirect_submission_buffer,
        authority_buffer: indirect_authority_buffer,
        draw_ref_buffer: indirect_draw_ref_buffer,
        segment_buffer: indirect_segment_buffer,
        execution_submission_buffer: indirect_execution_submission_buffer,
        execution_args_buffer: indirect_execution_args_buffer,
        execution_authority_buffer: indirect_execution_authority_buffer,
    } = virtual_geometry_indirect_stats;
    let node_and_cluster_cull_source = node_and_cluster_cull_pass.source;
    let node_and_cluster_cull_record_count = node_and_cluster_cull_pass.record_count;
    let node_and_cluster_cull_global_state = node_and_cluster_cull_pass.global_state;
    let node_and_cluster_cull_dispatch_setup = node_and_cluster_cull_pass.dispatch_setup;
    let node_and_cluster_cull_launch_worklist = node_and_cluster_cull_pass.launch_worklist;
    let node_and_cluster_cull_instance_seeds = node_and_cluster_cull_pass.instance_seeds;
    let node_and_cluster_cull_buffer = node_and_cluster_cull_pass.buffer;
    let node_and_cluster_cull_dispatch_setup_buffer =
        node_and_cluster_cull_pass.dispatch_setup_buffer;
    let node_and_cluster_cull_launch_worklist_buffer =
        node_and_cluster_cull_pass.launch_worklist_buffer;
    let node_and_cluster_cull_instance_seed_count = node_and_cluster_cull_pass.instance_seed_count;
    let node_and_cluster_cull_instance_seed_buffer =
        node_and_cluster_cull_pass.instance_seed_buffer;
    let node_and_cluster_cull_instance_work_item_count =
        node_and_cluster_cull_pass.instance_work_item_count;
    let node_and_cluster_cull_instance_work_items = node_and_cluster_cull_pass.instance_work_items;
    let node_and_cluster_cull_instance_work_item_buffer =
        node_and_cluster_cull_pass.instance_work_item_buffer;
    let node_and_cluster_cull_cluster_work_item_count =
        node_and_cluster_cull_pass.cluster_work_item_count;
    let node_and_cluster_cull_cluster_work_items = node_and_cluster_cull_pass.cluster_work_items;
    let node_and_cluster_cull_cluster_work_item_buffer =
        node_and_cluster_cull_pass.cluster_work_item_buffer;
    let node_and_cluster_cull_hierarchy_child_ids = node_and_cluster_cull_pass.hierarchy_child_ids;
    let node_and_cluster_cull_hierarchy_child_id_buffer =
        node_and_cluster_cull_pass.hierarchy_child_id_buffer;
    let node_and_cluster_cull_child_work_item_count =
        node_and_cluster_cull_pass.child_work_item_count;
    let node_and_cluster_cull_child_work_items = node_and_cluster_cull_pass.child_work_items;
    let node_and_cluster_cull_child_work_item_buffer =
        node_and_cluster_cull_pass.child_work_item_buffer;
    let node_and_cluster_cull_traversal_record_count =
        node_and_cluster_cull_pass.traversal_record_count;
    let node_and_cluster_cull_traversal_records = node_and_cluster_cull_pass.traversal_records;
    let node_and_cluster_cull_traversal_record_buffer =
        node_and_cluster_cull_pass.traversal_record_buffer;
    let node_and_cluster_cull_page_request_count = node_and_cluster_cull_pass.page_request_count;
    let node_and_cluster_cull_page_request_ids = node_and_cluster_cull_pass.page_request_ids;
    let node_and_cluster_cull_page_request_buffer = node_and_cluster_cull_pass.page_request_buffer;
    let hardware_rasterization_records = hardware_rasterization_pass.records;
    let hardware_rasterization_render_path_source = hardware_rasterization_pass.source;
    let hardware_rasterization_record_count = hardware_rasterization_pass.record_count;
    let hardware_rasterization_buffer = hardware_rasterization_pass.buffer;
    let visbuffer64_clear_value = visbuffer64_pass.clear_value;
    let visbuffer64_entries = visbuffer64_pass.entries;
    let visbuffer64_render_path_source = visbuffer64_pass.source;
    let visbuffer64_entry_count = visbuffer64_pass.entry_count;
    let visbuffer64_buffer = visbuffer64_pass.buffer;
    let fallback_readback_selected_clusters = if renderer
        .advanced_plugin_outputs
        .has_virtual_geometry_gpu_readback()
    {
        if selected_cluster_render_path_source
            == RenderVirtualGeometrySelectedClusterSource::RenderPathExecutionSelections
        {
            executed_selected_clusters.clone()
        } else {
            virtual_geometry_extract
                .map(|extract| {
                    rebuild_selected_clusters_from_execution_segments(
                        &RenderVirtualGeometryDebugSnapshot {
                            instances: extract.instances.clone(),
                            ..RenderVirtualGeometryDebugSnapshot::default()
                        },
                        Some(extract),
                        &execution_segments,
                    )
                })
                .unwrap_or_default()
        }
    } else {
        Vec::new()
    };
    let fallback_readback_visbuffer64_entries = if renderer
        .advanced_plugin_outputs
        .has_virtual_geometry_gpu_readback()
    {
        if !visbuffer64_entries.is_empty() {
            visbuffer64_entries.clone()
        } else {
            rebuild_visbuffer64_entries_from_selected_clusters(&fallback_readback_selected_clusters)
        }
    } else {
        Vec::new()
    };
    let fallback_readback_has_entries = !fallback_readback_visbuffer64_entries.is_empty();
    let mut virtual_geometry_debug_snapshot = virtual_geometry_debug_snapshot;
    if let Some(snapshot) = virtual_geometry_debug_snapshot.as_mut() {
        let draw_ref_index_by_original_index = indirect_draw_submission_records
            .iter()
            .map(|(_entity, _page_id, draw_ref_index, original_index)| {
                (*original_index, *draw_ref_index)
            })
            .collect::<HashMap<_, _>>();
        let instance_index_by_original_index = execution_segments
            .iter()
            .map(|segment| (segment.original_index as usize, segment.instance_index))
            .collect::<HashMap<_, _>>();
        snapshot.cluster_selection_input_source = cluster_selection_input_source;
        snapshot.cull_input.cluster_selection_input_source = cluster_selection_input_source;
        snapshot.execution_segment_count = execution_segment_count;
        snapshot.execution_page_count = execution_page_count;
        snapshot.execution_resident_segment_count = execution_resident_segment_count;
        snapshot.execution_pending_segment_count = execution_pending_segment_count;
        snapshot.execution_missing_segment_count = execution_missing_segment_count;
        snapshot.execution_repeated_draw_count = execution_repeated_draw_count;
        snapshot.execution_indirect_offsets = execution_indirect_offsets.clone();
        let selected_clusters = resolve_selected_clusters_for_store(
            snapshot,
            virtual_geometry_extract,
            &execution_segments,
            &executed_selected_clusters,
            selected_cluster_render_path_source,
        );
        snapshot.selected_clusters = selected_clusters.clone();
        snapshot.selected_clusters_source = selected_cluster_render_path_source;
        snapshot.node_and_cluster_cull_source = node_and_cluster_cull_source;
        snapshot.node_and_cluster_cull_record_count = node_and_cluster_cull_record_count;
        snapshot.node_and_cluster_cull_instance_seeds =
            node_and_cluster_cull_instance_seeds.clone();
        snapshot.node_and_cluster_cull_instance_work_items =
            node_and_cluster_cull_instance_work_items.clone();
        snapshot.node_and_cluster_cull_cluster_work_items =
            node_and_cluster_cull_cluster_work_items
                .iter()
                .copied()
                .map(RenderNodeAndClusterCullClusterWorkItem::from)
                .collect();
        snapshot.node_and_cluster_cull_child_work_items = node_and_cluster_cull_child_work_items
            .iter()
            .copied()
            .map(RenderNodeAndClusterCullChildWorkItem::from)
            .collect();
        snapshot.node_and_cluster_cull_traversal_records = node_and_cluster_cull_traversal_records
            .iter()
            .copied()
            .map(RenderNodeAndClusterCullTraversalRecord::from)
            .collect();
        snapshot.node_and_cluster_cull_hierarchy_child_ids =
            node_and_cluster_cull_hierarchy_child_ids.clone();
        snapshot.node_and_cluster_cull_page_request_ids =
            node_and_cluster_cull_page_request_ids.clone();
        snapshot.node_and_cluster_cull_dispatch_setup = node_and_cluster_cull_dispatch_setup;
        snapshot.node_and_cluster_cull_launch_worklist =
            node_and_cluster_cull_launch_worklist.clone();
        snapshot.node_and_cluster_cull_global_state = node_and_cluster_cull_global_state.clone();
        if let Some(global_state) = snapshot.node_and_cluster_cull_global_state.as_mut() {
            global_state.cull_input.cluster_selection_input_source = cluster_selection_input_source;
        }
        if let Some(launch_worklist) = snapshot.node_and_cluster_cull_launch_worklist.as_mut() {
            launch_worklist
                .global_state
                .cull_input
                .cluster_selection_input_source = cluster_selection_input_source;
        }
        snapshot.hardware_rasterization_records = hardware_rasterization_records.clone();
        snapshot.hardware_rasterization_source = hardware_rasterization_render_path_source;
        snapshot.visbuffer_debug_marks =
            rebuild_visbuffer_debug_marks_from_selected_clusters(snapshot, &selected_clusters);
        let resolved_visbuffer64_entries = resolve_visbuffer64_entries_for_store(
            &selected_clusters,
            &visbuffer64_entries,
            visbuffer64_render_path_source,
        );
        let resolved_visbuffer64_source = resolve_visbuffer64_buffer_source(
            visbuffer64_render_path_source,
            !resolved_visbuffer64_entries.is_empty(),
            fallback_readback_has_entries,
        );
        snapshot.visbuffer64_source = resolved_visbuffer64_source;
        snapshot.visbuffer64_clear_value = visbuffer64_clear_value;
        snapshot.visbuffer64_entries = resolved_visbuffer64_entries.clone();
        if let Some(readback) = renderer
            .advanced_plugin_outputs
            .virtual_geometry_gpu_readback_mut()
        {
            readback.hardware_rasterization_record_count = hardware_rasterization_record_count;
            readback.hardware_rasterization_source = hardware_rasterization_render_path_source;
            readback.selected_cluster_count = executed_selected_cluster_count;
            readback.selected_cluster_source = selected_cluster_render_path_source;
            readback.selected_clusters = selected_clusters.clone();
            readback.visbuffer64_entry_count = visbuffer64_entry_count;
            readback.visbuffer64_source = visbuffer64_render_path_source;
            readback.visbuffer64_clear_value = visbuffer64_clear_value;
            readback.visbuffer64_entries = resolved_visbuffer64_entries;
        }
        snapshot.execution_segments = execution_segments;
        snapshot.submission_order = indirect_draw_submission_order
            .iter()
            .map(
                |(instance_index, entity, page_id)| RenderVirtualGeometrySubmissionEntry {
                    instance_index: *instance_index,
                    entity: *entity,
                    page_id: *page_id,
                },
            )
            .collect();
        snapshot.submission_records = indirect_draw_submission_token_records
            .iter()
            .map(
                |(entity, page_id, submission_index, draw_ref_rank, original_index)| {
                    RenderVirtualGeometrySubmissionRecord {
                        instance_index: instance_index_by_original_index
                            .get(original_index)
                            .copied()
                            .flatten(),
                        entity: *entity,
                        page_id: *page_id,
                        draw_ref_index: draw_ref_index_by_original_index
                            .get(original_index)
                            .copied(),
                        submission_index: *submission_index,
                        draw_ref_rank: *draw_ref_rank,
                        original_index: *original_index as u32,
                    }
                },
            )
            .collect();
    }
    if let Some(readback) = renderer
        .advanced_plugin_outputs
        .virtual_geometry_gpu_readback_mut()
    {
        readback.hardware_rasterization_record_count = hardware_rasterization_record_count;
        readback.hardware_rasterization_source = hardware_rasterization_render_path_source;
        readback.selected_cluster_count = executed_selected_cluster_count;
        readback.selected_cluster_source = selected_cluster_render_path_source;
        if readback.selected_clusters.is_empty() {
            readback.selected_clusters = fallback_readback_selected_clusters;
        }
        readback.visbuffer64_entry_count = visbuffer64_entry_count;
        readback.visbuffer64_source = visbuffer64_render_path_source;
        readback.visbuffer64_clear_value = RenderVirtualGeometryVisBuffer64Entry::CLEAR_VALUE;
        if readback.visbuffer64_entries.is_empty() {
            readback.visbuffer64_entries = fallback_readback_visbuffer64_entries;
        }
    }
    let visbuffer64_packed_words = virtual_geometry_debug_snapshot
        .as_ref()
        .map(|snapshot| {
            snapshot
                .visbuffer64_entries
                .iter()
                .map(|entry| entry.packed_value)
                .collect::<Vec<_>>()
        })
        .filter(|entries| !entries.is_empty())
        .or_else(|| {
            renderer
                .advanced_plugin_outputs
                .virtual_geometry_gpu_readback()
                .map(|readback| {
                    readback
                        .visbuffer64_entries
                        .iter()
                        .map(|entry| entry.packed_value)
                        .collect::<Vec<_>>()
                })
                .filter(|entries| !entries.is_empty())
        })
        .unwrap_or_default();
    let visbuffer64_source = resolve_visbuffer64_buffer_source(
        visbuffer64_render_path_source,
        !visbuffer64_packed_words.is_empty(),
        fallback_readback_has_entries,
    );
    let selected_cluster_packed_words = virtual_geometry_debug_snapshot
        .as_ref()
        .map(|snapshot| {
            snapshot
                .selected_clusters
                .iter()
                .flat_map(RenderVirtualGeometrySelectedCluster::packed_words)
                .collect::<Vec<_>>()
        })
        .filter(|words| !words.is_empty())
        .unwrap_or_default();
    let selected_cluster_count = if executed_selected_cluster_buffer.is_some() {
        executed_selected_cluster_count
    } else {
        u32::try_from(
            selected_cluster_packed_words.len()
                / RenderVirtualGeometrySelectedCluster::GPU_WORD_COUNT,
        )
        .unwrap_or(u32::MAX)
    };
    let resolved_cull_input = virtual_geometry_debug_snapshot
        .as_ref()
        .map(|snapshot| snapshot.cull_input)
        .or_else(|| {
            virtual_geometry_cull_input.map(|mut cull_input| {
                cull_input.cluster_selection_input_source = cluster_selection_input_source;
                cull_input
            })
        });
    let cull_input_buffer = resolved_cull_input
        .as_ref()
        .and_then(|cull_input| create_cull_input_buffer(&renderer.backend.device, cull_input));
    let node_and_cluster_cull_dispatch_group_count = node_and_cluster_cull_dispatch_setup
        .map(|dispatch_setup| dispatch_setup.dispatch_group_count)
        .unwrap_or([0, 0, 0]);
    let node_and_cluster_cull_launch_worklist_buffer = node_and_cluster_cull_launch_worklist_buffer
        .or_else(|| {
            node_and_cluster_cull_launch_worklist
                .as_ref()
                .and_then(|launch_worklist| {
                    create_node_and_cluster_cull_launch_worklist_buffer(
                        &renderer.backend.device,
                        launch_worklist,
                    )
                })
        });
    let node_and_cluster_cull_instance_work_item_count =
        if node_and_cluster_cull_instance_work_item_buffer.is_some() {
            node_and_cluster_cull_instance_work_item_count
        } else {
            u32::try_from(node_and_cluster_cull_instance_work_items.len()).unwrap_or(u32::MAX)
        };
    let node_and_cluster_cull_instance_work_item_buffer =
        node_and_cluster_cull_instance_work_item_buffer.or_else(|| {
            create_node_and_cluster_cull_instance_work_item_buffer(
                &renderer.backend.device,
                &node_and_cluster_cull_instance_work_items,
            )
        });
    let node_and_cluster_cull_cluster_work_item_count =
        if node_and_cluster_cull_cluster_work_item_buffer.is_some() {
            node_and_cluster_cull_cluster_work_item_count
        } else {
            u32::try_from(node_and_cluster_cull_cluster_work_items.len()).unwrap_or(u32::MAX)
        };
    let node_and_cluster_cull_cluster_work_item_buffer =
        node_and_cluster_cull_cluster_work_item_buffer.or_else(|| {
            create_node_and_cluster_cull_cluster_work_item_buffer(
                &renderer.backend.device,
                &node_and_cluster_cull_cluster_work_items,
            )
        });
    let node_and_cluster_cull_hierarchy_child_id_count =
        u32::try_from(node_and_cluster_cull_hierarchy_child_ids.len()).unwrap_or(u32::MAX);
    let node_and_cluster_cull_hierarchy_child_id_buffer =
        node_and_cluster_cull_hierarchy_child_id_buffer.or_else(|| {
            create_node_and_cluster_cull_hierarchy_child_id_buffer(
                &renderer.backend.device,
                &node_and_cluster_cull_hierarchy_child_ids,
            )
        });
    let node_and_cluster_cull_child_work_item_count =
        if node_and_cluster_cull_child_work_item_buffer.is_some() {
            node_and_cluster_cull_child_work_item_count
        } else {
            u32::try_from(node_and_cluster_cull_child_work_items.len()).unwrap_or(u32::MAX)
        };
    let node_and_cluster_cull_child_work_item_buffer = node_and_cluster_cull_child_work_item_buffer
        .or_else(|| {
            create_node_and_cluster_cull_child_work_item_buffer(
                &renderer.backend.device,
                &node_and_cluster_cull_child_work_items,
            )
        });
    let node_and_cluster_cull_traversal_record_count =
        if node_and_cluster_cull_traversal_record_buffer.is_some() {
            node_and_cluster_cull_traversal_record_count
        } else {
            u32::try_from(node_and_cluster_cull_traversal_records.len()).unwrap_or(u32::MAX)
        };
    let node_and_cluster_cull_traversal_record_buffer =
        node_and_cluster_cull_traversal_record_buffer.or_else(|| {
            create_node_and_cluster_cull_traversal_record_buffer(
                &renderer.backend.device,
                &node_and_cluster_cull_traversal_records,
            )
        });
    let node_and_cluster_cull_page_request_count =
        if node_and_cluster_cull_page_request_buffer.is_some() {
            node_and_cluster_cull_page_request_count
        } else {
            u32::try_from(node_and_cluster_cull_page_request_ids.len()).unwrap_or(u32::MAX)
        };
    let node_and_cluster_cull_page_request_buffer = node_and_cluster_cull_page_request_buffer
        .or_else(|| {
            create_node_and_cluster_cull_page_request_buffer(
                &renderer.backend.device,
                &node_and_cluster_cull_page_request_ids,
            )
        });
    let selected_cluster_buffer = executed_selected_cluster_buffer.or_else(|| {
        create_selected_cluster_buffer(&renderer.backend.device, &selected_cluster_packed_words)
    });
    let visbuffer64_entry_count = if visbuffer64_buffer.is_some() {
        visbuffer64_entry_count
    } else {
        u32::try_from(visbuffer64_packed_words.len()).unwrap_or(u32::MAX)
    };
    let visbuffer64_buffer = visbuffer64_buffer
        .or_else(|| create_visbuffer64_buffer(&renderer.backend.device, &visbuffer64_packed_words));
    let hardware_rasterization_packed_words =
        pack_hardware_rasterization_records(&hardware_rasterization_records);
    let hardware_rasterization_record_count = if hardware_rasterization_buffer.is_some() {
        hardware_rasterization_record_count
    } else {
        u32::try_from(hardware_rasterization_records.len()).unwrap_or(u32::MAX)
    };
    let hardware_rasterization_buffer = hardware_rasterization_buffer.or_else(|| {
        create_hardware_rasterization_buffer(
            &renderer.backend.device,
            &hardware_rasterization_packed_words,
        )
    });
    renderer
        .advanced_plugin_outputs
        .store_virtual_geometry_last_outputs(VirtualGeometryLastOutputUpdate {
            node_and_cluster_cull: VirtualGeometryCullOutputUpdate {
                cluster_selection_input_source,
                cull_input_buffer,
                node_and_cluster_cull_source,
                node_and_cluster_cull_record_count,
                node_and_cluster_cull_global_state,
                node_and_cluster_cull_dispatch_group_count,
                node_and_cluster_cull_buffer,
                node_and_cluster_cull_dispatch_setup_buffer,
                node_and_cluster_cull_launch_worklist_buffer,
                node_and_cluster_cull_instance_seed_count,
                node_and_cluster_cull_instance_seed_buffer,
                node_and_cluster_cull_instance_work_item_count,
                node_and_cluster_cull_instance_work_item_buffer,
                node_and_cluster_cull_cluster_work_item_count,
                node_and_cluster_cull_cluster_work_item_buffer,
                node_and_cluster_cull_hierarchy_child_id_count,
                node_and_cluster_cull_hierarchy_child_id_buffer,
                node_and_cluster_cull_child_work_item_count,
                node_and_cluster_cull_child_work_item_buffer,
                node_and_cluster_cull_traversal_record_count,
                node_and_cluster_cull_traversal_record_buffer,
                node_and_cluster_cull_page_request_count,
                node_and_cluster_cull_page_request_ids,
                node_and_cluster_cull_page_request_buffer,
            },
            render_path: VirtualGeometryRenderPathOutputUpdate {
                selected_cluster_count,
                selected_cluster_source: selected_cluster_render_path_source,
                selected_cluster_buffer,
                visbuffer64_clear_value,
                visbuffer64_source,
                visbuffer64_entry_count,
                visbuffer64_buffer,
                hardware_rasterization_source: hardware_rasterization_render_path_source,
                hardware_rasterization_record_count,
                hardware_rasterization_buffer,
                debug_snapshot: virtual_geometry_debug_snapshot,
            },
            indirect: VirtualGeometryIndirectOutputUpdate {
                indirect_draw_count,
                indirect_buffer_count,
                indirect_segment_count,
                execution_segment_count,
                execution_page_count,
                execution_resident_segment_count,
                execution_pending_segment_count,
                execution_missing_segment_count,
                execution_repeated_draw_count,
                execution_indirect_offsets,
                mesh_draw_submission_order: indirect_draw_submission_order,
                mesh_draw_submission_records: indirect_draw_submission_records,
                mesh_draw_submission_token_records: indirect_draw_submission_token_records,
                indirect_args_buffer,
                indirect_args_count,
                indirect_submission_buffer,
                indirect_authority_buffer,
                indirect_draw_refs_buffer: indirect_draw_ref_buffer,
                indirect_segments_buffer: indirect_segment_buffer,
                indirect_execution_submission_buffer,
                indirect_execution_args_buffer,
                indirect_execution_authority_buffer,
            },
        });
    Ok(())
}
