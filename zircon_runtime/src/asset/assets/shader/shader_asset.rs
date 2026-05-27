use serde::{Deserialize, Serialize};

use crate::asset::AssetUri;
use crate::core::framework::render::{
    RenderShaderDefinitionValue, RenderShaderDependency, RenderShaderEntryPointDescriptor,
    RenderShaderPipelineLayoutDescriptor, RenderShaderVariantKey,
};

use super::{
    dependency, language::default_shader_language, ShaderDependencyAsset, ShaderEntryPointAsset,
    ShaderImportRedirectAsset, ShaderMaterialPropertyAsset, ShaderSourceFileAsset,
    ShaderSourceLanguage, ShaderTextureSlotAsset,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShaderAsset {
    pub uri: AssetUri,
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
    pub texture_slots: Vec<ShaderTextureSlotAsset>,
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
}
