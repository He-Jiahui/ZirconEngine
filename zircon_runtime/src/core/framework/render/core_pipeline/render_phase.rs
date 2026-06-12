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

pub const RENDER_PHASES_BY_QUEUE_ORDER: [RenderPhase; 13] = [
    RenderPhase::Prepass,
    RenderPhase::Shadow,
    RenderPhase::Opaque2d,
    RenderPhase::Opaque3d,
    RenderPhase::AlphaMask2d,
    RenderPhase::AlphaMask3d,
    RenderPhase::Deferred,
    RenderPhase::Transparent2d,
    RenderPhase::Transparent3d,
    RenderPhase::PostProcess,
    RenderPhase::Ui,
    RenderPhase::Overlay,
    RenderPhase::Debug,
];

impl RenderPhase {
    pub const fn diagnostic_name(self) -> &'static str {
        match self {
            Self::Opaque2d => "opaque-2d",
            Self::AlphaMask2d => "alpha-mask-2d",
            Self::Transparent2d => "transparent-2d",
            Self::Opaque3d => "opaque-3d",
            Self::AlphaMask3d => "alpha-mask-3d",
            Self::Transparent3d => "transparent-3d",
            Self::Prepass => "prepass",
            Self::Shadow => "shadow",
            Self::Deferred => "deferred",
            Self::PostProcess => "post-process",
            Self::Ui => "ui",
            Self::Overlay => "overlay",
            Self::Debug => "debug",
        }
    }

    /// Stable cross-phase submission precedence before per-phase sort keys are compared.
    pub const fn queue_order(self) -> u8 {
        match self {
            Self::Prepass => 0,
            Self::Shadow => 1,
            Self::Opaque2d | Self::Opaque3d => 2,
            Self::AlphaMask2d | Self::AlphaMask3d => 3,
            Self::Deferred => 4,
            Self::Transparent2d | Self::Transparent3d => 5,
            Self::PostProcess => 6,
            Self::Ui => 7,
            Self::Overlay => 8,
            Self::Debug => 9,
        }
    }

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
