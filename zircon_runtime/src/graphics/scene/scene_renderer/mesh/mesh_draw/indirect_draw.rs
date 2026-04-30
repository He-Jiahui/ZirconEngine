use std::sync::Arc;

use super::MeshDraw;

impl MeshDraw {
    pub(crate) fn indirect_args_buffer(&self) -> Option<&Arc<wgpu::Buffer>> {
        self.indirect_args_buffer.as_ref()
    }

    pub(crate) fn indirect_args_offset(&self) -> u64 {
        self.indirect_args_offset
    }

    pub(crate) fn assign_execution_owned_indirect_args(
        &mut self,
        buffer: Arc<wgpu::Buffer>,
        indirect_args_offset: u64,
    ) {
        self.indirect_args_buffer = Some(buffer);
        self.indirect_args_offset = indirect_args_offset;
    }
}
