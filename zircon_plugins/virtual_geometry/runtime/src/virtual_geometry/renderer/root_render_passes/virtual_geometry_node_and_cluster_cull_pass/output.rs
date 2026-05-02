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

use super::store_parts::VirtualGeometryNodeAndClusterCullPassStoreParts;

pub(in crate::virtual_geometry::renderer) struct VirtualGeometryNodeAndClusterCullPassOutput {
    source: RenderVirtualGeometryNodeAndClusterCullSource,
    record_count: u32,
    global_state: Option<RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot>,
    buffer: Option<Arc<wgpu::Buffer>>,
    dispatch_setup: Option<RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot>,
    launch_worklist: Option<RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot>,
    dispatch_setup_buffer: Option<Arc<wgpu::Buffer>>,
    launch_worklist_buffer: Option<Arc<wgpu::Buffer>>,
    instance_seed_count: u32,
    instance_seeds: Vec<RenderVirtualGeometryNodeAndClusterCullInstanceSeed>,
    instance_seed_buffer: Option<Arc<wgpu::Buffer>>,
    instance_work_item_count: u32,
    instance_work_items: Vec<RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem>,
    instance_work_item_buffer: Option<Arc<wgpu::Buffer>>,
    cluster_work_item_count: u32,
    cluster_work_items: Vec<VirtualGeometryNodeAndClusterCullClusterWorkItem>,
    cluster_work_item_buffer: Option<Arc<wgpu::Buffer>>,
    hierarchy_child_ids: Vec<u32>,
    hierarchy_child_id_buffer: Option<Arc<wgpu::Buffer>>,
    child_work_item_count: u32,
    child_work_items: Vec<VirtualGeometryNodeAndClusterCullChildWorkItem>,
    child_work_item_buffer: Option<Arc<wgpu::Buffer>>,
    traversal_record_count: u32,
    traversal_records: Vec<VirtualGeometryNodeAndClusterCullTraversalRecord>,
    traversal_record_buffer: Option<Arc<wgpu::Buffer>>,
    page_request_count: u32,
    page_request_ids: Vec<u32>,
    page_request_buffer: Option<Arc<wgpu::Buffer>>,
}

impl VirtualGeometryNodeAndClusterCullPassOutput {
    pub(super) fn from_store_parts(parts: VirtualGeometryNodeAndClusterCullPassStoreParts) -> Self {
        Self {
            source: parts.source,
            record_count: parts.record_count,
            global_state: parts.global_state,
            buffer: parts.buffer,
            dispatch_setup: parts.dispatch_setup,
            launch_worklist: parts.launch_worklist,
            dispatch_setup_buffer: parts.dispatch_setup_buffer,
            launch_worklist_buffer: parts.launch_worklist_buffer,
            instance_seed_count: parts.instance_seed_count,
            instance_seeds: parts.instance_seeds,
            instance_seed_buffer: parts.instance_seed_buffer,
            instance_work_item_count: parts.instance_work_item_count,
            instance_work_items: parts.instance_work_items,
            instance_work_item_buffer: parts.instance_work_item_buffer,
            cluster_work_item_count: parts.cluster_work_item_count,
            cluster_work_items: parts.cluster_work_items,
            cluster_work_item_buffer: parts.cluster_work_item_buffer,
            hierarchy_child_ids: parts.hierarchy_child_ids,
            hierarchy_child_id_buffer: parts.hierarchy_child_id_buffer,
            child_work_item_count: parts.child_work_item_count,
            child_work_items: parts.child_work_items,
            child_work_item_buffer: parts.child_work_item_buffer,
            traversal_record_count: parts.traversal_record_count,
            traversal_records: parts.traversal_records,
            traversal_record_buffer: parts.traversal_record_buffer,
            page_request_count: parts.page_request_count,
            page_request_ids: parts.page_request_ids,
            page_request_buffer: parts.page_request_buffer,
        }
    }

    pub(in crate::virtual_geometry::renderer::root_render_passes) fn source(
        &self,
    ) -> RenderVirtualGeometryNodeAndClusterCullSource {
        self.source
    }

    #[cfg(test)]
    pub(super) fn record_count(&self) -> u32 {
        self.record_count
    }

    #[cfg(test)]
    pub(super) fn global_state(
        &self,
    ) -> Option<&RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot> {
        self.global_state.as_ref()
    }

    #[cfg(test)]
    pub(super) fn buffer(&self) -> Option<&Arc<wgpu::Buffer>> {
        self.buffer.as_ref()
    }

    #[cfg(test)]
    pub(super) fn dispatch_setup(
        &self,
    ) -> Option<RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot> {
        self.dispatch_setup
    }

    #[cfg(test)]
    pub(super) fn launch_worklist(
        &self,
    ) -> Option<&RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot> {
        self.launch_worklist.as_ref()
    }

    #[cfg(test)]
    pub(super) fn dispatch_setup_buffer(&self) -> Option<&Arc<wgpu::Buffer>> {
        self.dispatch_setup_buffer.as_ref()
    }

    #[cfg(test)]
    pub(super) fn launch_worklist_buffer(&self) -> Option<&Arc<wgpu::Buffer>> {
        self.launch_worklist_buffer.as_ref()
    }

    #[cfg(test)]
    pub(super) fn instance_seed_count(&self) -> u32 {
        self.instance_seed_count
    }

    #[cfg(test)]
    pub(super) fn instance_seeds(&self) -> &[RenderVirtualGeometryNodeAndClusterCullInstanceSeed] {
        &self.instance_seeds
    }

    #[cfg(test)]
    pub(super) fn instance_seed_buffer(&self) -> Option<&Arc<wgpu::Buffer>> {
        self.instance_seed_buffer.as_ref()
    }

    #[cfg(test)]
    pub(super) fn instance_work_item_count(&self) -> u32 {
        self.instance_work_item_count
    }

    pub(in crate::virtual_geometry::renderer::root_render_passes) fn instance_work_items(
        &self,
    ) -> &[RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem] {
        &self.instance_work_items
    }

    #[cfg(test)]
    pub(super) fn instance_work_item_buffer(&self) -> Option<&Arc<wgpu::Buffer>> {
        self.instance_work_item_buffer.as_ref()
    }

    #[cfg(test)]
    pub(super) fn cluster_work_item_count(&self) -> u32 {
        self.cluster_work_item_count
    }

    pub(in crate::virtual_geometry::renderer::root_render_passes) fn cluster_work_items(
        &self,
    ) -> &[VirtualGeometryNodeAndClusterCullClusterWorkItem] {
        &self.cluster_work_items
    }

    #[cfg(test)]
    pub(super) fn hierarchy_child_ids(&self) -> &[u32] {
        &self.hierarchy_child_ids
    }

    #[cfg(test)]
    pub(super) fn hierarchy_child_id_buffer(&self) -> Option<&Arc<wgpu::Buffer>> {
        self.hierarchy_child_id_buffer.as_ref()
    }

    #[cfg(test)]
    pub(super) fn child_work_item_count(&self) -> u32 {
        self.child_work_item_count
    }

    #[cfg(test)]
    pub(super) fn child_work_items(&self) -> &[VirtualGeometryNodeAndClusterCullChildWorkItem] {
        &self.child_work_items
    }

    #[cfg(test)]
    pub(super) fn child_work_item_buffer(&self) -> Option<&Arc<wgpu::Buffer>> {
        self.child_work_item_buffer.as_ref()
    }

    #[cfg(test)]
    pub(super) fn traversal_record_count(&self) -> u32 {
        self.traversal_record_count
    }

    #[cfg(test)]
    pub(super) fn traversal_records(&self) -> &[VirtualGeometryNodeAndClusterCullTraversalRecord] {
        &self.traversal_records
    }

    #[cfg(test)]
    pub(super) fn page_request_count(&self) -> u32 {
        self.page_request_count
    }

    #[cfg(test)]
    pub(super) fn page_request_ids(&self) -> &[u32] {
        &self.page_request_ids
    }

    #[cfg(test)]
    pub(super) fn page_request_buffer(&self) -> Option<&Arc<wgpu::Buffer>> {
        self.page_request_buffer.as_ref()
    }

    pub(in crate::virtual_geometry::renderer) fn into_store_parts(
        self,
    ) -> VirtualGeometryNodeAndClusterCullPassStoreParts {
        VirtualGeometryNodeAndClusterCullPassStoreParts {
            source: self.source,
            record_count: self.record_count,
            global_state: self.global_state,
            buffer: self.buffer,
            dispatch_setup: self.dispatch_setup,
            launch_worklist: self.launch_worklist,
            dispatch_setup_buffer: self.dispatch_setup_buffer,
            launch_worklist_buffer: self.launch_worklist_buffer,
            instance_seed_count: self.instance_seed_count,
            instance_seeds: self.instance_seeds,
            instance_seed_buffer: self.instance_seed_buffer,
            instance_work_item_count: self.instance_work_item_count,
            instance_work_items: self.instance_work_items,
            instance_work_item_buffer: self.instance_work_item_buffer,
            cluster_work_item_count: self.cluster_work_item_count,
            cluster_work_items: self.cluster_work_items,
            cluster_work_item_buffer: self.cluster_work_item_buffer,
            hierarchy_child_ids: self.hierarchy_child_ids,
            hierarchy_child_id_buffer: self.hierarchy_child_id_buffer,
            child_work_item_count: self.child_work_item_count,
            child_work_items: self.child_work_items,
            child_work_item_buffer: self.child_work_item_buffer,
            traversal_record_count: self.traversal_record_count,
            traversal_records: self.traversal_records,
            traversal_record_buffer: self.traversal_record_buffer,
            page_request_count: self.page_request_count,
            page_request_ids: self.page_request_ids,
            page_request_buffer: self.page_request_buffer,
        }
    }

    #[cfg(test)]
    pub(in crate::virtual_geometry::renderer::root_render_passes) fn from_test_store_parts(
        parts: VirtualGeometryNodeAndClusterCullPassStoreParts,
    ) -> Self {
        Self::from_store_parts(parts)
    }

    #[cfg(test)]
    pub(in crate::virtual_geometry::renderer::root_render_passes) fn clear_instance_work_items_for_test(
        &mut self,
    ) {
        self.instance_work_item_count = 0;
        self.instance_work_items.clear();
        self.instance_work_item_buffer = None;
    }

    #[cfg(test)]
    pub(in crate::virtual_geometry::renderer::root_render_passes) fn clear_cluster_work_items_for_test(
        &mut self,
    ) {
        self.cluster_work_item_count = 0;
        self.cluster_work_items.clear();
        self.cluster_work_item_buffer = None;
    }
}
