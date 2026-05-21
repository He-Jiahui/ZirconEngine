use serde::{Deserialize, Serialize};

use crate::core::resource::{AssetReference, ResourceId};

use super::RenderMaterialDiagnosticSource;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum RenderMaterialValidationError {
    InvalidMaskCutoff {
        cutoff: f32,
    },
    UnresolvedMaterialReference {
        material: ResourceId,
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
    UnknownPropertyOverride {
        source: RenderMaterialDiagnosticSource,
        path: String,
        name: String,
    },
    PropertyOverrideTypeMismatch {
        source: RenderMaterialDiagnosticSource,
        path: String,
        name: String,
        expected: String,
    },
    MissingRequiredProperty {
        source: RenderMaterialDiagnosticSource,
        path: String,
        name: String,
    },
    UnknownTextureSlot {
        source: RenderMaterialDiagnosticSource,
        path: String,
        slot: String,
    },
    MissingWgslCapture {
        source: RenderMaterialDiagnosticSource,
        path: String,
        name: String,
    },
}
