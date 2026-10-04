//! shader 导入依赖先由作者文件解析，再投影为渲染依赖描述；资源流据此跟踪编译输入及失效关系。

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
