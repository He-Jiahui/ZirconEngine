use std::sync::Arc;

use wgpu::util::DeviceExt;
use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryCullInputSnapshot,
    RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    RenderVirtualGeometryNodeAndClusterCullSource,
};

use crate::virtual_geometry::renderer::{VirtualGeometryGpuResources, VirtualGeometryRenderFrame};

use super::output::VirtualGeometryNodeAndClusterCullPassOutput;
use super::page_requests::append_node_and_cluster_cull_page_requests;
use super::store_parts::VirtualGeometryNodeAndClusterCullPassStoreParts;
use super::{buffers, child_decision, child_worklist, startup_worklist, traversal};

const NODE_AND_CLUSTER_CULL_BASELINE_TRAVERSAL_WAVE_LIMIT: usize = 8;

pub(in crate::virtual_geometry::renderer::root_render_passes) fn execute_virtual_geometry_node_and_cluster_cull_pass(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    virtual_geometry_resources: &VirtualGeometryGpuResources,
    pass_enabled: bool,
    frame: &VirtualGeometryRenderFrame,
    cull_input: Option<&RenderVirtualGeometryCullInputSnapshot>,
    previous_global_state: Option<&RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot>,
) -> VirtualGeometryNodeAndClusterCullPassOutput {
    if !pass_enabled {
        return empty_node_and_cluster_cull_pass_output(
            RenderVirtualGeometryNodeAndClusterCullSource::Unavailable,
        );
    }

    let Some(cull_input) = cull_input else {
        return empty_node_and_cluster_cull_pass_output(
            RenderVirtualGeometryNodeAndClusterCullSource::RenderPathClearOnly,
        );
    };

    let global_state = startup_worklist::build_node_and_cluster_cull_global_state(
        frame,
        cull_input,
        previous_global_state,
    );
    let packed_words = global_state.packed_words();
    let instance_seeds =
        startup_worklist::build_node_and_cluster_cull_instance_seeds(frame, &global_state);
    let instance_seed_count = u32::try_from(instance_seeds.len()).unwrap_or(u32::MAX);
    let dispatch_setup = startup_worklist::build_node_and_cluster_cull_dispatch_setup(
        &global_state,
        instance_seed_count,
    );
    let launch_worklist = startup_worklist::build_node_and_cluster_cull_launch_worklist(
        &global_state,
        dispatch_setup,
        &instance_seeds,
    );
    let instance_work_items =
        startup_worklist::build_node_and_cluster_cull_instance_work_items(&launch_worklist);
    let instance_work_item_count = u32::try_from(instance_work_items.len()).unwrap_or(u32::MAX);
    let cluster_work_items = startup_worklist::build_node_and_cluster_cull_cluster_work_items(
        frame,
        &instance_work_items,
        global_state.cull_input.cluster_count,
    );
    let cluster_work_item_count = u32::try_from(cluster_work_items.len()).unwrap_or(u32::MAX);
    let clusters = frame
        .extract
        .geometry
        .virtual_geometry
        .as_ref()
        .map(|extract| extract.clusters.as_slice())
        .unwrap_or(&[]);
    let hierarchy_nodes = frame
        .extract
        .geometry
        .virtual_geometry
        .as_ref()
        .map(|extract| extract.hierarchy_nodes.as_slice())
        .unwrap_or(&[]);
    let hierarchy_child_ids = frame
        .extract
        .geometry
        .virtual_geometry
        .as_ref()
        .map(|extract| extract.hierarchy_child_ids.clone())
        .unwrap_or_default();
    let pages = frame
        .extract
        .geometry
        .virtual_geometry
        .as_ref()
        .map(|extract| extract.pages.as_slice())
        .unwrap_or(&[]);
    let mut traversal_records = traversal::build_node_and_cluster_cull_traversal_records(
        &cluster_work_items,
        hierarchy_nodes,
    );
    let mut child_work_items = Vec::new();
    let mut page_request_ids = Vec::new();
    let mut current_wave_records = traversal_records.clone();
    let page_request_budget = cull_input
        .page_budget
        .saturating_sub(cull_input.pending_page_request_count);

    for _ in 0..NODE_AND_CLUSTER_CULL_BASELINE_TRAVERSAL_WAVE_LIMIT {
        let wave_child_work_items = child_worklist::build_node_and_cluster_cull_child_work_items(
            &current_wave_records,
            &hierarchy_child_ids,
        );
        if wave_child_work_items.is_empty() {
            break;
        }

        let child_visit_records = child_worklist::build_node_and_cluster_cull_child_visit_records(
            &wave_child_work_items,
            hierarchy_nodes,
            traversal::next_node_and_cluster_cull_traversal_index(&traversal_records),
        );
        child_work_items.extend(wave_child_work_items);
        if child_visit_records.is_empty() {
            break;
        }

        let child_decision_output =
            child_decision::build_node_and_cluster_cull_child_decision_output(
                &child_visit_records,
                &global_state,
                &frame.extract.view.camera,
                clusters,
                hierarchy_nodes,
                pages,
                traversal::next_node_and_cluster_cull_traversal_index(&child_visit_records),
            );
        append_node_and_cluster_cull_page_requests(
            &mut page_request_ids,
            &child_decision_output.requested_page_ids,
            page_request_budget,
        );
        traversal_records.extend(child_visit_records);
        current_wave_records = child_decision_output.traversal_records.clone();
        traversal_records.extend(child_decision_output.traversal_records);
    }
    let child_work_item_count = u32::try_from(child_work_items.len()).unwrap_or(u32::MAX);
    let traversal_record_count = u32::try_from(traversal_records.len()).unwrap_or(u32::MAX);
    let page_request_count = u32::try_from(page_request_ids.len()).unwrap_or(u32::MAX);
    let launch_worklist_buffer =
        buffers::create_node_and_cluster_cull_launch_worklist_buffer(device, &launch_worklist);
    let instance_work_item_buffer = virtual_geometry_resources
        .create_node_and_cluster_cull_instance_work_item_buffer(
            device,
            encoder,
            launch_worklist_buffer.as_ref(),
            dispatch_setup,
            instance_work_item_count,
        );
    let cluster_work_item_buffer =
        buffers::create_node_and_cluster_cull_cluster_work_item_buffer(device, &cluster_work_items);
    let hierarchy_child_id_buffer = buffers::create_node_and_cluster_cull_hierarchy_child_id_buffer(
        device,
        &hierarchy_child_ids,
    );
    let child_work_item_buffer =
        buffers::create_node_and_cluster_cull_child_work_item_buffer(device, &child_work_items);
    let traversal_record_buffer =
        buffers::create_node_and_cluster_cull_traversal_record_buffer(device, &traversal_records);
    let page_request_buffer =
        buffers::create_node_and_cluster_cull_page_request_buffer(device, &page_request_ids);
    VirtualGeometryNodeAndClusterCullPassOutput::from_store_parts(
        VirtualGeometryNodeAndClusterCullPassStoreParts {
            source: RenderVirtualGeometryNodeAndClusterCullSource::RenderPathCullInput,
            record_count: 1,
            global_state: Some(global_state),
            buffer: Some(Arc::new(device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("zircon-vg-node-and-cluster-cull-buffer"),
                    contents: bytemuck::cast_slice(&packed_words),
                    usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
                },
            ))),
            dispatch_setup: Some(dispatch_setup),
            launch_worklist: Some(launch_worklist),
            dispatch_setup_buffer: buffers::create_node_and_cluster_cull_dispatch_setup_buffer(
                device,
                &dispatch_setup,
            ),
            launch_worklist_buffer,
            instance_seed_count,
            instance_seed_buffer: buffers::create_node_and_cluster_cull_instance_seed_buffer(
                device,
                &instance_seeds,
            ),
            instance_seeds,
            instance_work_item_count,
            instance_work_items,
            instance_work_item_buffer,
            cluster_work_item_count,
            cluster_work_items,
            cluster_work_item_buffer,
            hierarchy_child_ids,
            hierarchy_child_id_buffer,
            child_work_item_count,
            child_work_items,
            child_work_item_buffer,
            traversal_record_count,
            traversal_records,
            traversal_record_buffer,
            page_request_count,
            page_request_ids,
            page_request_buffer,
        },
    )
}

fn empty_node_and_cluster_cull_pass_output(
    source: RenderVirtualGeometryNodeAndClusterCullSource,
) -> VirtualGeometryNodeAndClusterCullPassOutput {
    VirtualGeometryNodeAndClusterCullPassOutput::from_store_parts(
        VirtualGeometryNodeAndClusterCullPassStoreParts {
            source,
            record_count: 0,
            global_state: None,
            buffer: None,
            dispatch_setup: None,
            launch_worklist: None,
            dispatch_setup_buffer: None,
            launch_worklist_buffer: None,
            instance_seed_count: 0,
            instance_seeds: Vec::new(),
            instance_seed_buffer: None,
            instance_work_item_count: 0,
            instance_work_items: Vec::new(),
            instance_work_item_buffer: None,
            cluster_work_item_count: 0,
            cluster_work_items: Vec::new(),
            cluster_work_item_buffer: None,
            hierarchy_child_ids: Vec::new(),
            hierarchy_child_id_buffer: None,
            child_work_item_count: 0,
            child_work_items: Vec::new(),
            child_work_item_buffer: None,
            traversal_record_count: 0,
            traversal_records: Vec::new(),
            traversal_record_buffer: None,
            page_request_count: 0,
            page_request_ids: Vec::new(),
            page_request_buffer: None,
        },
    )
}
