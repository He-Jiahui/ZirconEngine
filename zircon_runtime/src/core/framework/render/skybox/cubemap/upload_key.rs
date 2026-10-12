/// Cached GPU-texture identity; full artifact provenance remains outside the frame upload path.
/// 逐纹理内容变化更新对应 hash 或 source_revision，帧提交据此决定是否重用预编码上传字节。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SourceCubemapUploadKey {
    pub source_revision: u64,
    pub source_hash: [u32; 4],
    pub pmrem_hash: [u32; 4],
    pub irradiance_cube_hash: [u32; 4],
}
