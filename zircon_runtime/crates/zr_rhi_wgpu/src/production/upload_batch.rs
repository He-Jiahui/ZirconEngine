use core::ops::{Deref, Range};
use std::sync::Arc;

use zr_rhi::TextureCopyRegion;

/// Shared bytes or a retained asset exposing its existing upload bytes.
#[derive(Clone)]
pub enum WgpuTextureUploadPayload {
    Bytes(Arc<[u8]>),
    Shared(Arc<dyn AsRef<[u8]> + Send + Sync>),
}

impl WgpuTextureUploadPayload {
    pub fn shared<T: AsRef<[u8]> + Send + Sync + 'static>(source: Arc<T>) -> Self {
        Self::Shared(source)
    }
}
impl From<Arc<[u8]>> for WgpuTextureUploadPayload {
    fn from(bytes: Arc<[u8]>) -> Self {
        Self::Bytes(bytes)
    }
}
impl Deref for WgpuTextureUploadPayload {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        match self {
            Self::Bytes(bytes) => bytes,
            Self::Shared(source) => source.as_ref().as_ref(),
        }
    }
}

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
    payload: WgpuTextureUploadPayload,
    source_range: Range<usize>,
}

impl WgpuTextureUpload {
    /// 构造时只保证共享负载范围可切片；调用者须按所走的上传路径预先校验纹理区域和行布局。
    pub fn new(
        texture: wgpu::Texture,
        region: TextureCopyRegion,
        bytes_per_row: u32,
        rows_per_image: u32,
        payload: impl Into<WgpuTextureUploadPayload>,
        source_range: Range<usize>,
    ) -> Option<Self> {
        let payload = payload.into();
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
            payload: payload.into(),
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

    pub fn payload_byte_len(&self) -> u64 {
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

#[cfg(test)]
mod payload_tests {
    use super::*;
    #[test]
    fn shared_upload_payload_retains_source_without_copying_bytes() {
        let bytes = Arc::new(vec![1u8, 2, 3, 4]);
        let pointer = bytes.as_ptr();
        let payload = WgpuTextureUploadPayload::shared(Arc::clone(&bytes));
        let copy = payload.clone();
        drop(bytes);
        assert_eq!(copy.as_ptr(), pointer);
        assert_eq!(&copy[..], &[1, 2, 3, 4]);
    }
}
