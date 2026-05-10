use serde::{Deserialize, Serialize};

use crate::asset::AssetUri;
use crate::core::framework::render::{
    RenderShaderDependency, RenderShaderEntryPointDescriptor, RenderShaderPipelineLayoutDescriptor,
    RenderShaderVariantKey,
};

use super::{
    dependency, language::default_shader_language, ShaderDependencyAsset, ShaderEntryPointAsset,
    ShaderSourceLanguage,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShaderAsset {
    pub uri: AssetUri,
    #[serde(default = "default_shader_language")]
    pub source_language: ShaderSourceLanguage,
    pub source: String,
    #[serde(default)]
    pub wgsl_source: String,
    #[serde(default)]
    pub entry_points: Vec<ShaderEntryPointAsset>,
    #[serde(default)]
    pub dependencies: Vec<ShaderDependencyAsset>,
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
                defines: Vec::new(),
            })
            .collect()
    }

    pub fn pipeline_layout_descriptor(&self) -> RenderShaderPipelineLayoutDescriptor {
        self.pipeline_layout.clone()
    }
}
