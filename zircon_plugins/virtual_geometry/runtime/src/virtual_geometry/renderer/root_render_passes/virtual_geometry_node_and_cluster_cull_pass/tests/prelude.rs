pub(super) use super::super::{
    build_node_and_cluster_cull_global_state, execute_virtual_geometry_node_and_cluster_cull_pass,
    VirtualGeometryNodeAndClusterCullPassOutput,
};
pub(super) use crate::virtual_geometry::renderer::VirtualGeometryGpuResources;
pub(super) use crate::virtual_geometry::types::{
    VirtualGeometryNodeAndClusterCullChildWorkItem,
    VirtualGeometryNodeAndClusterCullClusterWorkItem,
    VirtualGeometryNodeAndClusterCullTraversalChildSource,
    VirtualGeometryNodeAndClusterCullTraversalOp, VirtualGeometryNodeAndClusterCullTraversalRecord,
};
pub(super) use zircon_runtime::core::framework::render::{
    ProjectionMode, RenderVirtualGeometryCluster, RenderVirtualGeometryClusterSelectionInputSource,
    RenderVirtualGeometryCullInputSnapshot, RenderVirtualGeometryDebugState,
    RenderVirtualGeometryExtract, RenderVirtualGeometryHierarchyNode,
    RenderVirtualGeometryInstance, RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
    RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    RenderVirtualGeometryNodeAndClusterCullInstanceSeed,
    RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem,
    RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot,
    RenderVirtualGeometryNodeAndClusterCullSource, RenderVirtualGeometryPage,
    RenderWorldSnapshotHandle,
};
pub(super) use zircon_runtime::core::math::{Transform, UVec2, Vec3};
pub(super) use zircon_runtime::graphics::GraphicsError;
pub(super) use zircon_runtime::scene::World;

pub(super) use crate::virtual_geometry::renderer::VirtualGeometryRenderFrame;

pub(super) struct RenderBackend {
    _instance: wgpu::Instance,
    _adapter: wgpu::Adapter,
    pub(super) device: wgpu::Device,
    _queue: wgpu::Queue,
}

impl RenderBackend {
    pub(super) fn new_offscreen() -> Result<Self, GraphicsError> {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .map_err(|_| GraphicsError::NoAdapter)?;
        let requested_features = adapter.features() & wgpu::Features::INDIRECT_FIRST_INSTANCE;
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("zircon-vg-node-and-cluster-cull-test-device"),
            required_features: requested_features,
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
        }))?;

        Ok(Self {
            _instance: instance,
            _adapter: adapter,
            device,
            _queue: queue,
        })
    }
}
