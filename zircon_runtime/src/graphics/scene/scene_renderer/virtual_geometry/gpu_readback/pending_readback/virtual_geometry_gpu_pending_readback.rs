pub(crate) struct VirtualGeometryGpuPendingReadback {
    pub(super) resident_entry_count: usize,
    pub(super) resident_slots: Vec<u32>,
    pub(super) page_table_word_count: usize,
    pub(super) page_table_buffer: wgpu::Buffer,
    pub(super) completed_word_count: usize,
    pub(super) completed_buffer: wgpu::Buffer,
}
