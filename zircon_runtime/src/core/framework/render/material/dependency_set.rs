use serde::{Deserialize, Serialize};

use crate::core::resource::AssetReference;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 材质的直接资源依赖快照：shader 始终在前，纹理按资产发现顺序去重，供加载与就绪检查共用。
pub struct RenderMaterialDependencySet {
    pub shader: AssetReference,
    pub textures: Vec<AssetReference>,
}

impl RenderMaterialDependencySet {
    pub fn new(shader: AssetReference) -> Self {
        Self {
            shader,
            textures: Vec::new(),
        }
    }

    pub fn push_texture(&mut self, texture: AssetReference) {
        if !self.textures.contains(&texture) {
            self.textures.push(texture);
        }
    }

    /// 供资产管理器遍历直接依赖；保留 shader 在首位和纹理的稳定顺序，避免重载时依赖次序漂移。
    pub fn all_references(&self) -> Vec<AssetReference> {
        let mut references = Vec::with_capacity(1 + self.textures.len());
        references.push(self.shader.clone());
        extend_cloned_values(&mut references, &self.textures);
        references
    }
}

fn extend_cloned_values<T: Clone>(target: &mut Vec<T>, source: &[T]) {
    target.extend_from_slice(source);
}

#[cfg(test)]
#[path = "dependency_set/tests/streaming_reference_tests.rs"]
mod streaming_reference_tests;
