use std::sync::Arc;

use crate::core::framework::render::{
    ProjectionMode, RenderVirtualGeometryCullInputSnapshot,
    RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
    RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    RenderVirtualGeometryNodeAndClusterCullInstanceSeed,
    RenderVirtualGeometryNodeAndClusterCullSource,
};
use crate::core::math::{view_matrix, Mat4, Real};
use crate::graphics::types::ViewportRenderFrame;
use wgpu::util::DeviceExt;

pub(super) struct VirtualGeometryNodeAndClusterCullPassOutput {
    pub(super) source: RenderVirtualGeometryNodeAndClusterCullSource,
    pub(super) record_count: u32,
    pub(super) global_state: Option<RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot>,
    pub(super) buffer: Option<Arc<wgpu::Buffer>>,
    pub(super) dispatch_setup: Option<RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot>,
    pub(super) dispatch_setup_buffer: Option<Arc<wgpu::Buffer>>,
    pub(super) instance_seed_count: u32,
    pub(super) instance_seeds: Vec<RenderVirtualGeometryNodeAndClusterCullInstanceSeed>,
    pub(super) instance_seed_buffer: Option<Arc<wgpu::Buffer>>,
}

const NODE_AND_CLUSTER_CULL_DISPATCH_WORKGROUP_SIZE: u32 = 64;

pub(super) fn execute_virtual_geometry_node_and_cluster_cull_pass(
    device: &wgpu::Device,
    pass_enabled: bool,
    frame: &ViewportRenderFrame,
    cull_input: Option<&RenderVirtualGeometryCullInputSnapshot>,
) -> VirtualGeometryNodeAndClusterCullPassOutput {
    if !pass_enabled {
        return VirtualGeometryNodeAndClusterCullPassOutput {
            source: RenderVirtualGeometryNodeAndClusterCullSource::Unavailable,
            record_count: 0,
            global_state: None,
            buffer: None,
            dispatch_setup: None,
            dispatch_setup_buffer: None,
            instance_seed_count: 0,
            instance_seeds: Vec::new(),
            instance_seed_buffer: None,
        };
    }

    let Some(cull_input) = cull_input else {
        return VirtualGeometryNodeAndClusterCullPassOutput {
            source: RenderVirtualGeometryNodeAndClusterCullSource::RenderPathClearOnly,
            record_count: 0,
            global_state: None,
            buffer: None,
            dispatch_setup: None,
            dispatch_setup_buffer: None,
            instance_seed_count: 0,
            instance_seeds: Vec::new(),
            instance_seed_buffer: None,
        };
    };

    let global_state = build_node_and_cluster_cull_global_state(frame, cull_input);
    let packed_words = global_state.packed_words();
    let instance_seeds = build_node_and_cluster_cull_instance_seeds(frame, &global_state);
    let instance_seed_count = u32::try_from(instance_seeds.len()).unwrap_or(u32::MAX);
    let dispatch_setup =
        build_node_and_cluster_cull_dispatch_setup(&global_state, instance_seed_count);
    VirtualGeometryNodeAndClusterCullPassOutput {
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
        dispatch_setup_buffer: create_node_and_cluster_cull_dispatch_setup_buffer(
            device,
            &dispatch_setup,
        ),
        instance_seed_count,
        instance_seed_buffer: create_node_and_cluster_cull_instance_seed_buffer(
            device,
            &instance_seeds,
        ),
        instance_seeds,
    }
}

fn build_node_and_cluster_cull_global_state(
    frame: &ViewportRenderFrame,
    cull_input: &RenderVirtualGeometryCullInputSnapshot,
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

    RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot {
        cull_input: *cull_input,
        viewport_size: [frame.viewport_size.x, frame.viewport_size.y],
        camera_translation: camera.transform.translation.to_array(),
        view_proj: (projection * view).to_cols_array_2d(),
    }
}

fn build_node_and_cluster_cull_instance_seeds(
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

fn build_node_and_cluster_cull_dispatch_setup(
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

fn create_node_and_cluster_cull_dispatch_setup_buffer(
    device: &wgpu::Device,
    dispatch_setup: &RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
) -> Option<Arc<wgpu::Buffer>> {
    let packed_words = dispatch_setup.packed_words();
    Some(Arc::new(device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("zircon-vg-node-and-cluster-cull-dispatch-setup"),
            contents: bytemuck::cast_slice(&packed_words),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
        },
    )))
}

fn create_node_and_cluster_cull_instance_seed_buffer(
    device: &wgpu::Device,
    instance_seeds: &[RenderVirtualGeometryNodeAndClusterCullInstanceSeed],
) -> Option<Arc<wgpu::Buffer>> {
    if instance_seeds.is_empty() {
        return None;
    }

    let packed_words = instance_seeds
        .iter()
        .flat_map(RenderVirtualGeometryNodeAndClusterCullInstanceSeed::packed_words)
        .collect::<Vec<_>>();
    Some(Arc::new(device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("zircon-vg-node-and-cluster-cull-instance-seeds"),
            contents: bytemuck::cast_slice(&packed_words),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
        },
    )))
}

#[cfg(test)]
mod tests {
    use super::{
        build_node_and_cluster_cull_global_state,
        execute_virtual_geometry_node_and_cluster_cull_pass,
    };
    use crate::core::framework::render::{
        RenderVirtualGeometryClusterSelectionInputSource, RenderVirtualGeometryCullInputSnapshot,
        RenderVirtualGeometryDebugState, RenderVirtualGeometryExtract,
        RenderVirtualGeometryInstance,
        RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
        RenderVirtualGeometryNodeAndClusterCullInstanceSeed,
        RenderVirtualGeometryNodeAndClusterCullSource, RenderWorldSnapshotHandle,
    };
    use crate::core::math::{Transform, UVec2, Vec3};
    use crate::graphics::backend::RenderBackend;
    use crate::graphics::types::ViewportRenderFrame;
    use crate::scene::world::World;

    #[test]
    fn node_and_cluster_cull_pass_publishes_cull_input_record_when_present() {
        let backend = RenderBackend::new_offscreen().expect("backend should initialize");
        let frame = viewport_frame(UVec2::new(96, 64));
        let cull_input = RenderVirtualGeometryCullInputSnapshot {
            cluster_budget: 4,
            page_budget: 2,
            instance_count: 3,
            cluster_count: 9,
            page_count: 5,
            visible_entity_count: 2,
            visible_cluster_count: 7,
            resident_page_count: 1,
            pending_page_request_count: 2,
            available_page_slot_count: 1,
            evictable_page_count: 0,
            debug: RenderVirtualGeometryDebugState {
                forced_mip: Some(6),
                freeze_cull: true,
                visualize_bvh: false,
                visualize_visbuffer: true,
                print_leaf_clusters: false,
            },
            cluster_selection_input_source:
                RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
        };
        let output = execute_virtual_geometry_node_and_cluster_cull_pass(
            &backend.device,
            true,
            &frame,
            Some(&cull_input),
        );
        assert_eq!(
            output.source,
            RenderVirtualGeometryNodeAndClusterCullSource::RenderPathCullInput
        );
        assert_eq!(output.record_count, 1);
        assert!(
            output.buffer.is_some(),
            "expected the first NodeAndClusterCull bridge to materialize a real GPU buffer from the stable cull-input layout"
        );
        assert_eq!(
            output.instance_seeds,
            vec![RenderVirtualGeometryNodeAndClusterCullInstanceSeed {
                instance_index: 0,
                entity: 7,
                cluster_offset: 10,
                cluster_count: 2,
                page_offset: 20,
                page_count: 1,
            }],
            "expected the next NodeAndClusterCull bridge step to publish one per-instance root seed row from the effective VG extract instead of leaving the typed global-state record without an instance worklist seam"
        );
        assert_eq!(output.instance_seed_count, 1);
        assert!(output.instance_seed_buffer.is_some());
    }

    #[test]
    fn node_and_cluster_cull_pass_reports_clear_only_without_cull_input() {
        let backend = RenderBackend::new_offscreen().expect("backend should initialize");
        let output = execute_virtual_geometry_node_and_cluster_cull_pass(
            &backend.device,
            true,
            &viewport_frame(UVec2::new(96, 64)),
            None,
        );

        assert_eq!(
            output.source,
            RenderVirtualGeometryNodeAndClusterCullSource::RenderPathClearOnly
        );
        assert_eq!(output.record_count, 0);
        assert!(output.buffer.is_none());
    }

    #[test]
    fn node_and_cluster_cull_pass_derives_dispatch_setup_from_global_state_and_seed_count() {
        let backend = RenderBackend::new_offscreen().expect("backend should initialize");
        let output = execute_virtual_geometry_node_and_cluster_cull_pass(
            &backend.device,
            true,
            &viewport_frame(UVec2::new(96, 64)),
            Some(&RenderVirtualGeometryCullInputSnapshot {
                cluster_budget: 4,
                page_budget: 2,
                instance_count: 1,
                cluster_count: 9,
                page_count: 5,
                visible_entity_count: 1,
                visible_cluster_count: 7,
                resident_page_count: 1,
                pending_page_request_count: 2,
                available_page_slot_count: 1,
                evictable_page_count: 0,
                debug: RenderVirtualGeometryDebugState {
                    forced_mip: Some(6),
                    freeze_cull: true,
                    visualize_bvh: false,
                    visualize_visbuffer: true,
                    print_leaf_clusters: false,
                },
                cluster_selection_input_source:
                    RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
            }),
        );

        assert_eq!(
            output.dispatch_setup,
            Some(RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot {
                instance_seed_count: 1,
                cluster_budget: 4,
                page_budget: 2,
                workgroup_size: 64,
                dispatch_group_count: [1, 1, 1],
            }),
            "expected the first explicit NodeAndClusterCull dispatch/setup seam to consume the typed global-state budget inputs together with the derived root-seed count instead of leaving dispatch dimensions as implicit host logic"
        );
        assert!(
            output.dispatch_setup_buffer.is_some(),
            "expected the dispatch/setup seam to materialize a dedicated GPU buffer beside the global-state and seed buffers so a later compute NodeAndClusterCull pass can bind explicit startup parameters"
        );
    }

    #[test]
    fn node_and_cluster_cull_global_state_uses_viewport_and_camera_from_frame() {
        let frame = viewport_frame(UVec2::new(320, 180));
        let snapshot = build_node_and_cluster_cull_global_state(
            &frame,
            &RenderVirtualGeometryCullInputSnapshot {
                cluster_budget: 4,
                page_budget: 2,
                instance_count: 3,
                cluster_count: 9,
                page_count: 5,
                visible_entity_count: 2,
                visible_cluster_count: 7,
                resident_page_count: 1,
                pending_page_request_count: 2,
                available_page_slot_count: 1,
                evictable_page_count: 0,
                debug: RenderVirtualGeometryDebugState {
                    forced_mip: Some(6),
                    freeze_cull: true,
                    visualize_bvh: false,
                    visualize_visbuffer: true,
                    print_leaf_clusters: false,
                },
                cluster_selection_input_source:
                    RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
            },
        );

        assert_eq!(snapshot.viewport_size, [320, 180]);
        assert_eq!(snapshot.camera_translation, [3.0, 4.0, 5.0]);
        assert!(
            snapshot.view_proj.iter().flatten().any(|value| *value != 0.0),
            "expected the upgraded NodeAndClusterCull startup record to carry a non-empty view-projection matrix derived from the frame camera"
        );
    }

    fn viewport_frame(viewport_size: UVec2) -> ViewportRenderFrame {
        let mut extract = crate::core::framework::render::RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(1),
            World::new().to_render_snapshot(),
        );
        extract.apply_viewport_size(viewport_size);
        extract.view.camera.transform = Transform::from_translation(Vec3::new(3.0, 4.0, 5.0));
        extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
            cluster_budget: 4,
            page_budget: 2,
            clusters: Vec::new(),
            pages: Vec::new(),
            instances: vec![RenderVirtualGeometryInstance {
                entity: 7,
                source_model: None,
                transform: Transform::default(),
                cluster_offset: 10,
                cluster_count: 2,
                page_offset: 20,
                page_count: 1,
                mesh_name: Some("NodeAndClusterCullUnitTestMesh".to_string()),
                source_hint: Some("unit-test".to_string()),
            }],
            debug: RenderVirtualGeometryDebugState::default(),
        });
        ViewportRenderFrame::from_extract(extract, viewport_size)
    }
}
