use std::sync::Arc;

use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
    RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem,
};

use super::VirtualGeometryGpuResources;
use zircon_runtime::graphics::{RenderPassGpuRecordingContext, RenderPassGpuResourceFactory};

pub(crate) struct DeviceCullRecording<'a> {
    pub(crate) device: &'a wgpu::Device,
    pub(crate) encoder: &'a mut wgpu::CommandEncoder,
}
impl RenderPassGpuRecordingContext for DeviceCullRecording<'_> {
    type ResourceFactory = wgpu::Device;
    fn resource_factory(&self) -> &wgpu::Device {
        self.device
    }
    fn command_encoder(&mut self) -> &mut wgpu::CommandEncoder {
        self.encoder
    }
}

impl VirtualGeometryGpuResources {
    // 从启动工作表生成 GPU 实例工作项；输出字数与 Rust 工作项布局、着色器步长必须一致。
    pub(in crate::virtual_geometry::renderer) fn create_node_and_cluster_cull_instance_work_item_buffer(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        launch_worklist_buffer: Option<&Arc<wgpu::Buffer>>,
        dispatch_setup: RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
        instance_work_item_count: u32,
    ) -> Option<Arc<wgpu::Buffer>> {
        let mut recording = DeviceCullRecording { device, encoder };
        Self::record_prepared_cull_instance_work_items(
            &mut recording,
            &self.node_and_cluster_cull_instance_work_item_bind_group_layout,
            &self.node_and_cluster_cull_instance_work_item_pipeline,
            launch_worklist_buffer,
            dispatch_setup,
            instance_work_item_count,
        )
    }
}

impl VirtualGeometryGpuResources {
    pub(crate) fn record_prepared_cull_instance_work_items(
        recording: &mut impl RenderPassGpuRecordingContext,
        layout: &wgpu::BindGroupLayout,
        pipeline: &wgpu::ComputePipeline,
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
        let buffer = Arc::new(recording.resource_factory().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("zircon-vg-node-and-cluster-cull-instance-work-items"),
                contents: &vec![0_u8; word_count.checked_mul(std::mem::size_of::<u32>())?],
                usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
            },
        ));
        let bind_group =
            recording
                .resource_factory()
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("zircon-vg-node-and-cluster-cull-instance-work-items-bind-group"),
                    layout: layout,
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
        let mut compute_pass =
            recording
                .command_encoder()
                .begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("zircon-vg-node-and-cluster-cull-instance-work-items-pass"),
                    timestamp_writes: None,
                });
        compute_pass.set_pipeline(pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        compute_pass.dispatch_workgroups(dispatch_setup.dispatch_group_count[0], 1, 1);
        drop(compute_pass);

        Some(buffer)
    }
}

impl VirtualGeometryGpuResources {
    pub(crate) fn materialize_prepared_cull_on_device(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        inspection: &zircon_runtime::core::framework::render::RenderVirtualGeometryPreparedCpuInspection,
    ) -> crate::virtual_geometry::renderer::MaterializedPreparedCull {
        let mut recording = DeviceCullRecording { device, encoder };
        crate::virtual_geometry::renderer::materialize_prepared_cull(&mut recording, inspection)
    }
}
