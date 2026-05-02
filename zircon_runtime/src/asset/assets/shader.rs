use serde::{Deserialize, Serialize};

use crate::asset::AssetUri;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShaderSourceLanguage {
    Wgsl,
    Glsl,
    SpirV,
    Hlsl,
    Cg,
}

impl ShaderSourceLanguage {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Wgsl => "wgsl",
            Self::Glsl => "glsl",
            Self::SpirV => "spir_v",
            Self::Hlsl => "hlsl",
            Self::Cg => "cg",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderEntryPointAsset {
    pub name: String,
    pub stage: String,
}

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
    pub validation_diagnostics: Vec<String>,
}

fn default_shader_language() -> ShaderSourceLanguage {
    ShaderSourceLanguage::Wgsl
}
