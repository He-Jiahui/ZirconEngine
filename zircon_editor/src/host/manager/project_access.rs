use std::path::{Path, PathBuf};
use std::sync::Arc;

use zircon_manager::{AssetManager, ConfigManager, ManagerResolver};
use zircon_scene::{DefaultLevelManager, LevelMetadata, Scene, DEFAULT_LEVEL_MANAGER_NAME};

use crate::project::{project_root_path, EditorProjectDocument};

use super::editor_error::EditorError;
use super::editor_manager::EditorManager;

impl EditorManager {
    pub fn open_project(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<EditorProjectDocument, EditorError> {
        let root =
            project_root_path(&path).map_err(|error| EditorError::Project(error.to_string()))?;
        self.asset_manager()?
            .open_project(root.to_string_lossy().as_ref())
            .map_err(|error| EditorError::Project(error.to_string()))?;
        EditorProjectDocument::load_from_path(&path)
            .map_err(|error| EditorError::Project(error.to_string()))
    }

    pub fn save_project(&self, path: impl AsRef<Path>, world: &Scene) -> Result<(), EditorError> {
        let workspace = self.project_workspace();
        let root =
            project_root_path(&path).map_err(|error| EditorError::Project(error.to_string()))?;
        EditorProjectDocument::save_to_path(&path, world, Some(&workspace))
            .map_err(|error| EditorError::Project(error.to_string()))?;
        self.asset_manager()?
            .open_project(root.to_string_lossy().as_ref())
            .map(|_| ())
            .map_err(|error| EditorError::Project(error.to_string()))
    }

    pub fn create_runtime_level(
        &self,
        scene: Scene,
    ) -> Result<zircon_scene::LevelSystem, EditorError> {
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
        ManagerResolver::new(self.core.clone())
            .asset()
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

pub(super) fn normalize_ui_asset_asset_id(asset_id: &str) -> &str {
    asset_id
        .split_once('#')
        .map(|(path, _)| path)
        .unwrap_or(asset_id)
}
