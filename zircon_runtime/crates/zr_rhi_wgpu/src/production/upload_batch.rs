use core::ops::Range;
use std::sync::Arc;

use zr_rhi::TextureCopyRegion;

/// One native subresource write backed by a shared immutable source payload.
///
/// Asset upload code may create many mip/layer writes from one cooked payload.
/// Keeping only a range per write avoids cloning that payload for every subresource while the
/// submission service retains it through queue acceptance.
#[derive(Clone)]
pub struct WgpuTextureUpload {
    texture: wgpu::Texture,
    region: TextureCopyRegion,
    bytes_per_row: u32,
    rows_per_image: u32,
    payload: Arc<[u8]>,
    source_range: Range<usize>,
}

impl WgpuTextureUpload {
    pub fn new(
        texture: wgpu::Texture,
        region: TextureCopyRegion,
        bytes_per_row: u32,
        rows_per_image: u32,
        payload: Arc<[u8]>,
        source_range: Range<usize>,
    ) -> Option<Self> {
        (source_range.start <= source_range.end && source_range.end <= payload.len()).then_some(
            Self {
                texture,
                region,
                bytes_per_row,
                rows_per_image,
                payload,
                source_range,
            },
        )
    }

    /// Moves one complete owned byte vector into an immutable texture-upload payload.
    pub fn from_owned_bytes(
        texture: wgpu::Texture,
        region: TextureCopyRegion,
        bytes_per_row: u32,
        rows_per_image: u32,
        bytes: Vec<u8>,
    ) -> Self {
        let payload: Arc<[u8]> = bytes.into();
        let payload_len = payload.len();
        Self {
            texture,
            region,
            bytes_per_row,
            rows_per_image,
            payload,
            source_range: 0..payload_len,
        }
    }

    pub(super) fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub(super) const fn region(&self) -> TextureCopyRegion {
        self.region
    }

    pub(super) const fn bytes_per_row(&self) -> u32 {
        self.bytes_per_row
    }

    pub(super) const fn rows_per_image(&self) -> u32 {
        self.rows_per_image
    }

    pub(super) fn payload(&self) -> &[u8] {
        &self.payload[self.source_range.clone()]
    }

    pub(super) fn payload_byte_len(&self) -> u64 {
        self.source_range.len() as u64
    }
}

/// One logical texture upload that receives one copy-queue ticket.
#[derive(Default)]
pub struct WgpuTextureUploadBatch {
    uploads: Vec<WgpuTextureUpload>,
}

impl WgpuTextureUploadBatch {
    pub const fn new() -> Self {
        Self {
            uploads: Vec::new(),
        }
    }

    pub fn push(&mut self, upload: WgpuTextureUpload) {
        self.uploads.push(upload);
    }

    pub fn append(&mut self, mut other: Self) {
        self.uploads.append(&mut other.uploads);
    }

    pub const fn is_empty(&self) -> bool {
        self.uploads.is_empty()
    }

    pub(super) const fn upload_count(&self) -> usize {
        self.uploads.len()
    }

    pub(super) fn payload_byte_len(&self) -> u64 {
        self.uploads
            .iter()
            .map(WgpuTextureUpload::payload_byte_len)
            .fold(0_u64, u64::saturating_add)
    }

    pub(super) fn into_uploads(self) -> Vec<WgpuTextureUpload> {
        self.uploads
    }
}

impl From<WgpuTextureUpload> for WgpuTextureUploadBatch {
    fn from(upload: WgpuTextureUpload) -> Self {
        Self {
            uploads: vec![upload],
        }
    }
}
