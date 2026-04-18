use zircon_core::CoreError;

use super::super::super::errors::asset_error_message;
use super::super::ProjectAssetManager;
use crate::{AssetId, AssetKind, ImportedAsset};

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
}
