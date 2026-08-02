use zircon_runtime::graphics::{GraphicsError, RuntimeGpuReadback, RuntimePrepareCollectorContext};

use super::super::decode::{completed_page_assignments, page_table_entries};
use super::super::readback::VirtualGeometryGpuReadback;
use super::VirtualGeometryGpuPendingReadback;

pub(crate) struct VirtualGeometryGpuReadbackFuture {
    resident_entry_count: usize,
    resident_slots: Vec<u32>,
    page_table_word_count: usize,
    page_table: RuntimeGpuReadback,
    completed_word_count: usize,
    completed: RuntimeGpuReadback,
}

impl VirtualGeometryGpuPendingReadback {
    pub(in crate::virtual_geometry::renderer) fn enqueue(
        self,
        context: &mut RuntimePrepareCollectorContext<'_>,
    ) -> Result<VirtualGeometryGpuReadbackFuture, GraphicsError> {
        let page_table = context.request_gpu_readback(
            "virtual-geometry.page-table",
            &self.page_table_buffer,
            0..word_byte_len(self.page_table_word_count),
        )?;
        let completed = context.request_gpu_readback(
            "virtual-geometry.completed-pages",
            &self.completed_buffer,
            0..word_byte_len(self.completed_word_count),
        )?;
        Ok(VirtualGeometryGpuReadbackFuture {
            resident_entry_count: self.resident_entry_count,
            resident_slots: self.resident_slots,
            page_table_word_count: self.page_table_word_count,
            page_table,
            completed_word_count: self.completed_word_count,
            completed,
        })
    }
}

impl VirtualGeometryGpuReadbackFuture {
    pub(crate) fn is_ready(&self) -> bool {
        self.page_table.is_ready() && self.completed.is_ready()
    }

    pub(crate) fn try_collect(self) -> Option<Result<VirtualGeometryGpuReadback, GraphicsError>> {
        if !self.is_ready() {
            return None;
        }
        Some(self.collect_ready())
    }

    fn collect_ready(self) -> Result<VirtualGeometryGpuReadback, GraphicsError> {
        let completed_bytes = self
            .completed
            .try_take()
            .expect("ready virtual geometry completion readback remains available")?;
        let page_table_bytes = self
            .page_table
            .try_take()
            .expect("ready virtual geometry page-table readback remains available")?;
        let (completed_page_assignments, completed_page_ids, completed_page_replacements) =
            completed_page_assignments(&completed_bytes, self.completed_word_count)?;
        let page_table_entries = page_table_entries(
            &page_table_bytes,
            self.page_table_word_count,
            self.resident_entry_count,
            self.resident_slots,
            &completed_page_assignments,
        )?;

        Ok(VirtualGeometryGpuReadback::new(
            page_table_entries,
            completed_page_ids,
            completed_page_assignments,
            completed_page_replacements,
        ))
    }
}

fn word_byte_len(word_count: usize) -> u64 {
    word_count.max(1) as u64 * std::mem::size_of::<u32>() as u64
}
