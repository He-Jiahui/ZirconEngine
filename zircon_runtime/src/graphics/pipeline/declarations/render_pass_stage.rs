#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RenderPassStage {
    DepthPrepass,
    Shadow,
    Deferred,
    AmbientOcclusion,
    Lighting,
    Opaque2d,
    AlphaMask2d,
    Transparent2d,
    Opaque3d,
    AlphaMask3d,
    Transparent3d,
    Opaque,
    Transparent,
    PostProcess,
    Ui,
    Overlay,
    Debug,
}

impl RenderPassStage {
    pub const ALL: &[Self] = &[
        Self::DepthPrepass,
        Self::Shadow,
        Self::Deferred,
        Self::AmbientOcclusion,
        Self::Lighting,
        Self::Opaque2d,
        Self::AlphaMask2d,
        Self::Transparent2d,
        Self::Opaque3d,
        Self::AlphaMask3d,
        Self::Transparent3d,
        Self::Opaque,
        Self::Transparent,
        Self::PostProcess,
        Self::Ui,
        Self::Overlay,
        Self::Debug,
    ];

    pub const RENDERER_DATA_AUTHORING_STAGES: &[Self] = &[
        Self::DepthPrepass,
        Self::Shadow,
        Self::Deferred,
        Self::AmbientOcclusion,
        Self::Lighting,
        Self::Opaque2d,
        Self::AlphaMask2d,
        Self::Transparent2d,
        Self::Opaque3d,
        Self::AlphaMask3d,
        Self::Transparent3d,
        Self::PostProcess,
        Self::Ui,
        Self::Overlay,
        Self::Debug,
    ];

    pub const fn authoring_name(self) -> &'static str {
        match self {
            Self::DepthPrepass => "DepthPrepass",
            Self::Shadow => "Shadow",
            Self::Deferred => "Deferred",
            Self::AmbientOcclusion => "AmbientOcclusion",
            Self::Lighting => "Lighting",
            Self::Opaque2d => "Opaque2d",
            Self::AlphaMask2d => "AlphaMask2d",
            Self::Transparent2d => "Transparent2d",
            Self::Opaque3d => "Opaque3d",
            Self::AlphaMask3d => "AlphaMask3d",
            Self::Transparent3d => "Transparent3d",
            Self::Opaque => "Opaque",
            Self::Transparent => "Transparent",
            Self::PostProcess => "PostProcess",
            Self::Ui => "Ui",
            Self::Overlay => "Overlay",
            Self::Debug => "Debug",
        }
    }

    pub fn from_renderer_data_authoring_name(value: &str) -> Option<Self> {
        Self::RENDERER_DATA_AUTHORING_STAGES
            .iter()
            .copied()
            .find(|stage| stage.authoring_name() == value)
    }
}
