use std::sync::Arc;

use crate::core::framework::render::{EXPOSURE_BUFFER_WORD_COUNT, PostProcessGraphResourceNames};
use crate::graphics::scene::scene_renderer::post_process::params::exposure_params::default_exposure_buffer_words;
use crate::rhi::{BufferDesc, BufferUsage};
use wgpu::util::DeviceExt;
use zr_rhi_wgpu::{WgpuBufferUpload, WgpuBufferUploadBatch};

pub(super) struct ExposureHistoryBuffers {
    read: wgpu::Buffer,
    write: wgpu::Buffer,
    reset_pending: bool,
}

impl ExposureHistoryBuffers {
    pub(super) fn new(device: &wgpu::Device) -> Self {
        Self {
            read: create_buffer(device, PostProcessGraphResourceNames::EXPOSURE_PREVIOUS),
            write: create_buffer(device, PostProcessGraphResourceNames::EXPOSURE_CURRENT),
            reset_pending: false,
        }
    }

    pub(super) const fn read(&self) -> &wgpu::Buffer {
        &self.read
    }

    pub(super) const fn write(&self) -> &wgpu::Buffer {
        &self.write
    }

    pub(super) fn request_reset(&mut self) {
        self.reset_pending = true;
    }

    pub(super) fn prepare_reset(&self, uploads: &mut WgpuBufferUploadBatch) -> bool {
        if !self.reset_pending {
            return false;
        }
        let payload: Arc<[u8]> = bytemuck::cast_slice(&default_exposure_buffer_words()).into();
        let source_range = 0..payload.len();
        let Some(read_upload) = WgpuBufferUpload::new(
            self.read.clone(),
            0,
            Arc::clone(&payload),
            source_range.clone(),
        ) else {
            return false;
        };
        let Some(write_upload) =
            WgpuBufferUpload::new(self.write.clone(), 0, payload, source_range)
        else {
            return false;
        };
        uploads.push(read_upload);
        uploads.push(write_upload);
        true
    }

    pub(super) fn commit_reset(&mut self) -> bool {
        std::mem::take(&mut self.reset_pending)
    }

    pub(super) fn flip_after_success(&mut self) {
        std::mem::swap(&mut self.read, &mut self.write);
    }

    pub(super) fn desc(&self, label: &'static str) -> BufferDesc {
        let size_bytes = u64::from(EXPOSURE_BUFFER_WORD_COUNT) * std::mem::size_of::<f32>() as u64;
        debug_assert_eq!(self.read.size(), size_bytes);
        debug_assert_eq!(self.write.size(), size_bytes);
        BufferDesc::new(
            label,
            size_bytes,
            BufferUsage::STORAGE | BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
        )
    }

    #[cfg(test)]
    pub(super) const fn reset_pending(&self) -> bool {
        self.reset_pending
    }
}

fn create_buffer(device: &wgpu::Device, label: &'static str) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(&default_exposure_buffer_words()),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST,
    })
}
