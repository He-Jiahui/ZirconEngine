use std::fs;
use std::path::PathBuf;

use zircon_asset::{ProjectManager, ProjectManifest, ProjectPaths};
use zircon_scene::{Scene, SceneProjectError};

use super::constants::DEFAULT_SCENE_URI;
use super::editor_project_document::EditorProjectDocument;
use super::project_root_path::project_root_path;
use super::runtime_asset_helpers::{invalid_data, parse_asset_uri};
use crate::workbench::startup::NewProjectDraft;

impl EditorProjectDocument {
    pub fn create_renderable_template(
        draft: &NewProjectDraft,
    ) -> Result<PathBuf, SceneProjectError> {
        let root = draft
            .validate_for_creation()
            .map_err(|error| invalid_data(error))?;
        if !root.exists() {
            fs::create_dir_all(&root)?;
        }
        Self::ensure_runtime_assets(&root)?;

        let paths = ProjectPaths::from_root(&root)?;
        if !paths.manifest_path().exists() {
            let default_scene = parse_asset_uri(DEFAULT_SCENE_URI)?;
            ProjectManifest::new(&draft.project_name, default_scene, 1)
                .save(paths.manifest_path())?;
        }

        let mut project = ProjectManager::open(&root)?;
        project.scan_and_import()?;
        let scene = Scene::default();
        let scene_asset = scene.to_scene_asset(&project)?;
        let scene_path = project.source_path_for_uri(&project.manifest().default_scene)?;
        if let Some(parent) = scene_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(
            scene_path,
            scene_asset
                .to_toml_string()
                .map_err(|error| invalid_data(error.to_string()))?,
        )?;
        Ok(project_root_path(root)?)
    }
}
