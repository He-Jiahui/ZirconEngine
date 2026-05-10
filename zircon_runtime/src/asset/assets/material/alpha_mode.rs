use serde::{Deserialize, Serialize};

use crate::core::framework::render::RenderMaterialAlphaMode;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum AlphaMode {
    Opaque,
    Mask { cutoff: f32 },
    Blend,
}

impl From<&AlphaMode> for RenderMaterialAlphaMode {
    fn from(value: &AlphaMode) -> Self {
        match value {
            AlphaMode::Opaque => Self::Opaque,
            AlphaMode::Mask { cutoff } => Self::Mask { cutoff: *cutoff },
            AlphaMode::Blend => Self::Blend,
        }
    }
}
