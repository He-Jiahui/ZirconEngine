/// Cached GPU-texture identity; full artifact provenance remains outside the frame upload path.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SourceCubemapUploadKey {
    pub source_revision: u64,
    pub source_hash: [u32; 4],
    pub pmrem_hash: [u32; 4],
    pub irradiance_cube_hash: [u32; 4],
}
