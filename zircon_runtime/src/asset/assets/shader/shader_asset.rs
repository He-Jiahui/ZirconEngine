use serde::{Deserialize, Serialize};

use crate::asset::AssetUri;
use crate::core::framework::render::{
    MaterialOptionTable, MaterialPropertyLayout, RenderShaderDefinitionValue,
    RenderShaderDependency, RenderShaderEntryPointDescriptor, RenderShaderPipelineLayoutDescriptor,
    RenderShaderVariantKey, ShaderAssetKind, ShaderQueueDescriptor, ShaderRenderStateDescriptor,
    ShaderResourceDescriptor,
};

use super::{
    classify_surface_source_contract, dependency, generate_material_artifact,
    language::default_shader_language, ShaderDependencyAsset, ShaderEntryPointAsset,
    ShaderImportRedirectAsset, ShaderMaterialPropertyAsset, ShaderOptionAsset,
    ShaderSourceFileAsset, ShaderSourceLanguage, ShaderSurfaceSourceContract,
    ShaderSurfaceSourceContractError, ShaderTextureSlotAsset,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShaderAsset {
    pub uri: AssetUri,
    #[serde(default = "default_shader_asset_kind")]
    pub kind: ShaderAssetKind,
    #[serde(default = "default_shader_language")]
    pub source_language: ShaderSourceLanguage,
    pub source: String,
    #[serde(default)]
    pub wgsl_source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub import_path: Option<String>,
    #[serde(default)]
    pub entry_points: Vec<ShaderEntryPointAsset>,
    #[serde(default)]
    pub dependencies: Vec<ShaderDependencyAsset>,
    #[serde(default)]
    pub source_files: Vec<ShaderSourceFileAsset>,
    #[serde(default)]
    pub imports: Vec<ShaderImportRedirectAsset>,
    #[serde(default)]
    pub shader_defs: Vec<RenderShaderDefinitionValue>,
    #[serde(default)]
    pub property_schema: Vec<ShaderMaterialPropertyAsset>,
    #[serde(default)]
    pub options: Vec<ShaderOptionAsset>,
    #[serde(default)]
    pub texture_slots: Vec<ShaderTextureSlotAsset>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shading_model: Option<String>,
    #[serde(default)]
    pub render_state: ShaderRenderStateDescriptor,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queue: Option<ShaderQueueDescriptor>,
    #[serde(default)]
    pub disabled_passes: Vec<String>,
    #[serde(default)]
    pub resources: Vec<ShaderResourceDescriptor>,
    #[serde(default)]
    pub material_property_layout: MaterialPropertyLayout,
    #[serde(default)]
    pub material_option_table: MaterialOptionTable,
    #[serde(default)]
    pub generated_material_wgsl: String,
    #[serde(default)]
    pub editor: toml::Table,
    #[serde(default)]
    pub pipeline_layout: RenderShaderPipelineLayoutDescriptor,
    #[serde(default)]
    pub validation_diagnostics: Vec<String>,
}

impl ShaderAsset {
    pub fn runtime_wgsl_source(&self) -> Option<&str> {
        if !self.wgsl_source.trim().is_empty() {
            Some(self.wgsl_source.as_str())
        } else if self.source_language == ShaderSourceLanguage::Wgsl
            && !self.source.trim().is_empty()
        {
            Some(self.source.as_str())
        } else {
            None
        }
    }

    pub fn dependencies(&self) -> Vec<RenderShaderDependency> {
        dependency::shader_dependencies(self)
    }

    pub fn entry_point_descriptors(&self) -> Vec<RenderShaderEntryPointDescriptor> {
        self.entry_points
            .iter()
            .filter_map(ShaderEntryPointAsset::descriptor)
            .collect()
    }

    pub fn variant_keys(&self) -> Vec<RenderShaderVariantKey> {
        self.entry_points
            .iter()
            .map(|entry| RenderShaderVariantKey {
                entry_point: Some(entry.name.clone()),
                stage: Some(entry.stage.clone()),
                defines: self.shader_defs.clone(),
            })
            .collect()
    }

    pub fn pipeline_layout_descriptor(&self) -> RenderShaderPipelineLayoutDescriptor {
        self.pipeline_layout.clone()
    }

    pub fn surface_source_contract(
        &self,
    ) -> Result<Option<ShaderSurfaceSourceContract>, ShaderSurfaceSourceContractError> {
        if self.kind != ShaderAssetKind::Surface {
            return Ok(None);
        }
        classify_surface_source_contract(
            self.runtime_wgsl_source().unwrap_or_default(),
            &self.entry_points,
        )
        .map(Some)
    }

    pub fn regenerate_material_artifact(&mut self) {
        if !self.kind.participates_in_material_variants() {
            self.material_property_layout = Default::default();
            self.material_option_table = Default::default();
            self.generated_material_wgsl.clear();
            return;
        }

        let generated =
            generate_material_artifact(&self.property_schema, &self.options, &self.texture_slots);
        self.material_property_layout = generated.property_layout;
        self.material_option_table = generated.option_table;
        self.generated_material_wgsl = generated.wgsl_source;
    }
}

fn default_shader_asset_kind() -> ShaderAssetKind {
    ShaderAssetKind::Surface
}
