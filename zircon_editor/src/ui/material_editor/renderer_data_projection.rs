use std::collections::{BTreeMap, HashMap};

use zircon_runtime::asset::AssetReference;
use zircon_runtime::core::framework::render::{
    RenderMaterialDiagnosticSource, RenderMaterialValidationError,
};
use zircon_runtime::graphics::{
    RenderPassStage, RendererAsset, RendererFeatureAsset, RendererFeatureContractDiagnostic,
    RendererFeatureContractDiagnosticSeverity, RendererFeatureSource,
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
    pub material_reference: Option<AssetReference>,
    pub shader_references: Vec<AssetReference>,
    pub source: Option<RenderMaterialDiagnosticSource>,
    pub severity: RendererFeatureContractDiagnosticSeverity,
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

    pub fn diagnostics_by_feature(&self) -> BTreeMap<&str, Vec<&RendererDataDiagnosticRow>> {
        let mut diagnostics = BTreeMap::new();
        for diagnostic in &self.diagnostics {
            diagnostics
                .entry(diagnostic.feature.as_str())
                .or_insert_with(Vec::new)
                .push(diagnostic);
        }
        diagnostics
    }

    pub fn diagnostics_by_source(
        &self,
    ) -> BTreeMap<RenderMaterialDiagnosticSource, Vec<&RendererDataDiagnosticRow>> {
        let mut diagnostics = BTreeMap::new();
        for diagnostic in &self.diagnostics {
            let Some(source) = diagnostic.source else {
                continue;
            };
            diagnostics
                .entry(source)
                .or_insert_with(Vec::new)
                .push(diagnostic);
        }
        diagnostics
    }

    pub fn diagnostics_by_severity(
        &self,
    ) -> BTreeMap<RendererFeatureContractDiagnosticSeverity, Vec<&RendererDataDiagnosticRow>> {
        let mut diagnostics = BTreeMap::new();
        for diagnostic in &self.diagnostics {
            diagnostics
                .entry(diagnostic.severity)
                .or_insert_with(Vec::new)
                .push(diagnostic);
        }
        diagnostics
    }

    pub fn diagnostics_by_material(
        &self,
    ) -> HashMap<AssetReference, Vec<&RendererDataDiagnosticRow>> {
        let mut diagnostics = HashMap::new();
        for diagnostic in &self.diagnostics {
            let Some(material) = diagnostic.material_reference.as_ref() else {
                continue;
            };
            diagnostics
                .entry(material.clone())
                .or_insert_with(Vec::new)
                .push(diagnostic);
        }
        diagnostics
    }

    pub fn diagnostics_by_shader(
        &self,
    ) -> HashMap<AssetReference, Vec<&RendererDataDiagnosticRow>> {
        let mut diagnostics = HashMap::new();
        for diagnostic in &self.diagnostics {
            for shader in &diagnostic.shader_references {
                diagnostics
                    .entry(shader.clone())
                    .or_insert_with(Vec::new)
                    .push(diagnostic);
            }
        }
        diagnostics
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
    let row = match diagnostic {
        RendererFeatureContractDiagnostic::ShaderMissing { feature, reference } => {
            RendererDataDiagnosticRow {
                feature: feature.clone(),
                material_reference: None,
                shader_references: Vec::new(),
                source: None,
                severity: RendererFeatureContractDiagnosticSeverity::Error,
                path: format!("features.{feature}.shader"),
                message: format!("shader `{}` could not be resolved", reference.locator),
            }
        }
        RendererFeatureContractDiagnostic::MaterialMissing { feature, reference } => {
            RendererDataDiagnosticRow {
                feature: feature.clone(),
                material_reference: None,
                shader_references: Vec::new(),
                source: None,
                severity: RendererFeatureContractDiagnosticSeverity::Error,
                path: format!("features.{feature}.material"),
                message: format!("material `{}` could not be resolved", reference.locator),
            }
        }
        RendererFeatureContractDiagnostic::MaterialShaderMissing {
            feature,
            material,
            shader,
        } => RendererDataDiagnosticRow {
            feature: feature.clone(),
            material_reference: None,
            shader_references: Vec::new(),
            source: None,
            severity: RendererFeatureContractDiagnosticSeverity::Error,
            path: format!("features.{feature}.material.shader"),
            message: format!(
                "material `{}` shader `{}` could not be resolved",
                material.locator, shader.locator
            ),
        },
        RendererFeatureContractDiagnostic::MaterialShaderMismatch {
            feature,
            feature_shader,
            material_shader,
            ..
        } => RendererDataDiagnosticRow {
            feature: feature.clone(),
            material_reference: None,
            shader_references: Vec::new(),
            source: None,
            severity: RendererFeatureContractDiagnosticSeverity::Error,
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
            material_reference: None,
            shader_references: Vec::new(),
            source: None,
            severity: RendererFeatureContractDiagnosticSeverity::Error,
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
            material_reference: None,
            shader_references: Vec::new(),
            source: None,
            severity: RendererFeatureContractDiagnosticSeverity::Error,
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
            material_reference: None,
            shader_references: Vec::new(),
            source: None,
            severity: RendererFeatureContractDiagnosticSeverity::Error,
            path: format!("features.{feature}.expected_texture_slots.{slot}"),
            message: format!(
                "shader `{}` is missing texture slot `{slot}`",
                shader.locator
            ),
        },
        RendererFeatureContractDiagnostic::MaterialValidation { feature, error, .. } => {
            material_validation_diagnostic_row(feature, error)
        }
        RendererFeatureContractDiagnostic::MaterialDiagnostic {
            feature,
            material,
            diagnostic,
        } => RendererDataDiagnosticRow {
            feature: feature.clone(),
            material_reference: None,
            shader_references: Vec::new(),
            source: None,
            severity: RendererFeatureContractDiagnosticSeverity::Error,
            path: format!("features.{feature}.material.validation_diagnostics"),
            message: format!("material `{}` validation: {diagnostic}", material.locator),
        },
        RendererFeatureContractDiagnostic::ShaderValidation {
            feature,
            shader,
            diagnostic,
        } => RendererDataDiagnosticRow {
            feature: feature.clone(),
            material_reference: None,
            shader_references: Vec::new(),
            source: None,
            severity: RendererFeatureContractDiagnosticSeverity::Error,
            path: format!("features.{feature}.shader.validation_diagnostics"),
            message: format!("shader `{}` validation: {diagnostic}", shader.locator),
        },
    };

    with_diagnostic_ownership(row, diagnostic)
}

fn with_diagnostic_ownership(
    mut row: RendererDataDiagnosticRow,
    diagnostic: &RendererFeatureContractDiagnostic,
) -> RendererDataDiagnosticRow {
    row.material_reference = diagnostic.material_reference().cloned();
    row.shader_references = diagnostic
        .shader_references()
        .into_iter()
        .cloned()
        .collect();
    row.source = diagnostic.source();
    row.severity = diagnostic.severity();
    row
}

fn material_validation_diagnostic_row(
    feature: &str,
    error: &RenderMaterialValidationError,
) -> RendererDataDiagnosticRow {
    match error {
        RenderMaterialValidationError::InvalidMaskCutoff { cutoff } => RendererDataDiagnosticRow {
            feature: feature.to_string(),
            material_reference: None,
            shader_references: Vec::new(),
            source: None,
            severity: RendererFeatureContractDiagnosticSeverity::Error,
            path: "overrides.alpha_mode.cutoff".to_string(),
            message: format!("alpha mask cutoff {cutoff} must be finite and within 0.0..=1.0"),
        },
        RenderMaterialValidationError::UnresolvedMaterialReference { material } => {
            RendererDataDiagnosticRow {
                feature: feature.to_string(),
                material_reference: None,
                shader_references: Vec::new(),
                source: None,
                severity: RendererFeatureContractDiagnosticSeverity::Error,
                path: "material".to_string(),
                message: format!("material `{material}` could not be resolved"),
            }
        }
        RenderMaterialValidationError::MissingRuntimeShaderSource => RendererDataDiagnosticRow {
            feature: feature.to_string(),
            material_reference: None,
            shader_references: Vec::new(),
            source: None,
            severity: RendererFeatureContractDiagnosticSeverity::Error,
            path: "shader".to_string(),
            message: "shader has no runtime WGSL source".to_string(),
        },
        RenderMaterialValidationError::UnresolvedShaderReference { reference } => {
            RendererDataDiagnosticRow {
                feature: feature.to_string(),
                material_reference: None,
                shader_references: Vec::new(),
                source: None,
                severity: RendererFeatureContractDiagnosticSeverity::Error,
                path: "shader".to_string(),
                message: format!("shader `{}` could not be resolved", reference.locator),
            }
        }
        RenderMaterialValidationError::UnresolvedTextureReference { slot, reference } => {
            RendererDataDiagnosticRow {
                feature: feature.to_string(),
                material_reference: None,
                shader_references: Vec::new(),
                source: None,
                severity: RendererFeatureContractDiagnosticSeverity::Error,
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
            material_reference: None,
            shader_references: Vec::new(),
            source: None,
            severity: RendererFeatureContractDiagnosticSeverity::Error,
            path: format!("textures.{slot}"),
            message: format!(
                "texture `{}` is not upload-ready: {reason}",
                reference.locator
            ),
        },
        RenderMaterialValidationError::InvalidLightingModel { path, value } => {
            RendererDataDiagnosticRow {
                feature: feature.to_string(),
                material_reference: None,
                shader_references: Vec::new(),
                source: None,
                severity: RendererFeatureContractDiagnosticSeverity::Error,
                path: path.clone(),
                message: format!("lighting model `{value}` is not supported"),
            }
        }
        RenderMaterialValidationError::RenderQueueAlphaModeConflict {
            source,
            path,
            alpha_mode,
            render_queue,
            expected,
        } => RendererDataDiagnosticRow {
            feature: feature.to_string(),
            material_reference: None,
            shader_references: Vec::new(),
            source: Some(*source),
            severity: RendererFeatureContractDiagnosticSeverity::Error,
            path: path.clone(),
            message: format!(
                "alpha mode `{alpha_mode}` uses render queue {render_queue}, expected {expected}"
            ),
        },
        RenderMaterialValidationError::UnregisteredShadingModel { path, token } => {
            RendererDataDiagnosticRow {
                feature: feature.to_string(),
                material_reference: None,
                shader_references: Vec::new(),
                source: None,
                severity: RendererFeatureContractDiagnosticSeverity::Error,
                path: path.clone(),
                message: format!("shading model `{token}` is not registered"),
            }
        }
        RenderMaterialValidationError::UnknownPropertyOverride { path, name, .. } => {
            RendererDataDiagnosticRow {
                feature: feature.to_string(),
                material_reference: None,
                shader_references: Vec::new(),
                source: None,
                severity: RendererFeatureContractDiagnosticSeverity::Error,
                path: path.clone(),
                message: format!("property override `{name}` is not declared by the shader"),
            }
        }
        RenderMaterialValidationError::PropertyOverrideTypeMismatch {
            path,
            name,
            expected,
            ..
        } => RendererDataDiagnosticRow {
            feature: feature.to_string(),
            material_reference: None,
            shader_references: Vec::new(),
            source: None,
            severity: RendererFeatureContractDiagnosticSeverity::Error,
            path: path.clone(),
            message: format!("property override `{name}` must match shader type `{expected}`"),
        },
        RenderMaterialValidationError::MissingRequiredProperty { path, name, .. } => {
            RendererDataDiagnosticRow {
                feature: feature.to_string(),
                material_reference: None,
                shader_references: Vec::new(),
                source: None,
                severity: RendererFeatureContractDiagnosticSeverity::Error,
                path: path.clone(),
                message: format!("required shader property `{name}` needs a material override"),
            }
        }
        RenderMaterialValidationError::MissingRequiredTextureSlot { path, slot, .. } => {
            RendererDataDiagnosticRow {
                feature: feature.to_string(),
                material_reference: None,
                shader_references: Vec::new(),
                source: None,
                severity: RendererFeatureContractDiagnosticSeverity::Error,
                path: path.clone(),
                message: format!(
                    "required texture slot `{slot}` needs a material texture reference"
                ),
            }
        }
        RenderMaterialValidationError::UnknownTextureSlot { path, slot, .. } => {
            RendererDataDiagnosticRow {
                feature: feature.to_string(),
                material_reference: None,
                shader_references: Vec::new(),
                source: None,
                severity: RendererFeatureContractDiagnosticSeverity::Error,
                path: path.clone(),
                message: format!("texture slot `{slot}` is not declared by the shader"),
            }
        }
        RenderMaterialValidationError::MissingWgslCapture { path, name, .. } => {
            RendererDataDiagnosticRow {
                feature: feature.to_string(),
                material_reference: None,
                shader_references: Vec::new(),
                source: None,
                severity: RendererFeatureContractDiagnosticSeverity::Error,
                path: path.clone(),
                message: format!("shader WGSL does not appear to capture `{name}`"),
            }
        }
        RenderMaterialValidationError::ShaderReadinessDiagnostic {
            path, diagnostic, ..
        } => RendererDataDiagnosticRow {
            feature: feature.to_string(),
            material_reference: None,
            shader_references: Vec::new(),
            source: None,
            severity: RendererFeatureContractDiagnosticSeverity::Error,
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
