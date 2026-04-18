use zircon_core::CoreError;
use zircon_resource::{
    MaterialMarker, ModelMarker, ResourceHandle, SceneMarker, ShaderMarker, TextureMarker,
    UiLayoutMarker, UiStyleMarker, UiWidgetMarker,
};

use super::super::ProjectAssetManager;
use crate::{
    AssetId, MaterialAsset, ModelAsset, SceneAsset, ShaderAsset, TextureAsset, UiLayoutAsset,
    UiStyleAsset, UiWidgetAsset,
};

impl ProjectAssetManager {
    pub fn load_model_asset(&self, id: AssetId) -> Result<ModelAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<ModelMarker>::new(id), "model")
    }

    pub fn load_material_asset(&self, id: AssetId) -> Result<MaterialAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<MaterialMarker>::new(id), "material")
    }

    pub fn load_texture_asset(&self, id: AssetId) -> Result<TextureAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<TextureMarker>::new(id), "texture")
    }

    pub fn load_shader_asset(&self, id: AssetId) -> Result<ShaderAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<ShaderMarker>::new(id), "shader")
    }

    pub fn load_scene_asset(&self, id: AssetId) -> Result<SceneAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<SceneMarker>::new(id), "scene")
    }

    pub fn load_ui_layout_asset(&self, id: AssetId) -> Result<UiLayoutAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<UiLayoutMarker>::new(id), "ui layout")
    }

    pub fn load_ui_widget_asset(&self, id: AssetId) -> Result<UiWidgetAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<UiWidgetMarker>::new(id), "ui widget")
    }

    pub fn load_ui_style_asset(&self, id: AssetId) -> Result<UiStyleAsset, CoreError> {
        self.load_typed(id, ResourceHandle::<UiStyleMarker>::new(id), "ui style")
    }
}
