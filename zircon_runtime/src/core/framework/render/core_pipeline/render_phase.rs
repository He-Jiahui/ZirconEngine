use serde::{Deserialize, Serialize};

use super::CorePipelineKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RenderPhase {
    Opaque2d,
    AlphaMask2d,
    Transparent2d,
    Opaque3d,
    AlphaMask3d,
    Transparent3d,
    Prepass,
    Shadow,
    Deferred,
    PostProcess,
    Ui,
    Overlay,
    Debug,
}

impl RenderPhase {
    pub const fn mesh_phase(
        pipeline: CorePipelineKind,
        alpha_mask: bool,
        transparent: bool,
    ) -> Self {
        match (pipeline, alpha_mask, transparent) {
            (CorePipelineKind::Core2d, true, _) => Self::AlphaMask2d,
            (CorePipelineKind::Core2d, false, true) => Self::Transparent2d,
            (CorePipelineKind::Core2d, false, false) => Self::Opaque2d,
            (CorePipelineKind::Core3d, true, _) => Self::AlphaMask3d,
            (CorePipelineKind::Core3d, false, true) => Self::Transparent3d,
            (CorePipelineKind::Core3d, false, false) => Self::Opaque3d,
        }
    }

    pub const fn is_transparent(self) -> bool {
        matches!(self, Self::Transparent2d | Self::Transparent3d)
    }

    pub const fn is_opaque_like(self) -> bool {
        matches!(
            self,
            Self::Opaque2d | Self::AlphaMask2d | Self::Opaque3d | Self::AlphaMask3d
        )
    }
}
