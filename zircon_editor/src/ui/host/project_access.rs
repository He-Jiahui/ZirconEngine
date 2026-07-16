use std::path::{Path, PathBuf};
use std::sync::Arc;

use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime::asset::{asset_manager_handle, AssetManager};
use zircon_runtime::asset::{AssetImportError, AssetUri};
use zircon_runtime::core::framework::foundation::ConfigManager;
use zircon_runtime::core::manager::{resolve_manager_service, ManagerResolver};
use zircon_runtime::scene::LevelMetadata;

use crate::core::project::ProjectAuthority;
use crate::ui::host::editor_asset_manager::{editor_asset_manager_handle, EditorAssetManager};
use crate::ui::workbench::project::EditorProjectDocument;

use super::editor_error::EditorError;
use super::editor_ui_host::EditorUiHost;

impl EditorUiHost {
    pub(super) fn open_project(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<EditorProjectDocument, EditorError> {
        let root = ProjectAuthority::default().open_project(&path)?.root;
        self.asset_manager()?
            .open_project(root.to_string_lossy().as_ref())?;
        self.editor_asset_manager()?
            .refresh_from_runtime_project()?;
        self.restart_ui_asset_workspace_watcher()?;
        Ok(EditorProjectDocument::load_from_path(&path)?)
    }

    pub(super) fn save_project(
        &self,
        path: impl AsRef<Path>,
        world: &zircon_runtime::scene::Scene,
    ) -> Result<(), EditorError> {
        let workspace = self.project_workspace();
        let root = ProjectAuthority::default().open_project(&path)?.root;
        EditorProjectDocument::save_to_path(&path, world, Some(&workspace))?;
        self.asset_manager()?
            .open_project(root.to_string_lossy().as_ref())
            .map(|_| ())?;
        self.editor_asset_manager()?
            .refresh_from_runtime_project()?;
        self.restart_ui_asset_workspace_watcher()
    }

    pub(super) fn create_runtime_level(
        &self,
        scene: zircon_runtime::scene::Scene,
    ) -> Result<zircon_runtime::scene::LevelSystem, EditorError> {
        Ok(zircon_runtime::scene::create_level(
            &self.runtime_core()?,
            scene,
            LevelMetadata::default(),
        )?)
    }

    pub(super) fn config_manager(&self) -> Result<Arc<dyn ConfigManager>, EditorError> {
        let resolver = ManagerResolver::new(self.runtime_core()?);
        Ok(resolver.resolve(resolver.config_handle()?)?)
    }

    pub(super) fn asset_manager(&self) -> Result<Arc<dyn AssetManager>, EditorError> {
        let core = self.runtime_core()?;
        Ok(resolve_manager_service(
            &core,
            asset_manager_handle(&core)?,
        )?)
    }

    pub(super) fn editor_asset_manager(&self) -> Result<Arc<dyn EditorAssetManager>, EditorError> {
        let core = self.runtime_core()?;
        Ok(resolve_manager_service(
            &core,
            editor_asset_manager_handle(&core)?,
        )?)
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
            let project = open_project_manager_for_paths(&project_root)?;
            return resolve_existing_project_asset_path(&project, &format!("res://{relative}"));
        }
        Ok(PathBuf::from(asset_id))
    }

    pub(super) fn resolve_asset_locator_path(
        &self,
        asset_locator: &AssetUri,
    ) -> Result<PathBuf, EditorError> {
        let project_root = self.current_project_root()?.ok_or_else(|| {
            EditorError::UiAsset(format!(
                "cannot resolve {asset_locator} without an open project"
            ))
        })?;
        let project = open_project_manager_for_paths(&project_root)?;
        Ok(project.source_path_for_uri(asset_locator)?)
    }
}

pub(crate) fn open_project_manager_for_paths(
    project_root: &Path,
) -> Result<ProjectManager, EditorError> {
    Ok(ProjectManager::open(project_root)?)
}

pub(crate) fn resolve_existing_project_asset_path(
    project: &ProjectManager,
    asset_id: &str,
) -> Result<PathBuf, EditorError> {
    let uri = AssetUri::parse(asset_id)?;
    Ok(project.source_path_for_uri(&uri)?)
}

pub(crate) fn resolve_project_asset_write_path(
    project: &ProjectManager,
    asset_id: &str,
) -> Result<PathBuf, EditorError> {
    let uri = AssetUri::parse(asset_id)?;
    Ok(project.existing_or_primary_project_source_path_for_uri(&uri)?)
}

pub(crate) fn project_asset_id_for_source_path(
    project: &ProjectManager,
    source_path: &Path,
) -> Result<Option<String>, EditorError> {
    match project.project_uri_for_source_path(source_path) {
        Ok(uri) => Ok(Some(uri.to_string())),
        Err(AssetImportError::SourceOutsideProjectAssetRoots { .. }) => Ok(None),
        Err(error) => Err(error.into()),
    }
}

pub(crate) fn normalize_ui_asset_asset_id(asset_id: &str) -> &str {
    asset_id
        .split_once('#')
        .map(|(path, _)| path)
        .unwrap_or(asset_id)
}
