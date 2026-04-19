pub(super) struct VirtualGeometryPrepareExecutionBuffers {
    pub(super) page_table_buffer: wgpu::Buffer,
    pub(super) page_table_readback: wgpu::Buffer,
    pub(super) request_buffer: wgpu::Buffer,
    pub(super) available_slot_buffer: wgpu::Buffer,
    pub(super) evictable_slot_buffer: wgpu::Buffer,
    pub(super) completed_buffer: wgpu::Buffer,
    pub(super) completed_readback: wgpu::Buffer,
}
