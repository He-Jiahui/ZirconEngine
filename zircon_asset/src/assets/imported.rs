use serde::{Deserialize, Serialize};

use super::{MaterialAsset, ModelAsset, SceneAsset, ShaderAsset, TextureAsset};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ImportedAsset {
    Texture(TextureAsset),
    Shader(ShaderAsset),
    Material(MaterialAsset),
    Scene(SceneAsset),
    Model(ModelAsset),
}
