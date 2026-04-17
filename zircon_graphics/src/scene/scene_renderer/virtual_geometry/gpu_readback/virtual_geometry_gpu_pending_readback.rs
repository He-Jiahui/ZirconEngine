pub(crate) struct VirtualGeometryGpuPendingReadback {
    pub(super) resident_entry_count: usize,
    pub(super) resident_slots: Vec<u32>,
    pub(super) page_table_word_count: usize,
    pub(super) page_table_buffer: wgpu::Buffer,
    pub(super) completed_word_count: usize,
    pub(super) completed_buffer: wgpu::Buffer,
}

impl VirtualGeometryGpuPendingReadback {
    pub(crate) fn new(
        resident_entry_count: usize,
        resident_slots: Vec<u32>,
        page_table_word_count: usize,
        page_table_buffer: wgpu::Buffer,
        completed_word_count: usize,
        completed_buffer: wgpu::Buffer,
    ) -> Self {
        Self {
            resident_entry_count,
            resident_slots,
            page_table_word_count,
            page_table_buffer,
            completed_word_count,
            completed_buffer,
        }
    }
}
