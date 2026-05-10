use serde::{Deserialize, Serialize};

use crate::core::framework::render::RenderShaderDependency;
use crate::core::resource::{AssetReference, ResourceKind};

use super::ShaderAsset;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderDependencyAsset {
    pub kind: ResourceKind,
    pub reference: AssetReference,
}

impl ShaderDependencyAsset {
    pub fn descriptor(&self) -> RenderShaderDependency {
        RenderShaderDependency::new(self.kind, self.reference.clone())
    }
}

pub fn shader_dependencies(shader: &ShaderAsset) -> Vec<RenderShaderDependency> {
    shader
        .dependencies
        .iter()
        .map(ShaderDependencyAsset::descriptor)
        .collect()
}
