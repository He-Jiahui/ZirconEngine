use super::prelude::*;

pub(super) fn viewport_frame(viewport_size: UVec2) -> ViewportRenderFrame {
    let mut extract = zircon_runtime::core::framework::render::RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    );
    extract.apply_viewport_size(viewport_size);
    extract.view.camera.transform = Transform::from_translation(Vec3::new(3.0, 4.0, 5.0));
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 4,
        page_budget: 2,
        clusters: Vec::new(),
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
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

pub(super) fn execute_pass(
    backend: &RenderBackend,
    pass_enabled: bool,
    frame: &ViewportRenderFrame,
    cull_input: Option<&RenderVirtualGeometryCullInputSnapshot>,
) -> VirtualGeometryNodeAndClusterCullPassOutput {
    execute_pass_with_previous(backend, pass_enabled, frame, cull_input, None)
}

pub(super) fn execute_pass_with_previous(
    backend: &RenderBackend,
    pass_enabled: bool,
    frame: &ViewportRenderFrame,
    cull_input: Option<&RenderVirtualGeometryCullInputSnapshot>,
    previous_global_state: Option<&RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot>,
) -> VirtualGeometryNodeAndClusterCullPassOutput {
    let mut encoder = backend
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-vg-node-and-cluster-cull-pass-test-encoder"),
        });
    let virtual_geometry_resources = VirtualGeometryGpuResources::new(&backend.device);

    execute_virtual_geometry_node_and_cluster_cull_pass(
        &backend.device,
        &mut encoder,
        &virtual_geometry_resources,
        pass_enabled,
        frame,
        cull_input,
        previous_global_state,
    )
}
