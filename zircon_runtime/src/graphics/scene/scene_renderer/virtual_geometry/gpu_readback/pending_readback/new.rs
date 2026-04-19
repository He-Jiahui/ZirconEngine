use super::VirtualGeometryGpuPendingReadback;

impl VirtualGeometryGpuPendingReadback {
    pub(in crate::graphics::scene::scene_renderer::virtual_geometry) fn new(
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
