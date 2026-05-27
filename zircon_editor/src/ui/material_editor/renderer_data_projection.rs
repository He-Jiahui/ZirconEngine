use std::collections::BTreeMap;

use zircon_runtime::asset::AssetReference;
use zircon_runtime::core::framework::render::{
    RenderMaterialDiagnosticSource, RenderMaterialValidationError,
};
use zircon_runtime::graphics::{
    RenderPassStage, RendererAsset, RendererFeatureAsset, RendererFeatureContractDiagnostic,
    RendererFeatureSource,
};

/// Read-only editor projection for runtime-owned SRP RendererData state.
#[derive(Clone, Debug, PartialEq)]
pub struct RendererDataEditorProjection {
    pub renderer_name: String,
    pub stages: Vec<String>,
    pub features: Vec<RendererDataFeatureRow>,
    pub diagnostics: Vec<RendererDataDiagnosticRow>,
}

/// One renderer feature row with feature-local shader/material contract expectations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RendererDataFeatureRow {
    pub name: String,
    pub source: String,
    pub enabled: bool,
    pub quality_gate: Option<String>,
    pub shader_reference: Option<AssetReference>,
    pub material_reference: Option<AssetReference>,
    pub required_entry_points: Vec<String>,
    pub expected_properties: Vec<String>,
    pub expected_texture_slots: Vec<String>,
    pub diagnostic_count: usize,
}

/// Editor-facing SRP diagnostic row keyed by runtime feature name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RendererDataDiagnosticRow {
    pub feature: String,
    pub source: Option<RenderMaterialDiagnosticSource>,
    pub path: String,
    pub message: String,
}

impl RendererDataEditorProjection {
    pub fn from_renderer_asset(
        renderer: &RendererAsset,
        diagnostics: &[RendererFeatureContractDiagnostic],
    ) -> Self {
        let diagnostics = diagnostics.iter().map(diagnostic_row).collect::<Vec<_>>();
        let diagnostic_counts = diagnostic_counts_by_feature(&diagnostics);
        let features = renderer
            .features
            .iter()
            .map(|feature| feature_row(feature, &diagnostic_counts))
            .collect();

        Self {
            renderer_name: renderer.name.clone(),
            stages: renderer.stages.iter().map(stage_name).collect(),
            features,
            diagnostics,
        }
    }
}

fn feature_row(
    feature: &RendererFeatureAsset,
    diagnostic_counts: &BTreeMap<String, usize>,
) -> RendererDataFeatureRow {
    let name = feature.feature_name();
    RendererDataFeatureRow {
        diagnostic_count: diagnostic_counts.get(&name).copied().unwrap_or_default(),
        name,
        source: feature_source_name(&feature.feature),
        enabled: feature.enabled,
        quality_gate: feature.quality_gate.map(|gate| format!("{gate:?}")),
        shader_reference: feature.asset_references.shader.clone(),
        material_reference: feature.asset_references.material.clone(),
        required_entry_points: feature.asset_references.required_entry_points.clone(),
        expected_properties: feature.asset_references.expected_properties.clone(),
        expected_texture_slots: feature.asset_references.expected_texture_slots.clone(),
    }
}

fn diagnostic_counts_by_feature(
    diagnostics: &[RendererDataDiagnosticRow],
) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for diagnostic in diagnostics {
        *counts.entry(diagnostic.feature.clone()).or_insert(0) += 1;
    }
    counts
}

fn diagnostic_row(diagnostic: &RendererFeatureContractDiagnostic) -> RendererDataDiagnosticRow {
    match diagnostic {
        RendererFeatureContractDiagnostic::ShaderMissing { feature, reference } => {
            RendererDataDiagnosticRow {
                feature: feature.clone(),
                source: Some(RenderMaterialDiagnosticSource::DependencyResolution),
                path: format!("features.{feature}.shader"),
                message: format!("shader `{}` could not be resolved", reference.locator),
            }
        }
        RendererFeatureContractDiagnostic::MaterialMissing { feature, reference } => {
            RendererDataDiagnosticRow {
                feature: feature.clone(),
                source: Some(RenderMaterialDiagnosticSource::DependencyResolution),
                path: format!("features.{feature}.material"),
                message: format!("material `{}` could not be resolved", reference.locator),
            }
        }
        RendererFeatureContractDiagnostic::MaterialShaderMismatch {
            feature,
            feature_shader,
            material_shader,
        } => RendererDataDiagnosticRow {
            feature: feature.clone(),
            source: Some(RenderMaterialDiagnosticSource::DependencyResolution),
            path: format!("features.{feature}.material.shader"),
            message: format!(
                "material shader `{}` does not match feature shader `{}`",
                material_shader.locator, feature_shader.locator
            ),
        },
        RendererFeatureContractDiagnostic::MissingEntryPoint {
            feature,
            shader,
            entry_point,
        } => RendererDataDiagnosticRow {
            feature: feature.clone(),
            source: Some(RenderMaterialDiagnosticSource::ShaderSchema),
            path: format!("features.{feature}.required_entry_points.{entry_point}"),
            message: format!(
                "shader `{}` is missing entry point `{entry_point}`",
                shader.locator
            ),
        },
        RendererFeatureContractDiagnostic::MissingProperty {
            feature,
            shader,
            property,
        } => RendererDataDiagnosticRow {
            feature: feature.clone(),
            source: Some(RenderMaterialDiagnosticSource::ShaderSchema),
            path: format!("features.{feature}.expected_properties.{property}"),
            message: format!(
                "shader `{}` is missing material property `{property}`",
                shader.locator
            ),
        },
        RendererFeatureContractDiagnostic::MissingTextureSlot {
            feature,
            shader,
            slot,
        } => RendererDataDiagnosticRow {
            feature: feature.clone(),
            source: Some(RenderMaterialDiagnosticSource::TextureSlot),
            path: format!("features.{feature}.expected_texture_slots.{slot}"),
            message: format!(
                "shader `{}` is missing texture slot `{slot}`",
                shader.locator
            ),
        },
        RendererFeatureContractDiagnostic::MaterialValidation { feature, error } => {
            material_validation_diagnostic_row(feature, error)
        }
        RendererFeatureContractDiagnostic::MaterialDiagnostic {
            feature,
            material,
            diagnostic,
        } => RendererDataDiagnosticRow {
            feature: feature.clone(),
            source: None,
            path: format!("features.{feature}.material.validation_diagnostics"),
            message: format!("material `{}` validation: {diagnostic}", material.locator),
        },
        RendererFeatureContractDiagnostic::ShaderValidation {
            feature,
            shader,
            diagnostic,
        } => RendererDataDiagnosticRow {
            feature: feature.clone(),
            source: diagnostic
                .starts_with("wgsl_capture ")
                .then_some(RenderMaterialDiagnosticSource::WgslCapture),
            path: format!("features.{feature}.shader.validation_diagnostics"),
            message: format!("shader `{}` validation: {diagnostic}", shader.locator),
        },
    }
}

fn material_validation_diagnostic_row(
    feature: &str,
    error: &RenderMaterialValidationError,
) -> RendererDataDiagnosticRow {
    match error {
        RenderMaterialValidationError::InvalidMaskCutoff { cutoff } => RendererDataDiagnosticRow {
            feature: feature.to_string(),
            source: None,
            path: "overrides.alpha_mode.cutoff".to_string(),
            message: format!("alpha mask cutoff {cutoff} must be finite and within 0.0..=1.0"),
        },
        RenderMaterialValidationError::UnresolvedMaterialReference { material } => {
            RendererDataDiagnosticRow {
                feature: feature.to_string(),
                source: Some(RenderMaterialDiagnosticSource::DependencyResolution),
                path: "material".to_string(),
                message: format!("material `{material}` could not be resolved"),
            }
        }
        RenderMaterialValidationError::MissingRuntimeShaderSource => RendererDataDiagnosticRow {
            feature: feature.to_string(),
            source: Some(RenderMaterialDiagnosticSource::DependencyResolution),
            path: "shader".to_string(),
            message: "shader has no runtime WGSL source".to_string(),
        },
        RenderMaterialValidationError::UnresolvedShaderReference { reference } => {
            RendererDataDiagnosticRow {
                feature: feature.to_string(),
                source: Some(RenderMaterialDiagnosticSource::DependencyResolution),
                path: "shader".to_string(),
                message: format!("shader `{}` could not be resolved", reference.locator),
            }
        }
        RenderMaterialValidationError::UnresolvedTextureReference { slot, reference } => {
            RendererDataDiagnosticRow {
                feature: feature.to_string(),
                source: Some(RenderMaterialDiagnosticSource::DependencyResolution),
                path: format!("textures.{slot}"),
                message: format!("texture `{}` could not be resolved", reference.locator),
            }
        }
        RenderMaterialValidationError::TextureNotUploadReady {
            slot,
            reference,
            reason,
        } => RendererDataDiagnosticRow {
            feature: feature.to_string(),
            source: Some(RenderMaterialDiagnosticSource::DependencyResolution),
            path: format!("textures.{slot}"),
            message: format!(
                "texture `{}` is not upload-ready: {reason}",
                reference.locator
            ),
        },
        RenderMaterialValidationError::UnknownPropertyOverride { source, path, name } => {
            RendererDataDiagnosticRow {
                feature: feature.to_string(),
                source: Some(*source),
                path: path.clone(),
                message: format!("property override `{name}` is not declared by the shader"),
            }
        }
        RenderMaterialValidationError::PropertyOverrideTypeMismatch {
            source,
            path,
            name,
            expected,
        } => RendererDataDiagnosticRow {
            feature: feature.to_string(),
            source: Some(*source),
            path: path.clone(),
            message: format!("property override `{name}` must match shader type `{expected}`"),
        },
        RenderMaterialValidationError::MissingRequiredProperty { source, path, name } => {
            RendererDataDiagnosticRow {
                feature: feature.to_string(),
                source: Some(*source),
                path: path.clone(),
                message: format!("required shader property `{name}` needs a material override"),
            }
        }
        RenderMaterialValidationError::MissingRequiredTextureSlot { source, path, slot } => {
            RendererDataDiagnosticRow {
                feature: feature.to_string(),
                source: Some(*source),
                path: path.clone(),
                message: format!(
                    "required texture slot `{slot}` needs a material texture reference"
                ),
            }
        }
        RenderMaterialValidationError::UnknownTextureSlot { source, path, slot } => {
            RendererDataDiagnosticRow {
                feature: feature.to_string(),
                source: Some(*source),
                path: path.clone(),
                message: format!("texture slot `{slot}` is not declared by the shader"),
            }
        }
        RenderMaterialValidationError::MissingWgslCapture { source, path, name } => {
            RendererDataDiagnosticRow {
                feature: feature.to_string(),
                source: Some(*source),
                path: path.clone(),
                message: format!("shader WGSL does not appear to capture `{name}`"),
            }
        }
        RenderMaterialValidationError::ShaderReadinessDiagnostic {
            source,
            path,
            diagnostic,
        } => RendererDataDiagnosticRow {
            feature: feature.to_string(),
            source: Some(*source),
            path: path.clone(),
            message: diagnostic.clone(),
        },
    }
}

fn feature_source_name(source: &RendererFeatureSource) -> String {
    match source {
        RendererFeatureSource::Builtin(feature) => format!("{feature:?}"),
        RendererFeatureSource::Plugin(name) => format!("plugin:{name}"),
    }
}

fn stage_name(stage: &RenderPassStage) -> String {
    format!("{stage:?}")
}
