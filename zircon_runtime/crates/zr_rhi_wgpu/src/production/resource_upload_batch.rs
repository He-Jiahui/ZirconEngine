use super::{WgpuBufferUploadBatch, WgpuTextureUploadBatch};

/// One logical resource upload packet admitted under a single Copy ticket.
///
/// Keeping buffer and texture domains in one ownership-move packet lets a frame publish all setup
/// writes before its consuming command buffers without exposing the native queue to feature code.
#[derive(Default)]
pub struct WgpuResourceUploadBatch {
    buffer_uploads: WgpuBufferUploadBatch,
    texture_uploads: WgpuTextureUploadBatch,
}

impl WgpuResourceUploadBatch {
    pub const fn new() -> Self {
        Self {
            buffer_uploads: WgpuBufferUploadBatch::new(),
            texture_uploads: WgpuTextureUploadBatch::new(),
        }
    }

    pub const fn from_batches(
        buffer_uploads: WgpuBufferUploadBatch,
        texture_uploads: WgpuTextureUploadBatch,
    ) -> Self {
        Self {
            buffer_uploads,
            texture_uploads,
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.buffer_uploads.is_empty() && self.texture_uploads.is_empty()
    }

    pub(super) const fn buffer_upload_count(&self) -> usize {
        self.buffer_uploads.upload_count()
    }

    pub(super) const fn texture_upload_count(&self) -> usize {
        self.texture_uploads.upload_count()
    }

    pub(super) fn payload_byte_len(&self) -> u64 {
        self.buffer_uploads
            .payload_byte_len()
            .saturating_add(self.texture_uploads.payload_byte_len())
    }

    pub(super) fn into_batches(self) -> (WgpuBufferUploadBatch, WgpuTextureUploadBatch) {
        (self.buffer_uploads, self.texture_uploads)
    }
}
