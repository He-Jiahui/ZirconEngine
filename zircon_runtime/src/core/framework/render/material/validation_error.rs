use serde::{Deserialize, Serialize};

use crate::core::resource::AssetReference;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum RenderMaterialValidationError {
    InvalidMaskCutoff {
        cutoff: f32,
    },
    MissingRuntimeShaderSource,
    UnresolvedShaderReference {
        reference: AssetReference,
    },
    UnresolvedTextureReference {
        slot: String,
        reference: AssetReference,
    },
    TextureNotUploadReady {
        slot: String,
        reference: AssetReference,
        reason: String,
    },
}
