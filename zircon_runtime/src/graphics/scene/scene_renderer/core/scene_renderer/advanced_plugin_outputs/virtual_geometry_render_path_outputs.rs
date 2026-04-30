use std::sync::Arc;

use crate::core::framework::render::{
    RenderVirtualGeometryDebugSnapshot, RenderVirtualGeometryHardwareRasterizationSource,
    RenderVirtualGeometrySelectedClusterSource, RenderVirtualGeometryVisBuffer64Source,
};
use crate::graphics::scene::scene_renderer::core::scene_renderer::VirtualGeometryRenderPathOutputUpdate;

#[derive(Default)]
pub(super) struct VirtualGeometryRenderPathOutputs {
    debug_snapshot: Option<RenderVirtualGeometryDebugSnapshot>,
    selected_cluster_source: RenderVirtualGeometrySelectedClusterSource,
    selected_cluster_count: u32,
    selected_cluster_buffer: Option<Arc<wgpu::Buffer>>,
    visbuffer64_clear_value: u64,
    visbuffer64_source: RenderVirtualGeometryVisBuffer64Source,
    visbuffer64_entry_count: u32,
    visbuffer64_buffer: Option<Arc<wgpu::Buffer>>,
    hardware_rasterization_record_count: u32,
    hardware_rasterization_source: RenderVirtualGeometryHardwareRasterizationSource,
    hardware_rasterization_buffer: Option<Arc<wgpu::Buffer>>,
}

#[cfg_attr(not(test), allow(dead_code))]
impl VirtualGeometryRenderPathOutputs {
    pub(in crate::graphics::scene::scene_renderer::core) fn debug_snapshot(
        &self,
    ) -> Option<RenderVirtualGeometryDebugSnapshot> {
        self.debug_snapshot.clone()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn debug_snapshot_node_and_cluster_cull_global_state(
        &self,
    ) -> Option<
        crate::core::framework::render::RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    > {
        self.debug_snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.node_and_cluster_cull_global_state.clone())
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn selected_cluster_source(
        &self,
    ) -> RenderVirtualGeometrySelectedClusterSource {
        self.selected_cluster_source
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn selected_cluster_count(&self) -> u32 {
        self.selected_cluster_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn selected_cluster_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        &self.selected_cluster_buffer
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn visbuffer64_clear_value(&self) -> u64 {
        self.visbuffer64_clear_value
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn visbuffer64_source(
        &self,
    ) -> RenderVirtualGeometryVisBuffer64Source {
        self.visbuffer64_source
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn visbuffer64_entry_count(&self) -> u32 {
        self.visbuffer64_entry_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn visbuffer64_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        &self.visbuffer64_buffer
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn hardware_rasterization_source(
        &self,
    ) -> RenderVirtualGeometryHardwareRasterizationSource {
        self.hardware_rasterization_source
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn hardware_rasterization_record_count(
        &self,
    ) -> u32 {
        self.hardware_rasterization_record_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn hardware_rasterization_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        &self.hardware_rasterization_buffer
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn store(
        &mut self,
        update: VirtualGeometryRenderPathOutputUpdate,
    ) {
        self.selected_cluster_count = update.selected_cluster_count;
        self.selected_cluster_source = update.selected_cluster_source;
        self.selected_cluster_buffer = update.selected_cluster_buffer;
        self.visbuffer64_clear_value = update.visbuffer64_clear_value;
        self.visbuffer64_source = update.visbuffer64_source;
        self.visbuffer64_entry_count = update.visbuffer64_entry_count;
        self.visbuffer64_buffer = update.visbuffer64_buffer;
        self.hardware_rasterization_source = update.hardware_rasterization_source;
        self.hardware_rasterization_record_count = update.hardware_rasterization_record_count;
        self.hardware_rasterization_buffer = update.hardware_rasterization_buffer;
        self.debug_snapshot = update.debug_snapshot;
    }
}
