use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginShaderPermutationManifest {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub geometry_source_ids: Vec<PluginShaderPermutationIdManifest>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shading_model_ids: Vec<PluginShaderPermutationIdManifest>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shader_modules: Vec<PluginShaderModuleManifest>,
}

impl PluginShaderPermutationManifest {
    pub fn is_empty(&self) -> bool {
        self.geometry_source_ids.is_empty()
            && self.shading_model_ids.is_empty()
            && self.shader_modules.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginShaderPermutationIdManifest {
    pub token: String,
    pub id: u8,
}

impl PluginShaderPermutationIdManifest {
    pub fn new(token: impl Into<String>, id: u8) -> Self {
        Self {
            token: token.into(),
            id,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginShaderModuleManifest {
    pub import_path: String,
    pub source: String,
}

impl PluginShaderModuleManifest {
    pub fn new(import_path: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            import_path: import_path.into(),
            source: source.into(),
        }
    }
}
