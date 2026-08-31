use zr_rhi::{BufferHandle, DiagnosticReadbackRequestId, TextureCopyRegion, TextureHandle};

use super::layout::{DiagnosticTextureMipChainReadbackLayout, DiagnosticTextureReadbackLayout};

/// A validated neutral source request waiting for the device owner to encode
/// it into exactly one submission-qualified native batch.
#[derive(Clone, Copy)]
pub(crate) struct DiagnosticBufferReadbackRequest {
    pub(super) request: DiagnosticReadbackRequestId,
    pub(super) source: BufferHandle,
    pub(super) source_offset: u64,
    pub(super) byte_len: u64,
}

impl DiagnosticBufferReadbackRequest {
    pub(crate) const fn request(&self) -> DiagnosticReadbackRequestId {
        self.request
    }

    pub(crate) const fn source(&self) -> BufferHandle {
        self.source
    }

    pub(crate) const fn source_offset(&self) -> u64 {
        self.source_offset
    }

    pub(crate) const fn byte_len(&self) -> u64 {
        self.byte_len
    }
}

pub(crate) struct DiagnosticNativeBufferReadbackRequest {
    pub(super) request: DiagnosticReadbackRequestId,
    pub(super) source: wgpu::Buffer,
    pub(super) source_offset: u64,
    pub(super) byte_len: u64,
}

impl DiagnosticNativeBufferReadbackRequest {
    pub(crate) const fn request(&self) -> DiagnosticReadbackRequestId {
        self.request
    }

    pub(crate) fn source(&self) -> &wgpu::Buffer {
        &self.source
    }

    pub(crate) const fn source_offset(&self) -> u64 {
        self.source_offset
    }

    pub(crate) const fn byte_len(&self) -> u64 {
        self.byte_len
    }
}

#[derive(Clone, Copy)]
pub(crate) struct DiagnosticTextureReadbackRequest {
    pub(super) request: DiagnosticReadbackRequestId,
    pub(super) source: TextureHandle,
    pub(super) region: TextureCopyRegion,
    pub(super) layout: DiagnosticTextureReadbackLayout,
}

impl DiagnosticTextureReadbackRequest {
    pub(crate) const fn request(&self) -> DiagnosticReadbackRequestId {
        self.request
    }

    pub(crate) const fn source(&self) -> TextureHandle {
        self.source
    }

    pub(crate) const fn region(&self) -> TextureCopyRegion {
        self.region
    }

    pub(crate) const fn layout(&self) -> DiagnosticTextureReadbackLayout {
        self.layout
    }
}

pub(crate) struct DiagnosticNativeTextureReadbackRequest {
    pub(super) request: DiagnosticReadbackRequestId,
    pub(super) source: wgpu::Texture,
    pub(super) region: TextureCopyRegion,
    pub(super) layout: DiagnosticTextureReadbackLayout,
}

impl DiagnosticNativeTextureReadbackRequest {
    pub(crate) const fn request(&self) -> DiagnosticReadbackRequestId {
        self.request
    }

    pub(crate) fn source(&self) -> &wgpu::Texture {
        &self.source
    }

    pub(crate) const fn region(&self) -> TextureCopyRegion {
        self.region
    }

    pub(crate) const fn layout(&self) -> DiagnosticTextureReadbackLayout {
        self.layout
    }
}

pub(crate) struct DiagnosticNativeTextureMipChainReadbackRequest {
    pub(super) request: DiagnosticReadbackRequestId,
    pub(super) source: wgpu::Texture,
    pub(super) layout: DiagnosticTextureMipChainReadbackLayout,
}

impl DiagnosticNativeTextureMipChainReadbackRequest {
    pub(crate) const fn request(&self) -> DiagnosticReadbackRequestId {
        self.request
    }

    pub(crate) fn source(&self) -> &wgpu::Texture {
        &self.source
    }

    pub(crate) const fn layout(&self) -> &DiagnosticTextureMipChainReadbackLayout {
        &self.layout
    }
}

pub(crate) enum DiagnosticReadbackSource {
    Buffer(DiagnosticBufferReadbackRequest),
    NativeBuffer(DiagnosticNativeBufferReadbackRequest),
    Texture(DiagnosticTextureReadbackRequest),
    NativeTexture(DiagnosticNativeTextureReadbackRequest),
    NativeTextureMipChain(DiagnosticNativeTextureMipChainReadbackRequest),
}

impl DiagnosticReadbackSource {
    pub(crate) const fn request(&self) -> DiagnosticReadbackRequestId {
        match self {
            Self::Buffer(request) => request.request(),
            Self::NativeBuffer(request) => request.request(),
            Self::Texture(request) => request.request(),
            Self::NativeTexture(request) => request.request(),
            Self::NativeTextureMipChain(request) => request.request(),
        }
    }

    pub(crate) const fn staging_byte_len(&self) -> u64 {
        match self {
            Self::Buffer(request) => request.byte_len(),
            Self::NativeBuffer(request) => request.byte_len(),
            Self::Texture(request) => request.layout().staging_byte_len(),
            Self::NativeTexture(request) => request.layout().staging_byte_len(),
            Self::NativeTextureMipChain(request) => request.layout().staging_byte_len(),
        }
    }

    pub(crate) fn copy_payload(&self, mapped: &[u8], staging_offset: u64) -> Option<Vec<u8>> {
        let start = usize::try_from(staging_offset).ok()?;
        let byte_len = usize::try_from(self.staging_byte_len()).ok()?;
        let end = start.checked_add(byte_len)?;
        let staging = mapped.get(start..end)?;
        match self {
            Self::Buffer(_) | Self::NativeBuffer(_) => Some(staging.to_vec()),
            Self::Texture(request) => request.layout().unpack(staging),
            Self::NativeTexture(request) => request.layout().unpack(staging),
            Self::NativeTextureMipChain(request) => request.layout().unpack(staging),
        }
    }
}
