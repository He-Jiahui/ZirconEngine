use crate::core::math::UVec2;
use crate::core::resource::{ResourceHandle, TextureMarker};
use serde::{Deserialize, Serialize};

use super::RenderCameraTargetKind;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderCameraTarget {
    PrimarySurface,
    Texture(ResourceHandle<TextureMarker>),
    Headless { size: UVec2 },
}

impl Default for RenderCameraTarget {
    fn default() -> Self {
        Self::PrimarySurface
    }
}

impl RenderCameraTarget {
    pub fn kind(&self) -> RenderCameraTargetKind {
        match self {
            Self::PrimarySurface => RenderCameraTargetKind::PrimarySurface,
            Self::Texture(_) => RenderCameraTargetKind::Texture,
            Self::Headless { .. } => RenderCameraTargetKind::Headless,
        }
    }
}
