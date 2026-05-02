use crate::virtual_geometry::types::VirtualGeometryNodeAndClusterCullClusterWorkItem;
use zircon_runtime::core::framework::render::{
    ProjectionMode, RenderVirtualGeometryCullInputSnapshot,
    RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
    RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    RenderVirtualGeometryNodeAndClusterCullInstanceSeed,
    RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem,
    RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot, ViewportCameraSnapshot,
};
use zircon_runtime::core::math::{view_matrix, Mat4, Real};
use zircon_runtime::graphics::ViewportRenderFrame;

const NODE_AND_CLUSTER_CULL_DISPATCH_WORKGROUP_SIZE: u32 = 64;
const NODE_AND_CLUSTER_CULL_BASELINE_CHILD_SPLIT_SCREEN_SPACE_ERROR_THRESHOLD: Real = 1.0;
const NODE_AND_CLUSTER_CULL_BASELINE_REFERENCE_VIEWPORT_HEIGHT: Real = 64.0;
const NODE_AND_CLUSTER_CULL_BASELINE_REFERENCE_ORTHO_SIZE: Real = 5.0;
const NODE_AND_CLUSTER_CULL_BASELINE_MIN_CHILD_SPLIT_SCREEN_SPACE_ERROR_THRESHOLD: Real = 0.03125;
const NODE_AND_CLUSTER_CULL_BASELINE_MAX_CHILD_SPLIT_SCREEN_SPACE_ERROR_THRESHOLD: Real = 32.0;
const NODE_AND_CLUSTER_CULL_BASELINE_CHILD_FRUSTUM_CULLING_ENABLED: bool = true;

pub(super) fn build_node_and_cluster_cull_global_state(
    frame: &ViewportRenderFrame,
    cull_input: &RenderVirtualGeometryCullInputSnapshot,
    previous_global_state: Option<&RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot>,
) -> RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot {
    let camera = &frame.scene.scene.camera;
    let aspect = frame.viewport_size.x as Real / frame.viewport_size.y.max(1) as Real;
    let projection = match camera.projection_mode {
        ProjectionMode::Perspective => Mat4::perspective_rh(
            camera.fov_y_radians,
            aspect.max(0.001),
            camera.z_near.max(0.001),
            camera.z_far,
        ),
        ProjectionMode::Orthographic => {
            let half_height = camera.ortho_size.max(0.01);
            let half_width = half_height * aspect.max(0.001);
            Mat4::orthographic_rh(
                -half_width,
                half_width,
                -half_height,
                half_height,
                camera.z_near.max(0.001),
                camera.z_far,
            )
        }
    };
    let view = view_matrix(camera.transform);
    let camera_translation = camera.transform.translation.to_array();
    let view_proj = (projection * view).to_cols_array_2d();
    let previous_camera_translation = previous_global_state
        .map(|state| state.camera_translation)
        .unwrap_or(camera_translation);
    let previous_view_proj = previous_global_state
        .map(|state| state.view_proj)
        .unwrap_or(view_proj);

    RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot {
        cull_input: *cull_input,
        viewport_size: [frame.viewport_size.x, frame.viewport_size.y],
        camera_translation,
        child_split_screen_space_error_threshold:
            node_and_cluster_cull_child_split_screen_space_error_threshold(
                camera,
                frame.viewport_size.y,
            ),
        child_frustum_culling_enabled: node_and_cluster_cull_child_frustum_culling_enabled(camera),
        view_proj,
        previous_camera_translation,
        previous_view_proj,
    }
}

fn node_and_cluster_cull_child_split_screen_space_error_threshold(
    camera: &ViewportCameraSnapshot,
    viewport_height: u32,
) -> Real {
    let viewport_scale =
        NODE_AND_CLUSTER_CULL_BASELINE_REFERENCE_VIEWPORT_HEIGHT / viewport_height.max(1) as Real;
    let projection_scale = match camera.projection_mode {
        ProjectionMode::Perspective => {
            let reference_projection_y = 1.0 / (std::f32::consts::FRAC_PI_3 * 0.5).tan();
            let projection_y = 1.0
                / (camera
                    .fov_y_radians
                    .clamp(0.001, std::f32::consts::PI - 0.001)
                    * 0.5)
                    .tan();
            reference_projection_y / projection_y.max(0.001)
        }
        ProjectionMode::Orthographic => {
            camera.ortho_size.max(0.01) / NODE_AND_CLUSTER_CULL_BASELINE_REFERENCE_ORTHO_SIZE
        }
    };

    (NODE_AND_CLUSTER_CULL_BASELINE_CHILD_SPLIT_SCREEN_SPACE_ERROR_THRESHOLD
        * viewport_scale
        * projection_scale)
        .clamp(
            NODE_AND_CLUSTER_CULL_BASELINE_MIN_CHILD_SPLIT_SCREEN_SPACE_ERROR_THRESHOLD,
            NODE_AND_CLUSTER_CULL_BASELINE_MAX_CHILD_SPLIT_SCREEN_SPACE_ERROR_THRESHOLD,
        )
}

fn node_and_cluster_cull_child_frustum_culling_enabled(camera: &ViewportCameraSnapshot) -> bool {
    if !NODE_AND_CLUSTER_CULL_BASELINE_CHILD_FRUSTUM_CULLING_ENABLED {
        return false;
    }
    if camera.z_far <= camera.z_near.max(0.001) {
        return false;
    }

    match camera.projection_mode {
        ProjectionMode::Perspective => {
            camera.fov_y_radians > 0.001 && camera.fov_y_radians < std::f32::consts::PI - 0.001
        }
        ProjectionMode::Orthographic => camera.ortho_size > 0.01,
    }
}

pub(super) fn build_node_and_cluster_cull_instance_seeds(
    frame: &ViewportRenderFrame,
    global_state: &RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
) -> Vec<RenderVirtualGeometryNodeAndClusterCullInstanceSeed> {
    let Some(extract) = frame.extract.geometry.virtual_geometry.as_ref() else {
        return Vec::new();
    };

    extract
        .instances
        .iter()
        .take(global_state.cull_input.instance_count as usize)
        .enumerate()
        .map(
            |(instance_index, instance)| RenderVirtualGeometryNodeAndClusterCullInstanceSeed {
                instance_index: u32::try_from(instance_index).unwrap_or(u32::MAX),
                entity: instance.entity,
                cluster_offset: instance.cluster_offset,
                cluster_count: instance.cluster_count,
                page_offset: instance.page_offset,
                page_count: instance.page_count,
            },
        )
        .collect()
}

pub(super) fn build_node_and_cluster_cull_dispatch_setup(
    global_state: &RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    instance_seed_count: u32,
) -> RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot {
    RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot {
        instance_seed_count,
        cluster_budget: global_state.cull_input.cluster_budget,
        page_budget: global_state.cull_input.page_budget,
        workgroup_size: NODE_AND_CLUSTER_CULL_DISPATCH_WORKGROUP_SIZE,
        dispatch_group_count: [
            instance_seed_count.div_ceil(NODE_AND_CLUSTER_CULL_DISPATCH_WORKGROUP_SIZE),
            1,
            1,
        ],
    }
}

pub(super) fn build_node_and_cluster_cull_launch_worklist(
    global_state: &RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    dispatch_setup: RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
    instance_seeds: &[RenderVirtualGeometryNodeAndClusterCullInstanceSeed],
) -> RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot {
    RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot {
        global_state: global_state.clone(),
        dispatch_setup,
        instance_seeds: instance_seeds.to_vec(),
    }
}

pub(super) fn build_node_and_cluster_cull_instance_work_items(
    launch_worklist: &RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot,
) -> Vec<RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem> {
    let cluster_budget = launch_worklist.dispatch_setup.cluster_budget;
    let page_budget = launch_worklist.dispatch_setup.page_budget;
    let forced_mip = launch_worklist.global_state.cull_input.debug.forced_mip;

    launch_worklist
        .instance_seeds
        .iter()
        .map(
            |seed| RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem {
                instance_index: seed.instance_index,
                entity: seed.entity,
                cluster_offset: seed.cluster_offset,
                cluster_count: seed.cluster_count,
                page_offset: seed.page_offset,
                page_count: seed.page_count,
                cluster_budget,
                page_budget,
                forced_mip,
            },
        )
        .collect()
}

pub(super) fn build_node_and_cluster_cull_cluster_work_items(
    frame: &ViewportRenderFrame,
    instance_work_items: &[RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem],
    cluster_work_item_limit: u32,
) -> Vec<VirtualGeometryNodeAndClusterCullClusterWorkItem> {
    let clusters = frame
        .extract
        .geometry
        .virtual_geometry
        .as_ref()
        .map(|extract| extract.clusters.as_slice())
        .unwrap_or(&[]);

    instance_work_items
        .iter()
        .flat_map(|work_item| {
            (0..work_item.cluster_count).map(move |cluster_local_index| {
                let cluster_array_index =
                    work_item.cluster_offset.saturating_add(cluster_local_index);
                VirtualGeometryNodeAndClusterCullClusterWorkItem {
                    instance_index: work_item.instance_index,
                    entity: work_item.entity,
                    cluster_array_index,
                    hierarchy_node_id: clusters
                        .get(cluster_array_index as usize)
                        .and_then(|cluster| cluster.hierarchy_node_id),
                    cluster_budget: work_item.cluster_budget,
                    page_budget: work_item.page_budget,
                    forced_mip: work_item.forced_mip,
                }
            })
        })
        .take(cluster_work_item_limit as usize)
        .collect()
}
