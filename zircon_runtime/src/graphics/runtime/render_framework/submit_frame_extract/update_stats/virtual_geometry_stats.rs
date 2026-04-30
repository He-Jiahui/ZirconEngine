use super::super::super::render_framework_state::RenderFrameworkState;
use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::submission_record_update::SubmissionRecordUpdate;
use std::collections::BTreeSet;

pub(super) fn update_virtual_geometry_stats(
    state: &mut RenderFrameworkState,
    context: &FrameSubmissionContext,
    record_update: &SubmissionRecordUpdate,
) {
    let virtual_geometry_stats = record_update.virtual_geometry_stats();
    let virtual_geometry_extract = context.virtual_geometry_extract();
    let cull_input_snapshot = virtual_geometry_extract
        .and_then(|_| state.renderer.last_virtual_geometry_debug_snapshot())
        .map(|snapshot| snapshot.cull_input);
    state.stats.last_virtual_geometry_cluster_budget = cull_input_snapshot
        .map(|snapshot| snapshot.cluster_budget as usize)
        .unwrap_or_else(|| {
            virtual_geometry_extract
                .map(|extract| extract.cluster_budget as usize)
                .unwrap_or(0)
        });
    state.stats.last_virtual_geometry_page_budget = cull_input_snapshot
        .map(|snapshot| snapshot.page_budget as usize)
        .unwrap_or_else(|| {
            virtual_geometry_extract
                .map(|extract| extract.page_budget as usize)
                .unwrap_or(0)
        });
    state.stats.last_virtual_geometry_input_cluster_count = cull_input_snapshot
        .map(|snapshot| snapshot.cluster_count as usize)
        .unwrap_or_else(|| {
            virtual_geometry_extract
                .map(|extract| extract.clusters.len())
                .unwrap_or(0)
        });
    state.stats.last_virtual_geometry_input_page_count = cull_input_snapshot
        .map(|snapshot| snapshot.page_count as usize)
        .unwrap_or_else(|| {
            virtual_geometry_extract
                .map(|extract| extract.pages.len())
                .unwrap_or(0)
        });
    state.stats.last_virtual_geometry_visible_cluster_count = context
        .visibility_context()
        .virtual_geometry_visible_clusters
        .len();
    state.stats.last_virtual_geometry_visible_entity_count = cull_input_snapshot
        .map(|snapshot| snapshot.visible_entity_count as usize)
        .unwrap_or_else(|| {
            virtual_geometry_extract
                .map(visible_entity_count_from_extract)
                .unwrap_or(0)
        });
    state.stats.last_virtual_geometry_instance_count = virtual_geometry_extract
        .map(|extract| extract.instances.len())
        .unwrap_or(0);
    state.stats.last_virtual_geometry_requested_page_count = context
        .virtual_geometry_page_upload_plan()
        .map(|plan| plan.requested_pages.len())
        .unwrap_or(0);
    state.stats.last_virtual_geometry_dirty_page_count = context
        .virtual_geometry_page_upload_plan()
        .map(|plan| plan.dirty_requested_pages.len())
        .unwrap_or(0);
    state.stats.last_virtual_geometry_forced_mip =
        virtual_geometry_extract.and_then(|extract| extract.debug.forced_mip);
    state.stats.last_virtual_geometry_freeze_cull = virtual_geometry_extract
        .map(|extract| extract.debug.freeze_cull)
        .unwrap_or(false);
    state.stats.last_virtual_geometry_visualize_bvh = virtual_geometry_extract
        .map(|extract| extract.debug.visualize_bvh)
        .unwrap_or(false);
    state.stats.last_virtual_geometry_visualize_visbuffer = virtual_geometry_extract
        .map(|extract| extract.debug.visualize_visbuffer)
        .unwrap_or(false);
    state.stats.last_virtual_geometry_print_leaf_clusters = virtual_geometry_extract
        .map(|extract| extract.debug.print_leaf_clusters)
        .unwrap_or(false);
    state.stats.last_virtual_geometry_page_table_entry_count =
        virtual_geometry_stats.page_table_entry_count();
    state.stats.last_virtual_geometry_resident_page_count =
        virtual_geometry_stats.resident_page_count();
    state.stats.last_virtual_geometry_pending_request_count =
        virtual_geometry_stats.pending_request_count();
    state.stats.last_virtual_geometry_completed_page_count =
        virtual_geometry_stats.completed_page_count();
    state.stats.last_virtual_geometry_replaced_page_count =
        virtual_geometry_stats.replaced_page_count();
    state.stats.last_virtual_geometry_indirect_draw_count =
        state.renderer.last_virtual_geometry_indirect_draw_count() as usize;
    state.stats.last_virtual_geometry_indirect_buffer_count =
        state.renderer.last_virtual_geometry_indirect_buffer_count() as usize;
    state.stats.last_virtual_geometry_indirect_args_count =
        state.renderer.last_virtual_geometry_indirect_args_count() as usize;
    state.stats.last_virtual_geometry_indirect_segment_count =
        virtual_geometry_stats.indirect_segment_count();
    state.stats.last_virtual_geometry_execution_segment_count = state
        .renderer
        .last_virtual_geometry_execution_segment_count()
        as usize;
    state.stats.last_virtual_geometry_execution_page_count =
        state.renderer.last_virtual_geometry_execution_page_count() as usize;
    state
        .stats
        .last_virtual_geometry_execution_resident_segment_count = state
        .renderer
        .last_virtual_geometry_execution_resident_segment_count()
        as usize;
    state
        .stats
        .last_virtual_geometry_execution_pending_segment_count = state
        .renderer
        .last_virtual_geometry_execution_pending_segment_count()
        as usize;
    state
        .stats
        .last_virtual_geometry_execution_missing_segment_count = state
        .renderer
        .last_virtual_geometry_execution_missing_segment_count()
        as usize;
    state
        .stats
        .last_virtual_geometry_execution_repeated_draw_count = state
        .renderer
        .last_virtual_geometry_execution_repeated_draw_count()
        as usize;
    state
        .stats
        .last_virtual_geometry_cluster_selection_input_source = virtual_geometry_extract
        .map(|_| {
            state
                .renderer
                .last_virtual_geometry_cluster_selection_input_source()
        })
        .unwrap_or_default();
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_source = virtual_geometry_extract
        .map(|_| {
            state
                .renderer
                .last_virtual_geometry_node_and_cluster_cull_source()
        })
        .unwrap_or_default();
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_record_count = virtual_geometry_extract
        .map(|_| {
            state
                .renderer
                .last_virtual_geometry_node_and_cluster_cull_record_count() as usize
        })
        .unwrap_or(0);
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_dispatch_group_count =
        virtual_geometry_extract
            .map(|_| {
                state
                    .renderer
                    .last_virtual_geometry_node_and_cluster_cull_dispatch_group_count()
                    .map(|group_count| group_count as usize)
            })
            .unwrap_or([0, 0, 0]);
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_instance_seed_count = virtual_geometry_extract
        .map(|_| {
            state
                .renderer
                .last_virtual_geometry_node_and_cluster_cull_instance_seed_count()
                as usize
        })
        .unwrap_or(0);
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_instance_work_item_count =
        virtual_geometry_extract
            .map(|_| {
                state
                    .renderer
                    .last_virtual_geometry_node_and_cluster_cull_instance_work_item_count()
                    as usize
            })
            .unwrap_or(0);
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_cluster_work_item_count =
        virtual_geometry_extract
            .map(|_| {
                state
                    .renderer
                    .last_virtual_geometry_node_and_cluster_cull_cluster_work_item_count()
                    as usize
            })
            .unwrap_or(0);
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_hierarchy_child_id_count =
        virtual_geometry_extract
            .map(|_| {
                state
                    .renderer
                    .last_virtual_geometry_node_and_cluster_cull_hierarchy_child_id_count()
                    as usize
            })
            .unwrap_or(0);
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_child_work_item_count =
        virtual_geometry_extract
            .map(|_| {
                state
                    .renderer
                    .last_virtual_geometry_node_and_cluster_cull_child_work_item_count()
                    as usize
            })
            .unwrap_or(0);
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_traversal_record_count =
        virtual_geometry_extract
            .map(|_| {
                state
                    .renderer
                    .last_virtual_geometry_node_and_cluster_cull_traversal_record_count()
                    as usize
            })
            .unwrap_or(0);
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_page_request_count = virtual_geometry_extract
        .map(|_| {
            state
                .renderer
                .last_virtual_geometry_node_and_cluster_cull_page_request_count()
                as usize
        })
        .unwrap_or(0);
    state.stats.last_virtual_geometry_selected_cluster_source = virtual_geometry_extract
        .map(|_| {
            state
                .renderer
                .last_virtual_geometry_selected_cluster_source()
        })
        .unwrap_or_default();
    state.stats.last_virtual_geometry_selected_cluster_count = virtual_geometry_extract
        .map(|_| {
            state
                .renderer
                .last_virtual_geometry_selected_cluster_count() as usize
        })
        .unwrap_or(0);
    state.stats.last_virtual_geometry_visbuffer64_source = virtual_geometry_extract
        .map(|_| state.renderer.last_virtual_geometry_visbuffer64_source())
        .unwrap_or_default();
    state.stats.last_virtual_geometry_visbuffer64_entry_count = virtual_geometry_extract
        .map(|_| {
            state
                .renderer
                .last_virtual_geometry_visbuffer64_entry_count() as usize
        })
        .unwrap_or(0);
    state
        .stats
        .last_virtual_geometry_hardware_rasterization_source = virtual_geometry_extract
        .map(|_| {
            state
                .renderer
                .last_virtual_geometry_hardware_rasterization_source()
        })
        .unwrap_or_default();
    state
        .stats
        .last_virtual_geometry_hardware_rasterization_record_count = virtual_geometry_extract
        .map(|_| {
            state
                .renderer
                .last_virtual_geometry_hardware_rasterization_record_count() as usize
        })
        .unwrap_or(0);
}

pub(super) fn reset_virtual_geometry_stats(state: &mut RenderFrameworkState) {
    state.stats.last_virtual_geometry_cluster_budget = 0;
    state.stats.last_virtual_geometry_page_budget = 0;
    state.stats.last_virtual_geometry_input_cluster_count = 0;
    state.stats.last_virtual_geometry_input_page_count = 0;
    state.stats.last_virtual_geometry_visible_cluster_count = 0;
    state.stats.last_virtual_geometry_visible_entity_count = 0;
    state.stats.last_virtual_geometry_instance_count = 0;
    state.stats.last_virtual_geometry_requested_page_count = 0;
    state.stats.last_virtual_geometry_dirty_page_count = 0;
    state.stats.last_virtual_geometry_forced_mip = None;
    state.stats.last_virtual_geometry_freeze_cull = false;
    state.stats.last_virtual_geometry_visualize_bvh = false;
    state.stats.last_virtual_geometry_visualize_visbuffer = false;
    state.stats.last_virtual_geometry_print_leaf_clusters = false;
    state.stats.last_virtual_geometry_page_table_entry_count = 0;
    state.stats.last_virtual_geometry_resident_page_count = 0;
    state.stats.last_virtual_geometry_pending_request_count = 0;
    state.stats.last_virtual_geometry_completed_page_count = 0;
    state.stats.last_virtual_geometry_replaced_page_count = 0;
    state.stats.last_virtual_geometry_indirect_draw_count = 0;
    state.stats.last_virtual_geometry_indirect_buffer_count = 0;
    state.stats.last_virtual_geometry_indirect_args_count = 0;
    state.stats.last_virtual_geometry_indirect_segment_count = 0;
    state.stats.last_virtual_geometry_execution_segment_count = 0;
    state.stats.last_virtual_geometry_execution_page_count = 0;
    state
        .stats
        .last_virtual_geometry_execution_resident_segment_count = 0;
    state
        .stats
        .last_virtual_geometry_execution_pending_segment_count = 0;
    state
        .stats
        .last_virtual_geometry_execution_missing_segment_count = 0;
    state
        .stats
        .last_virtual_geometry_execution_repeated_draw_count = 0;
    state
        .stats
        .last_virtual_geometry_cluster_selection_input_source = Default::default();
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_source = Default::default();
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_record_count = 0;
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_dispatch_group_count = [0, 0, 0];
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_instance_seed_count = 0;
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_instance_work_item_count = 0;
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_cluster_work_item_count = 0;
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_hierarchy_child_id_count = 0;
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_child_work_item_count = 0;
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_traversal_record_count = 0;
    state
        .stats
        .last_virtual_geometry_node_and_cluster_cull_page_request_count = 0;
    state.stats.last_virtual_geometry_selected_cluster_source = Default::default();
    state.stats.last_virtual_geometry_selected_cluster_count = 0;
    state.stats.last_virtual_geometry_visbuffer64_source = Default::default();
    state.stats.last_virtual_geometry_visbuffer64_entry_count = 0;
    state
        .stats
        .last_virtual_geometry_hardware_rasterization_source = Default::default();
    state
        .stats
        .last_virtual_geometry_hardware_rasterization_record_count = 0;
}

fn visible_entity_count_from_extract(
    extract: &crate::core::framework::render::RenderVirtualGeometryExtract,
) -> usize {
    if !extract.instances.is_empty() {
        return extract
            .instances
            .iter()
            .map(|instance| instance.entity)
            .collect::<BTreeSet<_>>()
            .len();
    }

    extract
        .clusters
        .iter()
        .map(|cluster| cluster.entity)
        .collect::<BTreeSet<_>>()
        .len()
}
