use serde::{Deserialize, Serialize};

use crate::core::resource::AssetReference;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

    pub fn all_references(&self) -> Vec<AssetReference> {
        let mut references = Vec::with_capacity(1 + self.textures.len());
        references.push(self.shader.clone());
        references.extend(self.textures.clone());
        references
    }
}
