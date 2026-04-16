use crate::{backend::read_buffer_u32s, types::GraphicsError};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct VirtualGeometryGpuReadback {
    pub(crate) page_table_entries: Vec<(u32, u32)>,
    pub(crate) completed_page_ids: Vec<u32>,
}

pub(crate) struct VirtualGeometryGpuPendingReadback {
    page_table_word_count: usize,
    page_table_buffer: wgpu::Buffer,
    completed_word_count: usize,
    completed_buffer: wgpu::Buffer,
}

impl VirtualGeometryGpuPendingReadback {
    pub(crate) fn new(
        page_table_word_count: usize,
        page_table_buffer: wgpu::Buffer,
        completed_word_count: usize,
        completed_buffer: wgpu::Buffer,
    ) -> Self {
        Self {
            page_table_word_count,
            page_table_buffer,
            completed_word_count,
            completed_buffer,
        }
    }

    pub(crate) fn collect(
        self,
        device: &wgpu::Device,
    ) -> Result<VirtualGeometryGpuReadback, GraphicsError> {
        let page_table_words =
            read_buffer_u32s(device, &self.page_table_buffer, self.page_table_word_count)?;
        let page_table_entries = page_table_words
            .chunks_exact(2)
            .map(|chunk| (chunk[0], chunk[1]))
            .collect();

        let completed_words =
            read_buffer_u32s(device, &self.completed_buffer, self.completed_word_count)?;
        let completed_count = completed_words.first().copied().unwrap_or_default() as usize;
        let completed_page_ids = completed_words
            .into_iter()
            .skip(1)
            .take(completed_count)
            .collect();

        Ok(VirtualGeometryGpuReadback {
            page_table_entries,
            completed_page_ids,
        })
    }
}
