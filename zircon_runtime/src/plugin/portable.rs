//! Runtime shader source contracts, independent of native plugin loading.
#[path = "package_manifest/plugin_shader_permutation_manifest.rs"]
mod shader;
pub use shader::{
    PluginShaderModuleManifest, PluginShaderModuleSource, PluginShaderPermutationIdManifest,
    PluginShaderPermutationManifest, ShaderModuleSourceBinding,
};
