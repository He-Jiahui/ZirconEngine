use core::ops::Range;
use std::sync::Arc;

use crate::{BufferHandle, TextureCopyRegion, TextureHandle};

#[derive(Clone, Debug)]
pub struct BufferUpload {
    buffer: BufferHandle,
    destination_offset: u64,
    payload: Arc<[u8]>,
    source_range: Range<usize>,
}

impl BufferUpload {
    pub fn from_payload(buffer: BufferHandle, destination_offset: u64, payload: Arc<[u8]>) -> Self {
        let payload_len = payload.len();
        Self {
            buffer,
            destination_offset,
            payload,
            source_range: 0..payload_len,
        }
    }

    pub fn new(
        buffer: BufferHandle,
        destination_offset: u64,
        payload: Arc<[u8]>,
        source_range: Range<usize>,
    ) -> Option<Self> {
        valid_source_range(&source_range, payload.len()).then_some(Self {
            buffer,
            destination_offset,
            payload,
            source_range,
        })
    }

    pub const fn buffer(&self) -> BufferHandle {
        self.buffer
    }

    pub const fn destination_offset(&self) -> u64 {
        self.destination_offset
    }

    pub fn payload_owner(&self) -> &Arc<[u8]> {
        &self.payload
    }

    pub fn source_range(&self) -> Range<usize> {
        self.source_range.clone()
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload[self.source_range.clone()]
    }

    pub fn payload_byte_len(&self) -> u64 {
        self.source_range.len() as u64
    }
}

#[derive(Clone, Debug, Default)]
pub struct BufferUploadBatch {
    uploads: Vec<BufferUpload>,
}

impl BufferUploadBatch {
    pub const fn new() -> Self {
        Self {
            uploads: Vec::new(),
        }
    }

    pub fn push(&mut self, upload: BufferUpload) {
        self.uploads.push(upload);
    }

    pub const fn is_empty(&self) -> bool {
        self.uploads.is_empty()
    }

    pub const fn len(&self) -> usize {
        self.uploads.len()
    }

    pub fn uploads(&self) -> &[BufferUpload] {
        &self.uploads
    }

    pub fn payload_byte_len(&self) -> Option<u64> {
        checked_payload_byte_len(self.uploads.iter().map(BufferUpload::payload_byte_len))
    }

    pub fn into_uploads(self) -> Vec<BufferUpload> {
        self.uploads
    }
}

impl From<BufferUpload> for BufferUploadBatch {
    fn from(upload: BufferUpload) -> Self {
        Self {
            uploads: vec![upload],
        }
    }
}

#[derive(Clone, Debug)]
pub struct TextureUpload {
    texture: TextureHandle,
    region: TextureCopyRegion,
    bytes_per_row: u64,
    payload: Arc<[u8]>,
    source_range: Range<usize>,
}

impl TextureUpload {
    pub fn from_payload(
        texture: TextureHandle,
        region: TextureCopyRegion,
        bytes_per_row: u64,
        payload: Arc<[u8]>,
    ) -> Self {
        let payload_len = payload.len();
        Self {
            texture,
            region,
            bytes_per_row,
            payload,
            source_range: 0..payload_len,
        }
    }

    pub fn new(
        texture: TextureHandle,
        region: TextureCopyRegion,
        bytes_per_row: u64,
        payload: Arc<[u8]>,
        source_range: Range<usize>,
    ) -> Option<Self> {
        valid_source_range(&source_range, payload.len()).then_some(Self {
            texture,
            region,
            bytes_per_row,
            payload,
            source_range,
        })
    }

    pub const fn texture(&self) -> TextureHandle {
        self.texture
    }

    pub const fn region(&self) -> TextureCopyRegion {
        self.region
    }

    pub const fn bytes_per_row(&self) -> u64 {
        self.bytes_per_row
    }

    pub fn payload_owner(&self) -> &Arc<[u8]> {
        &self.payload
    }

    pub fn source_range(&self) -> Range<usize> {
        self.source_range.clone()
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload[self.source_range.clone()]
    }

    pub fn payload_byte_len(&self) -> u64 {
        self.source_range.len() as u64
    }
}

#[derive(Clone, Debug, Default)]
pub struct TextureUploadBatch {
    uploads: Vec<TextureUpload>,
}

impl TextureUploadBatch {
    pub const fn new() -> Self {
        Self {
            uploads: Vec::new(),
        }
    }

    pub fn push(&mut self, upload: TextureUpload) {
        self.uploads.push(upload);
    }

    pub const fn is_empty(&self) -> bool {
        self.uploads.is_empty()
    }

    pub const fn len(&self) -> usize {
        self.uploads.len()
    }

    pub fn uploads(&self) -> &[TextureUpload] {
        &self.uploads
    }

    pub fn payload_byte_len(&self) -> Option<u64> {
        checked_payload_byte_len(self.uploads.iter().map(TextureUpload::payload_byte_len))
    }

    pub fn into_uploads(self) -> Vec<TextureUpload> {
        self.uploads
    }
}

impl From<TextureUpload> for TextureUploadBatch {
    fn from(upload: TextureUpload) -> Self {
        Self {
            uploads: vec![upload],
        }
    }
}

fn valid_source_range(range: &Range<usize>, payload_len: usize) -> bool {
    range.start <= range.end && range.end <= payload_len
}

fn checked_payload_byte_len(mut lengths: impl Iterator<Item = u64>) -> Option<u64> {
    lengths.try_fold(0_u64, u64::checked_add)
}

#[cfg(test)]
#[path = "tests/upload.rs"]
mod tests;
