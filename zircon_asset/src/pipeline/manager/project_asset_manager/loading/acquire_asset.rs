use zircon_core::CoreError;
use zircon_resource::{
    MaterialMarker, ModelMarker, ResourceHandle, ResourceLease, SceneMarker, ShaderMarker,
    TextureMarker, UiLayoutMarker, UiStyleMarker, UiWidgetMarker,
};

use super::super::ProjectAssetManager;
use crate::{
    AssetId, MaterialAsset, ModelAsset, SceneAsset, ShaderAsset, TextureAsset, UiLayoutAsset,
    UiStyleAsset, UiWidgetAsset,
};

impl ProjectAssetManager {
    pub fn acquire_model_asset(&self, id: AssetId) -> Result<ResourceLease<ModelAsset>, CoreError> {
        self.acquire_typed(id, ResourceHandle::<ModelMarker>::new(id), "model")
    }

    pub fn acquire_material_asset(
        &self,
        id: AssetId,
    ) -> Result<ResourceLease<MaterialAsset>, CoreError> {
        self.acquire_typed(id, ResourceHandle::<MaterialMarker>::new(id), "material")
    }

    pub fn acquire_texture_asset(
        &self,
        id: AssetId,
    ) -> Result<ResourceLease<TextureAsset>, CoreError> {
        self.acquire_typed(id, ResourceHandle::<TextureMarker>::new(id), "texture")
    }

    pub fn acquire_shader_asset(
        &self,
        id: AssetId,
    ) -> Result<ResourceLease<ShaderAsset>, CoreError> {
        self.acquire_typed(id, ResourceHandle::<ShaderMarker>::new(id), "shader")
    }

    pub fn acquire_scene_asset(&self, id: AssetId) -> Result<ResourceLease<SceneAsset>, CoreError> {
        self.acquire_typed(id, ResourceHandle::<SceneMarker>::new(id), "scene")
    }

    pub fn acquire_ui_layout_asset(
        &self,
        id: AssetId,
    ) -> Result<ResourceLease<UiLayoutAsset>, CoreError> {
        self.acquire_typed(id, ResourceHandle::<UiLayoutMarker>::new(id), "ui layout")
    }

    pub fn acquire_ui_widget_asset(
        &self,
        id: AssetId,
    ) -> Result<ResourceLease<UiWidgetAsset>, CoreError> {
        self.acquire_typed(id, ResourceHandle::<UiWidgetMarker>::new(id), "ui widget")
    }

    pub fn acquire_ui_style_asset(
        &self,
        id: AssetId,
    ) -> Result<ResourceLease<UiStyleAsset>, CoreError> {
        self.acquire_typed(id, ResourceHandle::<UiStyleMarker>::new(id), "ui style")
    }
}
