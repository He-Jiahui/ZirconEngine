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

/// CPU preparation is device independent; materialization remains renderer owned.
#[derive(Clone, Debug)]
pub(in crate::virtual_geometry) struct VirtualGeometryNodeAndClusterCullPreparedParts {
    pub(in crate::virtual_geometry) source: RenderVirtualGeometryNodeAndClusterCullSource,
    pub(in crate::virtual_geometry) record_count: u32,
    pub(in crate::virtual_geometry) global_state:
        Option<RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot>,
    pub(in crate::virtual_geometry) dispatch_setup:
        Option<RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot>,
    pub(in crate::virtual_geometry) launch_worklist:
        Option<RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot>,
    pub(in crate::virtual_geometry) instance_seeds:
        Vec<RenderVirtualGeometryNodeAndClusterCullInstanceSeed>,
    pub(in crate::virtual_geometry) instance_work_items:
        Vec<RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem>,
    pub(in crate::virtual_geometry) cluster_work_items:
        Vec<VirtualGeometryNodeAndClusterCullClusterWorkItem>,
    pub(in crate::virtual_geometry) hierarchy_child_ids: Vec<u32>,
    pub(in crate::virtual_geometry) child_work_items:
        Vec<VirtualGeometryNodeAndClusterCullChildWorkItem>,
    pub(in crate::virtual_geometry) traversal_records:
        Vec<VirtualGeometryNodeAndClusterCullTraversalRecord>,
    pub(in crate::virtual_geometry) page_request_ids: Vec<u32>,
}
impl VirtualGeometryNodeAndClusterCullPreparedParts {
    pub(super) fn empty(source: RenderVirtualGeometryNodeAndClusterCullSource) -> Self {
        Self {
            source,
            record_count: 0,
            global_state: None,
            dispatch_setup: None,
            launch_worklist: None,
            instance_seeds: Vec::new(),
            instance_work_items: Vec::new(),
            cluster_work_items: Vec::new(),
            hierarchy_child_ids: Vec::new(),
            child_work_items: Vec::new(),
            traversal_records: Vec::new(),
            page_request_ids: Vec::new(),
        }
    }
}
