use std::path::{Path, PathBuf};
use std::sync::Arc;

use zircon_runtime::asset::pipeline::manager::{resolve_asset_manager, AssetManager};
use zircon_runtime::core::framework::foundation::ConfigManager;
use zircon_runtime::core::manager::ManagerResolver;
use zircon_runtime::scene::{DefaultLevelManager, LevelMetadata, DEFAULT_LEVEL_MANAGER_NAME};

use crate::ui::host::editor_asset_manager::{resolve_editor_asset_manager, EditorAssetManager};
use crate::ui::workbench::project::{project_root_path, EditorProjectDocument};

use super::editor_error::EditorError;
use super::editor_ui_host::EditorUiHost;

impl EditorUiHost {
    pub(super) fn open_project(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<EditorProjectDocument, EditorError> {
        let root =
            project_root_path(&path).map_err(|error| EditorError::Project(error.to_string()))?;
        self.asset_manager()?
            .open_project(root.to_string_lossy().as_ref())
            .map_err(|error| EditorError::Project(error.to_string()))?;
        self.editor_asset_manager()?
            .refresh_from_runtime_project()
            .map_err(|error| EditorError::Project(error.to_string()))?;
        EditorProjectDocument::load_from_path(&path)
            .map_err(|error| EditorError::Project(error.to_string()))
    }

    pub(super) fn save_project(
        &self,
        path: impl AsRef<Path>,
        world: &zircon_runtime::scene::Scene,
    ) -> Result<(), EditorError> {
        let workspace = self.project_workspace();
        let root =
            project_root_path(&path).map_err(|error| EditorError::Project(error.to_string()))?;
        EditorProjectDocument::save_to_path(&path, world, Some(&workspace))
            .map_err(|error| EditorError::Project(error.to_string()))?;
        self.asset_manager()?
            .open_project(root.to_string_lossy().as_ref())
            .map(|_| ())
            .map_err(|error| EditorError::Project(error.to_string()))?;
        self.editor_asset_manager()?
            .refresh_from_runtime_project()
            .map_err(|error| EditorError::Project(error.to_string()))
    }

    pub(super) fn create_runtime_level(
        &self,
        scene: zircon_runtime::scene::Scene,
    ) -> Result<zircon_runtime::scene::LevelSystem, EditorError> {
        let manager = self
            .core
            .resolve_manager::<DefaultLevelManager>(DEFAULT_LEVEL_MANAGER_NAME)
            .map_err(|error| EditorError::Project(error.to_string()))?;
        Ok(manager.create_level(scene, LevelMetadata::default()))
    }

    pub(super) fn config_manager(&self) -> Result<Arc<dyn ConfigManager>, EditorError> {
        ManagerResolver::new(self.core.clone())
            .config()
            .map_err(|error| EditorError::Project(error.to_string()))
    }

    pub(super) fn asset_manager(&self) -> Result<Arc<dyn AssetManager>, EditorError> {
        resolve_asset_manager(&self.core).map_err(|error| EditorError::Project(error.to_string()))
    }

    pub(super) fn editor_asset_manager(&self) -> Result<Arc<dyn EditorAssetManager>, EditorError> {
        resolve_editor_asset_manager(&self.core)
            .map_err(|error| EditorError::Project(error.to_string()))
    }

    pub(super) fn current_project_root(&self) -> Result<Option<PathBuf>, EditorError> {
        let Some(project) = self.asset_manager()?.current_project() else {
            return Ok(None);
        };
        Ok(Some(PathBuf::from(project.root_path)))
    }

    pub(super) fn resolve_ui_asset_path(
        &self,
        asset_id: impl AsRef<str>,
    ) -> Result<PathBuf, EditorError> {
        let asset_id = normalize_ui_asset_asset_id(asset_id.as_ref());
        if let Some(relative) = asset_id.strip_prefix("res://") {
            let project_root = self.current_project_root()?.ok_or_else(|| {
                EditorError::UiAsset(format!("cannot resolve {asset_id} without an open project"))
            })?;
            return Ok(project_root.join("assets").join(relative));
        }
        Ok(PathBuf::from(asset_id))
    }
}

pub(crate) fn normalize_ui_asset_asset_id(asset_id: &str) -> &str {
    asset_id
        .split_once('#')
        .map(|(path, _)| path)
        .unwrap_or(asset_id)
}
