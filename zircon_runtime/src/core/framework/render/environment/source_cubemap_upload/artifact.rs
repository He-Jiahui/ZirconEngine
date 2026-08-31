use std::sync::Arc;

/// One RGBA16F cubemap mip, packed face-major with WGPU-aligned rows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceCubemapUploadMip {
    mip_level: u32,
    face_size: u32,
    bytes_per_row: u32,
    bytes: Arc<[u8]>,
}

impl SourceCubemapUploadMip {
    pub(super) fn new(mip_level: u32, face_size: u32, bytes_per_row: u32, bytes: Vec<u8>) -> Self {
        Self {
            mip_level,
            face_size,
            bytes_per_row,
            bytes: bytes.into(),
        }
    }

    pub const fn mip_level(&self) -> u32 {
        self.mip_level
    }

    pub const fn face_size(&self) -> u32 {
        self.face_size
    }

    pub const fn bytes_per_row(&self) -> u32 {
        self.bytes_per_row
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Immutable upload payload built before the render submission path consumes an environment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceCubemapUploadArtifact {
    source_mips: Vec<SourceCubemapUploadMip>,
    pmrem_mips: Vec<SourceCubemapUploadMip>,
    irradiance_mip: SourceCubemapUploadMip,
}

impl SourceCubemapUploadArtifact {
    pub(super) fn new(
        source_mips: Vec<SourceCubemapUploadMip>,
        pmrem_mips: Vec<SourceCubemapUploadMip>,
        irradiance_mip: SourceCubemapUploadMip,
    ) -> Self {
        Self {
            source_mips,
            pmrem_mips,
            irradiance_mip,
        }
    }

    pub fn source_mips(&self) -> &[SourceCubemapUploadMip] {
        &self.source_mips
    }

    pub fn pmrem_mips(&self) -> &[SourceCubemapUploadMip] {
        &self.pmrem_mips
    }

    pub const fn irradiance_mip(&self) -> &SourceCubemapUploadMip {
        &self.irradiance_mip
    }
}
