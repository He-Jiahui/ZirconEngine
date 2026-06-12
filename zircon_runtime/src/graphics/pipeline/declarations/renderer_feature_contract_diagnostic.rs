use crate::asset::AssetReference;
use crate::core::framework::render::{
    RenderMaterialDiagnosticSource, RenderMaterialValidationError,
};

/// Non-fatal shader/material contract problem found while compiling RendererData.
#[derive(Clone, Debug, PartialEq)]
pub enum RendererFeatureContractDiagnostic {
    ShaderMissing {
        feature: String,
        reference: AssetReference,
    },
    MaterialShaderMissing {
        feature: String,
        material: AssetReference,
        shader: AssetReference,
    },
    MaterialMissing {
        feature: String,
        reference: AssetReference,
    },
    MaterialShaderMismatch {
        feature: String,
        material: AssetReference,
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
        material: AssetReference,
        shader: Option<AssetReference>,
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

/// Editor/report triage for RendererData contract diagnostics.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RendererFeatureContractDiagnosticSeverity {
    Error,
    Warning,
}

impl RendererFeatureContractDiagnostic {
    pub fn feature(&self) -> &str {
        match self {
            Self::ShaderMissing { feature, .. }
            | Self::MaterialShaderMissing { feature, .. }
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

    pub fn severity(&self) -> RendererFeatureContractDiagnosticSeverity {
        match self {
            Self::MaterialDiagnostic { .. } | Self::ShaderValidation { .. } => {
                RendererFeatureContractDiagnosticSeverity::Warning
            }
            Self::ShaderMissing { .. }
            | Self::MaterialShaderMissing { .. }
            | Self::MaterialMissing { .. }
            | Self::MaterialShaderMismatch { .. }
            | Self::MissingEntryPoint { .. }
            | Self::MissingProperty { .. }
            | Self::MissingTextureSlot { .. }
            | Self::MaterialValidation { .. } => RendererFeatureContractDiagnosticSeverity::Error,
        }
    }

    pub fn material_reference(&self) -> Option<&AssetReference> {
        match self {
            Self::MaterialMissing { reference, .. } => Some(reference),
            Self::MaterialShaderMissing { material, .. }
            | Self::MaterialShaderMismatch { material, .. }
            | Self::MaterialValidation { material, .. }
            | Self::MaterialDiagnostic { material, .. } => Some(material),
            Self::ShaderMissing { .. }
            | Self::MissingEntryPoint { .. }
            | Self::MissingProperty { .. }
            | Self::MissingTextureSlot { .. }
            | Self::ShaderValidation { .. } => None,
        }
    }

    pub fn source(&self) -> Option<RenderMaterialDiagnosticSource> {
        match self {
            Self::ShaderMissing { .. }
            | Self::MaterialShaderMissing { .. }
            | Self::MaterialMissing { .. }
            | Self::MaterialShaderMismatch { .. } => {
                Some(RenderMaterialDiagnosticSource::DependencyResolution)
            }
            Self::MissingEntryPoint { .. } | Self::MissingProperty { .. } => {
                Some(RenderMaterialDiagnosticSource::ShaderSchema)
            }
            Self::MissingTextureSlot { .. } => Some(RenderMaterialDiagnosticSource::TextureSlot),
            Self::MaterialValidation { error, .. } => material_validation_error_source(error),
            Self::MaterialDiagnostic { .. } => None,
            Self::ShaderValidation { diagnostic, .. } => diagnostic
                .starts_with("wgsl_capture ")
                .then_some(RenderMaterialDiagnosticSource::WgslCapture),
        }
    }

    pub fn shader_references(&self) -> Vec<&AssetReference> {
        let mut references = Vec::new();
        match self {
            Self::ShaderMissing { reference, .. } => {
                push_unique_reference(&mut references, reference)
            }
            Self::MaterialShaderMissing { shader, .. } => {
                push_unique_reference(&mut references, shader)
            }
            Self::MaterialShaderMismatch {
                feature_shader,
                material_shader,
                ..
            } => {
                push_unique_reference(&mut references, feature_shader);
                push_unique_reference(&mut references, material_shader);
            }
            Self::MissingEntryPoint { shader, .. }
            | Self::MissingProperty { shader, .. }
            | Self::MissingTextureSlot { shader, .. }
            | Self::ShaderValidation { shader, .. } => {
                push_unique_reference(&mut references, shader)
            }
            Self::MaterialValidation { shader, error, .. } => {
                if let Some(shader) = shader {
                    push_unique_reference(&mut references, shader);
                }
                if let RenderMaterialValidationError::UnresolvedShaderReference { reference } =
                    error
                {
                    push_unique_reference(&mut references, reference);
                }
            }
            Self::MaterialMissing { .. } | Self::MaterialDiagnostic { .. } => {}
        }
        references
    }
}

fn material_validation_error_source(
    error: &RenderMaterialValidationError,
) -> Option<RenderMaterialDiagnosticSource> {
    match error {
        RenderMaterialValidationError::InvalidMaskCutoff { .. }
        | RenderMaterialValidationError::InvalidLightingModel { .. } => None,
        RenderMaterialValidationError::UnresolvedMaterialReference { .. }
        | RenderMaterialValidationError::MissingRuntimeShaderSource
        | RenderMaterialValidationError::UnresolvedShaderReference { .. }
        | RenderMaterialValidationError::UnresolvedTextureReference { .. }
        | RenderMaterialValidationError::TextureNotUploadReady { .. } => {
            Some(RenderMaterialDiagnosticSource::DependencyResolution)
        }
        RenderMaterialValidationError::UnknownPropertyOverride { source, .. }
        | RenderMaterialValidationError::PropertyOverrideTypeMismatch { source, .. }
        | RenderMaterialValidationError::MissingRequiredProperty { source, .. }
        | RenderMaterialValidationError::MissingRequiredTextureSlot { source, .. }
        | RenderMaterialValidationError::UnknownTextureSlot { source, .. }
        | RenderMaterialValidationError::MissingWgslCapture { source, .. }
        | RenderMaterialValidationError::ShaderReadinessDiagnostic { source, .. } => Some(*source),
    }
}

fn push_unique_reference<'a>(
    references: &mut Vec<&'a AssetReference>,
    reference: &'a AssetReference,
) {
    if !references.iter().any(|existing| *existing == reference) {
        references.push(reference);
    }
}
