use crate::asset::{AssetReference, MaterialAsset, ShaderAsset};
use crate::core::framework::render::RenderFrameExtract;
use crate::graphics::pipeline::declarations::{
    RenderPipelineAsset, RenderPipelineCompileOptions, RenderPipelineCompileReport,
    RendererFeatureAsset, RendererFeatureContractDiagnostic,
};

pub trait RenderPipelineAssetContext {
    fn load_shader_asset(&self, reference: &AssetReference) -> Option<ShaderAsset>;
    fn load_material_asset(&self, reference: &AssetReference) -> Option<MaterialAsset>;
}

impl RenderPipelineAsset {
    pub fn compile_with_asset_context(
        &self,
        extract: &RenderFrameExtract,
        options: &RenderPipelineCompileOptions,
        context: &impl RenderPipelineAssetContext,
    ) -> Result<RenderPipelineCompileReport, String> {
        let pipeline = self.compile_with_options(extract, options)?;
        let diagnostics = self
            .renderer
            .features
            .iter()
            .filter(|feature| {
                feature.enabled
                    && options.permits_feature_asset(feature)
                    && feature
                        .quality_gate
                        .is_none_or(|gate| options.permits_feature(gate))
            })
            .flat_map(|feature| collect_feature_contract_diagnostics(feature, context))
            .collect::<Vec<_>>();

        Ok(RenderPipelineCompileReport {
            pipeline,
            diagnostics,
        })
    }
}

fn collect_feature_contract_diagnostics(
    feature: &RendererFeatureAsset,
    context: &impl RenderPipelineAssetContext,
) -> Vec<RendererFeatureContractDiagnostic> {
    let references = &feature.asset_references;
    let feature_name = feature.feature_name();
    let shader = references.shader.as_ref().and_then(|reference| {
        context
            .load_shader_asset(reference)
            .map(|asset| (reference, asset))
    });
    let material = references.material.as_ref().and_then(|reference| {
        context
            .load_material_asset(reference)
            .map(|asset| (reference, asset))
    });
    let mut diagnostics = Vec::new();

    if let Some(reference) = references.shader.as_ref() {
        if shader.is_none() {
            diagnostics.push(RendererFeatureContractDiagnostic::ShaderMissing {
                feature: feature_name.clone(),
                reference: reference.clone(),
            });
        }
    }
    if let Some(reference) = references.material.as_ref() {
        if material.is_none() {
            diagnostics.push(RendererFeatureContractDiagnostic::MaterialMissing {
                feature: feature_name.clone(),
                reference: reference.clone(),
            });
        }
    }

    if let Some((material_reference, material)) = material.as_ref() {
        for error in material.validation_errors() {
            diagnostics.push(RendererFeatureContractDiagnostic::MaterialValidation {
                feature: feature_name.clone(),
                error,
            });
        }
        for diagnostic in &material.validation_diagnostics {
            diagnostics.push(RendererFeatureContractDiagnostic::MaterialDiagnostic {
                feature: feature_name.clone(),
                material: (*material_reference).clone(),
                diagnostic: diagnostic.clone(),
            });
        }
    }

    if let Some((shader_reference, shader)) = shader.as_ref() {
        for entry_point in &references.required_entry_points {
            if !shader
                .entry_points
                .iter()
                .any(|entry| entry.name == *entry_point)
            {
                diagnostics.push(RendererFeatureContractDiagnostic::MissingEntryPoint {
                    feature: feature_name.clone(),
                    shader: (*shader_reference).clone(),
                    entry_point: entry_point.clone(),
                });
            }
        }
        for property in &references.expected_properties {
            if !shader
                .property_schema
                .iter()
                .any(|schema| schema.name == *property)
            {
                diagnostics.push(RendererFeatureContractDiagnostic::MissingProperty {
                    feature: feature_name.clone(),
                    shader: (*shader_reference).clone(),
                    property: property.clone(),
                });
            }
        }
        for slot in &references.expected_texture_slots {
            if !shader
                .texture_slots
                .iter()
                .any(|schema| schema.name == *slot)
            {
                diagnostics.push(RendererFeatureContractDiagnostic::MissingTextureSlot {
                    feature: feature_name.clone(),
                    shader: (*shader_reference).clone(),
                    slot: slot.clone(),
                });
            }
        }
        for diagnostic in &shader.validation_diagnostics {
            diagnostics.push(RendererFeatureContractDiagnostic::ShaderValidation {
                feature: feature_name.clone(),
                shader: (*shader_reference).clone(),
                diagnostic: diagnostic.clone(),
            });
        }
    }

    if let (Some((feature_shader, shader)), Some((_material_reference, material))) =
        (shader.as_ref(), material.as_ref())
    {
        if material.shader != **feature_shader {
            diagnostics.push(RendererFeatureContractDiagnostic::MaterialShaderMismatch {
                feature: feature_name.clone(),
                feature_shader: (*feature_shader).clone(),
                material_shader: material.shader.clone(),
            });
        }
        for error in material.shader_contract_diagnostics(shader) {
            diagnostics.push(RendererFeatureContractDiagnostic::MaterialValidation {
                feature: feature_name.clone(),
                error,
            });
        }
    }

    diagnostics
}
