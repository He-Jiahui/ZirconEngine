use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
    pub source: Arc<str>,
}

impl PluginShaderModuleManifest {
    pub fn new(import_path: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            import_path: import_path.into(),
            source: source.into().into(),
        }
    }
}

/// Runtime-resolved shader-module text for either a project asset or a plugin package.
///
/// `PluginShaderModuleManifest` deliberately stays serializable and records a
/// package-relative source path. This binding is created before template assembly,
/// so render-time shader assembly never reads source files or drops source ownership.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShaderModuleSourceBinding {
    pub owner_id: String,
    pub import_path: String,
    pub source: Arc<str>,
    pub content_hash: String,
    pub diagnostic_origin: String,
}

impl ShaderModuleSourceBinding {
    pub fn new(
        owner_id: impl Into<String>,
        import_path: impl Into<String>,
        source: impl Into<Arc<str>>,
        diagnostic_origin: impl Into<String>,
    ) -> Self {
        let source = source.into();
        Self {
            owner_id: owner_id.into(),
            import_path: import_path.into(),
            content_hash: blake3::hash(source.as_bytes()).to_hex().to_string(),
            source,
            diagnostic_origin: diagnostic_origin.into(),
        }
    }
}

/// Compatibility name for plugin APIs. Project modules use the same underlying binding.
pub type PluginShaderModuleSource = ShaderModuleSourceBinding;
