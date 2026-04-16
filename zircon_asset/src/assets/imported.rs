use serde::{Deserialize, Serialize};

use super::{
    MaterialAsset, ModelAsset, SceneAsset, ShaderAsset, TextureAsset, UiLayoutAsset, UiStyleAsset,
    UiWidgetAsset,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ImportedAsset {
    Texture(TextureAsset),
    Shader(ShaderAsset),
    Material(MaterialAsset),
    Scene(SceneAsset),
    Model(ModelAsset),
    UiLayout(UiLayoutAsset),
    UiWidget(UiWidgetAsset),
    UiStyle(UiStyleAsset),
}
