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
    InvalidLightingModel {
        path: String,
        value: String,
    },
    RenderQueueAlphaModeConflict {
        source: RenderMaterialDiagnosticSource,
        path: String,
        alpha_mode: String,
        render_queue: u16,
        expected: String,
    },
    UnregisteredShadingModel {
        path: String,
        token: String,
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
    MissingRequiredTextureSlot {
        source: RenderMaterialDiagnosticSource,
        path: String,
        slot: String,
    },
    UnknownTextureSlot {
        source: RenderMaterialDiagnosticSource,
        path: String,
        slot: String,
    },
    UnknownMaterialOption {
        source: RenderMaterialDiagnosticSource,
        path: String,
        name: String,
    },
    MaterialOptionTypeMismatch {
        source: RenderMaterialDiagnosticSource,
        path: String,
        name: String,
        expected: String,
    },
    InvalidMaterialQueueOffset {
        source: RenderMaterialDiagnosticSource,
        path: String,
        offset: i16,
        expected: String,
    },
    InvalidMaterialParent {
        source: RenderMaterialDiagnosticSource,
        path: String,
        diagnostic: String,
    },
    MissingWgslCapture {
        source: RenderMaterialDiagnosticSource,
        path: String,
        name: String,
    },
    ShaderReadinessDiagnostic {
        source: RenderMaterialDiagnosticSource,
        path: String,
        diagnostic: String,
    },
}
