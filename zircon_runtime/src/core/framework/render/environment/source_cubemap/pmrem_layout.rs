use super::{
    source_cubemap_mip_count, SOURCE_CUBEMAP_PMREM_FACE_SIZE, SOURCE_CUBEMAP_PMREM_MIP_COUNT,
};

/// Independent result layout for a prefiltered radiance cubemap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceCubemapPmremLayout {
    face_size: u32,
    mip_count: u32,
}

impl SourceCubemapPmremLayout {
    pub fn new(face_size: u32, mip_count: u32) -> Self {
        let face_size = face_size.max(1);
        Self {
            face_size,
            mip_count: mip_count.clamp(1, source_cubemap_mip_count(face_size)),
        }
    }

    pub fn from_face_size(face_size: u32) -> Self {
        let face_size = face_size.max(1);
        Self::new(face_size, source_cubemap_mip_count(face_size))
    }

    pub const fn face_size(self) -> u32 {
        self.face_size
    }

    pub const fn mip_count(self) -> u32 {
        self.mip_count
    }
}

impl Default for SourceCubemapPmremLayout {
    fn default() -> Self {
        Self {
            face_size: SOURCE_CUBEMAP_PMREM_FACE_SIZE,
            mip_count: SOURCE_CUBEMAP_PMREM_MIP_COUNT,
        }
    }
}
