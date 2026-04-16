use zircon_core::CoreError;

use super::builtins::builtin_resources;
use super::errors::{asset_error, asset_error_message};
use super::resource_sync::store_runtime_payload;
use super::ProjectAssetManager;
use crate::{
    AssetId, AssetKind, AssetUri, ImportedAsset, MaterialAsset, MaterialMarker, ModelAsset,
    ModelMarker, ResourceHandle, ResourceLease, SceneAsset, SceneMarker, ShaderAsset, ShaderMarker,
    TextureAsset, TextureMarker, UiLayoutAsset, UiLayoutMarker, UiStyleAsset, UiStyleMarker,
    UiWidgetAsset, UiWidgetMarker,
};

impl ProjectAssetManager {
    pub fn load_imported_asset(&self, id: AssetId) -> Result<ImportedAsset, CoreError> {
        let kind = self
            .resource_manager()
            .registry()
            .get(id)
            .map(|record| record.kind)
            .ok_or_else(|| {
                asset_error_message(format!("missing resource record for asset id {id}"))
            })?;

        match kind {
            AssetKind::Model => self.load_model_asset(id).map(ImportedAsset::Model),
            AssetKind::Material => self.load_material_asset(id).map(ImportedAsset::Material),
            AssetKind::Texture => self.load_texture_asset(id).map(ImportedAsset::Texture),
            AssetKind::Shader => self.load_shader_asset(id).map(ImportedAsset::Shader),
            AssetKind::Scene => self.load_scene_asset(id).map(ImportedAsset::Scene),
            AssetKind::UiLayout => self.load_ui_layout_asset(id).map(ImportedAsset::UiLayout),
            AssetKind::UiWidget => self.load_ui_widget_asset(id).map(ImportedAsset::UiWidget),
            AssetKind::UiStyle => self.load_ui_style_asset(id).map(ImportedAsset::UiStyle),
        }
    }

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

    fn load_typed<TMarker, TAsset>(
        &self,
        id: AssetId,
        handle: ResourceHandle<TMarker>,
        label: &str,
    ) -> Result<TAsset, CoreError>
    where
        TMarker: crate::ResourceMarker,
        TAsset: crate::ResourceData + Clone,
    {
        self.ensure_resident(id)?;
        self.resource_manager()
            .get::<TMarker, TAsset>(handle)
            .map(|asset| asset.as_ref().clone())
            .ok_or_else(|| asset_error_message(format!("asset {id} was not a ready {label}")))
    }

    fn acquire_typed<TMarker, TAsset>(
        &self,
        id: AssetId,
        handle: ResourceHandle<TMarker>,
        label: &str,
    ) -> Result<ResourceLease<TAsset>, CoreError>
    where
        TMarker: crate::ResourceMarker,
        TAsset: crate::ResourceData,
    {
        self.ensure_resident(id)?;
        self.resource_manager()
            .acquire::<TMarker, TAsset>(handle)
            .ok_or_else(|| asset_error_message(format!("asset {id} was not a ready {label}")))
    }

    fn ensure_resident(&self, id: AssetId) -> Result<(), CoreError> {
        if self.resource_manager().get_untyped(id).is_some() {
            return Ok(());
        }

        let metadata = self
            .resource_manager()
            .registry()
            .get(id)
            .cloned()
            .ok_or_else(|| {
                asset_error_message(format!("missing resource record for asset id {id}"))
            })?;
        let imported = match metadata.primary_locator.scheme() {
            crate::AssetUriScheme::Builtin => builtin_resources()
                .into_iter()
                .find_map(|(locator_text, asset)| {
                    let locator = AssetUri::parse(locator_text).ok()?;
                    (locator == metadata.primary_locator).then_some(asset)
                })
                .ok_or_else(|| {
                    asset_error_message(format!(
                        "missing builtin runtime payload for {}",
                        metadata.primary_locator
                    ))
                })?,
            crate::AssetUriScheme::Res | crate::AssetUriScheme::Library => {
                let project = self.project_read();
                let project = project
                    .as_ref()
                    .ok_or_else(|| asset_error_message("no project is currently open"))?;
                project.load_artifact_by_id(id).map_err(asset_error)?
            }
            crate::AssetUriScheme::Memory => {
                return Err(asset_error_message(format!(
                    "memory resource {id} cannot be restored by ProjectAssetManager"
                )));
            }
        };
        store_runtime_payload(&self.resource_manager, id, imported);
        Ok(())
    }
}
