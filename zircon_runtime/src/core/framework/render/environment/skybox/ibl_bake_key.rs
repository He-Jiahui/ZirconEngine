use super::SkyboxMode;

/// 天空辐射源的烘焙缓存键，供场景提取、工件选择和运行时复水共用。
/// 展示强度与旋转在采样时施加，不属于改变源纹理的烘焙身份。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IblBakeKey {
    pub source_kind: u32,
    pub source_revision: u64,
    pub horizon_color: [u32; 4],
    pub zenith_color: [u32; 4],
    pub ground_color: [u32; 4],
    pub source_hash: [u32; 4],
}

impl IblBakeKey {
    pub const fn source_cubemap(source_revision: u64, source_hash: [u32; 4]) -> Self {
        Self {
            source_kind: SkyboxMode::SourceCubemap as u32,
            source_revision,
            horizon_color: [0; 4],
            zenith_color: [0; 4],
            ground_color: [0; 4],
            source_hash,
        }
    }
}
