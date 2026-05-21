use crate::asset::AssetReference;
use crate::core::framework::render::RenderMaterialValidationError;

/// Non-fatal shader/material contract problem found while compiling RendererData.
#[derive(Clone, Debug, PartialEq)]
pub enum RendererFeatureContractDiagnostic {
    ShaderMissing {
        feature: String,
        reference: AssetReference,
    },
    MaterialMissing {
        feature: String,
        reference: AssetReference,
    },
    MaterialShaderMismatch {
        feature: String,
        feature_shader: AssetReference,
        material_shader: AssetReference,
    },
    MissingEntryPoint {
        feature: String,
        shader: AssetReference,
        entry_point: String,
    },
    MissingProperty {
        feature: String,
        shader: AssetReference,
        property: String,
    },
    MissingTextureSlot {
        feature: String,
        shader: AssetReference,
        slot: String,
    },
    MaterialValidation {
        feature: String,
        error: RenderMaterialValidationError,
    },
    MaterialDiagnostic {
        feature: String,
        material: AssetReference,
        diagnostic: String,
    },
    ShaderValidation {
        feature: String,
        shader: AssetReference,
        diagnostic: String,
    },
}

impl RendererFeatureContractDiagnostic {
    pub fn feature(&self) -> &str {
        match self {
            Self::ShaderMissing { feature, .. }
            | Self::MaterialMissing { feature, .. }
            | Self::MaterialShaderMismatch { feature, .. }
            | Self::MissingEntryPoint { feature, .. }
            | Self::MissingProperty { feature, .. }
            | Self::MissingTextureSlot { feature, .. }
            | Self::MaterialValidation { feature, .. }
            | Self::MaterialDiagnostic { feature, .. }
            | Self::ShaderValidation { feature, .. } => feature,
        }
    }
}
