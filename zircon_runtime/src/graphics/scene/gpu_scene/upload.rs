use std::ops::Range;
use std::sync::Arc;

use bytemuck::Pod;
use zr_rhi_wgpu::{WgpuBufferUpload, WgpuBufferUploadBatch};

use super::update_queue::GpuSceneUploadRange;

struct GpuScenePendingBufferWrite {
    buffer: wgpu::Buffer,
    offset: u64,
    source_range: Range<usize>,
}

/// Packs every small GPU-scene write for one frame into immutable shared
/// storage before ownership moves to the backend submission coordinator.
pub(super) struct GpuSceneBufferUploadBatchBuilder {
    payload: Vec<u8>,
    writes: Vec<GpuScenePendingBufferWrite>,
}

impl GpuSceneBufferUploadBatchBuilder {
    pub(super) const fn new() -> Self {
        Self {
            payload: Vec::new(),
            writes: Vec::new(),
        }
    }

    pub(super) fn push_pod_slice<T: Pod>(
        &mut self,
        buffer: &wgpu::Buffer,
        offset: u64,
        values: &[T],
    ) -> u64 {
        self.push_bytes(buffer, offset, bytemuck::cast_slice(values))
    }

    pub(super) fn push_upload_ranges<T: Pod>(
        &mut self,
        buffer: &wgpu::Buffer,
        shadow: &[T],
        ranges: &[GpuSceneUploadRange],
    ) -> u64 {
        ranges.iter().fold(0_u64, |uploaded_bytes, range| {
            let start = range.start as usize;
            let end = start
                .checked_add(range.len as usize)
                .expect("gpu scene upload range overflowed usize");
            uploaded_bytes.saturating_add(self.push_pod_slice(
                buffer,
                range.byte_offset,
                &shadow[start..end],
            ))
        })
    }

    pub(super) fn push_changed_pod_slice<T: Pod + PartialEq>(
        &mut self,
        buffer: &wgpu::Buffer,
        previous: &[T],
        current: &[T],
    ) -> u64 {
        let mut uploaded_bytes = 0_u64;
        let mut index = 0;
        while index < current.len() {
            if previous.get(index) == Some(&current[index]) {
                index += 1;
                continue;
            }
            let start = index;
            index += 1;
            while index < current.len() && previous.get(index) != Some(&current[index]) {
                index += 1;
            }
            let byte_offset = start
                .checked_mul(std::mem::size_of::<T>())
                .and_then(|offset| u64::try_from(offset).ok())
                .expect("gpu scene dirty upload offset overflowed u64");
            uploaded_bytes = uploaded_bytes.saturating_add(self.push_pod_slice(
                buffer,
                byte_offset,
                &current[start..index],
            ));
        }
        uploaded_bytes
    }

    pub(super) fn push_bytes(&mut self, buffer: &wgpu::Buffer, offset: u64, bytes: &[u8]) -> u64 {
        if bytes.is_empty() {
            return 0;
        }

        let start = self.payload.len();
        let end = start
            .checked_add(bytes.len())
            .expect("gpu scene upload payload exceeded usize");
        self.payload.extend_from_slice(bytes);
        self.writes.push(GpuScenePendingBufferWrite {
            buffer: buffer.clone(),
            offset,
            source_range: start..end,
        });
        bytes.len() as u64
    }

    pub(super) fn into_batch(self) -> WgpuBufferUploadBatch {
        let payload: Arc<[u8]> = Arc::from(self.payload);
        let mut batch = WgpuBufferUploadBatch::new();
        for write in self.writes {
            let upload = WgpuBufferUpload::new(
                write.buffer,
                write.offset,
                Arc::clone(&payload),
                write.source_range,
            )
            .expect("gpu scene batch builder must create in-bounds payload ranges");
            batch.push(upload);
        }
        batch
    }
}
