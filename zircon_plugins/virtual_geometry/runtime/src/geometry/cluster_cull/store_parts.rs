use std::sync::Arc;

use crate::virtual_geometry::types::{
    VirtualGeometryNodeAndClusterCullChildWorkItem,
    VirtualGeometryNodeAndClusterCullClusterWorkItem,
    VirtualGeometryNodeAndClusterCullTraversalRecord,
};
use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
    RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    RenderVirtualGeometryNodeAndClusterCullInstanceSeed,
    RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem,
    RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot,
    RenderVirtualGeometryNodeAndClusterCullSource,
};

pub(in crate::virtual_geometry::renderer) struct VirtualGeometryNodeAndClusterCullPassStoreParts {
    pub(in crate::virtual_geometry::renderer) source: RenderVirtualGeometryNodeAndClusterCullSource,
    pub(in crate::virtual_geometry::renderer) record_count: u32,
    pub(in crate::virtual_geometry::renderer) global_state:
        Option<RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot>,
    pub(in crate::virtual_geometry::renderer) buffer: Option<Arc<wgpu::Buffer>>,
    pub(in crate::virtual_geometry::renderer) dispatch_setup:
        Option<RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot>,
    pub(in crate::virtual_geometry::renderer) launch_worklist:
        Option<RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot>,
    pub(in crate::virtual_geometry::renderer) dispatch_setup_buffer: Option<Arc<wgpu::Buffer>>,
    pub(in crate::virtual_geometry::renderer) launch_worklist_buffer: Option<Arc<wgpu::Buffer>>,
    pub(in crate::virtual_geometry::renderer) instance_seed_count: u32,
    pub(in crate::virtual_geometry::renderer) instance_seeds:
        Vec<RenderVirtualGeometryNodeAndClusterCullInstanceSeed>,
    pub(in crate::virtual_geometry::renderer) instance_seed_buffer: Option<Arc<wgpu::Buffer>>,
    pub(in crate::virtual_geometry::renderer) instance_work_item_count: u32,
    pub(in crate::virtual_geometry::renderer) instance_work_items:
        Vec<RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem>,
    pub(in crate::virtual_geometry::renderer) instance_work_item_buffer: Option<Arc<wgpu::Buffer>>,
    pub(in crate::virtual_geometry::renderer) cluster_work_item_count: u32,
    pub(in crate::virtual_geometry::renderer) cluster_work_items:
        Vec<VirtualGeometryNodeAndClusterCullClusterWorkItem>,
    pub(in crate::virtual_geometry::renderer) cluster_work_item_buffer: Option<Arc<wgpu::Buffer>>,
    pub(in crate::virtual_geometry::renderer) hierarchy_child_ids: Vec<u32>,
    pub(in crate::virtual_geometry::renderer) hierarchy_child_id_buffer: Option<Arc<wgpu::Buffer>>,
    pub(in crate::virtual_geometry::renderer) child_work_item_count: u32,
    pub(in crate::virtual_geometry::renderer) child_work_items:
        Vec<VirtualGeometryNodeAndClusterCullChildWorkItem>,
    pub(in crate::virtual_geometry::renderer) child_work_item_buffer: Option<Arc<wgpu::Buffer>>,
    pub(in crate::virtual_geometry::renderer) traversal_record_count: u32,
    pub(in crate::virtual_geometry::renderer) traversal_records:
        Vec<VirtualGeometryNodeAndClusterCullTraversalRecord>,
    pub(in crate::virtual_geometry::renderer) traversal_record_buffer: Option<Arc<wgpu::Buffer>>,
    pub(in crate::virtual_geometry::renderer) page_request_count: u32,
    pub(in crate::virtual_geometry::renderer) page_request_ids: Vec<u32>,
    pub(in crate::virtual_geometry::renderer) page_request_buffer: Option<Arc<wgpu::Buffer>>,
}
