use std::sync::Arc;

use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
    RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem,
};

use super::VirtualGeometryGpuResources;

impl VirtualGeometryGpuResources {
    pub(in crate::virtual_geometry::renderer) fn create_node_and_cluster_cull_instance_work_item_buffer(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        launch_worklist_buffer: Option<&Arc<wgpu::Buffer>>,
        dispatch_setup: RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
        instance_work_item_count: u32,
    ) -> Option<Arc<wgpu::Buffer>> {
        let launch_worklist_buffer = launch_worklist_buffer?;
        if instance_work_item_count == 0 {
            return None;
        }

        let word_count = usize::try_from(instance_work_item_count)
            .ok()?
            .checked_mul(RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem::GPU_WORD_COUNT)?;
        let buffer = Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-vg-node-and-cluster-cull-instance-work-items"),
            size: (word_count * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        }));
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-vg-node-and-cluster-cull-instance-work-items-bind-group"),
            layout: &self.node_and_cluster_cull_instance_work_item_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: launch_worklist_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffer.as_entire_binding(),
                },
            ],
        });
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("zircon-vg-node-and-cluster-cull-instance-work-items-pass"),
            timestamp_writes: None,
        });
        compute_pass.set_pipeline(&self.node_and_cluster_cull_instance_work_item_pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        compute_pass.dispatch_workgroups(dispatch_setup.dispatch_group_count[0], 1, 1);
        drop(compute_pass);

        Some(buffer)
    }
}
