use crate::core::framework::render::{
    RenderMaterialDependencySet, RenderMaterialFallbackPolicy, RenderMaterialFallbackReason,
    RenderMaterialFallbackUsage, RenderMaterialReadinessReport, RenderMaterialValidationError,
};
use crate::core::resource::{AssetReference, ResourceId};

use crate::graphics::types::GraphicsError;

use super::super::fallback_shader_uri;
use super::super::prepared::PreparedShader;
use super::super::runtime::ShaderRuntime;
use super::ResourceStreamer;

impl ResourceStreamer {
    pub(crate) fn ensure_shader_source(
        &mut self,
        reference: &AssetReference,
    ) -> Result<(ResourceId, u64, Option<RenderMaterialReadinessReport>), GraphicsError> {
        let uri = &reference.locator;
        let mut fallback_report = None;
        let (shader_id, shader) = match self.asset_manager.resolve_asset_id(uri) {
            Some(shader_id) => match self.asset_manager.load_shader_asset(shader_id) {
                Ok(shader) => (shader_id, shader),
                Err(_) => {
                    fallback_report = Some(missing_shader_readiness_report(reference));
                    self.load_fallback_shader()?
                }
            },
            None => {
                fallback_report = Some(missing_shader_readiness_report(reference));
                self.load_fallback_shader()?
            }
        };
        let (shader_id, shader) = if shader.runtime_wgsl_source().is_some() {
            (shader_id, shader)
        } else {
            fallback_report = Some(missing_runtime_shader_readiness_report(reference));
            self.load_fallback_shader()?
        };
        let revision = self.resource_revision(shader_id)?;

        if self
            .shaders
            .get(&shader_id)
            .is_some_and(|prepared| prepared.revision == revision)
        {
            return Ok((shader_id, revision, fallback_report));
        }
        self.shaders.insert(
            shader_id,
            PreparedShader {
                revision,
                runtime: ShaderRuntime {
                    source: shader
                        .runtime_wgsl_source()
                        .ok_or_else(|| {
                            GraphicsError::Asset(format!(
                                "shader {} has no runtime WGSL source",
                                shader.uri
                            ))
                        })?
                        .to_string(),
                },
            },
        );
        Ok((shader_id, revision, fallback_report))
    }

    fn load_fallback_shader(
        &self,
    ) -> Result<(ResourceId, crate::asset::ShaderAsset), GraphicsError> {
        let fallback_uri = fallback_shader_uri();
        let shader_id = self
            .asset_manager
            .resolve_asset_id(&fallback_uri)
            .ok_or_else(|| {
                GraphicsError::Asset(format!("missing shader resource for {fallback_uri}"))
            })?;
        let shader = self
            .asset_manager
            .load_shader_asset(shader_id)
            .map_err(|error| GraphicsError::Asset(error.to_string()))?;
        Ok((shader_id, shader))
    }
}

fn missing_shader_readiness_report(reference: &AssetReference) -> RenderMaterialReadinessReport {
    RenderMaterialReadinessReport {
        material_name: None,
        dependencies: RenderMaterialDependencySet::new(reference.clone()),
        fallback_policy: RenderMaterialFallbackPolicy::DefaultMaterial,
        validation_errors: vec![RenderMaterialValidationError::UnresolvedShaderReference {
            reference: reference.clone(),
        }],
        fallback_usages: vec![RenderMaterialFallbackUsage {
            reason: RenderMaterialFallbackReason::Shader {
                reference: reference.clone(),
            },
            fallback_policy: RenderMaterialFallbackPolicy::DefaultMaterial,
        }],
        property_value_summary: None,
        property_value_states: Vec::new(),
        uniform_summary: None,
        uniform_fields: Vec::new(),
        uniform_unsupported: Vec::new(),
        standard_texture_slot_summary: None,
        standard_texture_slot_states: Vec::new(),
        texture_slot_summary: None,
        non_standard_texture_slot_states: Vec::new(),
        diagnostics: Vec::new(),
    }
}

fn missing_runtime_shader_readiness_report(
    reference: &AssetReference,
) -> RenderMaterialReadinessReport {
    RenderMaterialReadinessReport {
        material_name: None,
        dependencies: RenderMaterialDependencySet::new(reference.clone()),
        fallback_policy: RenderMaterialFallbackPolicy::DefaultMaterial,
        validation_errors: vec![RenderMaterialValidationError::MissingRuntimeShaderSource],
        fallback_usages: vec![RenderMaterialFallbackUsage {
            reason: RenderMaterialFallbackReason::Shader {
                reference: reference.clone(),
            },
            fallback_policy: RenderMaterialFallbackPolicy::DefaultMaterial,
        }],
        property_value_summary: None,
        property_value_states: Vec::new(),
        uniform_summary: None,
        uniform_fields: Vec::new(),
        uniform_unsupported: Vec::new(),
        standard_texture_slot_summary: None,
        standard_texture_slot_states: Vec::new(),
        texture_slot_summary: None,
        non_standard_texture_slot_states: Vec::new(),
        diagnostics: Vec::new(),
    }
}
