use crate::graphics::types::{GraphicsError, VirtualGeometryPrepareFrame};

use super::super::super::super::gpu_readback::VirtualGeometryGpuPendingReadback;
use super::super::super::virtual_geometry_gpu_resources::VirtualGeometryGpuResources;
use super::collect_inputs::collect_inputs;
use super::copy_readbacks::copy_readbacks;
use super::create_bind_group::create_bind_group;
use super::create_buffers::create_buffers;
use super::dispatch::dispatch;
use super::queue_params::queue_params;

impl VirtualGeometryGpuResources {
    pub(crate) fn execute_prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        prepare: Option<&VirtualGeometryPrepareFrame>,
        page_budget: Option<u32>,
    ) -> Result<Option<VirtualGeometryGpuPendingReadback>, GraphicsError> {
        let Some(prepare) = prepare else {
            return Ok(None);
        };

        let inputs = collect_inputs(prepare);
        let buffers = create_buffers(device, &inputs);
        queue_params(self, queue, prepare, &inputs, page_budget);
        let bind_group = create_bind_group(self, device, &buffers);
        dispatch(self, encoder, &bind_group, &inputs);
        copy_readbacks(encoder, &buffers, &inputs);

        Ok(Some(VirtualGeometryGpuPendingReadback::new(
            inputs.resident_entries.len(),
            inputs.resident_slots,
            inputs.page_table_word_count,
            buffers.page_table_readback,
            inputs.completed_word_count.max(1),
            buffers.completed_readback,
        )))
    }
}
