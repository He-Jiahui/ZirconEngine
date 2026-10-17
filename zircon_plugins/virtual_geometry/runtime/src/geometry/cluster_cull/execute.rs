use std::collections::BTreeSet;

use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryCullInputSnapshot,
    RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    RenderVirtualGeometryNodeAndClusterCullSource,
};

use crate::virtual_geometry::renderer::{VirtualGeometryGpuResources, VirtualGeometryRenderFrame};

use super::output::VirtualGeometryNodeAndClusterCullPassOutput;
use super::page_requests::append_node_and_cluster_cull_page_requests;
use super::{child_decision, child_worklist, startup_worklist, traversal};

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
    let inspection = prepare_virtual_geometry_node_and_cluster_cull(
        frame.extract.geometry.virtual_geometry.as_ref(),
        &frame.scene.scene.camera,
        [frame.viewport_size.x, frame.viewport_size.y],
        pass_enabled,
        cull_input,
        previous_global_state,
    );
    let _ = virtual_geometry_resources;
    VirtualGeometryGpuResources::materialize_prepared_cull_on_device(device, encoder, &inspection)
        .into_pass_output()
}

/// Original CPU policy, shared by provider prepare and native materialization.
pub(crate) fn prepare_virtual_geometry_node_and_cluster_cull(
    extract: Option<&zircon_runtime::core::framework::render::RenderVirtualGeometryExtract>,
    camera: &zircon_runtime::core::framework::render::ViewportCameraSnapshot,
    viewport_size: [u32; 2],
    pass_enabled: bool,
    cull_input: Option<&RenderVirtualGeometryCullInputSnapshot>,
    previous_global_state: Option<&RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot>,
) -> zircon_runtime::core::framework::render::RenderVirtualGeometryPreparedCpuInspection {
    prepare_node_and_cluster_cull_parts(
        extract,
        camera,
        viewport_size,
        pass_enabled,
        cull_input,
        previous_global_state,
    )
    .into_neutral_inspection()
}

fn prepare_node_and_cluster_cull_parts(
    extract: Option<&zircon_runtime::core::framework::render::RenderVirtualGeometryExtract>,
    camera: &zircon_runtime::core::framework::render::ViewportCameraSnapshot,
    viewport_size: [u32; 2],
    pass_enabled: bool,
    cull_input: Option<&RenderVirtualGeometryCullInputSnapshot>,
    previous_global_state: Option<&RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot>,
) -> super::store_parts::VirtualGeometryNodeAndClusterCullPreparedParts {
    if !pass_enabled {
        return super::store_parts::VirtualGeometryNodeAndClusterCullPreparedParts::empty(
            RenderVirtualGeometryNodeAndClusterCullSource::Unavailable,
        );
    }

    let Some(cull_input) = cull_input else {
        return super::store_parts::VirtualGeometryNodeAndClusterCullPreparedParts::empty(
            RenderVirtualGeometryNodeAndClusterCullSource::RenderPathClearOnly,
        );
    };

    let global_state = startup_worklist::build_node_and_cluster_cull_global_state_from_camera(
        camera,
        viewport_size,
        cull_input,
        previous_global_state,
    );
    let instance_seeds = startup_worklist::build_node_and_cluster_cull_instance_seeds_from_extract(
        extract,
        &global_state,
    );
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
    let cluster_work_items =
        startup_worklist::build_node_and_cluster_cull_cluster_work_items_from_clusters(
            extract
                .map(|extract| extract.clusters.as_slice())
                .unwrap_or(&[]),
            &instance_work_items,
            global_state.cull_input.cluster_count,
        );
    let clusters = extract
        .map(|extract| extract.clusters.as_slice())
        .unwrap_or(&[]);
    let hierarchy_nodes = extract
        .map(|extract| extract.hierarchy_nodes.as_slice())
        .unwrap_or(&[]);
    let hierarchy_child_ids = extract
        .map(|extract| extract.hierarchy_child_ids.clone())
        .unwrap_or_default();
    let pages = extract
        .map(|extract| extract.pages.as_slice())
        .unwrap_or(&[]);
    let mut traversal_records = traversal::build_node_and_cluster_cull_traversal_records(
        &cluster_work_items,
        hierarchy_nodes,
    );
    let mut child_work_items = Vec::new();
    let mut page_request_ids = Vec::new();
    let mut seen_page_request_ids = BTreeSet::new();
    let mut current_wave_records = traversal_records.clone();
    // 当前帧的新请求额度先扣除已挂起页；同一去重集合贯穿所有子节点波次。
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
                camera,
                clusters,
                hierarchy_nodes,
                pages,
                traversal::next_node_and_cluster_cull_traversal_index(&child_visit_records),
            );
        append_node_and_cluster_cull_page_requests(
            &mut page_request_ids,
            &mut seen_page_request_ids,
            &child_decision_output.requested_page_ids,
            page_request_budget,
        );
        traversal_records.extend(child_visit_records);
        current_wave_records = child_decision_output.traversal_records.clone();
        traversal_records.extend(child_decision_output.traversal_records);
    }
    super::store_parts::VirtualGeometryNodeAndClusterCullPreparedParts {
        source: RenderVirtualGeometryNodeAndClusterCullSource::RenderPathCullInput,
        record_count: 1,
        global_state: Some(global_state),
        dispatch_setup: Some(dispatch_setup),
        launch_worklist: Some(launch_worklist),
        instance_seeds,
        instance_work_items,
        cluster_work_items,
        hierarchy_child_ids,
        child_work_items,
        traversal_records,
        page_request_ids,
    }
}
