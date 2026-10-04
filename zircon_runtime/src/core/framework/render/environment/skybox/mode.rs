/// 天空盒模式的稳定上传编号；更改判别值会改变 GPU 与烘焙键的协议。
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkyboxMode {
    Disabled = 0,
    ProceduralGradient = 1,
    SourceCubemap = 3,
}

impl SkyboxMode {
    pub(super) fn source_kind(self) -> u32 {
        match self {
            Self::Disabled => 0,
            Self::ProceduralGradient => 1,
            Self::SourceCubemap => 3,
        }
    }
}
