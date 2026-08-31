use core::ops::Range;
use std::sync::Arc;

/// One native buffer write backed by a shared immutable source payload.
#[derive(Clone)]
pub struct WgpuBufferUpload {
    buffer: wgpu::Buffer,
    offset: u64,
    payload: Arc<[u8]>,
    source_range: Range<usize>,
}

impl WgpuBufferUpload {
    pub fn new(
        buffer: wgpu::Buffer,
        offset: u64,
        payload: Arc<[u8]>,
        source_range: Range<usize>,
    ) -> Option<Self> {
        (source_range.start <= source_range.end && source_range.end <= payload.len()).then_some(
            Self {
                buffer,
                offset,
                payload,
                source_range,
            },
        )
    }

    /// Copies one complete byte slice into an immutable upload payload.
    pub fn from_bytes(buffer: wgpu::Buffer, offset: u64, bytes: &[u8]) -> Self {
        let payload: Arc<[u8]> = bytes.into();
        let payload_len = payload.len();
        Self {
            buffer,
            offset,
            payload,
            source_range: 0..payload_len,
        }
    }

    /// Moves one complete owned byte vector into an immutable upload payload.
    pub fn from_owned_bytes(buffer: wgpu::Buffer, offset: u64, bytes: Vec<u8>) -> Self {
        let payload: Arc<[u8]> = bytes.into();
        let payload_len = payload.len();
        Self {
            buffer,
            offset,
            payload,
            source_range: 0..payload_len,
        }
    }

    pub(super) fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub(super) const fn offset(&self) -> u64 {
        self.offset
    }

    pub(super) fn payload(&self) -> &[u8] {
        &self.payload[self.source_range.clone()]
    }

    pub(super) fn payload_byte_len(&self) -> u64 {
        self.source_range.len() as u64
    }
}

/// One logical buffer upload that receives one copy-queue ticket.
#[derive(Default)]
pub struct WgpuBufferUploadBatch {
    uploads: Vec<WgpuBufferUpload>,
}

impl WgpuBufferUploadBatch {
    pub const fn new() -> Self {
        Self {
            uploads: Vec::new(),
        }
    }

    pub fn push(&mut self, upload: WgpuBufferUpload) {
        self.uploads.push(upload);
    }

    /// Moves every write from `other` into this logical submission without
    /// cloning payloads or native buffer handles.
    pub fn append(&mut self, other: &mut Self) {
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
            .map(WgpuBufferUpload::payload_byte_len)
            .fold(0_u64, u64::saturating_add)
    }

    pub(super) fn into_uploads(self) -> Vec<WgpuBufferUpload> {
        self.uploads
    }
}

impl From<WgpuBufferUpload> for WgpuBufferUploadBatch {
    fn from(upload: WgpuBufferUpload) -> Self {
        Self {
            uploads: vec![upload],
        }
    }
}
